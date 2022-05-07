use proto::raft::{
    VoteRequest, 
    VoteResponse,
};
use serde::{de::DeserializeOwned, Serialize};
use tonic::{Request, Response, Status};
use crate::{node::RaftNode, storage::state::persistent::Log};
use log::{debug, info, trace};


pub async fn request_vote<S>(node: &RaftNode<S>, request: Request<VoteRequest>) -> Result<Response<VoteResponse>, Status> 
where
    S: state_machine::StateMachine,
    S::MutationCommand: Clone + Serialize + DeserializeOwned
{
    trace!("Got a Vote request: {:?}", request);

    let vote_request = request.into_inner();
    
    let mut node_persistent_state_guard = node.persistent_state.lock().unwrap();

    if vote_request.term < node_persistent_state_guard.current_term {
        return Ok(Response::new( VoteResponse { term: node_persistent_state_guard.current_term , vote_granted: false }));
    }

    let mut vote_response = VoteResponse { term: node_persistent_state_guard.current_term , vote_granted: false};

    if let Some(voted_for) = node_persistent_state_guard.voted_for {
        if 
        (
            vote_request.candidate_id == voted_for
        ) && 
        (
            vote_request.last_log_index >= (node_persistent_state_guard.log.len().try_into().expect("Couldn't convert length from usize into u64."))
            &&
            vote_request.last_log_term >= node_persistent_state_guard.current_term
        ) {
            vote_response.vote_granted = true;
        }
    } else {
        vote_response.vote_granted = true;
        node_persistent_state_guard.voted_for = Some(vote_request.candidate_id);
    }

    drop(node_persistent_state_guard);
    node.save().expect("Could not save persistent state.");
    trace!("Will send a vote response of {vote_response:?}");
    Ok(Response::new(vote_response))
}


#[cfg(test)]
pub mod tests {
    use crate::utils::test_utils::set_up_logging;
    use crate::node::{RaftNode, Int};
    use proto::raft::raft_rpc_server::RaftRpc;
    use state_machine::impls::key_value_store::*;
    use state_machine::StateMachine;
    use log::{info, debug};
    use super::*;
    use tonic::{Request, Response, Status};


    pub fn create_request(term: Int, candidate_id: Int, last_log_index: Int, last_log_term: Int) -> Request<VoteRequest> {
        let vote_request = VoteRequest {
            term, candidate_id, last_log_index, last_log_term
        };
        Request::new(vote_request)
    }

    pub fn create_response(term: Int, vote_granted: bool) -> VoteResponse {
        VoteResponse {
            term,
            vote_granted
        }
    }

    #[tokio::test]
    pub async fn test_request_vote_basic() {
        set_up_logging();

        // The node that receives the RPC.
        let receiver: RaftNode<KeyValueStore<String, usize>> = RaftNode::default();

        // Set some state for the receiver.


        // The sender node sends this request to the receiver node.
        let request = create_request(
            0, 
            0, 
            0, 
            0
        );

        // The response that the receiver node sends.
        let expected_response = create_response(0, true);

        let response = receiver.request_vote(request).await.unwrap();

        // Assert the expected response.
        assert_eq!(response.into_inner(), expected_response);

        // Assert the state of the receiver.
        

    }

}
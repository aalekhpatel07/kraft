use proto::raft::{
    VoteRequest, 
    VoteResponse,
};
use serde::de::DeserializeOwned;
use tonic::{Request, Response, Status};
use crate::node::Node;
use log::{debug, info, trace};


pub async fn request_vote<L: serde::Serialize + DeserializeOwned + Clone>(node: &Node<L>, request: Request<VoteRequest>) -> Result<Response<VoteResponse>, Status> {
    trace!("Got a Vote request: {:?}", request);

    let vote_request = request.into_inner();
    
    let mut node_persistent_state_guard = node.persistent_state.lock().unwrap();

    if vote_request.term < node_persistent_state_guard.current_term as u64 {
        return Ok(Response::new( VoteResponse { term: node_persistent_state_guard.current_term as u64, vote_granted: false }));
    }

    let mut vote_response = VoteResponse { term: node_persistent_state_guard.current_term as u64, vote_granted: false};

    if let Some(voted_for) = node_persistent_state_guard.voted_for {
        if 
        (
            vote_request.candidate_id == voted_for as u64
        ) && 
        (
            vote_request.last_log_index >= node_persistent_state_guard.log.len() as u64
            &&
            vote_request.last_log_term >= node_persistent_state_guard.current_term as u64
        ) {
            vote_response.vote_granted = true;
        }
    } else {
        vote_response.vote_granted = true;
        node_persistent_state_guard.voted_for = Some(vote_request.candidate_id as usize);
    }

    drop(node_persistent_state_guard);
    node.save();
    trace!("Will send a vote response of {vote_response:?}");
    Ok(Response::new(vote_response))
}
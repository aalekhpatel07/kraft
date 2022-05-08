use serde::{Serialize, de::DeserializeOwned};
use tonic::{Request, Response, Status};
use crate::{node::{RaftNode}, storage::state::persistent::Log};
use proto::raft::{
    HeartbeatRequest,
    HeartbeatResponse,
    VoteRequest,
    VoteResponse,
    AppendEntriesRequest,
    AppendEntriesResponse,
    raft_rpc_server::RaftRpc
};
pub mod request_vote;
pub mod append_entries;
pub mod heartbeat;


#[tonic::async_trait]
impl<S> RaftRpc for RaftNode<S>
where
    S: 'static + state_machine::StateMachine,
    S::MutationCommand: 'static + Send + Clone + Serialize + DeserializeOwned + std::fmt::Debug + From<Vec<u8>>
{
    async fn append_entries(&self, request: Request<AppendEntriesRequest>) -> Result<Response<AppendEntriesResponse>, Status> {
        append_entries::append_entries(&self, request).await
    }
    async fn heartbeat(&self, request: Request<HeartbeatRequest>) -> Result<Response<HeartbeatResponse>, Status> {
        heartbeat::heartbeat(&self, request).await
    }
    async fn request_vote(&self, request: Request<VoteRequest>) -> Result<Response<VoteResponse>, Status> {
        request_vote::request_vote(&self, request).await
    }
}

#[cfg(test)]
pub mod tests {
    use proto::raft::raft_rpc_server::RaftRpc;
    use state_machine::impls::key_value_store::KeyValueStore;
    use tonic::Request;

    use super::*;
    use crate::{utils::test_utils::set_up_logging, storage::state::persistent::Log};
    use crate::node::RaftNode;
    use log::{info, trace, debug};


    #[tokio::test]
    async fn test_append_entries_basic() {
        set_up_logging();
        let node: RaftNode<KeyValueStore<String, usize>> = RaftNode::default();
        let request = Request::new(AppendEntriesRequest::default());
        let response = node.append_entries(request).await.unwrap();

        let response = response.into_inner();
        assert_eq!(response.term, 0);
        assert_eq!(response.success, false);

        info!("response: {response:?}");
    }

    #[tokio::test]
    async fn test_heartbeat_basic() {
        set_up_logging();
        let node: RaftNode<KeyValueStore<String, usize>> = RaftNode::default();
        let request = Request::new(HeartbeatRequest::default());
        let response = node.heartbeat(request).await.unwrap();

        let response = response.into_inner();
        assert_eq!(response.term, 0);
        assert_eq!(response.success, false);

        info!("response: {response:?}");
    }

    #[tokio::test]
    async fn test_request_vote_basic() {
        set_up_logging();
        let node: RaftNode<KeyValueStore<String, usize>> = RaftNode::default();
        let request = Request::new(VoteRequest::default());
        let response = node.request_vote(request).await.unwrap();
        let response = response.into_inner();
        assert_eq!(response.term, 0);
        assert_eq!(response.vote_granted, true);
        info!("response: {response:?}");
    }
}
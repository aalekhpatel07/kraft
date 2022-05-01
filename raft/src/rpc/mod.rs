use serde::{Serialize, de::DeserializeOwned};
use tonic::{Request, Response, Status};
use crate::node::Node;
use proto::raft::{
    HeartbeatRequest,
    HeartbeatResponse,
    VoteRequest,
    VoteResponse,
    AppendEntriesRequest,
    AppendEntriesResponse, raft_server::Raft
};
mod request_vote;
mod append_entries;
mod heartbeat;


#[tonic::async_trait]
impl<L> Raft for Node<L> 
where
    L: Send + Sync + 'static + Serialize + DeserializeOwned + Clone
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
    use proto::raft::raft_server::Raft;
    use tonic::Request;

    use super::*;
    use crate::utils::test_utils::set_up_logging;
    use crate::node::Node;
    use log::{info, trace, debug};


    #[tokio::test]
    async fn test_append_entries_basic() {
        set_up_logging();
        let node = Node::<(String, usize)>::default();
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
        let node = Node::<(String, usize)>::default();
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
        let node = Node::<(String, usize)>::default();
        let request = Request::new(VoteRequest::default());
        let response = node.request_vote(request).await.unwrap();
        let response = response.into_inner();
        assert_eq!(response.term, 0);
        assert_eq!(response.vote_granted, true);
        info!("response: {response:?}");
    }
}
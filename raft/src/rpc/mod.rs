use serde::{Serialize, de::DeserializeOwned, Deserialize};
use tonic::{Request, Response, Status, Code};
use crate::{node::{Raft, Follower, Log, Candidate, Leader}};

use proto::{
    HeartbeatRequest,
    HeartbeatResponse,
    VoteRequest,
    VoteResponse,
    AppendEntriesRequest,
    AppendEntriesResponse,
    raft_rpc_server::{RaftRpc}
};
mod request_vote;
mod append_entries;
mod heartbeat;
use anyhow::Result;
use log::{info, error, warn};
use crate::utils::test_utils::set_up_logging;


// #[tonic::async_trait]
// impl<S, T> RaftRpc for Raft<S, T> 
// where
//     T: 'static + Send+ Clone + Serialize + DeserializeOwned + From<Vec<u8>> + std::fmt::Debug,
//     S: 'static + Send + Sync,
// {
//     async fn append_entries(&self, request: Request<AppendEntriesRequest>) -> Result<Response<AppendEntriesResponse>, Status> {
//         Ok(append_entries::append_entries(self, request).await?)
//     }
//     async fn heartbeat(&self, request: Request<HeartbeatRequest>) -> Result<Response<HeartbeatResponse>, Status> {
//         Ok(heartbeat::heartbeat(self, request).await?)
//     }

//     async fn request_vote(&self, request: Request<VoteRequest>) -> Result<Response<VoteResponse>, Status> {
//         Ok(request_vote::request_vote(self, request).await?)
//     }
// }

#[tonic::async_trait]
impl<T> RaftRpc for Raft<T> 
where
    T: 'static + Send + Clone + Serialize + DeserializeOwned + From<Vec<u8>> + std::fmt::Debug
{
    async fn append_entries(&self, request: Request<AppendEntriesRequest>) -> Result<Response<AppendEntriesResponse>, Status> {
        Ok(append_entries::append_entries(self, request).await?)
    }
    async fn heartbeat(&self, request: Request<HeartbeatRequest>) -> Result<Response<HeartbeatResponse>, Status> {
        Ok(heartbeat::heartbeat(self, request).await?)
    }
    async fn request_vote(&self, request: Request<VoteRequest>) -> Result<Response<VoteResponse>, Status> {
        Ok(request_vote::request_vote(self, request).await?)
    }
}

#[cfg(test)]
pub mod tests {
    use proto::{raft_rpc_server::RaftRpc};
    use state_machine::impls::key_value_store::KeyValueStore;
    use tonic::Request;

    use super::*;
    use crate::{utils::test_utils::set_up_logging};
    use crate::node::{Raft, Follower, Leader, Candidate, Log};
    use log::{info, trace, debug};


    #[tokio::test]
    async fn test_append_entries_basic() {
        set_up_logging();
        let node: Raft<Vec<u8>> = Raft::default();
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
        let node: Raft<Vec<u8>> = Raft::default();
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
        let node: Raft<Vec<u8>> = Raft::default();
        let request = Request::new(VoteRequest::default());
        let response = node.request_vote(request).await.unwrap();
        let response = response.into_inner();
        assert_eq!(response.term, 0);
        assert_eq!(response.vote_granted, true);
        info!("response: {response:?}");
    }
}
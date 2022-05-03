use serde::{Serialize, de::DeserializeOwned};
use tonic::{Request, Response, Status};
use crate::{node::{Node}, storage::state::persistent::LogEntry};
use proto::raft::{
    HeartbeatRequest,
    HeartbeatResponse,
    VoteRequest,
    VoteResponse,
    AppendEntriesRequest,
    AppendEntriesResponse,
    LogEntry as ProtoLogEntry,
    raft_server::Raft
};
mod request_vote;
mod append_entries;
mod heartbeat;


// impl<L> From<ProtoLogEntry> for LogEntry<L> 
// where
//     L: From<Vec<u8>> 
// {
//     fn from(entry: ProtoLogEntry) -> Self {
//         Self {
//             term: entry.term as usize,
//             command: entry.command.into()
//         }
//     }
// }

// impl<L> From<LogEntry<L>> for ProtoLogEntry 
// where
//     L: Into<Vec<u8>> + From<Vec<u8>>,
// {
//     fn from(entry: LogEntry<L>) -> Self {
//         Self {
//             term: entry.term as u64,
//             command: entry.command.into(),
//         }
//     }
// }


#[tonic::async_trait]
impl<L> Raft for Node<L>
where
    L: LogEntry + 'static + Send + Clone + Serialize + DeserializeOwned
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
    use crate::{utils::test_utils::set_up_logging, storage::state::persistent::LogEntryImpl};
    use crate::node::Node;
    use log::{info, trace, debug};


    #[tokio::test]
    async fn test_append_entries_basic() {
        set_up_logging();
        let node: Node<LogEntryImpl<String>> = Node::default();
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
        let mut node: Node<LogEntryImpl<String>> = Node::default();
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
        let node: Node<LogEntryImpl<String>> = Node::default();
        let request = Request::new(VoteRequest::default());
        let response = node.request_vote(request).await.unwrap();
        let response = response.into_inner();
        assert_eq!(response.term, 0);
        assert_eq!(response.vote_granted, true);
        info!("response: {response:?}");
    }
}
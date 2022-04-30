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
impl Raft for Node {
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
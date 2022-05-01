use proto::raft::{
    VoteRequest, 
    VoteResponse,
};
use tonic::{Request, Response, Status};
use crate::node::Node;

pub async fn request_vote(node: &Node, request: Request<VoteRequest>) -> Result<Response<VoteResponse>, Status> {
    println!("Got a request: {:?}", request);

    let reply = VoteResponse {
        term: 0,
        vote_granted: false
    };

    Ok(Response::new(reply))
}
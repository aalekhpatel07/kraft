use proto::raft::{
    AppendEntriesRequest, 
    AppendEntriesResponse,
};
use tonic::{Request, Response, Status};
use crate::node::Node;


pub async fn append_entries(node: &Node, request: Request<AppendEntriesRequest>) -> Result<Response<AppendEntriesResponse>, Status> {
    println!("Got a request: {:?}", request);

    let reply = AppendEntriesResponse {
        term: 0,
        success: false
    };

    Ok(Response::new(reply))
}
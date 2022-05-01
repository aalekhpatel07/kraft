use proto::raft::{
    AppendEntriesRequest, 
    AppendEntriesResponse,
};
use tonic::{Request, Response, Status};
use crate::node::Node;
use log::{info, trace, debug};


pub async fn append_entries<L: Clone>(node: &Node<L>, request: Request<AppendEntriesRequest>) -> Result<Response<AppendEntriesResponse>, Status> {
    info!("Got a request: {:?}", request);

    let reply = AppendEntriesResponse {
        term: 0,
        success: false
    };
    let response = Response::new(reply);
    info!("About to respond with: {response:?}");
    Ok(response)
}
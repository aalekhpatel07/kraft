use proto::raft::{
    HeartbeatRequest, 
    HeartbeatResponse,
};
use tonic::{Request, Response, Status};
use crate::node::Node;

pub async fn heartbeat(node: &Node, request: Request<HeartbeatRequest>) -> Result<Response<HeartbeatResponse>, Status> {
    println!("Got a request: {:?}", request);

    let reply = HeartbeatResponse {
        term: 0,
        success: false
    };

    Ok(Response::new(reply))
}
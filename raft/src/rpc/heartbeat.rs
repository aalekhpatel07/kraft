use proto::raft::{
    HeartbeatRequest, 
    HeartbeatResponse,
};
use tonic::{Request, Response, Status};
use crate::node::Node;
use log::{info, trace, debug};


pub async fn heartbeat<L: Clone>(node: &Node<L>, request: Request<HeartbeatRequest>) -> Result<Response<HeartbeatResponse>, Status> {
    debug!("Got a request: {:?}", request);

    let reply = HeartbeatResponse {
        term: 0,
        success: false
    };

    Ok(Response::new(reply))
}
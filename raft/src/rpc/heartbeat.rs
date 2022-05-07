use proto::raft::{
    HeartbeatRequest, 
    HeartbeatResponse,
};
use tonic::{Request, Response, Status};
use crate::{node::RaftNode, storage::state::persistent::Log};
use log::{info, trace, debug};


pub async fn heartbeat<S>(node: &RaftNode<S>, request: Request<HeartbeatRequest>) -> Result<Response<HeartbeatResponse>, Status> 
where
    S: state_machine::StateMachine,
    S::MutationCommand: Clone
{

    debug!("Got a request: {:?}", request);

    let reply = HeartbeatResponse {
        term: 0,
        success: false
    };

    Ok(Response::new(reply))
}
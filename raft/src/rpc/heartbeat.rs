use proto::{
    HeartbeatRequest, 
    HeartbeatResponse,
};
use tonic::{Request, Response, Status};
use log::{info, trace, debug};
use crate::node::{Raft};


pub async fn heartbeat<T>(node: &Raft<T>, request: Request<HeartbeatRequest>) -> Result<Response<HeartbeatResponse>, Status> 
{
    trace!("Got a Heartbeat request: {:?}", request);

    let mut response = HeartbeatResponse { 
        term: node.persistent_data.lock().unwrap().current_term as u64, 
        success: false,
    };
    Ok(Response::new(response))
}
// pub async fn heartbeat<S, T>(node: &Raft<S, T>, request: Request<HeartbeatRequest>) -> Result<Response<HeartbeatResponse>, Status> 
// where
//     S: state_machine::StateMachine,
//     S::MutationCommand: Clone + std::fmt::Debug + From<Vec<u8>>
// {

//     debug!("Got a request: {:?}", request);

//     let reply = HeartbeatResponse {
//         term: 0,
//         success: false
//     };

//     Ok(Response::new(reply))
// }
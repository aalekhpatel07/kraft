use proto::raft::{
    HeartbeatRequest, 
    HeartbeatResponse,
};
use tonic::{Request, Response, Status};

pub async fn heartbeat<T>(state: &T, request: Request<HeartbeatRequest>) -> Result<Response<HeartbeatResponse>, Status> {
    println!("Got a request: {:?}", request);

    let reply = HeartbeatResponse {
        term: 0,
        success: false
    };

    Ok(Response::new(reply))
}

// use crate::network::node::Server;
// use crate::rpc::*;
// use std::sync::Arc;
// use tokio::sync::Mutex;

// /// Invoked by leader to tell followers of its existence.
// #[derive(Copy, Clone, Default, Debug, Serialize, Deserialize)]
// pub struct HeartbeatRequest {
//     /// The leader's term.
//     pub term: usize,
//     /// Used by followers to redirect clients.
//     pub leader_id: usize,
//     /// The index of the log entry immediately preceding new ones.
//     pub previous_log_index: usize,
//     /// The term of `previous_log_index` entry.
//     pub previous_log_term: usize,
//     /// The leader's commit index.
//     pub leader_commit_index: usize,
// }

// /// The response sent by the followers to the leader when a heartbeat
// /// is acknowledged.
// #[derive(Clone, Default, Debug, Serialize, Deserialize)]
// pub struct HeartbeatResponse {
//     /// The current term for leader to update itself.
//     pub term: usize,
//     /// True if follower contained entry matching `previous_log_index`
//     /// and `previous_log_term`.
//     pub success: bool,
// }

// impl HeartbeatRequest {
//     pub fn new(
//         term: usize,
//         leader_id: usize,
//         previous_log_index: usize,
//         previous_log_term: usize,
//         leader_commit_index: usize,
//     ) -> Self {
//         Self {
//             term,
//             leader_id,
//             previous_log_index,
//             previous_log_term,
//             leader_commit_index,
//         }
//     }
// }

// pub fn create(_server: Arc<Mutex<Server>>) -> HeartbeatRequest {
//     HeartbeatRequest::default()
// }

// pub fn process(_server: Arc<Mutex<Server>>, _request: HeartbeatRequest) -> HeartbeatResponse {
//     HeartbeatResponse::default()
// }

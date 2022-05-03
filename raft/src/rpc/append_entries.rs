use color_eyre::owo_colors::colors::css::Wheat;
use proto::raft::{
    AppendEntriesRequest, 
    AppendEntriesResponse,
};
use tonic::{Request, Response, Status};
use crate::{node::Node, storage::state::{volatile::{VolatileState, NonLeaderState}, persistent::LogEntry}};
use crate::storage::state::volatile;
use log::{info, trace, debug, error};
// use crate::node::LogEntry;
use serde::Serialize;
use serde::de::DeserializeOwned;


pub async fn append_entries<L>(node: &Node<L>, request: Request<AppendEntriesRequest>) -> Result<Response<AppendEntriesResponse>, Status> 
where
    L: LogEntry + Clone + Serialize + DeserializeOwned
{
    info!("Got a request: {:?}", request);
    
    let mut persistent_state = node.persistent_state.lock().unwrap();

    let mut append_entries_request = request.into_inner();
    let mut response = AppendEntriesResponse { term: persistent_state.current_term as u64, success: false };

    // Reply false if term < currentTerm.
    if append_entries_request.term < persistent_state.current_term as u64 {
        return Ok(Response::new(response));
    }

    // Reply false if log doesn't contain an entry at prevLogIndex whose term matches prevLogTerm.

    if (persistent_state.log.len() as u64) <= append_entries_request.prev_log_index {
        return Ok(Response::new(response));
    } else {

        let log_entry_at_prev_log_index = persistent_state.log.get(append_entries_request.prev_log_index as usize).unwrap();
        if (log_entry_at_prev_log_index.term() as u64) != append_entries_request.prev_log_term {
            return Ok(Response::new(response));
        }
    }

    response.success = true;

    // If an existing entry conflicts with a new one (same index but different terms), delete
    // the existing entry and all that follow it.

    let mut first_faulty_log_entry_index: Option<usize> = None;

    for (index, (stored_entry, leader_entry)) in
        persistent_state
        .log
        .iter_mut()
        .zip(append_entries_request.entries.iter())
        .enumerate()
    {
        if (stored_entry.term() as u64) != leader_entry.term {
            first_faulty_log_entry_index = Some(index);
            break;
        }
    }

    // The logs of the leader and follower are the same up until this faulty index
    // but they differ starting here.
    // Remove this suffix in the follower's log.
    if let Some(first_faulty_index) = first_faulty_log_entry_index {
        persistent_state.log.drain(first_faulty_index..);
    }

    // Either the follower exhausted its log while trying to find a faulty entry,
    // or some first faulty entry was found and we removed all stored entries following
    // the first faulty entry above.

    // In the former case, the follower's log is extended by the remaining of the leader's log,
    // if the leader's log is longer than the follower's.

    // In the latter case, the follower's log is extended by those entries in the leader's log
    // that come after the first faulty index.
    let starting_index_to_append_entries = 
        first_faulty_log_entry_index
        .unwrap_or_else(
            || {
            persistent_state.log.len()
        }
    );

    // Append any new entries not already in the log.
    // let entries_to_append =
    // append_entries_request.entries
    // .iter()
    // .skip(starting_index_to_append_entries) 

    // // It is possible that the leader doesn't have long enough log. In which
    // // case, the map would be empty.
    // .map(|entry| {
    //     LogEntry {
    //         term: entry.term as usize,
    //         command: entry.command.clone().into()
    //     }
    // });
    // persistent_state.log.extend(entries_to_append);



    // If leaderCommit > commitIndex, set commitIndex = min(leaderCommit, index of last new entry).
    match &mut *node.volatile_state.lock().unwrap() {
        VolatileState::NonLeader(state) => {
            if append_entries_request.leader_commit_index > (state.commit_index as u64) {
                state.commit_index = (
                    append_entries_request.leader_commit_index as usize
                )
                .min(persistent_state.log.len());
            }
        },
        // Check that we are not a leader when responding to AppendEntriesRPC.
        VolatileState::Leader(leader_state) => {
            error!("Ended up in leader state while receiving AppendEntriesRPC: {leader_state:?}");
            unimplemented!("When responding to AppendEntries, the node can only be a Follower or Candidate, not Leader.");
        }
    };

    info!("About to respond with: {response:?}");
    // drop the guard before saving state.
    drop(persistent_state);

    node.save().expect("Couldn't save persistent state.");

    Ok(Response::new(response))
}


#[cfg(test)]
pub mod tests {
    use std::sync::{Arc, Mutex};
    use std::time::Duration;

    use log::info;
    use tonic::transport::Server;
    use tonic::{Request, Response};

    use crate::{node::{Node, NodeType, NodeMetadata}, utils::test_utils::set_up_logging, storage::state::persistent::LogEntry};
    // use crate::storage::state::persistent::State;
    // use crate::storage::state::volatile::*;
    use proto::raft::{raft_client::RaftClient, raft_server::RaftServer};
    use proto::raft::{LogEntry as ProtoLogEntry};
    use proto::raft::*;



    // pub fn create_log_entries(term_sequence: &[usize]) -> Vec<LogEntry<Vec<u8>>> {
    //     term_sequence
    //     .iter()
    //     .map(|term| {
    //         LogEntry { term: *term, command: vec![] }
    //     }).collect::<Vec<LogEntry<Vec<u8>>>>()
    // }
    
    // #[test]
    // pub fn test_create_log_entries() {
    //     set_up_logging();
    //     let terms: Vec<usize> = vec![1, 1, 1, 4, 4, 5, 5, 6, 6, 6];
    //     // let entries = create_log_entries(&terms);
    //     // info!("entries: {entries:?}");
    // }

    // fn create_leader(term_sequence: &[usize], addr: &str) -> Node<Vec<u8>> {

    //     let state = State {
    //         current_term: 0,
    //         voted_for: None,
    //         log: create_log_entries(term_sequence)
    //     };
    //     let volatile_state = VolatileState::Leader( LeaderState::default() );

    //     Node {
    //         node_type: NodeType::Leader,
    //         meta: NodeMetadata { id: 0, addr: addr.to_owned(), ..Default::default() },
    //         persistent_state: Arc::new(Mutex::new(state)),
    //         volatile_state: Arc::new(Mutex::new(volatile_state))
    //     }
    // }

    // fn create_follower(term_sequence: &[usize], addr: &str) -> Node<Vec<u8>> {

    //     let state = State {
    //         current_term: 0,
    //         voted_for: None,
    //         log: create_log_entries(term_sequence)
    //     };
    //     let volatile_state = VolatileState::NonLeader( NonLeaderState::default() );

    //     Node {
    //         node_type: NodeType::Follower,
    //         meta: NodeMetadata { id: 0, addr: addr.to_owned(), ..Default::default() },
    //         persistent_state: Arc::new(Mutex::new(state)),
    //         volatile_state: Arc::new(Mutex::new(volatile_state))
    //     }
    // }

    // #[tokio::test]
    // async fn test_append_entries_case_a() -> Result<(), Box<dyn std::error::Error>> {
    //     set_up_logging();
    //     let leader_terms: Vec<usize> = vec![1, 1, 1, 4, 4, 5, 5, 6, 6, 6];
    //     let follower_terms: Vec<usize> = vec![1, 1, 1, 4, 4, 5, 5, 6, 6];

    //     let follower_addr = "[::1]:50052";
    //     let follower = create_follower(&follower_terms, follower_addr.clone());
    //     let leader = create_leader(&leader_terms, "[::1]:50051");
        
    //     let server_handle = tokio::spawn( async move {
    //         Server::builder()
    //             .add_service(RaftServer::new(follower))
    //             .serve(follower_addr.parse().unwrap())
    //             .await
    //     });

    //     // let client_handle = tokio::spawn( async move {
    //     //     info!("Trying to connect to follower: {follower_addr:?}");
    //     //     let mut client = RaftClient::connect(
    //     //         format!("http://{}", follower_addr)
    //     //     ).await.unwrap();

    //     //     info!("Connected to follower: {follower_addr:?}");

    //     //     info!("Sending append entries to follower: {follower_addr:?}");
    //     //     let request = Request::new(
    //     //         AppendEntriesRequest {
    //     //             term: leader.persistent_state.lock().unwrap().current_term as u64,
    //     //             prev_log_index: 0,
    //     //             prev_log_term: 0,
    //     //             entries: leader.proto_log(),
    //     //             leader_commit_index: leader.leader_commit_index().unwrap() as u64,
    //     //             leader_id: leader.meta.id as u64
    //     //         }
    //     //     );

    //     //     let response = client.append_entries(request).await.unwrap();
    //     //     info!("Response received: {response:?}");

    //     // });

    //     // tokio::join!(server_handle, client_handle).0.expect("What A").expect(" What B");

    //     // tokio::join!().await;
    //     // RaftClient::connect(follower_addr);
    //     Ok(())
        
    // }
}
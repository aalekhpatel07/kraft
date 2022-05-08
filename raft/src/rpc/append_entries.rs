use color_eyre::owo_colors::colors::css::Wheat;
use proto::raft::{
    AppendEntriesRequest, 
    AppendEntriesResponse,
};
use tonic::{Request, Response, Status};
use crate::{node::{RaftNode}, storage::state::{volatile::{VolatileState, NonLeaderState}, persistent::Log}};
use crate::storage::state::volatile;
use log::{info, trace, debug, error};
// use crate::node::LogEntry;
use serde::Serialize;
use serde::de::DeserializeOwned;


pub async fn append_entries<S>(node: &RaftNode<S>, request: Request<AppendEntriesRequest>) -> Result<Response<AppendEntriesResponse>, Status> 
where
    S: state_machine::StateMachine,
    S::MutationCommand: Clone + Serialize + DeserializeOwned
{
    
    trace!("Got a request: {request:?}");

    let AppendEntriesRequest { 
        term, 
        leader_id, 
        prev_log_index, 
        prev_log_term, 
        entries, 
        leader_commit_index
    } = request.into_inner();


    let mut persistent_state = node.persistent_state.lock().expect("Could not lock persistent state.");
    let mut volatile_state = node.volatile_state.lock().expect("Could not lock volatile state.");

    let mut response = AppendEntriesResponse { 
        term: persistent_state.current_term as u64, 
        success: false
    };

    // // Reply false if term < currentTerm.
    // if append_entries_request.term < persistent_state.current_term as u64 {
    //     return Ok(Response::new(response));
    // }

    // // Reply false if log doesn't contain an entry at prevLogIndex whose term matches prevLogTerm.

    // if (persistent_state.log.len() as u64) <= append_entries_request.prev_log_index {
    //     return Ok(Response::new(response));
    // } else {

    //     let log_entry_at_prev_log_index = persistent_state.log.get(append_entries_request.prev_log_index as usize).unwrap();
    //     if (log_entry_at_prev_log_index.term() as u64) != append_entries_request.prev_log_term {
    //         return Ok(Response::new(response));
    //     }
    // }

    // response.success = true;

    // // If an existing entry conflicts with a new one (same index but different terms), delete
    // // the existing entry and all that follow it.

    // let mut first_faulty_log_entry_index: Option<usize> = None;

    // for (index, (stored_entry, leader_entry)) in
    //     persistent_state
    //     .log
    //     .iter_mut()
    //     .zip(append_entries_request.entries.iter())
    //     .enumerate()
    // {
    //     if (stored_entry.term() as u64) != leader_entry.term {
    //         first_faulty_log_entry_index = Some(index);
    //         break;
    //     }
    // }

    // // The logs of the leader and follower are the same up until this faulty index
    // // but they differ starting here.
    // // Remove this suffix in the follower's log.
    // if let Some(first_faulty_index) = first_faulty_log_entry_index {
    //     persistent_state.log.drain(first_faulty_index..);
    // }

    // // Either the follower exhausted its log while trying to find a faulty entry,
    // // or some first faulty entry was found and we removed all stored entries following
    // // the first faulty entry above.

    // // In the former case, the follower's log is extended by the remaining of the leader's log,
    // // if the leader's log is longer than the follower's.

    // // In the latter case, the follower's log is extended by those entries in the leader's log
    // // that come after the first faulty index.
    // let starting_index_to_append_entries = 
    //     first_faulty_log_entry_index
    //     .unwrap_or_else(
    //         || {
    //         persistent_state.log.len()
    //     }
    // );

    // // Append any new entries not already in the log.
    // // let entries_to_append =
    // // append_entries_request.entries
    // // .iter()
    // // .skip(starting_index_to_append_entries) 

    // // // It is possible that the leader doesn't have long enough log. In which
    // // // case, the map would be empty.
    // // .map(|entry| {
    // //     LogEntry {
    // //         term: entry.term as usize,
    // //         command: entry.command.clone().into()
    // //     }
    // // });
    // // persistent_state.log.extend(entries_to_append);



    // // If leaderCommit > commitIndex, set commitIndex = min(leaderCommit, index of last new entry).
    // match &mut *node.volatile_state.lock().unwrap() {
    //     VolatileState::NonLeader(state) => {
    //         if append_entries_request.leader_commit_index > (state.commit_index as u64) {
    //             state.commit_index = (
    //                 append_entries_request.leader_commit_index as usize
    //             )
    //             .min(persistent_state.log.len());
    //         }
    //     },
    //     // Check that we are not a leader when responding to AppendEntriesRPC.
    //     VolatileState::Leader(leader_state) => {
    //         error!("Ended up in leader state while receiving AppendEntriesRPC: {leader_state:?}");
    //         unimplemented!("When responding to AppendEntries, the node can only be a Follower or Candidate, not Leader.");
    //     }
    // };

    // info!("About to respond with: {response:?}");
    // // drop the guard before saving state.
    // drop(persistent_state);

    // node.save().expect("Couldn't save persistent state.");

    Ok(Response::new(response))
}


#[cfg(test)]
pub mod tests {
    use std::sync::{Arc, Mutex};
    use std::time::Duration;

    use log::info;
    use serde_json::Value;
    use tonic::transport::Server;
    use tonic::{Request, Response};

    use crate::storage::state::persistent::State;
    use crate::storage::state::volatile::{VolatileState, LeaderState, NonLeaderState};
    use crate::{
        node::{RaftNode, NodeType, NodeMetadata}, 
        utils::test_utils::set_up_logging, 
        storage::state::persistent::Log
    };
    use crate::rpc::RaftRpc;
    use proto::raft::{
        AppendEntriesRequest, 
        AppendEntriesResponse
    };
    // use crate::storage::state::persistent::State;
    // use crate::storage::state::volatile::*;
    use proto::raft::{raft_rpc_client::RaftRpcClient, raft_rpc_server::RaftRpcServer};
    use proto::raft::{LogEntry as ProtoLogEntry};
    use proto::raft::*;
    use state_machine::impls::key_value_store::*;
    use crate::node::Int;
    use std::path::PathBuf;

    use rand::{distributions::Alphanumeric, Rng};
    use super::append_entries;


    pub fn create_request<T: Into<Vec<u8>>>
    (
        term: Int, 
        leader_id: Int, 
        prev_log_index: Int, 
        prev_log_term: Int,
        entries: Vec<Log<T>>,
        leader_commit_index: Int
    ) -> Request<AppendEntriesRequest> 
    {
        let entries: Vec<LogEntry> = entries.into_iter().map(|entry| entry.into()).collect();

        let append_entries_request = AppendEntriesRequest {
            term, leader_id, prev_log_index, prev_log_term, entries, leader_commit_index
        };
        Request::new(append_entries_request)
    }

    pub fn create_response(term: Int, success: bool) -> AppendEntriesResponse {
        AppendEntriesResponse {
            term,
            success
        }
    }


    macro_rules! append_entries_test {
        (
            $(#[$meta:meta])*
            $func_name:ident, 
            $initial_persistent_state:expr, 
            $initial_volatile_state:expr, 
            $request:expr, 
            $response:expr, 
            $final_persistent_state:expr,
            $final_volatile_state:expr
        ) => {
                $(#[$meta])*
                #[tokio::test]
                pub async fn $func_name() {
                    set_up_logging();

                    // This node receives the RPC.
                    let mut receiver: RaftNode<KeyValueStore<String, String>> = RaftNode::default();
                    
                    let mut log_file_path = PathBuf::from("/tmp");
                    let log_file_base = rand::thread_rng().sample_iter(&Alphanumeric).take(15).map(char::from).collect::<String>();
                    log_file_path.push(format!("{}.raft", log_file_base));
                    receiver.meta.log_file = log_file_path.to_str().expect("Is invalid unicode.").to_owned();

                    // Set the initial states as given.
                    receiver.persistent_state = std::sync::Arc::new(std::sync::Mutex::new($initial_persistent_state));
                    receiver.volatile_state = std::sync::Arc::new(std::sync::Mutex::new($initial_volatile_state));

                    // Create a AppendEntriesRequest from the given argument.
                    let request = create_request::<Vec<u8>>(
                        $request.0, 
                        $request.1, 
                        $request.2, 
                        $request.3, 
                        $request.4, 
                        $request.5
                    );

                    // Create an expected response from the given response.
                    let expected_response = create_response($response.0, $response.1);

                    // Make the AppendEntriesRPC and get a response.
                    let observed_response = append_entries(&receiver, request).await.expect("AppendEntriesRPC failed to await.");

                    // Assert the observed response is the same as expected.
                    assert_eq!(
                        observed_response.into_inner(), 
                        expected_response,
                        "AppendEntriesResponse does not match up."
                    );

                    // Assert the final persistent state is the same as expected.
                    assert_eq!(
                        &*receiver.persistent_state.lock().expect("Couldn't lock persistent state."),
                        &$final_persistent_state,
                        "Persistent state does not match up."
                    );

                    // Check that the state change is persisted on stable storage.
                    assert_eq!(
                        &receiver.load_state().expect("Couldn't load state."),
                        &$final_persistent_state
                    );

                    // Assert the final volatile state is the same as expected.
                    assert_eq!(
                        &*receiver.volatile_state.lock().expect("Couldn't lock volatile state."),
                        &$final_volatile_state,
                        "Volatile state does not match up."
                    );
            }
        };
    }

    append_entries_test!(
        /// Test that append entries works fine.
        initial,
        State { current_term: 0, voted_for: None, log: vec![] },
        VolatileState::NonLeader(NonLeaderState { commit_index: 0, last_applied: 0}), 
        (0, 1, 0, 0, vec![(0u64, Vec::<u8>::new())], 0), 
        (0, false), 
        State { current_term: 0, voted_for: None, log: vec![] },
        VolatileState::NonLeader(NonLeaderState { commit_index: 0, last_applied: 0})
    );

    // pub fn create_log_entries(term_sequence: &[u64]) -> Vec<Log<Vec<u8>>> {
    //     term_sequence
    //     .iter()
    //     .map(|term| {
    //         (*term, Vec::<u8>::new())
    //     }).collect::<Vec<Log<Vec<u8>>>>()
    // }
    
    // #[test]
    // pub fn test_create_log_entries() {
    //     set_up_logging();
    //     let terms: Vec<u64> = vec![1, 1, 1, 4, 4, 5, 5, 6, 6, 6];
    //     let entries = create_log_entries(&terms);
    //     info!("entries: {entries:?}");
    // }

    // fn create_leader(term_sequence: &[u64], addr: &str) -> Node<Vec<u8>> {

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


    // fn create_key_value_store() -> KeyValueStore<String, serde_json::Value>{
    //     KeyValueStore::<String, serde_json::Value>::new()
    // }

}
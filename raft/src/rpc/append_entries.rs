use color_eyre::owo_colors::colors::css::Wheat;
use proto::{
    AppendEntriesRequest, 
    AppendEntriesResponse, LogEntry,
};
use tonic::{Request, Response, Status};
use crate::node::{Raft, Leader};
use log::{info, trace, debug, error, warn};
// use crate::node::LogEntry;
use serde::{Serialize, Deserialize};
use serde::de::DeserializeOwned;


#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct LogEntryBody {
    pub term: u64,
    pub command: Vec<u8>
}


/// Invoked by leader to replicate log entries.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AppendEntriesRequestBody {
    /// The election term that the leader node is in.
    pub term: u64,
    /// The ID of the leader so that follower can redirect clients.
    pub leader_id: u64,
    /// The index of the log entry immediately preceding new ones.
    pub prev_log_index: u64,
    /// The term of the prev_log_index.
    pub prev_log_term: u64,
    /// The log entries to store.
    pub entries: Vec<LogEntryBody>,
    /// The leader's commit index.
    pub leader_commit_index: u64,
}


/// Returned by candidates and followers when a leader requests to AppendEntries.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct AppendEntriesResponseBody {
    /// The current term for leader to update itself.
    pub term: u64,
    /// True, if a follower contained an entry matching prev_log_index and prev_log_term
    pub success: bool,
    /// The term for the earliest conflicting entry, if unsuccessful.
    pub conflicting_term: u64,
    /// The index of the first entry in the conflicting term, if unsuccessful.
    pub conflicting_term_first_index: u64,
}


impl From<&LogEntry> for LogEntryBody {
    fn from(entry: &LogEntry) -> Self {
        LogEntryBody {
            term: entry.term,
            command: entry.command.clone()
        }
    }
}

impl From<&AppendEntriesRequest> for AppendEntriesRequestBody {
    fn from(request: &AppendEntriesRequest) -> Self {
        AppendEntriesRequestBody {
            term: request.term,
            leader_id: request.leader_id,
            prev_log_index: request.prev_log_index,
            prev_log_term: request.prev_log_term,
            entries: request.entries.iter().map(|entry| entry.into()).collect(),
            leader_commit_index: request.leader_commit_index
        }
    }
}

impl From<&AppendEntriesResponse> for AppendEntriesResponseBody {
    fn from(response: &AppendEntriesResponse) -> Self {
        AppendEntriesResponseBody {
            term: response.term,
            success: response.success,
            conflicting_term: response.conflicting_term,
            conflicting_term_first_index: response.conflicting_term_first_index
        }
    }
}


pub async fn append_entries<T>(node: &Raft<T>, request: AppendEntriesRequest) -> Result<Response<AppendEntriesResponse>, Status> 
where
    T: Clone + DeserializeOwned + Serialize + From<Vec<u8>>
{
    trace!("Got an Append Entries request: {:?}", request);

    let mut persistent_data = node.persistent_data.lock().unwrap();
    let mut volatile_data = node.volatile_data.lock().unwrap();

    // Reply false if term < currentTerm.
    if request.term < persistent_data.current_term as u64 {
        let response = AppendEntriesResponse {
            term: persistent_data.current_term,
            success: false, 
            conflicting_term: 0, 
            conflicting_term_first_index: 0
        };
        drop(persistent_data);
        node.save().unwrap();
        return Ok(Response::new(response));
    }


    // Reply false if log doesn't contain an entry at prevLogIndex whose term matches prevLogTerm.
    if (persistent_data.log.len() as u64) <= request.prev_log_index {
        warn!("The log of the persistent data is not long enough.");
        let response = AppendEntriesResponse {
            term: persistent_data.current_term,
            success: false, 
            conflicting_term: 0, 
            conflicting_term_first_index: 0
        };

        drop(persistent_data);
        node.save().unwrap();
        return Ok(Response::new(response))
    }
    
    let (prev_log_term, _) = persistent_data.log.get(request.prev_log_index as usize).unwrap();

    // Log has an entry at prevLogIndex but is a different term than requested.
    if *prev_log_term != request.prev_log_term {
        warn!("The term for the entry at prevLogIndex does not match the prevLogTerm specified in the request.");

        let conflicting_term = *prev_log_term;
        let conflicting_term_first_index = persistent_data.log.partition_point(|(term, _)| *term <= *prev_log_term) + 1;

        trace!("The first index for conflicting term ({conflicting_term}) is : ({conflicting_term_first_index}).");
        
        let total_entries = persistent_data.log.len();
        let num_entries_to_delete = persistent_data.log.len() - conflicting_term_first_index;
        trace!("Draining ({num_entries_to_delete}) entries out of ({total_entries}) from the log, starting at index: ({conflicting_term_first_index}).");
        
        persistent_data.log.drain(conflicting_term_first_index..);

        // let response = AppendEntriesResponse {
        //     term: persistent_data.current_term,
        //     success: false, 
        //     conflicting_term, 
        //     conflicting_term_first_index: conflicting_term_first_index as u64
        // };

        // drop(persistent_data);
        // node.save().unwrap();
        // return Ok(Response::new(response))
    }


    // Append any new entries not already in the log.

    // TODO: This could use some more massaging as I'm not yet
    // 100% clear how the distinction is made.
    let mut all_after: Option<usize> = None;

    for (idx, entry) in request.entries.iter().enumerate() {
        // check at index: prevLogIndex + idx
        if let Some((stored_term, _)) = persistent_data.log.get(request.prev_log_index as usize + idx) {
            // an entry exists.
            if *stored_term != entry.term {
                all_after = Some(idx);
                break;
            }
        } else {
            all_after = Some(idx);
            break;
        }
    }

    let entries_as_tuples: Vec<(u64, T)> = request.entries.iter().map(|entry| (entry.term, entry.command.clone().into())).collect();
    let mut index_of_last_new_entry: Option<usize> = None;

    if let Some(idx) = all_after {
        let total_requested_entries = entries_as_tuples.len();
        let num_entries_to_append = total_requested_entries - idx;

        trace!("Found ({num_entries_to_append}) new entries to append to the log.");
        persistent_data.log.extend_from_slice(&entries_as_tuples[idx..]);
        index_of_last_new_entry = Some(persistent_data.log.len() - 1);
        trace!("Index of last new entry: {index_of_last_new_entry:#?}.");
    } else {
        trace!("No new entries to append to log.");
    }

    if request.leader_commit_index > volatile_data.commit_index {
        if let Some(idx) = index_of_last_new_entry {
            volatile_data.commit_index = request.leader_commit_index.min(idx as u64);
        } else {
            volatile_data.commit_index = request.leader_commit_index;
        }
    }

    let response = AppendEntriesResponse {
        term: persistent_data.current_term,
        success: true, 
        conflicting_term: 0, 
        conflicting_term_first_index: 0
    };

    drop(persistent_data);
    node.save().unwrap();
    return Ok(Response::new(response))

    // Err(Status::internal("Unhandled AppendEntries Implementation."))
}


#[cfg(test)]
pub mod tests {
    use std::sync::{Arc, Mutex};
    use std::time::Duration;

    use log::{info, debug};
    use serde_json::Value;
    use tonic::transport::Server;
    use tonic::{Request, Response};

    use crate::node::{VolatileState, Log, PersistentState};
    use crate::{
        node::{Raft, NodeMetadata}, 
        utils::test_utils::set_up_logging, 
    };
    use crate::rpc::RaftRpc;
    use proto::{
        AppendEntriesRequest, 
        AppendEntriesResponse
    };
    // use crate::storage::state::persistent::State;
    // use crate::storage::state::volatile::*;
    use proto::{raft_rpc_client::RaftRpcClient, raft_rpc_server::RaftRpcServer};
    use proto::{LogEntry as ProtoLogEntry};
    use proto::*;
    use state_machine::impls::key_value_store::*;
    use crate::node::Int;
    use std::path::PathBuf;

    use rand::{distributions::Alphanumeric, Rng};
    use super::append_entries;

    /// Given a term, leader_id, prev_log_index, prev_log_term,
    /// entries, and leader_commit_index, create an AppendEntriesRequest.
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
        let entries: Vec<LogEntry> = entries.into_iter().map(Into::into).collect();

        let append_entries_request = AppendEntriesRequest {
            term, leader_id, prev_log_index, prev_log_term, entries, leader_commit_index
        };
        Request::new(append_entries_request)
    }

    pub fn create_response(term: Int, success: bool) -> AppendEntriesResponse {
        AppendEntriesResponse {
            term,
            success,
            conflicting_term: 0,
            conflicting_term_first_index: 0
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
                #[ignore]
                #[tokio::test]
                pub async fn $func_name() {
                    set_up_logging();

                    // This node receives the RPC.
                    let mut receiver: Raft<Vec<u8>> = Raft::default();
                    
                    let mut log_file_path = PathBuf::from("/tmp");
                    let log_file_base = rand::thread_rng().sample_iter(&Alphanumeric).take(15).map(char::from).collect::<String>();
                    log_file_path.push(format!("{}.raft", log_file_base));

                    receiver.meta.log_file = log_file_path.to_str().expect("Is invalid unicode.").to_owned();

                    // Set the initial states as given.
                    receiver.persistent_data = std::sync::Arc::new(std::sync::Mutex::new($initial_persistent_state));
                    receiver.volatile_data = std::sync::Arc::new(std::sync::Mutex::new($initial_volatile_state));

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
                    let observed_response = append_entries(&receiver, request.into_inner()).await.expect("AppendEntriesRPC failed to await.");

                    // Assert the observed response is the same as expected.
                    assert_eq!(
                        observed_response.into_inner(), 
                        expected_response,
                        "AppendEntriesResponse does not match up."
                    );

                    // Assert the final persistent state is the same as expected.
                    assert_eq!(
                        &*receiver.persistent_data.lock().expect("Couldn't lock persistent state."),
                        &$final_persistent_state,
                        "Persistent state does not match up."
                    );

                    // Check that the state change is persisted on stable storage.
                    assert_eq!(
                        &receiver.load_state().expect("Couldn't load state."),
                        &$final_persistent_state,
                        "State change isn't persisted on stable storage."
                    );

                    // Assert the final volatile state is the same as expected.
                    assert_eq!(
                        &*receiver.volatile_data.lock().expect("Couldn't lock volatile state."),
                        &$final_volatile_state,
                        "Volatile state does not match up."
                    );
            }
        };
    }

    pub fn command(term: Int, s: &str) -> (Int, Vec<u8>) {
        let cmd = TryInto::<MutationCommand<String, serde_json::Value>>::try_into(s).expect("Could not parse PUT.");
        (term, cmd.into())
    }

    append_entries_test!(
        /// Test that append entries works fine.
        initial,
        PersistentState { current_term: 0, voted_for: None, log: vec![] },
        VolatileState { commit_index: 0, last_applied: 0 }, 
        (0, 1, 0, 0, vec![command(0, "PUT x 3")], 0), 
        (0, false, 0, 0), 
        PersistentState { current_term: 0, voted_for: None, log: vec![] },
        VolatileState { commit_index: 0, last_applied: 0}
    );
    append_entries_test!(
        /// Test that when a leader sends some entries to append after (term, index): (6, 10)
        /// and the follower does not have an entry (with term 6 at index 10), the follower refuses
        /// to append entries and notifies the leader of its latest term and the first index for the latest term.
        /// Since the follower does nothing else, there should be no state change (persistent or volatile).
        case_a,
        PersistentState { 
            current_term: 6, 
            voted_for: Some(1), 
            log: log_entries_from_term_sequence(&[1, 1, 1, 4, 4, 5, 5, 6, 6]).into_iter().map(|(term, cmd)| (term, cmd.into())).collect()
        },
        VolatileState { commit_index: 9, last_applied: 9 },
        (6, 1, 10, 6, log_entries_from_term_sequence(&[6, 6]), 9), 
        (6, false, 8, 6), 
        PersistentState { 
            current_term: 6, 
            voted_for: Some(1), 
            log: log_entries_from_term_sequence(&[1, 1, 1, 4, 4, 5, 5, 6, 6]).into_iter().map(|(term, cmd)| (term, cmd.into())).collect()
        },
        VolatileState { commit_index: 9, last_applied: 9 }
    );
    append_entries_test!(
        /// Test something
        case_b,
        PersistentState { 
            current_term: 4, 
            voted_for: Some(1),
            log: log_entries_from_term_sequence(&[1, 1, 1, 4]).into_iter().map(|(term, cmd)| (term, cmd.into())).collect()
        },
        VolatileState { commit_index: 3, last_applied: 3 },
        (6, 1, 10, 6, log_entries_from_term_sequence(&[6, 6]), 9),
        (4, false, 4, 4),
        PersistentState { 
            current_term: 4, 
            voted_for: Some(1), 
            log: log_entries_from_term_sequence(&[1, 1, 1, 4]).into_iter().map(|(term, cmd)| (term, cmd.into())).collect()
        },
        VolatileState { commit_index: 3, last_applied: 3 }
    );

    append_entries_test!(
        /// Test that when a leader sends some entries to append after (term, index): (6, 10)
        /// and the follower does have an entry (with term 6 at index 10), the follower appends only the remaining entries
        /// and notifies the leader of its latest term and the first index for the latest term.
        /// Since the follower appends one new entry, there should be a change in the log,
        /// as well as commit index should be moved up because leader sent a higher commit index.
        case_0,
        PersistentState { 
            current_term: 6, 
            voted_for: Some(1), 
            log: log_entries_from_term_sequence(&[1, 1, 1, 4, 4, 5, 5, 6, 6, 6, 6]).into_iter().map(|(term, cmd)| (term, cmd.into())).collect()
        },
        VolatileState { commit_index: 9, last_applied: 9 },
        (6, 1, 10, 6, log_entries_from_term_sequence(&[6, 6]), 11), 
        (6, true, 8, 6), 
        PersistentState { 
            current_term: 6, 
            voted_for: Some(1), 
            log: log_entries_from_term_sequence(&[1, 1, 1, 4, 4, 5, 5, 6, 6, 6, 6, 6]).into_iter().map(|(term, cmd)| (term, cmd.into())).collect()
        },
        VolatileState { commit_index: 11, last_applied: 9 }
    );

    // pub fn random_string(rng: &mut rand::rngs::ThreadRng, len: usize) -> String {
    //     rng
    //     .sample_iter(&Alphanumeric)
    //     .take(len)
    //     .map(char::from)
    //     .collect::<String>()
    // }

    pub fn log_entries_from_term_sequence(term_sequence: &[u64]) -> Vec<Log<Vec<u8>>> {
        // let mut rng = rand::thread_rng();

        term_sequence
        .iter()
        .map(|term| {
            // Don't even bother generating random commands
            // as we only need to specify the terms of the log entries,
            // and no actual command context is necessary in Raft.
            (*term, MutationCommand::PUT(PutCommand { key: "a", value: "b"}).into())
            // let cmd: MutationCommand<String, serde_json::Value> = match rng.gen_range(0..=1) {
            //     0 => {
            //         MutationCommand::PUT(PutCommand { key: random_string(&mut rng, 1), value: serde_json::json!(null) })
            //     },
            //     _ => {
            //         MutationCommand::DELETE(DeleteCommand { key: random_string(&mut rng, 1)})
            //     }
            // };

            // (*term, cmd.into())
        }).collect::<Vec<Log<Vec<u8>>>>()
    }
    
    #[test]
    pub fn test_log_entries_from_term_sequence() {
        set_up_logging();
        let terms: Vec<u64> = vec![1, 1, 1, 4, 4, 5, 5, 6, 6, 6];
        let entries = log_entries_from_term_sequence(&terms);
        debug!("entries: {entries:?}");
        
        let commands: Vec<Log<MutationCommand<String, serde_json::Value>>> = 
        entries
        .into_iter()
        .map(|(term, cmd)| {
            (term, Into::<MutationCommand<String, serde_json::Value>>::into(cmd))
        })
        .collect();
        debug!("commands: {commands:?}");
    }
}

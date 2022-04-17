use serde_derive::{Serialize, Deserialize};
use std::sync::Arc;
use tokio::sync::Mutex;
use crate::network::node::Server;



pub type LogType = String;

/// Invoked by leader to replicate log entries; also used as heartbeat.
#[derive(Clone, Default, Debug, Serialize, Deserialize)]
pub struct AppendEntriesRequest {
    /// The leader's term.
    pub term: usize,
    /// Used by followers to redirect clients.
    pub leader_id: usize,
    /// The index of the log entry immediately preceding new ones.
    pub previous_log_index: usize,
    /// The term of `previous_log_index` entry.
    pub previous_log_term: usize,
    /// The log entries to store (Empty for heartbeat; may send more
    /// than one for efficiency.)
    pub entries: Vec<LogType>,
    /// The leader's commit index.
    pub leader_commit_index: usize
}

/// Returned by followers.
#[derive(Clone, Default, Debug, Serialize, Deserialize)]
pub struct AppendEntriesResponse {
    /// The current term for leader to update itself.
    pub term: usize,
    /// True if follower contained entry matching `previous_log_index`
    /// and `previous_log_term`.
    pub success: bool
}

impl AppendEntriesRequest {
    pub fn new(term: usize, leader_id: usize, previous_log_index: usize, previous_log_term: usize, entries: Vec<LogType>, leader_commit_index: usize) -> Self {
        Self {
            term,
            leader_id,
            previous_log_index,
            previous_log_term,
            entries,
            leader_commit_index
        }
    }
}



pub fn process(server: Arc<Mutex<Server>>, request: AppendEntriesRequest) -> AppendEntriesResponse {

    AppendEntriesResponse::default()
}
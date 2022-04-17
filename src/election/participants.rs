use std::collections::HashMap;
use serde_derive::{Serialize, Deserialize};


#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum Election {
    Follower,
    Candidate,
    Leader
}

pub type StateMachineCommand = String;
pub type LogRecord = (StateMachineCommand, usize);

/// Updated on stable storage before responding to RPCs.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct PersistentState {
    /// The kind of the election entity that the current node is assigned.
    pub participant_type: Election,
    /// The latest term server has seen (initialized to 0 on first boot, increases 
    /// monotonically.)
    pub current_term: usize,
    /// The `candidate_id` that received vote in the current term (or None, if none exists.)
    pub voted_for: Option<usize>,
    /// The log entries, each entry contains command for state machine, and term when entry
    /// was received by leader.
    pub log: Vec<LogRecord>
}


/// Volatile state on all servers. The properties
/// `next_index` and `match_index` are only applicable
/// to leader nodes and as such will be None in the other
/// two cases.
#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct VolatileState {
    /// The index of the highest log entry
    /// known to be committed (initialized to 0, increases
    /// monotonically).
    pub commit_index: usize,
    /// The index of the highest log entry applied to
    /// state machine (initialized to 0, increases monotonically).
    pub last_applied: usize,
    /// For each server, the index of the next log entry
    /// to send to that server (initialized to leader's last log index + 1).
    pub next_index: Option<HashMap<usize, usize>>,
    /// For each server, the index of the highest log entry
    /// known to be to replicated on that server (initialized to 0, increases monotonically).
    pub match_index: Option<HashMap<usize, usize>>
}


impl Default for PersistentState {
    fn default() -> Self {
        Self {
            current_term: 0,
            voted_for: None,
            log: vec![],
            participant_type: Election::default()
        }
    }
}

impl Default for Election {
    fn default() -> Self {
        Self::Follower
    }
}

/// The time that a follower waits for receiving communication 
/// from a leader or candidate.
pub const ELECTION_TIMEOUT: usize = 1;

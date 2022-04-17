use serde_derive::{Serialize, Deserialize};


#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Election {
    Follower,
    Candidate,
    Leader
}

type LogType = String;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PersistentState {

    pub participant_type: Election,
    pub current_term: usize,
    pub voted_for: Option<usize>,
    pub log: Vec<LogType>
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

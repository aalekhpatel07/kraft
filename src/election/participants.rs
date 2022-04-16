use serde_derive::{Serialize, Deserialize};


#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Election {
    Follower,
    Candidate,
    Leader
}


#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PersistentState {

    pub participant_type: Election,
    pub current_term: usize,
    pub voted_for: Option<usize>,
    pub log: Vec<String>
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
// pub trait Vote {
//     fn request_vote(
//         &self,
//         args: &RequestVoteRPCRequest
//     ) -> RequestVoteRPCResult
//     {
//         (0, true)
//     }
// }



#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RequestVoteRPCRequest {
    pub participant_type: Election,
    pub term: usize,
    pub candidate_id: usize,
    pub last_log_index: usize,
    pub last_log_term: usize
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RequestVoteRPCResult {
    pub participant_type: Election,
    pub term: usize,
    pub vote_granted: bool
}


/// The time that a follower waits for receiving communication 
/// from a leader or candidate.
pub const ELECTION_TIMEOUT: usize = 1;


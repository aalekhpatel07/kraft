use serde_derive::{Serialize, Deserialize};
use tokio::sync::{Mutex};
use std::sync::Arc;
use crate::network::node::Server;

// use crate::


/// Invoked by candidate to gather votes.
#[derive(Clone, Default, Debug, Serialize, Deserialize)]
pub struct RequestVoteRequest {
    /// The candidate's term.
    pub term: usize,
    /// The candidate's id.
    pub candidate_id: usize,
    /// The index of the candidate's last log entry.
    pub last_log_index: usize,
    /// The term of the candidate's last log entry.
    pub last_log_term: usize
}


/// Returned by followers when a candidate requests for votes.
#[derive(Clone, Default, Debug, Serialize, Deserialize)]
pub struct RequestVoteResponse {
    /// The current term, for candidate to update itself.
    pub term: usize,

    /// True means candidate received vote.
    pub vote_granted: bool
}




impl RequestVoteRequest {
    pub fn new(term: usize, candidate_id: usize, last_log_index: usize, last_log_term: usize) -> Self {
        Self {
            term,
            candidate_id,
            last_log_index,
            last_log_term
        }
    }
}

pub fn process(server: Arc<Mutex<Server>>, request: RequestVoteRequest) -> RequestVoteResponse {

    RequestVoteResponse::default()
}
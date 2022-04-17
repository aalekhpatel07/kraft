use serde_derive::{Serialize, Deserialize};
use tokio::sync::{Mutex};
use std::sync::Arc;
use crate::network::node::Server;
use log::{trace};
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

/// Given the state of a server, and a RequestVote RPC request,
/// process the request according to Section 5.2 of the "Understandable Consensus"
/// paper.
pub async fn process(server: Arc<Mutex<Server>>, request: RequestVoteRequest) -> RequestVoteResponse {

    // trace!("About to acquire server mutex in process request_vote");
    // // trace!("Server lock status: {:?}", server);
    // let guard = server.lock().await;
    // trace!("Acquired server mutex in process request_vote.");

    {
        trace!("About to acquire server state mutex in process request_vote.");
        let server = server.lock().await.clone();

        let state_guard = server.state.lock().await;
        let mut state = state_guard.clone();
        drop(state_guard);

        trace!("Acquired server state mutex in process request_vote.");

        if (request.term) < state.current_term {
            return RequestVoteResponse { term: state.current_term, vote_granted: false };
        }
        match state.voted_for {
            Some(chosen_candidate) => {
                if (chosen_candidate == request.candidate_id) && (request.last_log_index >= state.log.len()) {

                    let mut state_guard = server.state.lock().await;
                    state_guard.voted_for = Some(request.candidate_id);
                    drop(state_guard);

                    trace!("About to save state.");
                    server.save_state().await.unwrap();
                    trace!("State saved successfully.");

                    return RequestVoteResponse { term: state.current_term, vote_granted: true };
                }
                return RequestVoteResponse { term: state.current_term, vote_granted: false };
            },
            None => {
                if request.last_log_index >= state.log.len() {

                    let mut state_guard = server.state.lock().await;
                    state_guard.voted_for = Some(request.candidate_id);
                    drop(state_guard);

                    trace!("About to save state: {:?}.", server);
                    server.save_state().await.unwrap();
                    trace!("State saved successfully. state: {:?}", server.state.lock().await);
                    
                    return RequestVoteResponse { term: state.current_term, vote_granted: true };
                }
                return RequestVoteResponse { term: state.current_term, vote_granted: false };
            }
        }
    }
}


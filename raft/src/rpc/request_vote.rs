use proto::{
    VoteRequest, 
    VoteResponse,
};
use serde::{de::DeserializeOwned, Serialize};
use tonic::{Request, Response, Status};
use crate::{node::{Raft, Log, Follower}};
use log::{debug, info, trace, warn};


/// Given a raft node that receives a request for a RequestVoteRPC,
/// determine if the receiving node should grant the vote to the requester
/// and update the persistent state of the receiver node depending on
/// how it responded to the request.
/// 
/// ### Example
/// ```
/// use raft::node::RaftNode;
/// use state_machine::impls::key_value_store::KeyValueStore;
/// use proto::raft::{VoteRequest, VoteResponse};
/// use tonic::{Request};
/// use raft::rpc::request_vote::request_vote;
/// 
/// #[tokio::main]
/// pub async fn main() {
/// 
///     let mut raft_node: RaftNode<KeyValueStore<String, String>> = RaftNode::default();
///     let request = Request::new(
///         VoteRequest {
///             term: 1,
///             candidate_id: 1,
///             last_log_index: 0,
///             last_log_term: 0
///         }
///     );
///     
///     // The receiver node has an initial state.
///     assert_eq!(raft_node.persistent_state.lock().unwrap().current_term, 0);
///     assert_eq!(raft_node.persistent_state.lock().unwrap().voted_for, None);
/// 
///     // Initiate the RPC.
///     let response = request_vote(&raft_node, request).await.expect("Request Vote RPC failed.");
/// 
///     let response = response.into_inner();
///     // We must've received the vote because this was the initial vote request.
///     assert_eq!(response, VoteResponse { term: 1, vote_granted: true });    
/// 
///     // The receiver node has its state updated.
///     assert_eq!(raft_node.persistent_state.lock().unwrap().current_term, 1);
///     assert_eq!(raft_node.persistent_state.lock().unwrap().voted_for, Some(1));
/// 
/// }
/// ```
/// 
pub async fn request_vote<S, T>(node: &Raft<S, T>, request: Request<VoteRequest>) -> Result<Response<VoteResponse>, Status> 
where
    T: Clone + Serialize + DeserializeOwned + From<Vec<u8>> + std::fmt::Debug,

{
    trace!("Got a Vote request: {:?}", request);

    let VoteRequest {
        term, 
        candidate_id, 
        last_log_index, 
        last_log_term
    } = request.into_inner();

    let mut current_state_guard = node.persistent_data.lock().expect("Could not lock persistent state.");

    let latest_term = current_state_guard.current_term;
    let my_id = node.meta.id;
    // The requesting node is out of date from the latest
    // election term.
    if term < current_state_guard.current_term {
        warn!("Requesting node ({candidate_id}) is in an out-of-date election term ({term}). Informing it of my ({my_id}) latest term ({latest_term})");
        
        // Make sure we drop the guard because `node.save()` takes the lock again.
        // If we don't drop, we deadlock ;(
        drop(current_state_guard);
        node.save().expect("Couldn't save node to stable storage.");
        return Ok(Response::new(VoteResponse { term: latest_term , vote_granted: false }));
    }

    let receiver_log = &current_state_guard.log;

    let num_entries = receiver_log.len() as u64;


    // Determine if the candidate's log is at least as up-to-date as the receiver's log.
    let candidate_log_is_up_to_date: bool = {
        (
            {
                // Last log index must appear on the right of the receiver's log.
                let have_less_entries = last_log_index >= num_entries;

                if !have_less_entries {
                    warn!("Candidate's ({candidate_id}) log has latest entry's index ({last_log_index}), and term ({last_log_term}). We ({my_id}) have ({num_entries}) entries in our log, which is more than the candidate's.");
                }

                have_less_entries
            }
        )
        && 
        (
            {
                // If non-empty log, then make sure the candidate's term
                // comes later than the latest entry's term.
                if !receiver_log.is_empty() {
                    let our_last_entry_term = receiver_log.last().expect("Last entry in the log is None.").0;
                    let have_lower_term = last_log_term >= our_last_entry_term;
                    if !have_lower_term {
                        warn!("Candidate's ({candidate_id}) log has latest entry's index ({last_log_index}), and term ({last_log_term}). Our ({my_id}) last entry's term ({our_last_entry_term}) is larger than that of the candidate.");
                    }
                    have_lower_term
                } else {
                    true
                }
            }
        )
    };

    if !candidate_log_is_up_to_date {
        warn!("Candidate's ({candidate_id}) log has latest entry's index ({last_log_index}), and term ({last_log_term}). The candidate's log is more stale than ours. Notify candidate of that.");
    }

    if let Some(voted_for) = current_state_guard.voted_for {

        let latest_term = current_state_guard.current_term;
        let my_id = node.meta.id;

        // We have already voted for someone.
        if voted_for != candidate_id {
            // We have already voted in this term.
            if latest_term == term {
                warn!("We ({my_id}) have already voted for ({voted_for}) in election term ({latest_term}) so we deny candidate ({candidate_id}) a vote for term ({term}).");
                drop(current_state_guard);
                node.save().expect("Couldn't save node to stable storage.");
                return Ok(Response::new(VoteResponse { term: latest_term , vote_granted: false }));
            } 
            // We have already voted but in some term before the one that the candidate presented.
            // So basically, some elections have happened without us knowing.
            // In this case, we might benefit from updating ourselves.
            else {
                trace!("Our ({my_id}) most recent vote was for ({voted_for}) and happened in election term ({latest_term}). The candidate ({candidate_id}) is in the election term ({term}), which is higher than ours, so we may be able to grant a vote and update ourselves.");
                if candidate_log_is_up_to_date {
                    trace!("Since the candidate's ({candidate_id}) log is fresher than ours ({my_id}), we grant it a vote.");

                    current_state_guard.current_term = term;
                    current_state_guard.voted_for = Some(candidate_id);
                    drop(current_state_guard);

                    node.save().expect("Couldn't save node to stable storage.");
                    return Ok(Response::new(VoteResponse { term, vote_granted: true }))
                } else {
                    warn!("Since the candidate's ({candidate_id}) log is stale compared to ours ({my_id}), we DO NOT grant it a vote.");

                    current_state_guard.current_term = term;
                    drop(current_state_guard);
                    node.save().expect("Couldn't save node to stable storage.");
                    return Ok(Response::new(VoteResponse { term, vote_granted: false }))
                }
            }
        }
        // We have voted for the candidate.
        else {
            trace!("Our ({my_id}) most recent vote was for ({voted_for}) and happened in election term ({latest_term}). The candidate ({candidate_id}) is in the election term ({term}).");
            if latest_term == term {
                trace!("The candidate is once again asking a vote from us, for the same term. -_-");
                drop(current_state_guard);

                if candidate_log_is_up_to_date {

                    node.save().expect("Couldn't save node to stable storage.");
                    return Ok(Response::new(VoteResponse { term, vote_granted: true }))
                } else {

                    node.save().expect("Couldn't save node to stable storage.");
                    return Ok(Response::new(VoteResponse { term, vote_granted: false }))
                }
            } else {
                // We have voted for the candidate but in some previous term.
                if candidate_log_is_up_to_date {
                    current_state_guard.current_term = term;
                    current_state_guard.voted_for = Some(candidate_id);
                    drop(current_state_guard);

                    node.save().expect("Couldn't save node to stable storage.");
                    return Ok(Response::new(VoteResponse { term, vote_granted: true }))
                } else {
                    current_state_guard.current_term = term;
                    drop(current_state_guard);

                    node.save().expect("Couldn't save node to stable storage.");
                    return Ok(Response::new(VoteResponse { term, vote_granted: false }))
                }                
            }
        }
    }
    else {
        // We haven't voted for anybody yet.
        // Grant a vote as long as candidate's log is up-to-date.
        if candidate_log_is_up_to_date {
            trace!("Candidate's ({candidate_id}) log is at least as fresh as ours ({my_id}). Along with the fact that we haven't voted for anybody yet, we grant a vote to the candidate.");
            current_state_guard.current_term = term;
            current_state_guard.voted_for = Some(candidate_id);
            drop(current_state_guard);

            node.save().expect("Couldn't save node to stable storage.");
            return Ok(Response::new(VoteResponse { term, vote_granted: true }))
        } else {

            warn!("Candidate's ({candidate_id}) log is stale compared to ours ({my_id}). Even though we haven't voted for anybody yet, we DO NOT vote for the candidate because we know we have a \"better\" log than the candidate.");

            current_state_guard.current_term = term;
            drop(current_state_guard);

            node.save().expect("Couldn't save node to stable storage.");
            return Ok(Response::new(VoteResponse { term, vote_granted: false }))
        }
    }
}


#[cfg(test)]
pub mod tests {
    use crate::utils::test_utils::set_up_logging;
    use crate::node::{Raft, Int, Follower, PersistentState, VolatileState};
    use proto::leader_rpc_server::LeaderRpc;
    use proto::candidate_rpc_server::CandidateRpc;
    use crate::storage::state::raft_io::ReadWriteState;

    use state_machine::impls::key_value_store::*;
    use state_machine::StateMachine;
    use log::{info, debug};
    use super::*;
    use tonic::{Request, Response, Status};
    use std::path::PathBuf;
    use crate::storage::state::raft_io::*;

    use rand::{distributions::Alphanumeric, Rng};


    pub fn create_request(term: Int, candidate_id: Int, last_log_index: Int, last_log_term: Int) -> Request<VoteRequest> {
        let vote_request = VoteRequest {
            term, candidate_id, last_log_index, last_log_term
        };
        Request::new(vote_request)
    }

    pub fn create_response(term: Int, vote_granted: bool) -> VoteResponse {
        VoteResponse {
            term,
            vote_granted
        }
    }

    macro_rules! request_vote_test {
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
                    let mut receiver: Raft<Follower, Vec<u8>> = Raft::default();
                    
                    let mut log_file_path = PathBuf::from("/tmp");
                    let log_file_base = rand::thread_rng().sample_iter(&Alphanumeric).take(15).map(char::from).collect::<String>();
                    log_file_path.push(format!("{}.raft", log_file_base));

                    receiver.meta.log_file = log_file_path.to_str().expect("Is invalid unicode.").to_owned();

                    // Set the initial states as given.
                    receiver.persistent_data = std::sync::Arc::new(std::sync::Mutex::new($initial_persistent_state));
                    receiver.volatile_data = std::sync::Arc::new(std::sync::Mutex::new($initial_volatile_state));

                    // Create a VoteRequest from the given argument.
                    let request = create_request($request.0, $request.1, $request.2, $request.3);

                    // Create an expected response from the given response.
                    let expected_response = create_response($response.0, $response.1);

                    // Make the RequestVoteRPC and get a response.
                    let observed_response = request_vote(&receiver, request).await.expect("RequestVoteRPC failed to await.");

                    // Assert the observed response is the same as expected.
                    assert_eq!(
                        observed_response.into_inner(), 
                        expected_response,
                        "VoteResponse does not match up."
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
                        &$final_persistent_state
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

    request_vote_test!(
        /// Test that a receiver node grants the vote to any vote request initially.
        initial, 
        PersistentState { current_term: 0, voted_for: None, log: vec![] },
        VolatileState { commit_index: 0, last_applied: 0}, 
        (0, 0, 0, 0),
        (0, true),
        PersistentState { current_term: 0, voted_for: Some(0), log: vec![] },
        VolatileState { commit_index: 0, last_applied: 0}
    );

    request_vote_test!(
        /// Test that a receiver node grants 
        /// the vote to a vote request when the receiver
        /// hasn't voted for anybody yet.
        grant_vote_if_not_voted_yet_and_log_at_least_up_to_date, 
        PersistentState { current_term: 0, voted_for: None, log: vec![] },
        VolatileState { commit_index: 0, last_applied: 0}, 
        (1, 0, 0, 0),
        (1, true),
        PersistentState { current_term: 1, voted_for: Some(0), log: vec![] },
        VolatileState { commit_index: 0, last_applied: 0}
    );


    request_vote_test!(
        /// Test that a receiver node denies 
        /// a vote request if the candidate
        /// is in a stale election term.
        stale_election_term_denial, 
        PersistentState { current_term: 2, voted_for: None, log: vec![] },
        VolatileState { commit_index: 0, last_applied: 0}, 
        (1, 1, 0, 0),
        (2, false),
        PersistentState { current_term: 2, voted_for: None, log: vec![] },
        VolatileState { commit_index: 0, last_applied: 0}
    );

    request_vote_test!(
        /// Test that a receiver node approves 
        /// a vote request to the candidate
        /// if the receiver has already voted for that term
        /// but the vote was for the same candidate.
        approve_vote_if_already_voted_for_same_candidate, 
        PersistentState { current_term: 2, voted_for: Some(1), log: vec![] },
        VolatileState { commit_index: 0, last_applied: 0}, 
        (2, 1, 0, 0), // Candidate is in election term 3.
        (2, true), // Send the latest election term.
        PersistentState { current_term: 2, voted_for: Some(1), log: vec![] },
        VolatileState { commit_index: 0, last_applied: 0}
    );

    request_vote_test!(
        /// Test that a receiver node denies
        /// a vote request to the candidate
        /// if the receiver has already voted for that term
        /// but the vote was for the other candidate.
        deny_vote_if_already_voted_for_other_candidate, 
        PersistentState { current_term: 2, voted_for: Some(3), log: vec![] },
        VolatileState { commit_index: 0, last_applied: 0}, 
        (2, 1, 0, 0), // Candidate is in election term 3.
        (2, false), // Send the latest election term.
        PersistentState { current_term: 2, voted_for: Some(3), log: vec![] },
        VolatileState { commit_index: 0, last_applied: 0}
    );

    request_vote_test!(
        /// Test that a receiver node approves
        /// a vote request to the candidate
        /// if the receiver has already voted for the candidate
        /// but in a previous term.
        approve_vote_if_already_voted_for_same_candidate_but_in_a_previous_term, 
        PersistentState { current_term: 0, voted_for: Some(1), log: vec![] },
        VolatileState { commit_index: 0, last_applied: 0}, 
        (2, 1, 0, 0), // Candidate is in election term 3.
        (2, true), // Send the latest election term.
        PersistentState { current_term: 2, voted_for: Some(1), log: vec![] },
        VolatileState { commit_index: 0, last_applied: 0}
    );

    request_vote_test!(
        /// Test that a receiver node approves
        /// a vote request of the candidate
        /// if the receiver has already voted for some other node
        /// but in a previous term.
        approve_vote_if_already_voted_for_other_candidate_in_a_previous_term, 
        PersistentState { current_term: 0, voted_for: Some(3), log: vec![] }, // Already voted for Node 3.
        VolatileState { commit_index: 0, last_applied: 0}, 
        (2, 1, 0, 0), // Candidate is in election term 3.
        (2, true), // Send the latest election term.
        PersistentState { current_term: 2, voted_for: Some(1), log: vec![] },
        VolatileState { commit_index: 0, last_applied: 0}
    );

    request_vote_test!(
        /// Test that a receiver node denies
        /// a vote request of the candidate
        /// if the receiver has not already voted
        /// and has a fresher log than the candidate.
        deny_vote_if_not_already_voted_and_candidate_log_is_stale, 
        PersistentState { current_term: 2, voted_for: None, log: vec![(1, vec![])] }, // Already voted for Node 2.
        VolatileState { commit_index: 0, last_applied: 0}, 
        (2, 1, 0, 0), // Candidate is in election term 2.
        (2, false), // Send the latest election term.
        PersistentState { current_term: 2, voted_for: None, log: vec![(1, vec![])] },
        VolatileState { commit_index: 0, last_applied: 0}
    );

    request_vote_test!(
        /// Test that a receiver node approves
        /// a vote request of the candidate
        /// if the receiver has not already voted
        /// and has candidate has the same log as receiver's.
        approve_vote_if_not_already_voted_and_candidate_log_is_same_as_receiver_log, 
        PersistentState { current_term: 2, voted_for: None, log: vec![(1, vec![])] }, // Already voted for Node 2.
        VolatileState { commit_index: 0, last_applied: 0}, 
        (2, 1, 1, 1), // Candidate is in election term 2.
        (2, true), // Send the latest election term.
        PersistentState { current_term: 2, voted_for: Some(1), log: vec![(1, vec![])] },
        VolatileState { commit_index: 0, last_applied: 0}
    );

    request_vote_test!(
        /// Test that a receiver node denies
        /// a vote request of the candidate
        /// if the receiver has not already voted
        /// and has the candidate's last log term is earlier than the receiver's last entry's.
        deny_vote_if_not_already_voted_and_candidates_last_log_term_is_earlier_than_receiver_logs_last_entry, 
        PersistentState { current_term: 2, voted_for: None, log: vec![(2, vec![])] }, // Already voted for Node 2.
        VolatileState { commit_index: 0, last_applied: 0}, 
        (2, 1, 1, 1), // Candidate is in election term 2.
        (2, false), // Send the latest election term.
        PersistentState { current_term: 2, voted_for: None, log: vec![(2, vec![])] },
        VolatileState { commit_index: 0, last_applied: 0}
    );


    request_vote_test!(
        /// Test that a receiver node denies
        /// a vote request of the candidate
        /// if the receiver has not already voted
        /// and has the candidate has less entries than us.
        deny_vote_if_not_already_voted_and_candidate_has_less_entries_than_receiver, 
        PersistentState { current_term: 2, voted_for: None, log: vec![(2, vec![]), (2, vec![])] }, // Already voted for Node 2.
        VolatileState { commit_index: 0, last_applied: 0}, 
        (2, 1, 1, 2), // Candidate is in election term 2.
        (2, false), // Send the latest election term.
        PersistentState { current_term: 2, voted_for: None, log: vec![(2, vec![]), (2, vec![])] },
        VolatileState { commit_index: 0, last_applied: 0}
    );

    request_vote_test!(
        /// Test that a receiver node approves
        /// a vote request of the candidate
        /// if the receiver has not already voted
        /// and has the candidate has more entries than receiver.
        approve_vote_if_not_already_voted_and_candidate_has_more_entries_than_receiver, 
        PersistentState { current_term: 1, voted_for: Some(2), log: vec![(2, vec![]), (2, vec![])] }, // Already voted for Node 2.
        VolatileState { commit_index: 0, last_applied: 0}, 
        (2, 1, 4, 3), // Candidate is in election term 2.
        (2, true), // Send the latest election term.
        PersistentState { current_term: 2, voted_for: Some(1), log: vec![(2, vec![]), (2, vec![])] },
        VolatileState { commit_index: 0, last_applied: 0}
    );

    request_vote_test!(
        /// Test that a receiver node denies
        /// a vote request of the candidate
        /// despite itself having a stale log,
        /// because it was able to determine that
        /// the candidate has even more stale log.
        deny_vote_if_already_voted_in_some_previous_term_but_candidate_has_more_stale_log_than_receiver, 
        PersistentState { current_term: 2, voted_for: Some(4), log: vec![(2, vec![]), (2, vec![])] }, // Already voted for Node 2.
        VolatileState { commit_index: 0, last_applied: 0}, 
        (6, 1, 2, 1), // Candidate is in election term 2.
        (6, false), // Send the latest election term.
        PersistentState { current_term: 6, voted_for: Some(4), log: vec![(2, vec![]), (2, vec![])] },
        VolatileState { commit_index: 0, last_applied: 0}
    );

    request_vote_test!(
        /// Test that a receiver node denies
        /// a vote request of the candidate
        /// if it already voted for it in the same
        /// term but the candidate has more stale logs than the
        /// receiver.
        deny_vote_if_already_voted_for_the_same_candidate_in_this_term_but_candidate_has_more_stale_log_than_receiver,
        PersistentState { current_term: 2, voted_for: Some(1), log: vec![(2, vec![]), (2, vec![])] }, // Already voted for Node 2.
        VolatileState { commit_index: 0, last_applied: 0}, 
        (2, 1, 2, 1), // Candidate is in election term 2.
        (2, false), // Send the latest election term.
        PersistentState { current_term: 2, voted_for: Some(1), log: vec![(2, vec![]), (2, vec![])] },
        VolatileState { commit_index: 0, last_applied: 0}
    );

    request_vote_test!(
        /// Test that a receiver node denies
        /// a vote request of the candidate
        /// if it already voted for it in some previous term
        /// and the candidate has stale logs compared to the receiver.
        deny_vote_if_already_voted_for_the_same_candidate_in_some_previous_term_and_candidate_has_stale_logs,
        PersistentState { current_term: 1, voted_for: Some(1), log: vec![(2, vec![]), (2, vec![])] }, // Already voted for Node 2.
        VolatileState { commit_index: 0, last_applied: 0}, 
        (2, 1, 2, 1), // Candidate is in election term 2.
        (2, false), // Send the latest election term.
        PersistentState { current_term: 2, voted_for: Some(1), log: vec![(2, vec![]), (2, vec![])] },
        VolatileState { commit_index: 0, last_applied: 0}
    );
}
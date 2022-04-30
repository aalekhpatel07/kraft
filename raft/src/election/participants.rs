// use serde_derive::{Deserialize, Serialize};
// use std::collections::HashMap;
// use crate::node::NodeType;

// pub type StateMachineCommand = String;
// pub type LogRecord = (StateMachineCommand, usize);

// /// Updated on stable storage before responding to RPCs.
// #[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
// pub struct PersistentState {
//     /// The kind of the election entity that the current node is assigned.
//     pub participant_type: NodeType,
//     /// The latest term server has seen (initialized to 0 on first boot, increases
//     /// monotonically.)
//     pub current_term: usize,
//     /// The `candidate_id` that received vote in the current term (or None, if none exists.)
//     pub voted_for: Option<usize>,
//     /// The log entries, each entry contains command for state machine, and term when entry
//     /// was received by leader.
//     pub log: Vec<LogRecord>,
// }


// impl Default for PersistentState {
//     fn default() -> Self {
//         Self {
//             current_term: 0,
//             voted_for: None,
//             log: vec![],
//             participant_type: NodeType::default(),
//         }
//     }
// }

// /// The time that a follower waits for receiving communication
// /// from a leader or candidate.
// pub const ELECTION_TIMEOUT: usize = 5;

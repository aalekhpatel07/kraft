use std::sync::{Arc, Mutex};
use serde::Serialize;
use serde::de::DeserializeOwned;
use serde_derive::{Deserialize, Serialize};
use crate::storage::state::persistent::{self, State};
use crate::storage::state::persistent::Log;
use crate::storage::state::{volatile};
use crate::storage::state::raft_io::ReadWriteState;
use std::fs::File;
use anyhow::Result;
use std::env::temp_dir;
use proto::raft::{LogEntry as ProtoLogEntry};
use log::{trace};


pub type Int = u64;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum NodeType {
    Candidate,
    Follower,
    Leader
}

impl Default for NodeType {
    fn default() -> Self {
        Self::Follower
    }
}

impl NodeType {
    pub fn new() -> Self {
        Default::default()
    }
}

// impl<T> From<ProtoLogEntry> for Log<T> 
// where
//     T: Clone + From<Vec<u8>>
// {
//     fn from(entry: ProtoLogEntry) -> Self {
//         LogEntryImpl { term: entry.term as usize, command: entry.command.clone().into() }
//     }
// }

// impl<T> From<LogEntryImpl<T>> for ProtoLogEntry 
// where
//     T: Clone + Into<Vec<u8>>
// {
//     fn from(entry: LogEntryImpl<T>) -> Self {
//         Self {
//             term: entry.term() as u64,
//             command: entry.command().clone().into()
//         }
//     }
// }

// #[derive(Debug, Clone, Default, Serialize, Deserialize)]
// pub struct Node<L> 
// where
//     L: Clone
// {
//     pub node_type: NodeType,
//     pub meta: NodeMetadata,
//     pub persistent_state: Arc<Mutex<persistent::State<L>>>,
//     pub volatile_state: Arc<Mutex<volatile::VolatileState>>
// }


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RaftNode<S>
where
    S: state_machine::StateMachine,
{
    pub node_type: NodeType,
    pub meta: NodeMetadata,
    pub persistent_state: Arc<Mutex<persistent::State<S::MutationCommand>>>,
    pub volatile_state: Arc<Mutex<volatile::VolatileState>>
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct NodeMetadata {
    pub id: Int,
    pub addr: String,
    pub log_file: String
}

impl NodeMetadata {
    pub fn new() -> Self {
        Default::default()
    }
}

impl Default for NodeMetadata {
    fn default() -> Self {

        let mut log_file = temp_dir();
        log_file.push("raft.log");

        Self {
            id: 0,
            addr: "".to_owned(),
            log_file: log_file.to_str().expect("Path to log file may not be valid UTF-8.").to_owned()
        }
    }
}


impl<S> Default for RaftNode<S>
where
    S: state_machine::StateMachine
{
    fn default() -> RaftNode<S> {
        Self {
            node_type: NodeType::default(),
            meta: NodeMetadata::default(),
            persistent_state: Arc::new(Mutex::new(State::default())),
            volatile_state: Arc::new(Mutex::new(volatile::VolatileState::default()))
        }
    }
}


impl<S> RaftNode<S>
where
    S: state_machine::StateMachine,
    S::MutationCommand: Clone + Serialize + DeserializeOwned
{
    pub fn save(&self) -> Result<usize> {
        let state = self.persistent_state.lock().unwrap();
        let log_file = &self.meta.log_file;
        trace!("Writing state to {log_file:?}");
        let mut file = File::create(log_file)?;

        let bytes_written = file
            .write_state(&state.clone())
            .expect("Could not write persistent state.");
        Ok(bytes_written)
    }

    pub fn leader_commit_index(&self) -> Option<Int> {
        match &*self.volatile_state.lock().unwrap() {
            volatile::VolatileState::Leader(state) => {
                Some(state.commit_index)
            },
            volatile::VolatileState::NonLeader(_) => {
                None
            }
        }
    }

    pub fn new() -> Self {
        Default::default()
    }

    pub fn set_logs(&self, logs: &[Log<S::MutationCommand>]) -> Result<()> {
        let mut state = self.persistent_state.lock().expect("Couldn't lock persistent_state.");
        state.log = logs.to_vec();
        Ok(())
    }

    pub fn load_state(&self) -> Result<State<S::MutationCommand>> {
        let mut file = File::open(&self.meta.log_file)?;
        let observed_state: State<S::MutationCommand> = file.read_state()?;
        Ok(observed_state)
    }
    // pub fn proto_log(&self) -> Vec<ProtoLogEntry> {
    //     self
    //     .log()
    //     .iter()
    //     .map(|entry| {
    //         ProtoLogEntry::from(entry.clone())
    //     })
    //     .collect::<Vec<ProtoLogEntry>>()
    // }

    // pub fn log(&self) -> Vec<LogEntry<L>> {
    //     self.persistent_state.lock().unwrap().log.clone()
    // }
}


#[cfg(feature = "random")]
pub mod random {
    use rand::{Rng};
    use rand::distributions::{Distribution, Standard};
    use super::RaftNode;
    use super::NodeMetadata;
    use super::{NodeType, Int};
    use std::sync::{Arc, Mutex};
    use crate::storage::state::persistent::{State, Log};
    use crate::storage::state::volatile::VolatileState;
    use crate::storage::state::volatile::{LeaderState, NonLeaderState};

    #[cfg(feature = "hashbrown")]
    use hashbrown::HashMap;

    #[cfg(not(feature = "hashbrown"))]
    use std::collections::HashMap;

    impl Distribution<NodeMetadata> for Standard {
        fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> NodeMetadata {
            NodeMetadata {
                id: rng.gen_range(0..=10),
                ..Default::default()
            }
        }
    }

    impl Distribution<NodeType> for Standard {
        fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> NodeType {
            match rng.gen_range(0usize..=2) {
                0 => NodeType::Follower,
                1 => NodeType::Candidate,
                _ => NodeType::Leader
            }
        }
    }

    impl Distribution<LeaderState> for Standard {
        fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> LeaderState {

            let mut next_index_hmap: HashMap<Int, Option<Int>> =  HashMap::new();
            let mut match_index_hmap: HashMap<Int, Option<Int>> =  HashMap::new();

            (0..=5)
            .for_each(|id| {
                next_index_hmap.insert(id, rng.gen::<Option<Int>>());
                match_index_hmap.insert(id, rng.gen::<Option<Int>>());
            });

            LeaderState {
                commit_index: rng.gen_range(0..=15),
                last_applied: rng.gen_range(0..=15),
                next_index: next_index_hmap,
                match_index: match_index_hmap
            }
        }
    }

    impl Distribution<NonLeaderState> for Standard {
        fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> NonLeaderState {

            NonLeaderState {
                commit_index: rng.gen_range(0..=15),
                last_applied: rng.gen_range(0..=15)
            }
        }
    }

    impl Distribution<VolatileState> for Standard {
        fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> VolatileState {
            match rng.gen_range(0..=1) {
                0 => VolatileState::Leader(rng.gen::<LeaderState>()),
                _ => VolatileState::NonLeader(rng.gen::<NonLeaderState>())
            }
        }
    }


    impl<T> Distribution<State<T>> for Standard 
    where
        Standard: Distribution<T>
    {
        fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> State<T> {
            let mut log: Vec<Log<T>> = vec![];

            (0..rng.gen_range::<>(0..=10u64))
            .for_each(|idx| {
                log.push((rng.gen_range((idx+1)..=100), rng.gen::<T>()));
            });

            State {
                current_term: rng.gen_range(0..=10),
                voted_for: {
                    match rng.gen_range(0..=1) {
                        0 => Some(rng.gen_range(0..=5)),
                        _ => None
                    }
                },
                log
            }
        }
    }

    impl<S> Distribution<RaftNode<S>> for Standard 
    where
        S: state_machine::StateMachine,
        Standard: Distribution<S::MutationCommand>
    {
        fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> RaftNode<S> {
            
            let node_type: NodeType = rng.gen::<NodeType>();
            let meta: NodeMetadata = rng.gen::<NodeMetadata>();
            let persistent_state: State<S::MutationCommand> = rng.gen::<State<S::MutationCommand>>();
            let volatile_state: VolatileState = rng.gen::<VolatileState>();

            RaftNode {
                node_type,
                meta,
                persistent_state: Arc::new(Mutex::new(persistent_state)),
                volatile_state: Arc::new(Mutex::new(volatile_state)),
            }
        }
    }
}

#[cfg(test)]
pub mod tests {
    use crate::node::{NodeType, NodeMetadata};
    use crate::storage::state;

    use super::RaftNode;
    use state_machine::StateMachine;
    use state_machine::impls::key_value_store::*;
    use serde_json::Value;
    use std::sync::{Arc, Mutex};
    use crate::utils::test_utils::set_up_logging;
    use log::{debug, info};

    #[cfg(feature = "random")]
    use rand::{Rng, thread_rng};


    #[test]
    fn test_create_new_raft_node() {
        let node: RaftNode<KeyValueStore<String, Value>> = RaftNode::new();

        assert_eq!(node.node_type, NodeType::new());
        assert_eq!(node.meta, NodeMetadata::new());
        assert_eq!(&*node.persistent_state.lock().unwrap(), &state::persistent::State::new());
        assert_eq!(&*node.volatile_state.lock().unwrap(), &state::volatile::VolatileState::new());

    }

    #[test]
    fn test_create_default_raft_node() {
        let node: RaftNode<KeyValueStore<String, Value>> = RaftNode::default();

        assert_eq!(node.node_type, NodeType::default());
        assert_eq!(node.meta, NodeMetadata::default());
        assert_eq!(&*node.persistent_state.lock().unwrap(), &state::persistent::State::default());
        assert_eq!(&*node.volatile_state.lock().unwrap(), &state::volatile::VolatileState::default());

    }


    #[test]
    fn test_create_random_raft_node() {
        set_up_logging();
        let mut rng = thread_rng();
        let node: RaftNode<KeyValueStore<usize, usize>> = rng.gen();
        info!("Random node: {node:?}");
    }
    #[test]
    fn test_set_logs() {
        set_up_logging();
        let node: RaftNode<KeyValueStore<String, Value>> = RaftNode::default();
        let logs = vec![
            (0, MutationCommand::PUT(PutCommand { key: "a".to_owned(), value: serde_json::json!({ "a": "doggie" })})),
            (0, MutationCommand::PUT(PutCommand { key: "a".to_owned(), value: serde_json::json!({ "a": "cat" })})),
            (1, MutationCommand::PUT(PutCommand { key: "a".to_owned(), value: serde_json::json!({ "a": "doggie" })})),
        ];
        node.set_logs(&logs).unwrap();
        info!("node: {node:?}");
    }
}
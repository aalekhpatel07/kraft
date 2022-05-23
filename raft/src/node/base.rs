use std::fmt::Debug;
use std::sync::{Arc, Mutex};
use serde::Serialize;
use serde::de::DeserializeOwned;
use serde_derive::{Deserialize, Serialize};
// use crate::storage::state::persistent::{self, State};
use crate::storage::state::raft_io::ReadWriteState;
use std::fs::File;
use anyhow::Result;
use std::env::temp_dir;
use proto::{LogEntry as ProtoLogEntry};
use log::{trace};
use hashbrown::HashMap;


pub type Int = u64;

pub type Log<T> = (Int, T);
// #[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
// pub enum NodeType {
//     Candidate,
//     Follower,
//     Leader
// }

// impl Default for NodeType {
//     fn default() -> Self {
//         Self::Follower
//     }
// }

// impl NodeType {
//     pub fn new() -> Self {
//         Default::default()
//     }
// }

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

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct PersistentState<T>
{
    /// The log entries, each entry contains command for state machine, and term when entry
    /// was received by leader.
    pub log: Vec<Log<T>>,
    // /// The kind of the election entity that the current node is assigned.
    // pub participant_type: NodeType,
    /// The latest term server has seen (initialized to 0 on first boot, increases
    /// monotonically.)
    pub current_term: Int,
    /// The `candidate_id` that received vote in the current term (or None, if none exists.)
    pub voted_for: Option<Int>,
}

impl<T> PersistentState<T> 
where
    T: Serialize + DeserializeOwned
{
    pub fn new() -> Self {
        Default::default()
    }
}

impl<T> Default for PersistentState<T>
where
    T: Serialize + DeserializeOwned
{
    fn default() -> Self {
        Self {
            current_term: 0,
            voted_for: None,
            log: vec![],
        }
    }
}


#[derive(Default, Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct VolatileState
{
    /// The index of the highest log entry
    /// known to be committed (initialized to 0, increases
    /// monotonically).
    pub commit_index: Int,
    /// The index of the highest log entry applied to
    /// state machine (initialized to 0, increases monotonically).
    pub last_applied: Int,
}

impl VolatileState {
    pub fn new() -> Self {
        Default::default()
    }
}


/// The state stored by Leaders.
#[derive(Default, Clone, Debug, Serialize, Deserialize)]
pub struct LeaderVolatileState {
    pub next_index: HashMap<Int, Option<Int>>,
    /// For each server, the index of the highest log entry
    /// known to be to replicated on that server (initialized to 0, increases monotonically).
    pub match_index: HashMap<Int, Option<Int>>
}

impl LeaderVolatileState {
    pub fn new() -> Self {
        Default::default()
    }
}

impl From<HashMap<Int, ClusterNode>> for LeaderVolatileState {
    fn from(cluster: HashMap<Int, ClusterNode>) -> Self {

        let mut next_index_map = HashMap::new();
        let mut match_index_map = HashMap::new();

        for &key in cluster.keys() {
            next_index_map.insert(key, None);
            match_index_map.insert(key, None);
        }

        Self {
            next_index: next_index_map,
            match_index: match_index_map
        }
    }
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Raft<S, T> {
    /// For each server, the index of the next log entry
    /// to send to that server (initialized to leader's last log index + 1).
    pub meta: NodeMetadata,
    pub cluster: HashMap<Int, ClusterNode>,
    pub persistent_data: Arc<Mutex<PersistentState<T>>>,
    pub volatile_data: Arc<Mutex<VolatileState>>,
    pub node_state: S
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Leader {
    pub neighbor_data: Arc<Mutex<LeaderVolatileState>>
}

impl Default for Leader {
    fn default() -> Self {
        Self {
            neighbor_data: Arc::new(Mutex::new(LeaderVolatileState::default()))
        }
    }
}

#[derive(Default, Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Candidate;

#[derive(Default, Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Follower;


impl<T> From<Raft<Follower, T>> for Raft<Candidate, T> {
    fn from(raft: Raft<Follower, T>) -> Self {
        Raft {
            volatile_data: raft.volatile_data,
            meta: raft.meta,
            cluster: raft.cluster,
            persistent_data: raft.persistent_data,
            node_state: Candidate
        }
    }
}


impl<T> From<Raft<Candidate, T>> for Raft<Leader, T> {
    fn from(raft: Raft<Candidate, T>) -> Self {
        
        Raft {
            volatile_data: raft.volatile_data,
            meta: raft.meta,
            cluster: raft.cluster.clone(),
            persistent_data: raft.persistent_data,
            node_state: Leader {
                neighbor_data: Arc::new(Mutex::new(raft.cluster.into()))
            }
        }
    }
}


impl<T> From<Raft<Candidate, T>> for Raft<Follower, T> {
    fn from(raft: Raft<Candidate, T>) -> Self {
        Raft {
            volatile_data: raft.volatile_data,
            meta: raft.meta,
            cluster: raft.cluster,
            persistent_data: raft.persistent_data,
            node_state: Follower
        }
    }
}



impl<T> From<Raft<Leader, T>> for Raft<Follower, T> {
    fn from(raft: Raft<Leader, T>) -> Self {
        Raft {
            volatile_data: raft.volatile_data,
            meta: raft.meta,
            cluster: raft.cluster,
            persistent_data: raft.persistent_data,
            node_state: Follower
        }
    }
}

impl<S, T> Default for Raft<S, T> 
where
    T: Serialize + DeserializeOwned,
    S: Default
{
    fn default() -> Self {
        Self { 
            persistent_data: Arc::new(Mutex::new(PersistentState::new())), 
            meta: NodeMetadata::new(), 
            cluster: HashMap::new(),
            volatile_data: Arc::new(Mutex::new(VolatileState::default())), 
            node_state: S::default()
        }
    }
}


impl<S, T> Raft<S, T> 
where
    T: Serialize + DeserializeOwned,
    S: Default
{
    pub fn new() -> Self {
        Default::default()
    }
}


impl<S, T> Raft<S, T> 
where
    T: Clone + Serialize + DeserializeOwned
{
    pub fn save(&self) -> Result<usize> {
        let state = self.persistent_data.lock().unwrap();
        let log_file = &self.meta.log_file;
        trace!("Writing state to {log_file:?}");
        let mut file = File::create(log_file)?;

        let bytes_written = file
            .write_state(&state.clone())
            .expect("Could not write persistent state.");
        Ok(bytes_written)
    }
}

impl<S, T> Raft<S, T>
where
    T: Serialize + DeserializeOwned 
{
    pub fn load_state(&self) -> Result<PersistentState<T>> {
        let mut file = File::open(&self.meta.log_file)?;
        let observed_state: PersistentState<T> = file.read_state()?;
        Ok(observed_state)
    }
}

// impl<T> Default for Raft<Follower, T> {
//     fn default() -> Self {
        
//     }
// }

// #[derive(Debug, Clone, Serialize, Deserialize)]
// pub struct RaftNode<S>
// where
//     S: state_machine::StateMachine,
//     S::MutationCommand: std::fmt::Debug + From<Vec<u8>>
// {
//     pub node_type: NodeType,
//     pub meta: NodeMetadata,
//     pub persistent_state: Arc<Mutex<State<S::MutationCommand>>>,
//     pub volatile_state: Arc<Mutex<volatile::VolatileState>>
// }

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


#[derive(Default, Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct ClusterNode {
    /// The id of the node in the raft cluster.
    pub id: Int,
    /// The address of a node in the cluster from the perspective of a given node.
    pub addr: String
}

impl From<NodeMetadata> for ClusterNode {
    fn from(node: NodeMetadata) -> Self {
        ClusterNode { id: node.id, addr: node.addr }
    }
}

#[cfg(test)]
pub mod tests {
    use crate::storage::state;

    use super::{Raft, PersistentState, VolatileState, NodeMetadata, Follower, Candidate, Leader, LeaderVolatileState};
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
        let node: Raft<Follower, usize> = Raft::new();

        assert_eq!(node.node_state, Follower);
        assert_eq!(node.meta, NodeMetadata::new());
        assert_eq!(&*node.persistent_data.lock().unwrap(), &PersistentState::new());
        assert_eq!(&*node.volatile_data.lock().unwrap(), &VolatileState::new());

    }

    #[test]
    fn transform_follower_into_candidate() {
        let node: Raft<Follower, usize> = Raft::new();
        assert_eq!(node.node_state, Follower);
        assert_eq!(node.meta, NodeMetadata::new());
        assert_eq!(&*node.persistent_data.lock().unwrap(), &PersistentState::new());
        assert_eq!(&*node.volatile_data.lock().unwrap(), &VolatileState::new());
        let node: Raft<Candidate, usize> = node.into();
        assert_eq!(node.node_state, Candidate);
        assert_eq!(node.meta, NodeMetadata::new());
        assert_eq!(&*node.persistent_data.lock().unwrap(), &PersistentState::new());
        assert_eq!(&*node.volatile_data.lock().unwrap(), &VolatileState::new());
    }

    #[test]
    fn transform_candidate_into_leader() {
        let node: Raft<Candidate, usize> = Raft::new();
        assert_eq!(node.node_state, Candidate);
        assert_eq!(node.meta, NodeMetadata::new());
        assert_eq!(&*node.persistent_data.lock().unwrap(), &PersistentState::new());
        assert_eq!(&*node.volatile_data.lock().unwrap(), &VolatileState::new());
        let node: Raft<Leader, usize> = node.into();
        // assert_eq!(node.node_state, Leader);
        assert_eq!(node.meta, NodeMetadata::new());
        assert_eq!(&*node.persistent_data.lock().unwrap(), &PersistentState::new());
        assert_eq!(&*node.volatile_data.lock().unwrap(), &VolatileState::new());
    }
    // #[test]
    // fn test_create_default_raft_node() {
    //     let node = Raft::default();

    //     assert_eq!(node.node_type, NodeType::default());
    //     assert_eq!(node.meta, NodeMetadata::default());
    //     assert_eq!(&*node.persistent_state.lock().unwrap(), &state::persistent::State::default());
    //     assert_eq!(&*node.volatile_state.lock().unwrap(), &state::volatile::VolatileState::default());

    // }


    // #[test]
    // fn test_create_random_raft_node() {
    //     set_up_logging();
    //     let mut rng = thread_rng();
    //     let node: RaftNode<KeyValueStore<usize, usize>> = rng.gen();
    //     info!("Random node: {node:?}");
    // }
    // #[test]
    // fn test_set_logs() {
    //     set_up_logging();
    //     let node: RaftNode<KeyValueStore<String, Value>> = RaftNode::default();
    //     let logs = vec![
    //         (0, MutationCommand::PUT(PutCommand { key: "a".to_owned(), value: serde_json::json!({ "a": "doggie" })})),
    //         (0, MutationCommand::PUT(PutCommand { key: "a".to_owned(), value: serde_json::json!({ "a": "cat" })})),
    //         (1, MutationCommand::PUT(PutCommand { key: "a".to_owned(), value: serde_json::json!({ "a": "doggie" })})),
    //     ];
    //     node.set_logs(&logs).unwrap();
    //     info!("node: {node:?}");
    // }
}
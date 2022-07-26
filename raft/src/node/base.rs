use std::fmt::Debug;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use serde::{Serialize, Deserialize};
use serde::de::DeserializeOwned;
use serde_prefix::prefix_all;
use crate::config::Config;
// use crate::storage::state::persistent::{self, State};
use crate::storage::state::raft_io::ReadWriteState;
use std::fs::File;
use anyhow::Result;
use std::env::temp_dir;
use proto::{LogEntry as ProtoLogEntry, AppendEntriesRequest, AppendEntriesResponse, HeartbeatResponse, HeartbeatRequest, VoteRequest, VoteResponse};
use log::{trace, info, error};
use hashbrown::HashMap;
use crate::rpc;
use proto::raft_rpc_server::RaftRpc;
use tokio::sync::mpsc;


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

#[derive(Debug, Clone)]
pub enum Election {
    Leader(Leader),
    Candidate(Candidate),
    Follower(Follower)
}

impl Default for Election {
    fn default() -> Self {
        Self::Follower(Follower::default())
    }
}

impl PartialEq for Election {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Leader(_), Self::Leader(_)) => true,
            (Self::Candidate(_), Self::Candidate(_)) => true,
            (Self::Follower(_), Self::Follower(_)) => true,
            _ => false
        }
    }
}

impl Eq for Election {}

#[derive(Debug)]
pub struct Raft<T> {
    /// For each server, the index of the next log entry
    /// to send to that server (initialized to leader's last log index + 1).
    pub meta: NodeMetadata,
    pub cluster: HashMap<Int, ClusterNode>,
    pub persistent_data: Arc<Mutex<PersistentState<T>>>,
    pub volatile_data: Arc<Mutex<VolatileState>>,
    pub node_state: Election,
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



impl From<Follower> for Candidate {
    fn from(_: Follower) -> Self {
        Candidate {}
    }
}
impl From<Candidate> for Follower {
    fn from(_: Candidate) -> Self {
        Follower {}
    }
}

impl From<Leader> for Follower {
    fn from(_: Leader) -> Self {
        Follower {}
    }
}

impl<T> Default for Raft<T> 
where
    T: Serialize + DeserializeOwned,
{
    fn default() -> Self {

        Self { 
            persistent_data: Arc::new(Mutex::new(PersistentState::new())), 
            meta: NodeMetadata::new(), 
            cluster: HashMap::new(),
            volatile_data: Arc::new(Mutex::new(VolatileState::default())), 
            node_state: Default::default(),
        }
    }
}


impl<T> Raft<T> 
where
    T: Serialize + DeserializeOwned,
{
    pub fn new() -> Self {
        Default::default()
    }

    pub fn promote(self) -> Self {

        let node_state = match self.node_state {

            Election::Follower(node) => {
                Election::Candidate(node.into())
            },
            Election::Candidate(_) => {
                Election::Leader(Leader {
                    neighbor_data: Arc::new(Mutex::new(self.cluster.clone().into()))
                })
            },
            Election::Leader(candidate) => {
                panic!("A leader {candidate:?} cannot be promoted any further.");
            }
        };

        Self {
            persistent_data: self.persistent_data,
            cluster: self.cluster,
            meta: self.meta,
            volatile_data: self.volatile_data,
            node_state
        }

    }

    pub fn demote(self) -> Self {
        let node_state = match self.node_state {
            Election::Follower(node) => {
                panic!("A follower {node:?} cannot be promoted any further.");
            },
            Election::Candidate(node) => {
                Election::Follower(node.into())
            },
            Election::Leader(node) => {
                Election::Follower(node.into())
            }
        };

        Self {
            persistent_data: self.persistent_data,
            cluster: self.cluster,
            meta: self.meta,
            volatile_data: self.volatile_data,
            node_state
        }
    }
}


impl<T> Raft<T> 
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

impl<T> Raft<T>
where
    T: Serialize + DeserializeOwned 
{
    pub fn load_state(&self) -> Result<PersistentState<T>> {
        let mut file = File::open(&self.meta.log_file)?;
        let observed_state: PersistentState<T> = file.read_state()?;
        Ok(observed_state)
    }
}


impl<T> From<Config> for Raft<T>
where
    T: Serialize + DeserializeOwned,
{
    fn from(config: Config) -> Self {

        Raft { 
            meta: NodeMetadata {id: config.id, addr: "0.0.0.0:60000".to_owned(), log_file: config.log_file.clone() }, 
            cluster: config.rafts.iter().map(|raft| (raft.id, raft.clone())).collect(), 
            persistent_data: Arc::new(Mutex::new(PersistentState::new())), 
            volatile_data: Arc::new(Mutex::new(VolatileState::new())), 
            node_state: Default::default(),
        }
    }
}


#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct State<T> {
    #[serde(flatten)]
    pub persistent: PersistentState<T>,
    #[serde(flatten)]
    pub volatile: VolatileState,
    #[cfg(feature = "monitor")]
    pub node: NodeMetadata,
}

impl<T> From<&Raft<T>> for State<T> 
where
    T: Serialize + DeserializeOwned + Clone,
{
    fn from(raft: &Raft<T>) -> Self {
        let persistent = raft.persistent_data.lock().unwrap();
        let volatile = raft.volatile_data.lock().unwrap();
        Self {
            persistent: persistent.clone(),
            volatile: volatile.clone(),
            node: raft.meta.clone()
        }
    }
}

pub trait RaftLogEntry: Clone + Send + Serialize + DeserializeOwned + std::fmt::Debug + From<Vec<u8>> {}
impl<T> RaftLogEntry for T 
where
    T: 'static + Clone + Send + Serialize + DeserializeOwned + std::fmt::Debug + From<Vec<u8>>
{}


use tokio::time::{timeout, Timeout};

impl<T> Raft<T> 
where
    T: RaftLogEntry
{
    // pub async fn start(&self) {
    //     info!("Starting the election timer of 2s for the follower.");
    //     while let Ok(recvd) = 
    //         timeout(
    //             Duration::from_secs(2), 
    //             self.channels_vote_request.lock().await.incoming_vote_request.recv()
    //         )
    //         .await 
    //     {
    //         info!("Stuff received {:#?}", recvd);
    //         self
    //             .channels_vote_request
    //             .lock()
    //             .await
    //             .outbound_vote_response
    //             .send(
    //                 VoteResponse { term: 0, vote_granted: false }
    //             )
    //             .await
    //             .unwrap()
    //         ;
    //     }
    //     error!("Didn't receive any vote request in 2s. Now should become candidate.");
    //     // let candidate: Raft<Candidate, T> = self.into();
    //     // loop {
    //     //     if let Err
    //     //     self.request_vote().await
    //     // }
    // }

    pub fn reset_election_timer(&self) {

    }
}


#[prefix_all("node_")]
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

    use super::{Raft, PersistentState, VolatileState, NodeMetadata, Follower, Candidate, Leader, LeaderVolatileState, Election};
    use state_machine::StateMachine;
    use state_machine::impls::key_value_store::*;
    use serde_json::Value;
    use std::sync::{Arc, Mutex};
    use crate::utils::test_utils::set_up_logging;
    use log::{debug, info};


    #[test]
    fn test_create_new_raft_node() {
        let node: Raft<usize> = Raft::new();

        assert_eq!(node.node_state, Election::Follower(Follower::default()));
        assert_eq!(node.meta, NodeMetadata::new());
        assert_eq!(&*node.persistent_data.lock().unwrap(), &PersistentState::new());
        assert_eq!(&*node.volatile_data.lock().unwrap(), &VolatileState::new());

    }

    #[test]
    fn transform_follower_into_candidate() {
        let node: Raft<usize> = Raft::new();
        assert_eq!(node.node_state, Election::Follower(Follower::default()));
        assert_eq!(node.meta, NodeMetadata::new());
        assert_eq!(&*node.persistent_data.lock().unwrap(), &PersistentState::new());
        assert_eq!(&*node.volatile_data.lock().unwrap(), &VolatileState::new());
        let node: Raft<usize> = node.promote();
        assert_eq!(node.node_state, Election::Candidate(Candidate::default()));
        assert_eq!(node.meta, NodeMetadata::new());
        assert_eq!(&*node.persistent_data.lock().unwrap(), &PersistentState::new());
        assert_eq!(&*node.volatile_data.lock().unwrap(), &VolatileState::new());
    }

    #[test]
    fn transform_candidate_into_leader() {
        let mut node: Raft<usize> = Raft::new();
        node = node.promote();
        assert_eq!(node.node_state, Election::Candidate(Candidate::default()));
        assert_eq!(node.meta, NodeMetadata::new());
        assert_eq!(&*node.persistent_data.lock().unwrap(), &PersistentState::new());
        assert_eq!(&*node.volatile_data.lock().unwrap(), &VolatileState::new());
        let node: Raft<usize> = node.promote();
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
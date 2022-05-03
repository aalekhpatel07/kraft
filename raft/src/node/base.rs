use std::sync::{Arc, Mutex};
use serde::Deserialize;
use serde::Serialize;
use serde::de::DeserializeOwned;
use serde_derive::{Deserialize, Serialize};
use crate::storage::state::persistent;
use crate::storage::state::persistent::LogEntry;
use crate::storage::state::persistent::LogEntryImpl;
use crate::storage::state::volatile;
use crate::storage::state::raft_io::ReadWriteState;
use std::path::Path;
use std::fs::File;
use anyhow::Result;
use std::env::temp_dir;
use proto::raft::{LogEntry as ProtoLogEntry};


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

impl<T> From<ProtoLogEntry> for LogEntryImpl<T> 
where
    T: Clone + From<Vec<u8>>
{
    fn from(entry: ProtoLogEntry) -> Self {
        LogEntryImpl { term: entry.term as usize, command: entry.command.clone().into() }
    }
}

impl<T> From<LogEntryImpl<T>> for ProtoLogEntry 
where
    T: Clone + Into<Vec<u8>>
{
    fn from(entry: LogEntryImpl<T>) -> Self {
        Self {
            term: entry.term() as u64,
            command: entry.command().clone().into()
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Node<L> 
where
    L: LogEntry + Clone
{
    pub node_type: NodeType,
    pub meta: NodeMetadata,
    pub persistent_state: Arc<Mutex<persistent::State<L>>>,
    pub volatile_state: Arc<Mutex<volatile::VolatileState>>
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct NodeMetadata {
    pub id: usize,
    pub addr: String,
    pub log_file: String
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

impl<L> Node<L>
where
    L: LogEntry + Clone + Serialize + DeserializeOwned
{
    pub fn save(&self) -> Result<usize> {
        let state = self.persistent_state.lock().unwrap();
        
        let mut file = File::create(&self.meta.log_file)?;

        let bytes_written = file
            .write_state(&state.clone())
            .expect("Could not write persistent state.");
        Ok(bytes_written)
    }

    pub fn leader_commit_index(&self) -> Option<usize> {
        match &*self.volatile_state.lock().unwrap() {
            volatile::VolatileState::Leader(state) => {
                Some(state.commit_index)
            },
            volatile::VolatileState::NonLeader(_) => {
                None
            }
        }
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
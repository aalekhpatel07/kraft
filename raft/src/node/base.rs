use std::sync::{Arc, Mutex};
use serde::Deserialize;
use serde::Serialize;
use serde::de::DeserializeOwned;
use serde_derive::{Deserialize, Serialize};
use crate::storage::state::persistent;
use crate::storage::state::volatile;
use crate::storage::state::raft_io::ReadWriteState;
use std::path::Path;
use std::fs::File;
use anyhow::Result;


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


#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Node<L: Clone> {
    pub node_type: NodeType,
    pub meta: NodeMetadata,
    pub persistent_state: Arc<Mutex<persistent::State<L>>>,
    pub volatile_state: Arc<Mutex<volatile::VolatileState>>
}

#[derive(Debug, Default, Serialize, Deserialize, Clone)]
pub struct NodeMetadata {
    pub id: usize,
    pub addr: String,
    pub log_file: String
}

impl<L: serde::Serialize + DeserializeOwned + Clone> Node<L> {
    pub fn save(&self) -> Result<usize> {
        let state = self.persistent_state.lock().unwrap();
        
        let mut file = File::create(&self.meta.log_file)?;

        let bytes_written = file
            .write_state(&state.clone())
            .expect("Could not write persistent state.");
        Ok(bytes_written)
    }
}
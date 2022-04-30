use rmp_serde::{Serializer, Deserializer};
use serde_derive::{Deserialize, Serialize};


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
pub struct Node {
    pub node_type: NodeType,
    pub id: usize
}
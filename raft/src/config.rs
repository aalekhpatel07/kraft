use std::io::Read;
use std::path::Path;
use anyhow::Result;
use crate::node::ClusterNode;

/// The configuration that provides the information about a particular Raft node in a cluster.
#[derive(Debug, Clone, serde_derive::Deserialize, serde_derive::Serialize)]
pub struct Config {
    pub rafts: Vec<ClusterNode>,
    pub id: u64,
    pub log_file: String
}

impl Config {
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self> {
        let mut file = std::fs::File::open(path)?;
        let mut buffer = vec![];
        file.read_to_end(&mut buffer)?;
        Ok(toml::from_slice(&buffer)?)
    }
}

impl Default for Config {
    fn default() -> Self {
        Self::load("/etc/raft/config.toml").unwrap()
    }
}
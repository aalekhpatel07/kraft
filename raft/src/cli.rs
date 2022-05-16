// use std::net::SocketAddr;

use clap::Parser;

#[derive(Parser, Debug)]
#[clap(author = "Aalekh Patel", version, about = "A raft implementation.", long_about = None)]
pub struct Args {
    /// The ID of this node.
    #[clap(long)]
    pub id: usize,

    /// The port to use for server.
    #[clap(short, long)]
    pub port: u16,

    /// The remote nodes in this topology.
    #[clap(long)]
    pub remote_node: Vec<String>,

    /// The path to the log file.
    #[clap(long, default_value = "/tmp/raft_log.gz")]
    pub log_file: String,

    #[clap(long, default_value = "/etc/raft/config.toml")]
    pub config: String
    
}

use std::net::SocketAddr;

use clap::Parser;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
pub struct Args {
    /// The ID of this node.
    #[clap(long)]
    pub id: usize,

    /// The port to use for server.
    #[clap(short, long)]
    pub port: String,

    /// The remote nodes in this topology.
    #[clap(long)]
    pub remote_nodes: Vec<String>
}
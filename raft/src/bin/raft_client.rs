use std::io::Write;
use std::ops::{Deref, DerefMut};
use std::str::FromStr;
use std::sync::atomic::AtomicU64;
use std::{net::{TcpStream, SocketAddr}};
use std::process::exit;

use clap::Parser;
use hashbrown::HashMap;
use raft::utils::io::input;
use serde::ser::Error;
use serde_derive::Deserialize;
use state_machine::impls::key_value_store;
use std::{thread, io};
use crossbeam_channel::{unbounded, Receiver, Sender};
use log::{trace, debug, warn, info, error};
use raft::utils::test_utils::set_up_logging;
use state_machine::impls::key_value_store::{
    MutationCommand,
    PutCommand,
    QueryCommand,
    GetCommand,
    DeleteCommand,
    Command
};
use anyhow::Result;
use raft::node::{Int, Raft};


#[derive(Parser, Debug)]
#[clap(
    name = "raft_client",
    author = "Aalekh Patel", 
    version = "0.1.0", 
    about = "A client to issue commands to the state machine managed by the Raft cluster.", 
    long_about = None,
    override_help = "Stuff"
)]
pub struct Args {
    /// One of the hosts of the raft cluster.
    #[clap(long)]
    pub node: SocketAddr,

}

pub type NodeId = (Int, SocketAddr);


// #[derive(Debug, Clone, serde_derive::Serialize, serde_derive::Deserialize)]
// pub struct Client {
//     pub node: NodeId,
// }

pub fn exit_gracefully() {
    debug!("Closing the connection to raft cluster... Exiting raft client.");
    exit(0);
}


pub fn process(stream: &mut TcpStream, s: &str)
{
    match s {
        "exit" | "quit" => { exit_gracefully() },
        _ => {}
    };

    if let Ok(cmd) = TryInto::<Command<String, serde_json::Value>>::try_into(s) {

        debug!("About to write \"{cmd:?}\" to stream: {stream:?}");
        let mut cmd_serialized: Vec<u8> = cmd.into();
        cmd_serialized.extend("\n".as_bytes());
        
        stream.write_all(&cmd_serialized).expect("Couldn't write to stream");
    }
    else {
        error!("Welp! The following command is not recognized by the state machine: \"{s}\"");
    }
}


pub fn main() -> Result<(), Box<dyn std::error::Error>> {



    set_up_logging();

    let lines = input();

    let args = Args::parse();
    println!("{args:?}");

    let node = args.node;
    let mut stream = TcpStream::connect(&node).expect(&format!("Couldn't connect to {node:?}"));

    for result in lines {
        let inp = result.trim();
        process(&mut stream, inp);
        info!("Processed {inp}");
    }

    Ok(())
}
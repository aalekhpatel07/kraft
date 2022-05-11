use std::io::Write;
use std::ops::{Deref, DerefMut};
use std::str::FromStr;
use std::sync::atomic::AtomicU64;
use std::{net::SocketAddr, collections::BTreeSet};
use std::sync::{Arc, Mutex};

use clap::Parser;
use raft::utils::io::input;
use serde::ser::Error;
use serde_derive::Deserialize;
use state_machine::impls::key_value_store;
use tokio::io::AsyncWriteExt;
// use tokio::io::AsyncWriteExt;
use std::{thread, io};
use crossbeam_channel::{unbounded, Receiver, Sender};
use log::{trace, debug, warn, info, error};
use raft::utils::test_utils::set_up_logging;
use state_machine::impls::key_value_store::{
    MutationCommand,
    PutCommand,
    QueryCommand,
    GetCommand,
    DeleteCommand
};
use tokio::net::TcpStream;
use anyhow::Result;
use raft::node::{Int, RaftNode};


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


// pub async fn setup_stream(node: &SocketAddr) -> TcpStream {
//     TcpStream::connect(node).await.expect(&format!("Couldn't connect to {node:?}"))
// }

#[derive(Debug, Clone, serde_derive::Serialize, serde_derive::Deserialize)]
pub enum Command<K, V> {
    GET(GetCommand<K>),
    PUT(PutCommand<K, V>),
    DELETE(DeleteCommand<K>)
}


impl<K, V> TryFrom<&str> for Command<K, V>
where
    K: FromStr,
    K::Err: std::fmt::Debug,
    V: FromStr,
    V::Err: std::fmt::Debug
{
    type Error = key_value_store::parse::KeyValueError<'static, K::Err, V::Err>;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        if let Ok(PutCommand { key, value: val }) = value.try_into() {
            Ok(Command::PUT(PutCommand {key, value: val}))
        } else if let Ok(DeleteCommand { key }) = value.try_into() {
            Ok(Command::DELETE(DeleteCommand { key }))
        } else if let Ok(GetCommand { key }) = value.try_into() {
            Ok(Command::GET(GetCommand { key }))
        } else {
            Err(Self::Error::CommandError(key_value_store::parse::ParseError { error: "Could not parse given value into a valid command." }))
        }
    }
}


impl<K, V> From<Vec<u8>> for Command<K, V> 
where
    K: serde::de::DeserializeOwned,
    V: serde::de::DeserializeOwned
{

    fn from(serialized: Vec<u8>) -> Self {
        let data: Command<K, V> = serde_json::from_slice(&serialized).expect("Couldn't deserialize into a Command.");
        data
    }
}


impl<K, V> Into<Vec<u8>> for Command<K, V> 
where
    K: serde::ser::Serialize,
    V: serde::ser::Serialize
{

    fn into(self) -> Vec<u8> {
        let serialized = serde_json::to_string(&self).expect("Couldn't serialize Command.");
        serialized.into_bytes()
    }
}




pub async fn process(stream: Arc<tokio::sync::Mutex<tokio::net::TcpStream>>, s: String)
{

    if let Ok(cmd) = TryInto::<Command<String, serde_json::Value>>::try_into(s.as_str()) {

        stream.lock().await.writable().await.expect("Couldn't get writable stream.");

        debug!("Writing \"{cmd:?}\" to stream: {stream:?}");

        let mut cmd_serialized: Vec<u8> = cmd.into();
        cmd_serialized.extend("\n".as_bytes());
        
        let mut stream_guard = stream.lock().await;

        match stream_guard.write_all(&cmd_serialized).await {
            Ok(()) => {
                let bytes_written = cmd_serialized.len();
                trace!("Written ({bytes_written}) bytes for \"{s}\"");
            },
            Err(err) => {
                warn!("Some error occurred while trying to write {s}.\n {err}");
            }
        };
        drop(stream_guard);
    }
    else {
        error!("Welp! The following command is not recognized by the state machine: \"{s}\"");
    }

    // let cmd = match &s.try_into() {
    //     Ok(Command::GET(cmd)) => {},
    //     Ok(Command::DELETE(cmd)) => {},
    //     Ok(Command::PUT(cmd)) => {}
    // };

    // let cmd = s.try_into().expect("Couldn't convert string into command.");

    // match cmd {
    //     PutCommand { key, value } => {},
    //     GetCommand { key } => {}
    // }
    // match s.try_into() {
    //     Ok(PutCommand { key, value }) => {},
    //     Ok(GetCommand { key }) => {},
    //     Ok(DeleteCommand { key }) => {},
    //     Ok(PutCommand { key, value }) => {},

    // }
    // if let Ok(put_cmd: PutCommand<String, serde_json::Value>) = s.try_into() {

    // }
    // stream.writable().await.expect("Couldn't get writable stream.");

    // let cmd: S::MutationCommand = ;
    // let cmd = match FromStr::from_str(&s) {
    //     S::MutationCommand(mutation_cmd) => {

    //     },
    //     S::QueryCommand(query_cmd) => {

    //     }
    // };


}

pub async fn accept(s: String) -> Result<(), std::io::Error> {

    Ok(())
}



#[tokio::main]
pub async fn main() -> Result<(), Box<dyn std::error::Error>> {
    set_up_logging();

    let lines = input();

    let args = Args::parse();
    println!("{args:?}");

    let node = args.node;
    let stream = Arc::new(tokio::sync::Mutex::new(TcpStream::connect(&node).await.expect(&format!("Couldn't connect to {node:?}"))));

    for result in lines {
        let res = result.clone();
        let stream = stream.clone();

        tokio::spawn(async move {
            process(stream, result.trim().to_owned()).await;
            info!("Processed {res}");
        });
    }
    // drain(prints);
    // let (tx, rx) = crossbeam_channel::unbounded();


    // handle_message(rx, &std::io::stdin);

    Ok(())
}
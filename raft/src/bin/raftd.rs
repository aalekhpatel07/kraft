use std::net::SocketAddr;

use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use log::{trace, debug, info, warn};
use raft::utils::test_utils::set_up_logging;
use raft::node::RaftNode;
use state_machine::impls::key_value_store::*;
use serde_json::Value;
use tokio::io::BufReader;
use tokio::io::AsyncBufReadExt;


pub async fn process_client(socket: TcpStream) {
    let mut stream = BufReader::new(socket);

    let mut line = String::new();

    while let Ok(bytes_read) = stream.read_line(&mut line).await {
        if bytes_read == 0 {
            trace!("End of input from socket. line: {line:?}, stream: {stream:?}");
            return;
        } 
        else {
            trace!("Read {bytes_read} into buffer: {line:?}");
            info!("Should process: {line:?}");
            // stream.write_all(line.as_bytes()).await.expect("Couldn't echo back.");
            line.clear();
        }
    }
    // else {
    //     warn!("Couldn't read stream: {stream:?}, line: {line:?}");
    // }

    // while let Ok(bytes_read) = stream.read_line(&mut line).await {
        // if bytes_read == 0 {
        //     trace!("End of input from socket.");
        //     break
        //     // vreturn
        // }
        // else {
        //     trace!("Read {bytes_read} into buffer: {line:?}");
        //     info!("Should process: {line:?}");
        //     line.clear();
        // }
    // }

}

pub async fn start_server(addr: SocketAddr) -> tokio::task::JoinHandle<()> {

    let listener = TcpListener::bind(addr).await.unwrap();
    info!("Starting raft server at {addr} ...");
    tokio::spawn(async move {
        loop {
            let (socket, peer) = listener.accept().await.expect("Couldn't accept socket connection.");
            trace!("Received connection from {peer}");
            tokio::spawn(async move {
                process_client(socket).await;
            });
        }
    })
}

#[tokio::main]
pub async fn main() {
    set_up_logging();

    let server_handle = start_server("127.0.0.1:60000".parse().expect("Couldn't parse into SocketAddr.")).await;

    let mut node: RaftNode<KeyValueStore<String, Value>> = RaftNode::default();
    node.meta.addr = "127.0.0.1:60001".to_owned();
    node.meta.id = 1;



    server_handle.await.expect("Couldn't join server handle.");
    // tokio::join(server_handle);
    // loop {
    //     tokio::spawn(async move {
    //         node.
    //     });
    // }
}
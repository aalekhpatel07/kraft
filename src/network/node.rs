use std::{sync::Arc, net::{SocketAddr}};

use rmp_serde::Serializer;
use serde::Serialize;
use tokio::{net::{TcpStream, TcpListener}, io::AsyncReadExt, io::AsyncWriteExt};
use std::sync::Mutex;
use crate::rpc::{self, RPCRequest, *};
use crate::election::PersistentState;
use log::{info, warn, error, debug, trace};


pub async fn process(server: Arc<tokio::sync::Mutex<Server>>, stream: &mut TcpStream) {
    let mut buffer: Vec<u8> = vec![];

    while let Ok(size) = stream.read_buf(&mut buffer).await {

        if size == 0 {
            return;
        }

        let result = rmp_serde::decode::from_slice::<rpc::RPCRequest>(&buffer);
        

        match result {
            Err(err) => {
                error!("Couldn't understand rpc request. {:?}", err);
            },
            Ok(rpc) => {
                match rpc {
                    RPCRequest::RequestVote(request_vote_rpc) => {
                        debug!("Received RequestVoteRPC {:?}", request_vote_rpc);
                        
                        // Let's keep this RPC process blocking.                        
                        // Its scary to hold locks across await.
                        // I don't think I'm bold enough yet.
                        let response = request_vote::process(server.clone(), request_vote_rpc);
                        let wrapped_response = RPCResponse::RequestVote(response);
                        
                        debug!("Response to send: {:?}", wrapped_response);

                        let mut buf = Vec::new();
                        wrapped_response.serialize(&mut Serializer::new(&mut buf)).unwrap();
                        stream.write_all(&mut buf).await.unwrap();

                    },

                    RPCRequest::AppendEntries(append_entries_rpc) => {
                        debug!("Received AppendEntriesRPC {:?}", append_entries_rpc);

                        // Let's keep this RPC process blocking.                        
                        // Its scary to hold locks across await.
                        // I don't think I'm bold enough yet.
                        let response = append_entries::process(server.clone(), append_entries_rpc);
                        let wrapped_response = RPCResponse::AppendEntries(response);
                        
                        debug!("Response to send: {:?}", wrapped_response);

                        let mut buf = Vec::new();
                        wrapped_response.serialize(&mut Serializer::new(&mut buf)).unwrap();
                        stream.write_all(&mut buf).await.unwrap();

                    },
                    _ => {
                        error!("Couldn't identify the RPC: {:?}", rpc);
                    }
                }
            }
        }
        buffer.clear();
    }
}




#[derive(Debug, Default, Clone)]
pub struct Server {
    pub port: String,
    pub id: usize,
    pub state: Arc<tokio::sync::Mutex<PersistentState>>,
    pub remote_nodes: Vec<(usize, SocketAddr)>,
}

impl Server {
    
    pub fn new(port: &str, id: usize, remote_nodes: Vec<(usize, SocketAddr)>) -> Self {
        Self {
            port: port.to_string(),
            id,
            state: Arc::new(tokio::sync::Mutex::new(PersistentState::default())),
            remote_nodes,
        }
    }

    pub fn get_socket_addr(&self, id: usize) -> SocketAddr {
        let &(_, socket_addr) = self.remote_nodes.iter().find(|(node_id, socket_addr)| *node_id == id).unwrap();
        socket_addr.clone()
    }

    pub async fn start_server(server: Arc<tokio::sync::Mutex<Server>>) {
        let server_guard = server.lock().await;

        let listener =
            TcpListener::bind(format!("0.0.0.0:{}", &server_guard.port))
            .await
            .unwrap();

        debug!("TCP Server listening on: {}", &server_guard.port);

        loop {
            let (mut stream, _) = listener.accept().await.unwrap();

            tokio::spawn({
                let me = Arc::clone(&server);
                async move {
                    process(me, &mut stream).await
                }
            });
        }
    }


}


#[cfg(test)]
mod tests {
    // use crate::network::node::*;

    // #[ignore = "Requires netcat (or another external TCPServer). Run `nc 127.0.0.1 9000 -l` before this test."]
    // #[tokio::test]
    // async fn test_connect() {

    //     dbg!("Run `nc 127.0.0.1 9000 -l` and receive messages from the TCPClient.");
    //     let node = TcpNode::new(1, "127.0.0.1", 9000);
    //     node.connect(&node.addr).await;
    // }

    // #[ignore = "Requires netcat (or another external TCPClient). Run `nc 127.0.0.1 9001` before this test."]
    // #[tokio::test]
    // async fn test_listen() {
    //     dbg!("Run `nc 127.0.0.1 9001` and send messages to see the TCPServer logging the messages.");
    //     let node = TcpNode::new(1, "127.0.0.1", 9001);
    //     node.listen().await;
    // }
}
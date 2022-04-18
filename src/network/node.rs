use std::{sync::Arc, net::{SocketAddr}, time::Duration};

use rmp_serde::Serializer;
use serde::Serialize;
use tokio::{net::{TcpStream, TcpListener}, io::AsyncReadExt, io::AsyncWriteExt};
use crate::{rpc::{self, RPCRequest, *}, election::VolatileState};
use crate::election::PersistentState;
use log::{error, debug, trace};
use std::fs::File;
use crate::storage::persistent_state::ReadWritePersistentState;
use futures::future::{Abortable, AbortHandle, AbortRegistration};
use crate::rpc::heartbeat;


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
                        trace!("Server state before processing RPC: {:?}", server.clone());
                        
                        let response = request_vote::process(server.clone(), request_vote_rpc).await;
                        let wrapped_response = RPCResponse::RequestVote(response);
                        
                        debug!("Response to send: {:?}", wrapped_response);
                        debug!("Server state after processing RequestVoteRPC: {:?}", server.clone());

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
                    RPCRequest::Heartbeat(heartbeat_rpc) => {
                        debug!("Received HeartbeatRPC {:?}", heartbeat_rpc);

                        let response = heartbeat::process(server.clone(), heartbeat_rpc);
                        let wrapped_response = RPCResponse::Heartbeat(response);

                        debug!("Response to send: {:?}", wrapped_response);

                        let mut buf = Vec::new();
                        wrapped_response.serialize(&mut Serializer::new(&mut buf)).unwrap();
                        stream.write_all(&mut buf).await.unwrap();

                    }
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
    pub port: u16,
    pub id: usize,
    pub state: Arc<tokio::sync::Mutex<PersistentState>>,
    pub volatile_state: Arc<tokio::sync::Mutex<VolatileState>>,
    pub remote_nodes: Vec<(usize, SocketAddr)>,
    pub storage_path: String
}

impl Server {
    
    pub fn new(port: u16, id: usize, remote_nodes: Vec<(usize, SocketAddr)>, storage_path: &str) -> Self {
        Self {
            port,
            id,
            state: Arc::new(tokio::sync::Mutex::new(PersistentState::default())),
            volatile_state: Arc::new(tokio::sync::Mutex::new(VolatileState::default())),
            remote_nodes,
            storage_path: storage_path.to_string()
        }
    }

    pub fn get_socket_addr(&self, id: usize) -> SocketAddr {
        let &(_, socket_addr) = self.remote_nodes.iter().find(|(node_id, _)| *node_id == id).unwrap();
        socket_addr
    }

    pub async fn save_state(&self) -> Result<usize, std::io::Error> {
        let mut storage = File::create(&self.storage_path)?;
        let state_guard = self.state.lock().await;
        let bytes_written = storage.write_persistent_state(&state_guard)?;
        // drop(state_guard);
        Ok(bytes_written)
    }

    pub async fn load_state(&self) -> Result<PersistentState, std::io::Error> {

        let mut storage = File::open(&self.storage_path)?;
        let mut state_guard = self.state.lock().await;
        let state = storage.read_persistent_state()?;
        *state_guard = state;
        // drop(state_guard);
        Ok(state_guard.clone())
    }

    pub async fn start_server(server: Arc<tokio::sync::Mutex<Server>>) {
        let server_guard = server.lock().await;

        let loaded_state = server_guard.load_state().await;
        
        // Load the persistent state if the log file is available.
        if loaded_state.is_err() {
            let mut state = server_guard.state.lock().await;
            *state = PersistentState::default();
            debug!("Could not load state from log file: {:?}\n Initializing state to: {:?}", server_guard.storage_path, state);
        } else {
            debug!("Loaded state: {:?}, from log file: {:?}", loaded_state.unwrap(), server_guard.storage_path);
        }

        let listener =
            TcpListener::bind(format!("0.0.0.0:{}", &server_guard.port))
            .await
            .unwrap();

        debug!("TCP Server listening on: {}", &server_guard.port);
        drop(server_guard);
        trace!("server_state before process loop: {:?}", server);

        loop {
            let (mut stream, _) = listener.accept().await.unwrap();

            tokio::spawn({
                let me = Arc::clone(&server);

                async move {
                    trace!("server_state right before process call: {:?}", me.clone());
                    process(me, &mut stream).await
                }
            });
        }
    }

    pub async fn send_heartbeat(server: Arc<tokio::sync::Mutex<Server>>) {
        let heartbeat_request = heartbeat::create(server.clone());

        let server_guard = server.lock().await;
        let remote_nodes = server_guard.remote_nodes.clone();

        drop(server_guard);

        for (node_id, node_addr) in remote_nodes {
            tokio::spawn({
                async move {
                    if let Ok(mut stream) = TcpStream::connect(node_addr).await {
                        
                        let mut buf = Vec::new();
                        RPCRequest::Heartbeat(heartbeat_request.clone()).serialize(&mut Serializer::new(&mut buf)).unwrap();
                        stream.write_all(&mut buf).await.unwrap();

                        trace!("Heartbeat {:?} sent to Node ID: {:?} at {:?}", heartbeat_request.clone(), node_id, node_addr);
                    }
                }
            }).await.unwrap();
        }
        // let (abort_handle, abort_registration) = AbortHandle::new_pair();
        // let future = Abortable::new(async { 2 }, abort_registration);
    }

    pub async fn send_heartbeats(server: Arc<tokio::sync::Mutex<Server>>, duration: Duration) {
        let mut interval = tokio::time::interval(duration);
        let heartbeat_loop = async move {
            loop {
                interval.tick().await;
                tokio::spawn({
                    let server = server.clone();
                    async move {
                        Server::send_heartbeat(server).await;
                    }
                }).await.unwrap();
            }
        };

        heartbeat_loop.await;
    }

}


#[cfg(test)]
mod tests {

    use super::*;
    use log::info;
    use simple_logger::SimpleLogger;
    use std::time::Duration;

    /// Test that we can save persistent state from the server API.
    #[tokio::test]
    async fn test_save_state() {

        SimpleLogger::new().init().unwrap();

        let server = Server::new(
            9000,
            1,
            vec![], 
            "/tmp/save_state.gz"
        );

        let state_guard = server.state.lock().await;
        info!("State before saving: {:?}", state_guard);
        let state_before_saving = state_guard.clone();

        drop(state_guard);

        let bytes_saved = server.save_state().await.expect("Could not save state.");
        
        let state_guard = server.state.lock().await;
        info!("State after saving: {:?}", state_guard);
        let state_after_saving = state_guard.clone();
        drop(state_guard);

        assert_eq!(state_after_saving, state_before_saving);
        info!("Written {} bytes to {}", bytes_saved, server.storage_path);

        tokio::time::sleep(Duration::from_secs(3)).await;

        let server = Server::new(
            9000,
            1,
            vec![], 
            "/tmp/save_state.gz"
        );
        server.load_state().await.expect("Could not save state.");
        trace!("Waiting to lock state. Server state: {:?}", server);

        let state_guard = server.state.lock().await;

        let state_when_loaded = state_guard.clone();
        debug!("Loaded state: {:?}", state_guard);
        drop(state_guard);

        assert_eq!(state_after_saving, state_when_loaded);
    }

    /// A utility to generate data for the load_state test.
    async fn write_state() -> PersistentState {

        const OUT_FILE: &str = "/tmp/bulky_persistent_state.gz";
        let state: PersistentState = PersistentState { 
            participant_type: crate::election::Election::Candidate,
            current_term: 2,
            voted_for: Some(3),
            log: vec![("Some string".to_owned(), 0)]
        };

        let mut file = File::create(OUT_FILE).expect("Unable to create file.");
        let bytes_written = file.write_persistent_state(&state).expect("Could not write persistent state.");
        
        println!("{} bytes written to {}", bytes_written, OUT_FILE);
        state
    }

    /// Test that we can load persistent state from the server API.
    #[tokio::test]
    async fn load_state() {

        let expected = write_state().await;

        let server = Server::new(
            9000,
            1,
            vec![], 
            "/tmp/bulky_persistent_state.gz"
        );
        server.load_state().await.expect("Could not save state.");
        println!("Waiting to lock state. Server state: {:?}", server);
        let state_guard = server.state.lock().await;

        println!("Loaded state: {:?}", state_guard);
        assert_eq!(expected, state_guard.clone());
        drop(state_guard);
    }

    // #[tokio::test]
    // async fn test_save_and_load_state() {
    //     save_state().await;
    //     load_state().await;

    // }

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
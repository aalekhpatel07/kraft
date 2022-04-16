use std::{sync::Arc, net::{SocketAddr}, collections::HashMap};

use clap::StructOpt;
use tokio::{net::{TcpStream, TcpListener}, io::AsyncReadExt, io::AsyncWriteExt};
use kraft::election::*;
use kraft::cli::Args;
use log::{info, warn, error, debug, trace};
use simple_logger::SimpleLogger;



async fn process(server: Arc<tokio::sync::Mutex<Server>>, stream: &mut TcpStream) {
    let mut buffer: Vec<u8> = vec![];

    while let Ok(size) = stream.read_buf(&mut buffer).await {

        if size == 0 {
            return;
        }
        match rmp_serde::decode::from_slice(&buffer) {
            Ok::<RequestVoteRPCRequest, _>(request) => {
                debug!("Received RequestVote RPC: {:?}", &request);
                let server_guard = server.lock().await;
                let mut server_state = server_guard.state.lock().await;

                trace!("Server state: {:?}", &server_state);
                
                // let state_guard = server.state.lock().unwrap();
                
                

            },
            Err(err) => {
                error!("could not decode data: {:?}", err);
            }
        }

        buffer.clear();

        // let mut fizz_buzz_counter: FizzBuzzCounter = 
        // rmp_serde::decode::from_slice(&buffer).unwrap();
        // fizz_buzz_counter.print();

        // buffer.clear();

        // fizz_buzz_counter.value += 1;
        // fizz_buzz_counter.time_stamp = SystemTime::now();

        // let mut buf = Vec::new();
        // fizz_buzz_counter.serialize(&mut Serializer::new(&mut buf)).unwrap();

        // stream.write_all(&buf).await.unwrap();
        // let mut deserializer = Deserializer::new();

        // println!("Read {} bytes: {:?}", size, &buffer);
    }

}

#[derive(Debug, Default, Clone)]
pub struct Server {
    pub port: String,
    pub id: usize,
    pub state: Arc<tokio::sync::Mutex<PersistentState>>,
    pub remote_nodes: Vec<(usize, SocketAddr)>,
    pub streams: HashMap<usize, Arc<tokio::sync::Mutex<Option<TcpStream>>>>
}

impl Server {
    
    pub fn new(port: &str, id: usize, remote_nodes: Vec<(usize, SocketAddr)>) -> Self {
        Self {
            port: port.to_string(),
            id,
            state: Arc::new(tokio::sync::Mutex::new(PersistentState::default())),
            remote_nodes,
            streams: HashMap::new()
        }
    }

    pub fn get_socket_addr(&self, id: usize) -> SocketAddr {
        let &(_, socket_addr) = self.remote_nodes.iter().find(|(node_id, socket_addr)| *node_id == id).unwrap();
        socket_addr.clone()
    }

    pub async fn connect(server: Arc<tokio::sync::Mutex<Server>>, id: usize, node: SocketAddr) {
        
        let server = server.clone();

        // tokio::spawn(async move {
            let stream = TcpStream::connect(node).await.ok();
            trace!("Started waiting on server_guard.");
            trace!("server: {:?}", server);
            let mut guard = server.lock().await;
            trace!("Finished waiting on server_guard.");
            
            let hmap = &mut guard.streams;

            if hmap.contains_key(&id) {

                let prev_value = hmap.get_mut(&id).unwrap();
                trace!("Waiting for prev_stream lock");
                let mut prev_stream = prev_value.lock().await;

                // Only persist connection if not already exists.
                if prev_stream.is_none() {
                    trace!("Updating a TCPStream for Node: {} at {:?}", id, node);
                    *prev_stream = stream;
                    trace!("Updated prev_stream = {:?}", prev_stream);
                }

            } else {
                trace!("Storing a new TCPStream for Node: {} at {:?}", id, node);
                hmap.insert(id, Arc::new(tokio::sync::Mutex::new(stream)));
                trace!("Updated hmap = {:?}", hmap);
            }

            drop(guard);
        // });
    }

    pub async fn send(server: Arc<tokio::sync::Mutex<Server>>, id: usize, data: &[u8]) {
        trace!("Invoking Server::send for Node: {}, with {} bytes.", id, data.len());        
        let server_guard = server.lock().await;
        let hmap = &server_guard.streams;
        let peer_addr = server_guard.get_socket_addr(id);
        if let Some(stream_record) = hmap.get(&id) {
            let mut guard = stream_record.lock().await;

            trace!("Attempting to send {:?} bytes to Node: {:?} at {:?}", data.len(), id, &peer_addr);     
            
            if let Some(stream) = guard.as_mut() {
                stream.write_all(data).await.expect("Could not write to stream.");
                trace!("Sent {:?} bytes to Node: {:?} at {:?}", data.len(), id, &peer_addr);
            } else {
                error!("No TCP connection open to Node: {:?} at {:?}", id, &peer_addr);
            }
            // let stream = guard.as_mut().expect("Stream should exist.");
            drop(guard);
        } else {
            
            // Server::connect(server.clone(), id, server_guard.get_socket_addr(id)).await;
        }
        drop(server_guard);
    }

    pub async fn start_server(server: Arc<tokio::sync::Mutex<Server>>) {
        let server_guard = server.lock().await;

        let listener =
            TcpListener::bind(format!("0.0.0.0:{}", &server_guard.port))
            .await
            .unwrap();

        debug!("TCP Server listening on: {}", &server_guard.port);
        trace!("Server streams: {:?}", &server_guard.streams);

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

    pub async fn start_client(server: Arc<tokio::sync::Mutex<Server>>) {

        trace!("In start_client guard assignment: server: {:?}", server);
        let guard = server.lock().await;

        let nodes = guard.remote_nodes.clone();
        
        // It is kinda weird that guard isn't automatically dropped here.
        // Without this explicit drop, there the mutex is never unlocked
        // and then freezes on the following attempts at acquiring the lock.
        // I thought since guard's life ends here, it might be auto-dropped
        // but I guess not.
        drop(guard);

        trace!("After start_client drop guard: server {:?}", server);

        for (id, socket_addr) in nodes {
                let srvr = server.clone();
                trace!("Establishing connection to client: {} at {:?}", id, socket_addr);

                // Each Server::connect call mutates the server state. So probably just run it concurrently, instead of fully parallel.
                Server::connect(srvr.clone(), id, socket_addr).await;
                Server::send(srvr.clone(), id, b"stuffstuff").await;
        }
    }

}

fn parse_socket_and_id(s: &String) -> (usize, SocketAddr) {
    let (socket_addr, id) = s.split_once(',').unwrap();
    (id.parse::<usize>().unwrap(), socket_addr.parse::<SocketAddr>().unwrap())
}

#[tokio::main]
async fn main() {
    SimpleLogger::new().init().unwrap();
    let args: Args = Args::parse();

    let remote_nodes = 
    args
    .remote_node
    .iter()
    .map(parse_socket_and_id)
    .collect::<Vec<(usize, SocketAddr)>>();

    let server = {
        Arc::new(tokio::sync::Mutex::new(Server::new(&args.port, args.id, remote_nodes)))
    };

    // Start clients asynchronously.
    // debug!("Just before spawn start_client.");
    // tokio::spawn( {
        // let server = server.clone();
        // async move {
            Server::start_client(server.clone()).await;
        // }
    // });
    // debug!("After spawn start_client.");
    // tokio::spawn({
        // async move {
    Server::start_server(server.clone()).await;
        // }
    // });
    // Server::send(server.clone(), 1, b"blahblah").await;
    // Start server.


}

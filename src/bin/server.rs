use std::{sync::{Arc, Mutex}, net::{SocketAddr, SocketAddrV4}};

use clap::StructOpt;
use serde_derive::{Serialize, Deserialize};
use tokio::{net::{TcpStream, TcpListener}, io::AsyncReadExt, io::AsyncWriteExt};
use kraft::election::*;
use kraft::cli::Args;
// use std::time::SystemTime;
use rmp_serde::{Serializer};
use log::{info, warn, error, debug};
use simple_logger::SimpleLogger;



async fn process(server: Arc<Server>, stream: &mut TcpStream) {
    let mut buffer: Vec<u8> = vec![];

    while let Ok(size) = stream.read_buf(&mut buffer).await {

        if size == 0 {
            return;
        }
        match rmp_serde::decode::from_slice(&buffer) {
            Ok::<RequestVoteRPCRequest, _>(request) => {
                debug!("Received RequestVote RPC: {:?}", &request);

                let mut server_state = server.state.lock().unwrap();
                debug!("Server state: {:?}", &server_state);
                
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

#[derive(Clone, Debug, Default)]
pub struct Server {
    pub port: String,
    pub id: usize,
    pub state: Arc<Mutex<PersistentState>>,
    pub remote_nodes: Vec<SocketAddr>
}

impl Server {
    
    pub fn new(port: &str, id: usize) -> Self {
        Self {
            port: port.to_string(),
            id,
            state: Arc::new(Mutex::new(PersistentState::default())),
            remote_nodes: Vec::default()
        }
    }
    pub fn with_remote_nodes(self, remote_nodes: Vec<SocketAddr>) -> Self {

        Self {
            remote_nodes,
            port: self.port,
            id: self.id,
            state: self.state,
        }
    }

    pub async fn start(self: Arc<Self>) {
        let listener = 
            TcpListener::bind(format!("0.0.0.0:{}", &self.port))
            .await
            .unwrap();

        debug!("TCP Server listening on: {}", &self.port);

        loop {
            let (mut stream, _) = listener.accept().await.unwrap();

            tokio::spawn({
                let me = Arc::clone(&self);
                async move {
                    process(me, &mut stream).await
                }
            });
        }
    }
}


#[tokio::main]
async fn main() {
    SimpleLogger::new().init().unwrap();
    let args: Args = Args::parse();

    let server = 
        Server::new(&args.port, args.id)
            .with_remote_nodes(
                args
                .remote_nodes
                .iter()
                .map(|s| s.parse().unwrap())
                .collect::<Vec<SocketAddr>>()
            );
    Server::start(Arc::new(server)).await;
}




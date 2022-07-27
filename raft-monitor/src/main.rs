use std::fmt::Debug;
use std::ops::Deref;
use std::sync::Arc;
use std::{
    io::Error as IoError,
    net::SocketAddr,
};

use raft::rpc::diagnostic::{RPCDiagnostic, VoteRequestBody, VoteResponseBody};
use raft::node::{State};
use log::{debug, error, info, warn, trace};
use serde::{Serialize, de::DeserializeOwned, Deserialize};
use simple_logger::SimpleLogger;
use meilisearch_sdk::client::*;

use tokio::net::{TcpListener, TcpStream, UdpSocket};
use uuid::Uuid;

pub fn set_up_logging() {
    SimpleLogger::new()
        .with_level(log::LevelFilter::Debug)
        .init()
        .unwrap();
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Document<T, D> {
    pub id: String,
    #[serde(flatten)]
    pub content: RPCDiagnostic<State<T>, D>,
}

impl<T, D> Document<T, D> {
    pub fn new(content: RPCDiagnostic<State<T>, D>) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            content
        }
    }
}


pub async fn process_message<T, D>(mut data: RPCDiagnostic<State<T>, D>, len: usize, addr: SocketAddr, backend_conn: Arc<Client>) 
where
    T: Serialize + DeserializeOwned + Debug + Clone,
    D: std::fmt::Debug + Serialize + DeserializeOwned,
{
    data.state.node.addr = addr.to_string();
    info!("Data: {:#?}, Len: {:#?}\nAddr: {}", data, len, addr);

    let documents = [Document::new(data)];

    match backend_conn.index("raft_diagnostics").add_documents(&documents, Some("id")).await {
        Ok(_) => {
            info!("Document added to backend");
        }
        Err(e) => {
            error!("Error adding document to backend: {}", e);
        }
    }
    
}

pub fn connect_backend(addr: &str) -> Client {
    Client::new(addr, "masterKey")
}

#[tokio::main]
async fn main() -> Result<(), IoError> {
    set_up_logging();
    // let peer_map = PeerMap::new(Mutex::new(HashMap::new()));
    let bind_addr: String = "0.0.0.0:8000".into();
    let backend_addr: String = "http://127.0.0.1:7700".into();

    let backend_conn = Arc::new(connect_backend(&backend_addr));

    // Create the event loop and UDP listener we'll accept connections on.
    let socket = UdpSocket::bind(&bind_addr).await?;
    debug!("Listening UDP on: {}", bind_addr);

    let mut buf = [0; 1024];

    loop {
        let (len, addr) = socket.recv_from(&mut buf).await?;
        // info!("Received {} bytes from {}", len, addr);
        if let Ok(diagnostic_message) = serde_json::from_slice::<RPCDiagnostic<State<Vec<u8>>, VoteRequestBody>>(&buf[..len]) {
            tokio::spawn(
                process_message(diagnostic_message, len, addr, backend_conn.clone())
            );
        }
        else if let Ok(diagnostic_message) = serde_json::from_slice::<RPCDiagnostic<State<Vec<u8>>, VoteResponseBody>>(&buf[..len]) {
            tokio::spawn(
                process_message(diagnostic_message, len, addr, backend_conn.clone())
            );
        }
        else {
            warn!("Couldn't deserialize the message.");
        }
        // buf.clear();
    }
}

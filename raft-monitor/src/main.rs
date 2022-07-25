use std::{
    collections::HashMap,
    io::Error as IoError,
    net::SocketAddr,
    sync::{Arc, Mutex},
};

use raft_monitor::rpc::{RPCDiagnostic, DiagnosticKind};
use log::{debug, error, info, warn, trace};
use simple_logger::SimpleLogger;
use futures::{future, pin_mut, StreamExt, TryStreamExt};
use futures_channel::mpsc::{unbounded, UnboundedSender};

use tokio::net::{TcpListener, TcpStream};

// type Tx = UnboundedSender<Message>;
// type PeerMap = Arc<Mutex<HashMap<SocketAddr, Tx>>>;

pub fn set_up_logging() {
    SimpleLogger::new()
        .with_level(log::LevelFilter::Debug)
        .init()
        .unwrap();
}


pub async fn process_stream(mut stream: TcpStream, addr: SocketAddr) {
    info!("Stream: {:#?}\nAddr: {}", stream, addr);
    let (read_stream, _) = stream.split();
    
    let mut buffer = [0u8; 1024];
    while let Ok(bytes_read) = read_stream.try_read(&mut buffer) {
        if bytes_read == 0 {
            warn!("No bytes read from stream");
            return;
        }
        trace!("Bytes read: {}", bytes_read);
        if let Ok(msg) = serde_json::from_slice::<RPCDiagnostic<usize>>(&buffer) {
            info!("Message: {:#?}", msg);
        } else {
            error!("Error deserializing message.");
        }
    }
}


#[tokio::main]
async fn main() -> Result<(), IoError> {
    set_up_logging();
    // let peer_map = PeerMap::new(Mutex::new(HashMap::new()));
    let addr: String = "127.0.0.1:8000".into();

    // Create the event loop and TCP listener we'll accept connections on.
    let try_socket = TcpListener::bind(&addr).await;
    let listener = try_socket.expect("Failed to bind");
    info!("Listening on: {}", addr);

    // Let's spawn the handling of each connection in a separate task.
    while let Ok((stream, addr)) = listener.accept().await {
        tokio::spawn(process_stream(
            stream,
            addr,
        ));
    }

    Ok(())
}

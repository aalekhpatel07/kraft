use std::{sync::Arc, net::{SocketAddr}, fmt};

use clap::StructOpt;



use kraft::network::node::{Server};
use kraft::cli::Args;
use log::{debug};
use simple_logger::SimpleLogger;

#[derive(Debug)]
pub struct ParseError;


impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Could not parse the given input.")
    }
}

impl From<std::io::Error> for ParseError {
    fn from(_error: std::io::Error) -> Self {
        ParseError {}
    }
}


fn parse_socket_and_id(s: &String) -> Result<(usize, SocketAddr), ParseError> {
    s
    .split_once(',')
    .ok_or(ParseError)
    .map(|(socket_addr, id)| {
        let id = id.parse::<usize>().expect("Couldn't parse id.");
        let socket_addr = socket_addr.parse::<SocketAddr>().expect("Couldn't parse socket address.");
        (id, socket_addr)
    })
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
    .map(|x| x.unwrap())
    .collect::<Vec<(usize, SocketAddr)>>();

    let server = {
        Arc::new(
            tokio::sync::Mutex::new(
                Server::new(
                    args.port,
                    args.id, 
                    remote_nodes, 
                    &args.log_file
                )
            )
        )
    };
    debug!("Server state: {:?}", server.clone());
    Server::start_server(server.clone()).await;
}


#[cfg(test)]
mod tests {

    use std::time::Duration;

    use super::*;

    use kraft::{rpc::{RPCRequest, request_vote::{self}, RPCResponse, append_entries}};
    use rmp_serde::Serializer;
    use serde::Serialize;
    
    use log::{trace, warn};
    use futures::future::{AbortHandle, AbortRegistration, Abortable};
    use tokio::{net::{TcpStream}, io::AsyncReadExt, io::AsyncWriteExt};

    #[test]
    fn test_parse_socket_and_id() {
        let inputs: Vec<String> = vec![
            "192.168.1.113:9000,1".to_owned(),
            "192.168.0.223:9000,2".to_owned(),
            "192.168.0.223:9000,22".to_owned(),
            "192.168.0.223:9000,0".to_owned(),
        ];
        let expected_outputs = vec![
            (1, "192.168.1.113:9000".parse::<SocketAddr>().unwrap()),
            (2, "192.168.0.223:9000".parse::<SocketAddr>().unwrap()),
            (22, "192.168.0.223:9000".parse::<SocketAddr>().unwrap()),
            (0, "192.168.0.223:9000".parse::<SocketAddr>().unwrap()),
        ];

        let observed_outputs = inputs.iter().map(parse_socket_and_id).map(|x| x.unwrap()).collect::<Vec<(usize, SocketAddr)>>();
        assert_eq!(expected_outputs, observed_outputs);
    }
    
    #[test]
    fn test_parse_socket_and_id_invalid() {

        let input = "192.168.0.223:9000-0".to_owned();
        parse_socket_and_id(&input).expect_err("Some kind of error");
    }

    async fn connect() -> TcpStream {
        TcpStream::connect("192.168.1.113:9000").await.unwrap()
    }

    #[ignore = "Requires a server setup, at least until we choose to mock it."]
    #[tokio::test]
    async fn test_server_understands_request_vote_rpc() {
        let mut stream = connect().await;

        let rpc_request_vote = RPCRequest::RequestVote(
            request_vote::RequestVoteRequest {
                term: 0,
                candidate_id: 0,
                last_log_index: 0,
                last_log_term: 0
            }
        );
        
        let mut buf = Vec::new();
        rpc_request_vote.serialize(&mut Serializer::new(&mut buf)).unwrap();

        stream.write_all(&buf).await.unwrap();
        
        trace!("Waiting for readable stream.");
        stream.readable().await.unwrap();
        trace!("Finished waiting for readable stream.");

        buf.clear();

        trace!("Waiting for read_buf");
        stream.read_buf(&mut buf).await.unwrap();
        trace!("Finished Waiting for read_buf");

        let result = rmp_serde::decode::from_slice::<RPCResponse>(&buf).unwrap();

        assert!(matches!(result, RPCResponse::RequestVote(..)));
    }

    #[ignore = "Requires a server setup, at least until we choose to mock it."]
    #[tokio::test]
    async fn test_server_understands_append_entries_rpc() {
        let mut stream = connect().await;

        let rpc_append_entries = RPCRequest::AppendEntries(
            append_entries::AppendEntriesRequest::default()
        );
        
        let mut buf = Vec::new();
        rpc_append_entries.serialize(&mut Serializer::new(&mut buf)).unwrap();
        stream.write_all(&buf).await.unwrap();
        
        trace!("Waiting for readable stream.");
        stream.readable().await.unwrap();
        trace!("Finished waiting for readable stream.");

        buf.clear();

        trace!("Waiting for read_buf");
        stream.read_buf(&mut buf).await.unwrap();
        trace!("Finished Waiting for read_buf");

        let result = rmp_serde::decode::from_slice::<RPCResponse>(&buf).unwrap();
        assert!(matches!(result, RPCResponse::AppendEntries(..)));
    }

    #[ignore = "Requires a server setup, at least until we choose to mock it."]
    #[tokio::test]
    async fn test_server_sends_heartbeat() {
        SimpleLogger::new().init().unwrap();

        let server = {
            Arc::new(
                tokio::sync::Mutex::new(
                    Server::new(
                        9000,
                        0, 
                        vec![
                            // (1, "0.0.0.0:9000".parse::<SocketAddr>().unwrap()),
                            (2, "192.168.1.113:9000".parse::<SocketAddr>().unwrap()),
                        ],
                        "/tmp/storage.gz"
                    )
                )
            )
        };
        debug!("Server state: {:?}", server.clone());
        Server::send_heartbeat(server.clone()).await;
    }
    
    #[ignore = "Requires a server setup, at least until we choose to mock it."]
    #[tokio::test]
    async fn test_server_sends_heartbeats_every_second_for_10_seconds() {
        SimpleLogger::new().init().unwrap();

        let server = {
            Arc::new(
                tokio::sync::Mutex::new(
                    Server::new(
                        9000,
                        0, 
                        vec![
                            (1, "0.0.0.0:9000".parse::<SocketAddr>().unwrap()),
                            (2, "192.168.1.113:9000".parse::<SocketAddr>().unwrap()),
                        ],
                        "/tmp/storage.gz"
                    )
                )
            )
        };
        debug!("Server state before sending heartbeats: {:?}", server.clone());


        let (abort_handle, abort_registration) = AbortHandle::new_pair();
        let future = Abortable::new(Server::send_heartbeats(server, Duration::from_secs(1)), abort_registration);
        
        tokio::spawn({
            async move {
                tokio::time::sleep(Duration::from_secs(10)).await;
                abort_handle.abort();
            }
        });

        match future.await {
            Ok(_) => {
                unreachable!("This is a potentially infinite heartbeat loop which may only ever return an AbortError. ");
            },
            Err(err) => {
                warn!("Could not send heartbeat.");
                warn!("AbortError: {:?}", err);
            }
        }
    }
}

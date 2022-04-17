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

    use super::*;

    use kraft::{election::*, rpc::{RPCRequest, request_vote::{RequestVoteRequest, self}, RPCResponse, append_entries}};
    use rmp_serde::Serializer;
    use serde::Serialize;
    use std::future::Future;
    use log::{trace};

    use tokio::{net::{TcpStream, TcpListener}, io::AsyncReadExt, io::AsyncWriteExt};

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
        TcpStream::connect("192.168.1.113:9001").await.unwrap()
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
}

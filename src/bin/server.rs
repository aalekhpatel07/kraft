use std::{sync::Arc, net::{SocketAddr}, collections::HashMap, fmt, error::Error};

use clap::StructOpt;
use tokio::{net::{TcpStream, TcpListener}, io::AsyncReadExt, io::AsyncWriteExt};
use kraft::rpc::*;
use kraft::election::PersistentState;
use kraft::network::node::{Server, process};
use kraft::cli::Args;
use log::{info, warn, error, debug, trace};
use simple_logger::SimpleLogger;

#[derive(Debug)]
pub struct ParseError;


impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Could not parse the given input.")
    }
}

impl From<std::io::Error> for ParseError {
    fn from(error: std::io::Error) -> Self {
        ParseError {}
    }
}


fn parse_socket_and_id(s: &String) -> Result<(usize, SocketAddr), ParseError> {
    s
    .split_once(',')
    .ok_or(ParseError)
    .and_then(|(socket_addr, id)| {
        let id = id.parse::<usize>().expect("Couldn't parse id.");
        let socket_addr = socket_addr.parse::<SocketAddr>().expect("Couldn't parse socket address.");
        Ok((id, socket_addr))
    })
    // if let Some((socket_addr, id)) = s.split_once(',') {

        // (
        //     id.parse::<usize>().expect(format!("Couldn't parse ID (usize): {:?}", id).as_str()), 
        //     socket_addr.parse::<SocketAddr>().expect(format!("Couldn't parse socket address: {:?}", socket_addr).as_str())
        // )
    // }else {
    //     Err(std::io::ErrorKind)
    // }


    // let id = 
    

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
        Arc::new(tokio::sync::Mutex::new(Server::new(&args.port, args.id, remote_nodes)))
    };

    Server::start_server(server.clone()).await;
}


#[cfg(test)]
mod tests {

    use super::*;
    use crate::*;
    use kraft::{election::*, rpc::{RPCRequest, request_vote::RequestVoteRequest}};
    use rmp_serde::Serializer;
    use serde::Serialize;
    use std::future::Future;

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

    #[tokio::test]
    async fn test_server_understands_request_vote_rpc() {
        let mut stream = connect().await;

        let rpc_request_vote = RPCRequest::RequestVote(
            request_vote::RequestVoteRequest::new(
                0,
                0,
                0,
                0
            )
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
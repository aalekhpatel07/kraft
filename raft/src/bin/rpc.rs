use tonic::{transport::Server, Request, Response, Status};

use proto::raft::raft_server::RaftServer;
use raft::node::Node;


#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    
    let addr = "[::1]:50051".parse()?;
    let node = Node::<String>::default();

    Server::builder()
        .add_service(RaftServer::new(node))
        .serve(addr)
        .await?;

    Ok(())
}
use tonic::{transport::Server, Request, Response, Status};

use proto::raft::raft_rpc_server::RaftRpcServer;
use raft::{node::RaftNode, storage::state::persistent::Log};


#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    
    // let addr = "[::1]:50051".parse()?;
    // let node = Node::<Vec<u8>>::default();
    // let node = RaftNode::default();
    Ok(())

    // Server::builder()
    //     .add_service(RaftServer::new(node))
    //     .serve(addr)
    //     .await?;

    // Ok(())
}
use tonic::{transport::Server, Request, Response, Status};
// use proto::{raft_rcp_*};
use proto::{raft_rpc_client::RaftRpcClient, *};
use anyhow::Result;
use raft::utils::test_utils::set_up_logging;
use log::{info, trace, warn, debug, error};


#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    set_up_logging();

    let addr = "http://127.0.0.1:60000";
    let mut client = RaftRpcClient::connect(addr).await.unwrap();

    let request = Request::new(
        VoteRequest {
            term: 0,
            candidate_id: 0,
            last_log_index: 0,
            last_log_term: 0
        }
    );

    info!("Sending request: {:#?}", request);
    let response = client.request_vote(request).await?;
    info!("Received response: {:#?}", response);

    // let node = Node::<Vec<u8>>::default();
    // let node = RaftNode::default();
    Ok(())

    // Server::builder()
    //     .add_service(RaftServer::new(node))
    //     .serve(addr)
    //     .await?;

    // Ok(())
}
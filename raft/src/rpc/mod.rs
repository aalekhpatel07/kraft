use serde::{Serialize, de::DeserializeOwned, Deserialize};
use tonic::{Request, Response, Status, Code};
use crate::{node::{Raft, Follower, Log, Candidate, Leader}};

use proto::{
    HeartbeatRequest,
    HeartbeatResponse,
    VoteRequest,
    VoteResponse,
    AppendEntriesRequest,
    AppendEntriesResponse,
    raft_rpc_server::{RaftRpc}
};
mod request_vote;
mod append_entries;
mod heartbeat;
pub mod diagnostic;
use anyhow::Result;
use log::{info, error, warn};
use crate::rpc::diagnostic::{RPCDiagnostic, DiagnosticKind, RPCKind, DataKind};
use crate::node::State;
use std::net::UdpSocket;

use self::request_vote::VoteResponseBody;

#[cfg(feature = "monitor")]
pub fn report_state<T, D>(diagnostic_message: &RPCDiagnostic<State<T>, D>) 
where
    T: DeserializeOwned + Serialize + std::fmt::Debug + Clone,
    D: Serialize + DeserializeOwned
{

    let sock = UdpSocket::bind("0.0.0.0:8001").expect("Couldn't bind to UDP socket");
    let diag_serialized = serde_json::to_string(&diagnostic_message).unwrap();
    info!("Sending state to monitor: {}", diag_serialized);
    sock.send_to(diag_serialized.as_bytes(), "0.0.0.0:8000").expect("Couldn't send UDP message");
}

#[tonic::async_trait]
impl<T> RaftRpc for Raft<T> 
where
    T: 'static + Send + Clone + Serialize + DeserializeOwned + From<Vec<u8>> + std::fmt::Debug
{
    async fn append_entries(&self, request: Request<AppendEntriesRequest>) -> Result<Response<AppendEntriesResponse>, Status> {

        let request_data = request.into_inner();

        let rpc_id = uuid::Uuid::new_v4();

        #[cfg(feature = "monitor")]
        {
            let diagnostic_message = RPCDiagnostic::new(

                DiagnosticKind::ReceiverBeforeProcessing,
                DataKind::Request,
                RPCKind::AppendEntries,
                Some(append_entries::AppendEntriesRequestBody::from(&request_data)),
                self.into()
            )
            .with_id(rpc_id);
            report_state(&diagnostic_message);
        }

        let res = append_entries::append_entries(self, request_data.clone()).await;

        #[cfg(feature = "monitor")]
        {
            let diagnostic_message = RPCDiagnostic::new(

                DiagnosticKind::ReceiverAfterProcessing,
                DataKind::Response,
                RPCKind::AppendEntries,
                Some(append_entries::AppendEntriesRequestBody::from(&request_data)),
                self.into()
            )
            .with_id(rpc_id);
            report_state(&diagnostic_message);
        }

        res   
    }

    async fn heartbeat(&self, request: Request<HeartbeatRequest>) -> Result<Response<HeartbeatResponse>, Status> {
        Ok(heartbeat::heartbeat(self, request).await?)
    }
    async fn request_vote(&self, request: Request<VoteRequest>) -> Result<Response<VoteResponse>, Status> {

        let request_data = request.into_inner();
        let rpc_id = uuid::Uuid::new_v4();
        #[cfg(feature = "monitor")]
        {
            let diagnostic_message = RPCDiagnostic::new(
                DiagnosticKind::ReceiverBeforeProcessing,
                DataKind::Request,
                RPCKind::RequestVote,
                Some(request_vote::VoteRequestBody::from(&request_data)),
                self.into()
            )
            .with_id(rpc_id);
            report_state(&diagnostic_message);
        }

        let mut data: Option<VoteResponseBody> = None;

        let response = match request_vote::request_vote(self, request_data.clone()).await {
            Ok(vote_response) => {
                data = Some((&vote_response).into());
                Ok(Response::new(vote_response))
            },
            Err(err) => {
                Err(err)
            }
        };

        #[cfg(feature = "monitor")]
        {
            let diagnostic_message = RPCDiagnostic::new(
                DiagnosticKind::ReceiverBeforeProcessing,
                DataKind::Request,
                RPCKind::RequestVote,
                data,
                self.into()
            )
            .with_id(rpc_id);
            report_state(&diagnostic_message);
        }

        response
    }
}

#[cfg(test)]
pub mod tests {
    use proto::{raft_rpc_server::RaftRpc};
    use state_machine::impls::key_value_store::KeyValueStore;
    use tonic::Request;

    use super::*;
    use crate::{utils::test_utils::set_up_logging};
    use crate::node::{Raft, Follower, Leader, Candidate, Log};
    use log::{info, trace, debug};


    #[tokio::test]
    async fn test_append_entries_basic() {
        set_up_logging();
        let node: Raft<Vec<u8>> = Raft::default();
        let request = Request::new(AppendEntriesRequest::default());
        let response = node.append_entries(request).await.unwrap();

        let response = response.into_inner();
        assert_eq!(response.term, 0);
        assert_eq!(response.success, false);

        info!("response: {response:?}");
    }

    #[tokio::test]
    async fn test_heartbeat_basic() {
        set_up_logging();
        let node: Raft<Vec<u8>> = Raft::default();
        let request = Request::new(HeartbeatRequest::default());
        let response = node.heartbeat(request).await.unwrap();

        let response = response.into_inner();
        assert_eq!(response.term, 0);
        assert_eq!(response.success, false);

        info!("response: {response:?}");
    }

    #[tokio::test]
    async fn test_request_vote_basic() {
        set_up_logging();
        let node: Raft<Vec<u8>> = Raft::default();
        let request = Request::new(VoteRequest::default());
        let response = node.request_vote(request).await.unwrap();
        let response = response.into_inner();
        assert_eq!(response.term, 0);
        assert_eq!(response.vote_granted, true);
        info!("response: {response:?}");
    }
}

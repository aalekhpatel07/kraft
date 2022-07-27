use uuid::Uuid;
use std::time::SystemTime;
use serde::{Serialize, Deserialize, de::DeserializeOwned};
pub use crate::rpc::append_entries::{AppendEntriesRequestBody, LogEntryBody};
pub use crate::rpc::request_vote::{VoteRequestBody, VoteResponseBody};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DiagnosticKind {
    SenderBeforeSending,
    ReceiverBeforeProcessing,
    ReceiverAfterProcessing,
    SenderAfterReceiving
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DataKind {
    Request,
    Response
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RPCKind {
    AppendEntries,
    Heartbeat,
    RequestVote
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RPCDiagnostic<T, D> {
    pub rpc_id: Uuid,
    pub rpc_kind: RPCKind,
    pub kind: DiagnosticKind,
    pub time_stamp: SystemTime,
    pub data: Option<D>,
    pub data_kind: DataKind,
    #[serde(flatten)]
    pub state: T
}


impl<T, D> RPCDiagnostic<T, D> 
where
    T: Serialize + DeserializeOwned,
    D: Serialize + DeserializeOwned
{
    pub fn new(
        diagnostic_kind: DiagnosticKind,
        data_kind: DataKind,
        rpc_kind: RPCKind,
        data: Option<D>,
        state: T
    ) -> RPCDiagnostic<T, D> {
        RPCDiagnostic {
            rpc_id: Uuid::new_v4(),
            kind: diagnostic_kind,
            state,
            data,
            data_kind,
            rpc_kind,
            time_stamp: SystemTime::now()
        }
    }

    pub fn with_id(self, rpc_id: Uuid) -> RPCDiagnostic<T, D> {
        RPCDiagnostic {
            rpc_id,
            ..self
        }
    }
}
use uuid::Uuid;
use std::time::SystemTime;
use serde::{Serialize, Deserialize, de::DeserializeOwned};


#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DiagnosticKind {
    SenderBeforeSending,
    ReceiverBeforeProcessing,
    ReceiverAfterProcessing,
    SenderAfterReceiving
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RPCDiagnostic<T> {
    pub rpc_id: Uuid,
    pub kind: DiagnosticKind,
    pub time_stamp: SystemTime,
    #[serde(flatten)]
    pub state: T
}


impl<T> RPCDiagnostic<T> 
where
    T: Serialize + DeserializeOwned
{
    pub fn new(
        kind: DiagnosticKind,
        state: T
    ) -> RPCDiagnostic<T> {
        RPCDiagnostic {
            rpc_id: Uuid::new_v4(),
            kind,
            state,
            time_stamp: SystemTime::now()
        }
    }

    pub fn with_id(self, rpc_id: Uuid) -> RPCDiagnostic<T> {
        RPCDiagnostic {
            rpc_id,
            ..self
        }
    }

    pub fn new_sender_before_sending(state: T) -> RPCDiagnostic<T> {
        RPCDiagnostic::new(DiagnosticKind::SenderBeforeSending, state)
    }

    pub fn new_receiver_before_processing(state: T) -> RPCDiagnostic<T> {
        RPCDiagnostic::new(DiagnosticKind::ReceiverBeforeProcessing, state)
    }

    pub fn new_receiver_after_processing(state: T) -> RPCDiagnostic<T> {
        RPCDiagnostic::new(DiagnosticKind::ReceiverAfterProcessing, state)
    }

    pub fn new_sender_after_receiving(state: T) -> RPCDiagnostic<T> {
        RPCDiagnostic::new(DiagnosticKind::SenderAfterReceiving, state)
    }

}
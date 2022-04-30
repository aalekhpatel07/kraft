pub mod request_vote;
pub mod append_entries;
pub mod heartbeat;

use append_entries::*;
use request_vote::*;
use heartbeat::*;
use serde_derive::{Serialize, Deserialize};


#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RPCRequest {
    RequestVote(RequestVoteRequest),
    AppendEntries(AppendEntriesRequest),
    Heartbeat(HeartbeatRequest)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RPCResponse {
    RequestVote(RequestVoteResponse),
    AppendEntries(AppendEntriesResponse),
    Heartbeat(HeartbeatResponse)
}
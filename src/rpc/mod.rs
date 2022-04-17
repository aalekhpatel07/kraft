pub mod request_vote;
pub mod append_entries;

use append_entries::*;
use request_vote::*;
use serde_derive::{Serialize, Deserialize};


#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RPCRequest {
    RequestVote(RequestVoteRequest),
    AppendEntries(AppendEntriesRequest)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RPCResponse {
    RequestVote(RequestVoteResponse),
    AppendEntries(AppendEntriesResponse)
}
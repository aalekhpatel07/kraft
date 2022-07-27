use log::{info};
use serde::{Serialize, de::DeserializeOwned};
use crate::rpc::diagnostic::*;
use crate::node::State;
use std::net::UdpSocket;


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
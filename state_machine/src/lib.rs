//! Generic behaviour for the State Machine module used in Raft.
//! 
//! Optionally, a key-value store (Arc<Mutex<HashMap<K, V>>>), and its state machine adaptation
//! is also provided and can be enabled with the "impls" feature.
//! 
//! Enable the hashbrown feature to use the faster `hashbrown::HashMap` 
//! instead of `std::collections::HashMap`.


/// Contains implementations of various databases that also speak StateMachine.
/// TODO: Add more databases. Only contains a Key-Value store right now.
#[cfg(feature = "impls")]
pub mod impls;

use anyhow::Result;

use serde::ser::Serialize;
use serde::de::DeserializeOwned;

/// A generic behaviour for a State Machine that accepts a command that mutates it,
/// or a query that only reads from it, and allows to generate a snapshot of its state
/// that may be used to recover the state machine in any given state.
pub trait StateMachine {
    /// A command type provided by an implementation of a state machine.
    /// This command should not mutate the state machine and only query it.
    type QueryCommand;

    /// A command type provided by an implementation of a state machine.
    /// This command may mutate the state machine.
    type MutationCommand: Serialize + DeserializeOwned;

    /// The response type returned by the state machine when a query or mutation is performed.
    type Response;

    /// A (de)serializable representation of the State machine.
    type Snapshot: Serialize + DeserializeOwned;
    
    /// Apply a command to the state machine.
    fn apply(&self, command: Self::MutationCommand) -> Result<Self::Response>;

    /// Query the state machine for some data.
    fn query(&self, command: Self::QueryCommand) -> Result<Self::Response>;

    /// Capture the snapshot of the state machine at a point in time.
    fn snapshot(&self) -> Result<Self::Snapshot>;


    /// Restore a state machine given a snapshot.
    fn restore_from_snapshot(&self, snapshot: Self::Snapshot) -> Result<()>;
}
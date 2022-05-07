
pub mod raft_io {
    use std::result;
    use serde::de::DeserializeOwned;
    use serde::Serialize;
    use flate2::read::GzDecoder;
    use flate2::write::GzEncoder;
    use flate2::Compression;
    use rmp_serde::Serializer;
    use std::io::{self, Read, Write};

    pub type Result<T> = result::Result<T, std::io::Error>;

    /// The API to read/write a state.
    pub trait ReadWriteState<T> {
        /// Write the given persistent state to self.
        fn write_state(&mut self, state: &T) -> Result<usize>;
        /// Read a persistent state object from self.
        fn read_state(&mut self) -> result::Result<T, rmp_serde::decode::Error>;
    }

    /// Let any stateful struct that is serializable be serialized
    /// via messagepack and then gzipped before passing on to anything that writes.
    /// Let anything that allows read provide bytes that are first ungzipped
    /// and then deserialized via messagepack into a stateful struct.
    impl<S: Serialize + DeserializeOwned, T: io::Read + io::Write> ReadWriteState<S> for T {
        /// Serialize the given serialiable stateful struct
        /// with msgpack and then gz encode it before
        /// passing it to a writer.
        fn write_state(&mut self, state: &S) -> Result<usize> {
            let mut buf = Vec::new();
            state
                .serialize(&mut Serializer::new(&mut buf))
                .unwrap_or_else(|_| panic!("Could not serialize data."));

            let mut e = GzEncoder::new(Vec::new(), Compression::default());
            e.write_all(&buf)?;

            let compressed_bytes = e.finish()?;
            let length = compressed_bytes.len();

            self.write_all(&compressed_bytes)?;

            Ok(length)
        }

        /// Decode the gzipped bytes from a reader into something messagepacked
        /// and then further deserialize it into a stateful (de)serializable struct.
        fn read_state(&mut self) -> result::Result<S, rmp_serde::decode::Error> {
            let mut decoder = GzDecoder::new(self);

            let mut buf = Vec::new();
            decoder.read_to_end(&mut buf).expect("Could not");
            rmp_serde::decode::from_slice::<S>(&buf)
        }
    }
}


pub mod persistent {
    use serde::de::DeserializeOwned;
    use serde_derive::{Deserialize, Serialize};
    use crate::node::NodeType;
    // use crate::node::LogEntry;
    use state_machine::StateMachine;
    use proto::raft::LogEntry;

    // impl<T> From<LogEntry> for (u64, T)
    // where
    //     T: Clone + From<Vec<u8>>
    // {
    //     fn from(entry: LogEntry) -> Self {
    //         (entry.term, entry.command.into())
    //     }
    // }

    pub type Log<T> = (u64, T);

    /// Updated on stable storage before responding to RPCs.
    #[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
    pub struct State<T>
    {
        // /// The kind of the election entity that the current node is assigned.
        // pub participant_type: NodeType,
        /// The latest term server has seen (initialized to 0 on first boot, increases
        /// monotonically.)
        pub current_term: usize,
        /// The `candidate_id` that received vote in the current term (or None, if none exists.)
        pub voted_for: Option<usize>,

        /// The log entries, each entry contains command for state machine, and term when entry
        /// was received by leader.
        pub log: Vec<Log<T>>,
    }
    
    impl<T> Default for State<T>
    {
        fn default() -> Self {
            Self {
                current_term: 0,
                voted_for: None,
                log: vec![],
                // participant_type: NodeType::default(),
            }
        }
    }

    impl<T> State<T>
    {
        pub fn new() -> Self {
            Default::default()
        }
    }

}

pub mod volatile { 
    use serde_derive::{Deserialize, Serialize};
    use std::collections::HashMap;

    use crate::node::NodeMetadata;

    #[derive(Default, Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
    pub struct LeaderState {
        /// The index of the highest log entry
        /// known to be committed (initialized to 0, increases
        /// monotonically).
        pub commit_index: usize,
        /// The index of the highest log entry applied to
        /// state machine (initialized to 0, increases monotonically).
        pub last_applied: usize,
        /// For each server, the index of the next log entry
        /// to send to that server (initialized to leader's last log index + 1).
        pub next_index: HashMap<usize, Option<usize>>,
        /// For each server, the index of the highest log entry
        /// known to be to replicated on that server (initialized to 0, increases monotonically).
        pub match_index: HashMap<usize, Option<usize>>,

    }
    /// Volatile state on all servers. The properties
    /// `next_index` and `match_index` are only applicable
    /// to leader nodes and as such will be None in the other
    /// two cases.
    #[derive(Default, Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
    pub struct NonLeaderState {
        /// The index of the highest log entry
        /// known to be committed (initialized to 0, increases
        /// monotonically).
        pub commit_index: usize,
        /// The index of the highest log entry applied to
        /// state machine (initialized to 0, increases monotonically).
        pub last_applied: usize,
    }

    impl LeaderState {
        pub fn servers(self, servers: &[NodeMetadata]) -> Self {
            let mut next_index: HashMap<usize, Option<usize>> = HashMap::new();
            let mut match_index: HashMap<usize, Option<usize>> = HashMap::new();
            
            servers
            .iter()
            .for_each(|server| {
                next_index.insert(server.id, None);
                match_index.insert(server.id, None);
            });
            Self {
                commit_index: self.commit_index,
                last_applied: self.last_applied,
                next_index,
                match_index
            }
        }
    }

    impl From<NonLeaderState> for LeaderState {
        fn from(state: NonLeaderState) -> Self {
            Self {
                commit_index: state.commit_index,
                last_applied: state.last_applied,
                next_index: HashMap::new(),
                match_index: HashMap::new()
            }
        }
    }

    impl From<LeaderState> for NonLeaderState {
        fn from(state: LeaderState) -> Self {
            Self {
                commit_index: state.commit_index,
                last_applied: state.last_applied
            }
        }
    }

    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
    pub enum VolatileState {
        Leader(LeaderState),
        NonLeader(NonLeaderState)
    }

    impl Default for VolatileState {
        fn default() -> Self {
            Self::NonLeader(NonLeaderState::default())
        }
    }

    impl VolatileState {
        pub fn new() -> Self {
            Default::default()
        }
    }

}


#[cfg(test)]
mod tests {

    use crate::node::NodeType;
    use crate::storage::state::persistent::{State};
    use std::fs::File;
    use std::thread::sleep;
    use std::time::Duration;
    use super::raft_io::*;
    use state_machine::impls::key_value_store::*;
    use super::persistent::Log;
    // use crate::storage::state_machine::{
    //     CommitLog,
    //     Mutation,
    //     SetCommand,
    //     DeleteCommand
    // };

    /// Test that we can write a gzipped msgpack stream to a file on disk
    /// and then read it back immediately without losing any data. In other words,
    /// test for persistence.
    #[test]
    pub fn test_write_and_read_persistent_state() {

        // let key_value_store = KeyValueStore::new();

        const OUT_FILE: &str = "/tmp/.storage.gz";
        let persistent_state: State<String> = State {
            // participant_type: NodeType::Candidate,
            current_term: 2,
            voted_for: Some(3),
            log: vec![ (0, "".to_owned()) ],
        };

        let mut file = File::create(OUT_FILE).expect("Unable to create file.");
        let bytes_written = file
            .write_state(&persistent_state)
            .expect("Could not write persistent state.");

        println!("{} bytes written to {}", bytes_written, OUT_FILE);

        let mut file = File::open(OUT_FILE).expect("Unable to open file.");
        let observed_state = file.read_state().expect("Could not read persistent state");

        assert_eq!(
            persistent_state, 
            observed_state,
            "Persistent state is different from observed state."
        );
    }

    /// Test that we can write a gzipped msgpack stream to a file on disk
    /// and then read it back after a period of wait without losing any data.
    /// In other words, test for on-disk persistence after a period of wait.
    #[test]
    pub fn test_write_and_read_persistent_state_after_a_while() {
        const OUT_FILE: &str = "/tmp/foo_wait.gz";
        let persistent_state: State<Vec<u8>> = State {
            // participant_type: NodeType::Candidate,
            current_term: 2,
            voted_for: Some(3),
            log: vec![ (0, vec![]) ],
        };

        let mut file = File::create(OUT_FILE).expect("Unable to create file.");
        let bytes_written = file
            .write_state(&persistent_state)
            .expect("Could not write persistent state.");

        const WAIT_SECONDS: u64 = 2;

        println!("{} bytes written to {}", bytes_written, OUT_FILE);
        println!("Sleeping for {} seconds", WAIT_SECONDS);

        sleep(Duration::from_secs(WAIT_SECONDS));
        println!("Woken up after {} seconds", WAIT_SECONDS);

        let mut file = File::open(OUT_FILE).expect("Unable to open file.");
        let observed_state = file.read_state().expect("Could not read persistent state");

        assert_eq!(
            persistent_state, observed_state,
            "Persistent state is different from observed state."
        );
    }
}

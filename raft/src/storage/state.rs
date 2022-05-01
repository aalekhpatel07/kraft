
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
    use serde_derive::{Deserialize, Serialize};
    use crate::node::NodeType;

    
    pub type StateMachineCommand = String;
    pub type Term = usize;
    pub type LogRecord = (StateMachineCommand, Term);

    /// Updated on stable storage before responding to RPCs.
    #[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
    pub struct State {
        /// The kind of the election entity that the current node is assigned.
        pub participant_type: NodeType,
        /// The latest term server has seen (initialized to 0 on first boot, increases
        /// monotonically.)
        pub current_term: usize,
        /// The `candidate_id` that received vote in the current term (or None, if none exists.)
        pub voted_for: Option<usize>,

        /// The log entries, each entry contains command for state machine, and term when entry
        /// was received by leader.
        pub log: Vec<LogRecord>,
    }
    
    impl Default for State
    {
        fn default() -> Self {
            Self {
                current_term: 0,
                voted_for: None,
                log: vec![],
                participant_type: NodeType::default(),
            }
        }
    }
}

pub mod volatile { 
    use serde_derive::{Deserialize, Serialize};
    use std::collections::HashMap;

    #[derive(Default, Debug, Clone, Serialize, Deserialize)]
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
        pub next_index: HashMap<usize, usize>,
        /// For each server, the index of the highest log entry
        /// known to be to replicated on that server (initialized to 0, increases monotonically).
        pub match_index: HashMap<usize, usize>,

    }
    /// Volatile state on all servers. The properties
    /// `next_index` and `match_index` are only applicable
    /// to leader nodes and as such will be None in the other
    /// two cases.
    #[derive(Default, Debug, Clone, Serialize, Deserialize)]
    pub struct NonLeaderState {
        /// The index of the highest log entry
        /// known to be committed (initialized to 0, increases
        /// monotonically).
        pub commit_index: usize,
        /// The index of the highest log entry applied to
        /// state machine (initialized to 0, increases monotonically).
        pub last_applied: usize,
    }

}


#[cfg(test)]
mod tests {

    use crate::node::NodeType;
    use crate::storage::state::persistent::State;
    use std::fs::File;
    use std::thread::sleep;
    use std::time::Duration;
    use super::raft_io::*;
    use crate::storage::state_machine::state_machine_impls::key_value::*;
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
        let persistent_state = State {
            participant_type: NodeType::Candidate,
            current_term: 2,
            voted_for: Some(3),
            log: vec![("Some string".to_owned(), 0)],
        };

        let mut file = File::create(OUT_FILE).expect("Unable to create file.");
        let bytes_written = file
            .write_state(&persistent_state)
            .expect("Could not write persistent state.");

        println!("{} bytes written to {}", bytes_written, OUT_FILE);

        let mut file = File::open(OUT_FILE).expect("Unable to open file.");
        let observed_state = file.read_state().expect("Could not read persistent state");

        assert_eq!(
            persistent_state, observed_state,
            "Persistent state is different from observed state."
        );
    }

    /// Test that we can write a gzipped msgpack stream to a file on disk
    /// and then read it back after a period of wait without losing any data.
    /// In other words, test for on-disk persistence after a period of wait.
    #[test]
    pub fn test_write_and_read_persistent_state_after_a_while() {
        const OUT_FILE: &str = "/tmp/foo_wait.gz";
        let persistent_state = State {
            participant_type: NodeType::Candidate,
            current_term: 2,
            voted_for: Some(3),
            log: vec![("Some string".to_owned(), 0)],
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

    // /// Test that we can write a gzipped msgpack stream to a file on disk
    // /// and then read it back immediately without losing any data. In other words,
    // /// test for persistence.
    // #[test]
    // pub fn test_write_and_read_commit_log() {
    //     const OUT_FILE: &str = "/tmp/commit-log.gz";

    //     let commit_log = CommitLog {
    //         mutations: vec![
    //             Mutation::SET(SetCommand { key: "GE", value: "Germany" }),
    //             Mutation::SET(SetCommand { key: "IN", value: "India" }),
    //             Mutation::SET(SetCommand { key: "CA", value: "Canada" }),
    //             Mutation::SET(SetCommand { key: "US", value: "United States" }),
    //             Mutation::DELETE(DeleteCommand { key: "IN" }),
    //             Mutation::DELETE(DeleteCommand { key: "GE" }),
    //         ]
    //     };

    //     // let persistent_state = PersistentState {
    //     //     participant_type: Election::Candidate,
    //     //     current_term: 2,
    //     //     voted_for: Some(3),
    //     //     log: vec![("Some string".to_owned(), 0)],
    //     // };
    //     let mut file = File::create(OUT_FILE).expect("Unable to create file.");
    //     let bytes_written = file
    //         .write_commit_log(&commit_log.clone())
    //         .expect("Could not write commit log.");

    //     println!("{} bytes written to {}", bytes_written, OUT_FILE);

    //     let mut file = File::open(OUT_FILE).expect("Unable to open file.");
    //     let observed_state: CommitLog<&str, &str> = file
    //         .read_commit_log()
    //         .expect("Could not read commit log");

    //     assert_eq!(
    //         commit_log, observed_state,
    //         "Commit log is different from observed state."
    //     );
    // }
}

use crate::election::PersistentState;
use crate::storage::state_machine;
use flate2::read::GzDecoder;
use flate2::write::GzEncoder;
use flate2::Compression;
use rmp_serde::Serializer;
use serde::de::DeserializeOwned;
use serde::{Serialize};
use std::io::Write;
use std::io::{self, Read};
use std::result;

pub type Result<T> = result::Result<T, io::Error>;

// There is a low-hanging fruit of genericizing 
// the "gzipped + msgpacked reading/writing" 
// over arbitrary rmp (de)serializable state.
// I don't wanna do that right now because that 
// will end up introducing deserializer lifetimes
// and I'm scared of lifetimes ;(

// EDIT: The neat part is there's no need for lifetimes in that:
// https://stackoverflow.com/q/71909265/14045826

/// The API to read/write a state.
pub trait ReadWriteState<T> 
{
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


#[cfg(test)]
mod tests {

    use super::*;
    use crate::election::Election;
    use std::fs::File;
    use std::thread::sleep;
    use std::time::Duration;
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
        const OUT_FILE: &str = "/tmp/.storage.gz";
        let persistent_state = PersistentState {
            participant_type: Election::Candidate,
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
        let observed_state = file
            .read_state()
            .expect("Could not read persistent state");

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
        let persistent_state = PersistentState {
            participant_type: Election::Candidate,
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
        let observed_state = file
            .read_state()
            .expect("Could not read persistent state");

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

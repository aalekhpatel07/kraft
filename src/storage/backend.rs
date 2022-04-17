use crate::election::PersistentState;
use std::io::{self, Read};
use std::io::Write;
use std::result;
use rmp_serde::Serializer;
use serde::Serialize;
use flate2::Compression;
use flate2::write::GzEncoder;
use flate2::read::GzDecoder;


pub type Result<T> = result::Result<T, io::Error>;


/// The behaviours to read/write persistent storage to/from disk.
pub trait ReadWritePersistentState {
    /// Write the given persistent state to a stream.
    fn write_persistent_state(&mut self, state: &PersistentState) -> Result<usize>;
    /// Read a persistent state object from a stream.
    fn read_persistent_state(&mut self) -> Result<PersistentState>;
}

/// All read/write-able streams should be able to read/write our persistent storage on disk.
impl<T: io::Read + io::Write> ReadWritePersistentState for T {

    /// Serialize the given state with msgpack and then gz encode it before writing to the stream.
    fn write_persistent_state(&mut self, state: &PersistentState) -> Result<usize> {

        let mut buf = Vec::new();
        state.serialize(&mut Serializer::new(&mut buf)).expect(format!("Could not serialize: {:?}", state).as_str());

        let mut e = GzEncoder::new(Vec::new(), Compression::default());
        e.write_all(&buf)?;

        let compressed_bytes = e.finish()?;
        let length = compressed_bytes.len();

        self.write_all(&compressed_bytes)?;

        Ok(length)
    }

    /// Decode the gzipped stream into msgpack and then further deserialize it into the state object.
    fn read_persistent_state(&mut self) -> Result<PersistentState> {

        let mut decoder = GzDecoder::new(self);
        let mut buf = Vec::new();
        decoder.read_to_end(&mut buf)?;

        if let Ok(result) = rmp_serde::decode::from_slice::<PersistentState>(&buf) {
            Ok(result)
        } else {
            Err(io::Error::new(io::ErrorKind::Other, "Could not read persistent state."))
        }
    }
}


#[cfg(test)]
mod tests {

    use super::*;
    use std::fs::File;
    use crate::election::Election;
    use std::thread::sleep;
    use std::time::Duration;

    /// Test that we can write a gzipped msgpack stream to a file on disk
    /// and then read it back immediately without losing any data. In other words,
    /// test for persistence.
    #[test]
    pub fn test_write_and_read_persistent_state() {
        const OUT_FILE: &str = "./.storage.gz";
        let persistent_state = PersistentState { 
            participant_type: Election::Candidate, 
            current_term: 2, 
            voted_for: Some(3), 
            log: vec![("Some string".to_owned(), 0)]
        };
        let mut file = File::create(OUT_FILE).expect("Unable to create file.");
        let bytes_written = file.write_persistent_state(&persistent_state).expect("Could not write persistent state.");
        
        println!("{} bytes written to {}", bytes_written, OUT_FILE);
        
        let mut file = File::open(OUT_FILE).expect("Unable to open file.");
        let observed_state = file.read_persistent_state().expect("Could not read persistent state");

        assert_eq!(persistent_state, observed_state, "Persistent state is different from observed state.");
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
            log: vec![("Some string".to_owned(), 0)]
        };

        let mut file = File::create(OUT_FILE).expect("Unable to create file.");
        let bytes_written = file.write_persistent_state(&persistent_state).expect("Could not write persistent state.");
        
        const WAIT_SECONDS: u64 = 2;

        println!("{} bytes written to {}", bytes_written, OUT_FILE);
        println!("Sleeping for {} seconds", WAIT_SECONDS);
        
        sleep(Duration::from_secs(WAIT_SECONDS));
        println!("Woken up after {} seconds", WAIT_SECONDS);

        let mut file = File::open(OUT_FILE).expect("Unable to open file.");
        let observed_state = file.read_persistent_state().expect("Could not read persistent state");

        assert_eq!(persistent_state, observed_state, "Persistent state is different from observed state.");
    }
}
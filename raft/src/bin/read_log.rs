use raft::storage::state::{persistent::{State, Log}, raft_io::ReadWriteState};
use std::fs::File;

use clap::Parser;

#[derive(Parser, Debug)]
#[clap(author = "Aalekh Patel", version, about = "Read log file generated by this implementation of Raft.", long_about = None)]
pub struct Args {
    /// The path to the log file.
    pub log_file: String,
}

pub fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    let mut file = File::open(args.log_file)?;
    let state: State<String> = file.read_state().expect("Could not read persistent state");
    println!("{:?}", state);
    Ok(())
}

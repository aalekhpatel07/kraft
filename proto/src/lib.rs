use tonic;
mod raft;

pub use raft::*;

impl<T> From<raft::LogEntry> for (u64, T)
where
    T: Clone + From<Vec<u8>>
{
    fn from(entry: raft::LogEntry) -> Self {
        (entry.term, entry.command.into())
    }
}
impl<T> From<(u64, T)> for raft::LogEntry 
where
    T: core::convert::Into<Vec<u8>>
{
    fn from(entry: (u64, T)) -> Self {
        Self {
            term: entry.0,
            command: entry.1.into()
        }
    }
}


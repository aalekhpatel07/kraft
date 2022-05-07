use tonic;

pub mod raft {
    super::tonic::include_proto!("raft");
    
    impl<T> From<LogEntry> for (u64, T)
    where
        T: Clone + From<Vec<u8>>
    {
        fn from(entry: LogEntry) -> Self {
            (entry.term, entry.command.into())
        }
    }
}


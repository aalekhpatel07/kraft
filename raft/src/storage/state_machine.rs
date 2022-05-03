use anyhow::Result;
use serde::Serialize;
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


pub mod state_machine_impls {
    use super::*;
    use std::collections::HashMap;
    use std::hash::Hash;
    use std::sync::{Arc, Mutex};

    pub mod key_value {
        use serde_derive::{Deserialize, Serialize};

        use super::*;
        use std::fmt::Debug;

        #[derive(Debug, Clone, Serialize, Deserialize)]
        pub struct KeyValueStore<K: Hash + Eq + PartialEq, V> {
            pub(crate) inner: Arc<Mutex<HashMap<K, V>>>
        }

        #[derive(Debug, Clone, Serialize, Deserialize)]
        pub struct GetCommand<K> {
            pub key: K
        }

        #[derive(Debug, Clone, Serialize, Deserialize)]
        pub enum MutationCommand<K, V> {
            PUT(PutCommand<K, V>),
            DELETE(DeleteCommand<K>)
        }

        #[derive(Debug, Clone, Serialize, Deserialize)]
        pub enum QueryCommand<K> {
            GET(GetCommand<K>)
        }

        #[derive(Debug, Clone, Serialize, Deserialize)]
        pub struct PutCommand<K, V> {
            pub key: K,
            pub value: V
        }

        #[derive(Debug, Clone, Serialize, Deserialize)]
        pub struct DeleteCommand<K> {
            pub key: K
        }

        #[derive(Debug, Clone)]
        pub struct Response<V> {
            pub value: V
        }

        impl<K: Hash + PartialEq + Eq, V> KeyValueStore<K, V>
        {
            /// Create a new empty KeyValueStore with keys of type K, and values of type V.
            pub fn new() -> Self {
                Self {
                    inner: Arc::new(Mutex::new(HashMap::new()))
                }
            }
            /// Create a new empty KeyValueStore with keys of type K, and values of type V.
            pub fn empty() -> Self {
                Self::new()
            }
        }

        impl<K, V> KeyValueStore<K, V>
        where
            K: Hash + PartialEq + Eq + Debug + Clone,
            V: Clone
        {
            /// Given a key, get its corresponding value from the store.
            /// If the key does not exist, return an error.
            pub(crate) fn get(&self, key: K) -> Result<V> {
                let guard = self.inner.lock().unwrap();
                if let Some(value) = guard.get(&key) {
                    Ok(value.clone())
                } else {
                    Err(anyhow::Error::msg(format!("No key {:?} exists", &key)))
                }
            }
            /// Given a key, and a value, insert the pair into the store.
            /// If a key existed previously, return the value corresponding to it.
            /// Otherwise, return an error if this is the first time this key is inserted.
            pub(crate) fn put(&self, key: K, value: V) -> Result<V> {
                let mut guard = self.inner.lock().unwrap();

                if let Some(previous_value) = guard.insert(key.clone(), value) {
                    Ok(previous_value)
                } else {
                    Err(anyhow::Error::msg(format!("Did not have this key present {key:?}.")))
                }
            }
            /// Given a key, delete it and its corresponding value from the store.
            /// If the key existed, then return its corresponding value before deletion.
            /// Otherwise, return an error since there was an attempt to delete a key that didn't exist.
            pub(crate) fn delete(&self, key: K) -> Result<V> {
                let mut guard = self.inner.lock().unwrap();
                if let Some(previous_value) = guard.remove(&key) {
                    return Ok(previous_value)
                }
                else {
                    return Err(anyhow::Error::msg(format!("Key {key:?} does not exist.")))
                }
            }
        }


        impl<K, V> StateMachine for KeyValueStore<K, V> 
        where
            K: Hash + Serialize + DeserializeOwned + PartialEq + Eq + Clone + Debug,
            V: Serialize + DeserializeOwned + Clone
        {
            type QueryCommand = QueryCommand<K>;
            type MutationCommand = MutationCommand<K, V>;
            type Response = Response<V>;
            type Snapshot = HashMap<K, V>;

            /// Given a mutation command, apply the command to the state machine
            /// and return an application specific response.
            fn apply(&self, command: Self::MutationCommand) -> Result<Self::Response> {

                match command {
                    MutationCommand::PUT(cmd) => {
                        
                        let value = self.put(cmd.key, cmd.value)?;

                        Ok(Self::Response {
                            value
                        })
                    },
                    MutationCommand::DELETE(cmd) => {
                        let value = self.delete(cmd.key)?;
                        Ok(Self::Response {
                            value
                        })
                    },
                    _ => {
                        unreachable!("Only PUT and DELETE are implemented as a Mutation Command.")
                    }
                }
            }

            /// Given a query command, query the state machine with the given command
            /// and return an application specific response.
            fn query(&self, command: Self::QueryCommand) -> Result<Self::Response> {
                match command {
                    QueryCommand::GET(cmd) => {

                        let value = self.get(cmd.key)?;
                        Ok(Self::Response {
                            value
                        })

                    },
                    _ => {
                        unreachable!("Only GET is implemented as a Query Command.")
                    }
                }
            }

            /// Capture the state of the state machine into a serializable representation.
            fn snapshot(&self) -> Result<Self::Snapshot> {
                Ok(self.inner.lock().unwrap().clone())
            }

            /// Given a particular state of the machine, update the state of self to match
            /// the given state.
            fn restore_from_snapshot(&self, snapshot: Self::Snapshot) -> Result<()> {
                let mut guard = self.inner.lock().unwrap();
                *guard = snapshot;
                Ok(())
            }
        }
    }
}


#[cfg(test)]
pub mod tests {

    use serde_derive::{Serialize, Deserialize};
    use serde_json::Value;
    use std::hash::Hash;
    use std::fmt::Debug;

    use super::state_machine_impls::key_value::*;
    use super::StateMachine;
    use log::{debug, info, error, warn};
    use simple_logger::SimpleLogger;
    use std::collections::HashMap;
    use serde::Serialize;
    use serde::de::DeserializeOwned;
    use crate::utils::test_utils::set_up_logging;

    /// A test data structure that wraps the query command
    /// and attaches an expected value to it.
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct TestQueryCommand<K, V> {
        command: QueryCommand<K>,
        expected_value: V
    }

    /// A test data structure that wraps the mutation command
    /// and attaches an expected value to it.
    #[derive(Debug, Clone, Deserialize, Serialize)]
    pub struct TestMutationCommand<K, V> {
        command: MutationCommand<K, V>,
        expected_value: V
    }

    /// Enumerable command types
    /// that may be sent to a state machine.
    #[derive(Debug, Clone, Deserialize, Serialize)]
    pub enum Command<K, V> {
        Query(TestQueryCommand<K, V>),
        Mutation(TestMutationCommand<K, V>)
    }


    pub fn put<K, V>(key: K, value: V, expected_value: V) -> Command<K, V> {
        Command::Mutation(
            TestMutationCommand { 
                command: MutationCommand::PUT(
                    PutCommand { key, value }
                ), 
                expected_value
            }
        )
    }

    pub fn delete<K, V>(key: K, expected_value: V) -> Command<K, V> {
        Command::Mutation(
            TestMutationCommand { 
                command: MutationCommand::DELETE(
                    DeleteCommand { key }
                ), 
                expected_value
            }
        )
    }

    pub fn get<K, V>(key: K, expected_value: V) -> Command<K, V> {
        Command::Query(
            TestQueryCommand { 
                command: QueryCommand::GET(
                    GetCommand { key }
                ), 
                expected_value
            }
        )
    }

    /// Generate a log of entries and the state of the database after
    /// each log entry is applied to it.
    fn get_command_log_and_state_pairs() -> Vec<(Command<usize, String>, HashMap<usize, String>)> {

        let command_log: Vec<(Command<usize, String>, HashMap<usize, String>)> = vec![
            (
                put(1, "a".to_owned(), "".to_owned()),
                HashMap::from([(1, "a".to_owned())])
            ),
            (
                put(1, "b".to_owned(), "a".to_owned()),
                HashMap::from([(1, "b".to_owned())])
            ),
            (
                get(1, "b".to_owned()),
                HashMap::from([(1, "b".to_owned())])
            ),
            (
                put(2, "a".to_owned(), "".to_owned()),
                HashMap::from([(1, "b".to_owned()), (2, "a".to_owned())])
            ),
            (
                delete(1, "b".to_owned()),
                HashMap::from([(2, "a".to_owned())])
            ),
            (
                delete(1, "".to_owned()),
                HashMap::from([(2, "a".to_owned())])
            ),
            (
                delete(2, "a".to_owned()),
                HashMap::from([])
            )
        ];
        command_log
    }


    fn get_command_log_and_state_pairs_json() -> Vec<(Command<usize, serde_json::Value>, HashMap<usize, serde_json::Value>)> {

        let some_obj = r#"
        {
            "name": "John Doe",
            "age": 43,
            "phones": [
                "12345",
                "67890"
            ]
        }
        "#;

        let v: serde_json::Value = serde_json::from_str(some_obj).unwrap();

        let empty_object: serde_json::Value = serde_json::json!(null);
        debug!("{v:?}");

        let command_log: Vec<(Command<usize, serde_json::Value>, HashMap<usize, serde_json::Value>)> = vec![
            (
                put(1, v.clone(), empty_object),
                HashMap::from([(1, v)])
            )
        ];
        command_log
    }

    fn verify_command_logs<K, V>(command_log: Vec<(Command<K, V>, HashMap<K, V>)>) 
    where
        K: Hash + Serialize + DeserializeOwned + PartialEq + Eq + Copy + Debug,
        V: Serialize + DeserializeOwned + Clone + PartialEq + Debug
    {
        let db = KeyValueStore::new();

        for (command, snapshot) in command_log {
            match command {
                Command::Query(query) => {
                    let query_cmd = query.command;
                    let expected_value = query.expected_value;

                    let value = db.query(query_cmd).expect("Could not query.");

                    assert_eq!(value.value, expected_value);
                    assert_eq!(snapshot, db.snapshot().expect("Could not capture snapshot"), "Snapshots don't match up.");
                    debug!("Snapshot: {snapshot:?}");

                },

                Command::Mutation(mutation) => {
                    let mutation_cmd = mutation.command;
                    let expected_value = mutation.expected_value;

                    let (put_cmd, key_check_cmd) =
                        match mutation_cmd {
                            MutationCommand::PUT(PutCommand { key, value }) => {
                                (MutationCommand::PUT(PutCommand {key, value}), QueryCommand::GET(GetCommand { key }))
                            },
                            MutationCommand::DELETE(DeleteCommand { key }) => {
                                (MutationCommand::DELETE(DeleteCommand { key }), QueryCommand::GET(GetCommand { key }))
                            },
                            _ => {
                                unimplemented!("Only PUT and DELETE are implemented for mutation.");
                            }
                        };

                    let key_did_not_exist = db.query(key_check_cmd).is_err();


                    if let Ok(previous_value) = db.apply(put_cmd) {
                        assert_eq!(previous_value.value, expected_value);

                    } else {
                        // This should only happen if:
                        // this is the first time we're inserting a key for this PUT,
                        // or,
                        // we tried to DELETE a key that did not exist.
                        // 
                        assert!(key_did_not_exist, "Key existed but still the mutating \"apply\" call on the state machine failed.");
                    }

                    assert_eq!(snapshot, db.snapshot().expect("Could not capture snapshot"), "Snapshots don't match up.");
                    debug!("Snapshot: {snapshot:?}");
                }
            }
        }
    }

    #[test]
    fn test_command_log_json() {
        set_up_logging();
        let command_log = get_command_log_and_state_pairs_json();
        verify_command_logs(command_log);
    }

    #[test]
    fn test_command_log() {
        set_up_logging();
        let command_log = get_command_log_and_state_pairs();
        verify_command_logs(command_log);
    }



}

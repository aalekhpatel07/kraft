use anyhow::Result;
use serde::Serialize;
use serde::de::DeserializeOwned;


pub trait StateMachine: {
    /// A command type provided by an implementation of a state machine.
    /// This command should not mutate the state machine and only query it.
    type QueryCommand;

    /// A command type provided by an implementation of a state machine.
    /// This command may mutate the state machine.
    type MutationCommand;

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


pub mod StateMachineImpls {
    use super::*;
    use std::collections::HashMap;
    use std::hash::Hash;
    use std::sync::{Arc, Mutex};

    pub mod KeyValue {
        use super::*;
        use std::fmt::Debug;

        #[derive(Debug, Clone)]
        pub struct KeyValueStore<K, V> {
            inner: Arc<Mutex<HashMap<K, V>>>
        }

        #[derive(Debug, Clone)]
        pub struct GetCommand<K> {
            pub key: K
        }

        #[derive(Debug, Clone)]
        pub enum MutationCommand<K, V> {
            PUT(PutCommand<K, V>)
        }

        #[derive(Debug, Clone)]
        pub enum QueryCommand<K> {
            GET(GetCommand<K>)
        }

        #[derive(Debug, Clone)]
        pub struct PutCommand<K, V> {
            pub key: K,
            pub value: V
        }

        #[derive(Debug, Clone)]
        pub struct Response<V> {
            pub value: V
        }

        impl<K, V> KeyValueStore<K, V>
        {
            pub fn new() -> Self {
                Self {
                    inner: Arc::new(Mutex::new(HashMap::new()))
                }
            }
        }

        impl<K, V> KeyValueStore<K, V>
        where
            K: Hash + PartialEq + Eq + Debug + Copy,
            V: Clone
        {
            fn get(&self, key: K) -> Result<V> {
                let guard = self.inner.lock().unwrap();
                if let Some(value) = guard.get(&key) {
                    Ok(value.clone())
                } else {
                    Err(anyhow::Error::msg(format!("No key {:?} exists", &key)))
                }
            }
            fn put(&self, key: K, value: V) -> Result<V> {
                let mut guard = self.inner.lock().unwrap();

                if let Some(previous_value) = guard.insert(key, value) {
                    Ok(previous_value)
                } else {
                    Err(anyhow::Error::msg(format!("Did not have this key present {:?}.", &key)))
                }
            }
        }


        impl<K, V> StateMachine for KeyValueStore<K, V> 
        where
            K: Hash + Serialize + DeserializeOwned + PartialEq + Eq + Copy + Debug,
            V: Serialize + DeserializeOwned + Clone
        {
            type QueryCommand = QueryCommand<K>;
            type MutationCommand = MutationCommand<K, V>;
            type Response = Response<V>;
            type Snapshot = HashMap<K, V>;

            fn apply(&self, command: Self::MutationCommand) -> Result<Self::Response> {

                match command {
                    MutationCommand::PUT(cmd) => {
                        
                        let value = self.put(cmd.key, cmd.value)?;

                        Ok(Self::Response {
                            value
                        })
                    },
                    _ => {
                        unreachable!("Only PUT is implemented for a Mutation Command.")
                    }
                }
            }

            fn query(&self, command: Self::QueryCommand) -> Result<Self::Response> {
                match command {
                    QueryCommand::GET(cmd) => {

                        let value = self.get(cmd.key)?;
                        Ok(Self::Response {
                            value
                        })

                    },
                    _ => {
                        unreachable!("Only GET is implemented for a Query Command.")
                    }
                }
            }

            fn snapshot(&self) -> Result<Self::Snapshot> {
                Ok(self.inner.lock().unwrap().clone())
            }

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

    use super::StateMachineImpls::KeyValue::*;
    use super::StateMachine;
    use log::{debug, info, error, warn};
    use simple_logger::SimpleLogger;
    use std::collections::HashMap;


    fn init() -> KeyValueStore<usize, String> {
        SimpleLogger::new().init().unwrap();
        KeyValueStore::new()
    }

    #[test]
    fn stuff() {
        let machine = init();

        let put1 = PutCommand {
            key: 2usize,
            value: "Hello!".to_owned()
        };

        let get1 = GetCommand {
            key: 2usize
        };

        let response = 
            machine
                .apply(MutationCommand::PUT(put1))
                .expect_err("The key was inserted for the first time.");

        info!("response: {response:?}");

        let response = machine.query(QueryCommand::GET(get1)).expect("Could not GET");
        info!("response: {response:?}");
        // machine
        let snapshot = machine.snapshot().expect("Could not create snapshot of empty db.");

        info!("Snapshot: {snapshot:?}");
    }

    #[derive(Debug, Clone)]
    pub struct TestQueryCommand<K, V> {
        command: QueryCommand<K>,
        expected_value: V
    }

    #[derive(Debug, Clone)]
    pub struct TestMutationCommand<K, V> {
        command: MutationCommand<K, V>,
        expected_value: V
    }

    pub enum Command<K, V> {
        Query(TestQueryCommand<K, V>),
        Mutation(TestMutationCommand<K, V>)
    }


    pub fn put(key: usize, value: &str, expected_value: &str) -> Command<usize, String> {
        Command::Mutation(
            TestMutationCommand { 
                command: MutationCommand::PUT(
                    PutCommand { key, value: value.to_owned() }
                ), 
                expected_value: expected_value.to_owned()
            }
        )
    }

    pub fn get(key: usize, expected_value: &str) -> Command<usize, String> {
        Command::Query(
            TestQueryCommand { 
                command: QueryCommand::GET(
                    GetCommand { key }
                ), 
                expected_value: expected_value.to_owned()
            }
        )
    }

    /// TODO: Write a macro for this to clean up a lil bit.
    fn get_command_log_and_state_pairs() -> Vec<(Command<usize, String>, HashMap<usize, String>)> {

        let command_log: Vec<(Command<usize, String>, HashMap<usize, String>)> = vec![
            (
                put(1usize, "a", ""),
                HashMap::from([(1usize, "a".to_owned())])
            ),
            (
                put(1usize, "b", "a"),
                HashMap::from([(1usize, "b".to_owned())])
            ),
            (
                get(1usize, "b"),
                HashMap::from([(1usize, "b".to_owned())])
            )
        ];
        command_log
    }

    #[test]
    fn test_command_log() {
        let command_log = get_command_log_and_state_pairs();

        let db: KeyValueStore<usize, String> = KeyValueStore::new();

        for (command, snapshot) in command_log {
            match command {
                Command::Query(query) => {
                    let query_cmd = query.command;
                    let expected_value = query.expected_value;

                    let cmd = 
                        match query_cmd {
                            QueryCommand::GET(q) => {
                                QueryCommand::GET(q)
                            },
                            _ => {
                                unimplemented!("Only GET implemented for Query.");
                            }
                        };

                    let value = db.query(cmd).expect("Could not query.");

                    assert_eq!(value.value, expected_value);
                    assert_eq!(snapshot, db.snapshot().expect("Could not capture snapshot"), "Snapshots don't match up.");

                },

                Command::Mutation(mutation) => {
                    let mutation_cmd = mutation.command;
                    let expected_value = mutation.expected_value;

                    let (put_cmd, key_check_cmd) =
                        match mutation_cmd {
                            MutationCommand::PUT(PutCommand { key, value }) => {
                                (MutationCommand::PUT(PutCommand {key, value}), QueryCommand::GET(GetCommand { key }))
                            },
                            _ => {
                                unimplemented!("Only PUT implemented for mutation.");
                            }
                        };

                    let key_did_not_exist = db.query(key_check_cmd).is_err();


                    if let Ok(previous_value) = db.apply(put_cmd) {
                        assert_eq!(previous_value.value, expected_value);

                    } else {
                        // This should only happen if this is the first time we're inserting a key for this PUT.
                        assert!(key_did_not_exist);
                    }

                    assert_eq!(snapshot, db.snapshot().expect("Could not capture snapshot"), "Snapshots don't match up.");
                    
                }
            }
        }
    }
}

// use serde_derive::{Deserialize, Serialize};

// pub trait Commit<E = std::io::Error> {
//     fn commit(&self) -> Result<(), E>;
// }

// #[derive(Clone, Debug, Default)]
// pub struct KeyValueStorage<K, V> {
//     pub storage_path: String,
//     pub uncommitted: Vec<Mutation<K, V>>,
//     pub commit_log: Vec<Mutation<K, V>>,
//     pub state: HashMap<K, V>,
// }

// #[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
// pub enum Mutation<K, V> {
//     SET(SetCommand<K, V>),
//     DELETE(DeleteCommand<K>),
// }

// #[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
// pub struct SetCommand<K, V> {
//     pub key: K,
//     pub value: V,
// }

// #[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
// pub struct DeleteCommand<K> {
//     pub key: K,
// }

// #[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
// pub struct CommitLog<K, V> {
//     pub mutations: Vec<Mutation<K, V>>,
// }

// /// Let's just do a comparison based on the time
// /// instant since it is guaranteed to be monotonic
// /// and we're ensuring that only one commit log is generated
// /// at any given point in time.
// // impl<K, V> PartialEq for CommitLog<K, V>
// //     where
// //         K: Clone + Hash + Eq
// // {
// //     fn eq(&self, other: &Self) -> bool {
// //         self.instant.eq(&other.instant)
// //     }
// // }

// // impl<K, V> Eq for CommitLog<K, V>
// //     where
// //         K: Clone + Hash + Eq
// // {
// // }

// // impl<K, V> PartialOrd for CommitLog<K, V>
// //     where
// //         K: Clone + Hash + Eq
// // {
// //     fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
// //         self.instant.partial_cmp(&other.instant)
// //     }
// // }

// /// Since PartialOrd is provably infallible
// /// if we base the comparison on the instant,
// /// we can safely unwrap it to get Ord for free.
// // impl<K, V> Ord for CommitLog<K, V>
// //     where
// //         K: Clone + Hash + Eq
// // {
// //     fn cmp(&self, other: &Self) -> std::cmp::Ordering {
// //         self.partial_cmp(other).unwrap()
// //     }
// // }

// pub trait KeyValueTrait<K, V> // where
// //     K: Hash + Clone + Eq,
// {
//     fn set(&self, key: K, value: V) -> Option<V>;
//     fn get(&self, key: K) -> Option<V>;
//     fn delete(&self, key: K) -> Option<V>;
//     fn list(&self) -> Option<Vec<(K, V)>>;
// }

// impl<K, V> KeyValueStorage<K, V>
// where
//     K: Clone + Hash + Eq,
// {
//     pub fn apply(&mut self) {}
// }

// impl<K, V, E> Commit<E> for KeyValueStorage<K, V>
// where
//     K: Clone + Hash + Eq,
// {
//     fn commit(&self) -> Result<(), E> {
//         todo!()
//     }
// }

// // #[derive(Clone, Debug)]
// // pub enum KeyValueStoreCommand<K, V> {
// //     GET(GetCommand<K>),
// //     ALL(AllCommand)
// // }

// // #[derive(Debug, Clone)]
// // pub enum KeyValueStoreQuery<K: Key + Clone + Hash + Eq> {
// //     GET(GetCommand<K>),
// //     LIST(ListCommand)
// // }

// // #[derive(Clone, Debug)]
// // pub enum KeyValueStoreMutation<K: Key + Clone + Hash + Eq, V> {
// //     SET(SetCommand<K, V>),
// // }

// // #[derive(Debug, Clone)]
// // pub enum KeyValueStoreCommand<K: Key + Clone + Hash + Eq, V> {
// //     KeyValueStoreMutation(KeyValueStoreMutation<K, V>),
// //     KeyValueStoreQuery(KeyValueStoreQuery<K>),
// // }

// // #[derive(Clone, Debug)]
// // pub struct SetCommand<K: Key + Clone + Hash + Eq, V> {
// //     pub key: K,
// //     pub value: V
// // }

// // #[derive(Clone, Debug)]
// // pub struct GetCommand<K: Key + Clone> {
// //     pub key: K
// // }

// // pub trait Key{}

// // impl<K: Hash + Eq + Clone> Key for K {}
// // // pub trait SetCommand<K, V> {
// // //     pub type Output = Option<>
// // // }

// // #[derive(Clone, Debug)]
// // pub struct ListCommand;

// // #[derive(Clone, Debug)]
// // pub struct KeyValueStorage<K: Key + Clone, V>(HashMap<K, V>);

// // impl<K: Key + Clone + Eq + Hash, V> KeyValueStorage<K, V> {
// //     pub fn transition(&mut self, mutation: KeyValueStoreMutation<K, V>) {
// //         match mutation {
// //             KeyValueStoreMutation::SET(set) => {
// //                 self.0.insert(set.key, set.value).unwrap();
// //             },
// //             _ => {}
// //         }
// //     }
// // }

// // #[derive(Clone, Debug)]
// // pub struct KeyValueStateMachine<K: Key + Clone + Hash + Eq, V> {
// //    pub uncommitted: Arc<Mutex<Vec<KeyValueStoreCommand<K, V>>>>,
// //    pub commit_log: Arc<Mutex<Vec<KeyValueStoreCommand<K, V>>>>,
// //    pub state: Arc<Mutex<KeyValueStorage<K, V>>>
// // }

// // pub trait KeyValueStorageImpl<K: Key + Hash + Eq, V> {
// //     type STATE;

// //     fn set(&self, key: K, value: V) -> Option<V>;
// //     fn get(&self, key: K) -> Option<V>;
// //     fn list(&self) -> Option<Self::STATE>;
// // }

// // impl<K: Key + Clone + Hash + Eq, V: Clone> KeyValueStateMachine<K, V> {
// //     // pub fn process(&self, command: KeyValueStoreCommand<K, V>) {
// //     //     match command {
// //     //         KeyValueStoreCommand::GET(get) => {

// //     //             // old_state.insert(set.key.clone(), set.value.clone());
// //     //         },
// //     //         KeyValueStoreCommand::ALL(all) => {
// //     //             // old_state.insert(set.key.clone(), set.value.clone());
// //     //         },
// //     //         other_command => {
// //     //             let mut uncommitted = self.uncommitted.lock().unwrap();
// //     //             uncommitted.push(other_command);
// //     //         }
// //     //     }
// //     // }
// //     // pub fn apply(&self, command: &KeyValueStoreCommand<K, V>) {
// //     //     match command {
// //     //         KeyValueStoreCommand::KeyValueStoreMutation(mutation) => {
// //     //             match mutation {
// //     //                 KeyValueStoreMutation::SET(set) => {
// //     //                     self.state.lock().unwrap().0.insert(set.key.clone(), set.value.clone());
// //     //                     self.commit_log.lock().unwrap().push(command.clone());
// //     //                 },
// //     //                 _ => {}
// //     //             }
// //     //         },
// //     //         KeyValueStoreCommand::KeyValueStoreQuery(query) => {
// //     //             match query  {
// //     //                 KeyValueStoreQuery::GET(get) => {
// //     //                     self.state.lock().unwrap().0.insert(set.key.clone(), set.value.clone());
// //     //                     self.commit_log.lock().unwrap().push(command.clone());

// //     //                 },
// //     //                 KeyValueStoreQuery::LIST(list) => {

// //     //                 },

// //     //             }
// //     //         }

// //     //         _ => {}
// //     //     }
// //     // }

// //     // pub fn apply_all(&self, commands: &[KeyValueStoreCommand<K, V>]) -> KeyValueStorage<K, V> {
// //     //     let mut old_state = self.state.lock().unwrap().clone();

// //     //     commands
// //     //     .iter()
// //     //     .for_each( |command|  {
// //     //         self.transition(command)
// //     //     });
// //     //     old_state
// //     // }
// // }

// // impl<K: Key + Clone + Hash + Eq, V> KeyValueStorageImpl<K, V> for KeyValueStateMachine<K, V> {
// //     type STATE = KeyValueStorage<K, V>;

// //     fn set(&self, key: K, value: V) -> Option<V> {
// //         todo!()
// //     }

// //     fn get(&self, key: K) -> Option<V> {

// //         todo!()
// //     }

// //     fn list(&self) -> Option<Self::STATE> {
// //         todo!()
// //     }
// // }

// // impl<K: Key + Clone + Hash + Eq, V: Clone, E> Commit<E> for KeyValueStateMachine<K, V> {
// //     fn commit(&self) -> Result<(), E> {
// //         let uncommitted = self.uncommitted.lock().unwrap().clone();
// //         let new_state = self.apply_all(&uncommitted);
// //         *self.state.lock().unwrap() = new_state;

// //         Ok(())
// //     }
// // }

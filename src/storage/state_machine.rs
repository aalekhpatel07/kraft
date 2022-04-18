use std::{sync::{ Arc, Mutex}, collections::HashMap, process::{CommandArgs, Output}};
use std::hash::Hash;

use tokio::time::Instant;

pub trait Commit<E = std::io::Error> {
    fn commit(&self) -> Result<(), E>;
}


#[derive(Clone, Debug, Default)]
pub struct KeyValueStorage<K, V> 
    where
        K: Clone + Hash + Eq
{
    pub storage_path: String,
    pub uncommitted_mutations: Vec<Mutation<K, V>>,
    pub state: Arc<Mutex<HashMap<K, V>>>
}

#[derive(Clone, Debug)]
pub enum Mutation<K, V> 
    where
        K: Clone + Hash + Eq,
{
    SET(SetCommand<K, V>),
    DELETE(DeleteCommand<K>)
}

#[derive(Clone, Debug)]
pub struct SetCommand<K, V>
    where
        K: Clone + Hash + Eq
{
    pub key: K,
    pub value: V
}

#[derive(Clone, Debug)]
pub struct DeleteCommand<K>
    where
        K: Clone + Hash + Eq
{
    pub key: K,
}


#[derive(Clone, Debug)]
pub struct CommitLog<K, V> 
    where
        K: Clone + Hash + Eq,
{
    pub mutations: Vec<Mutation<K, V>>,
    pub instant: Instant
}

/// Let's just do a comparison based on the time
/// instant since it is guaranteed to be monotonic
/// and we're ensuring that only one commit log is generated
/// at any given point in time.
impl<K, V> PartialEq for CommitLog<K, V> 
    where
        K: Clone + Hash + Eq
{
    fn eq(&self, other: &Self) -> bool { 
        self.instant.eq(&other.instant)
    }
}

impl<K, V> Eq for CommitLog<K, V>
    where
        K: Clone + Hash + Eq 
{
}

impl<K, V> PartialOrd for CommitLog<K, V> 
    where
        K: Clone + Hash + Eq
{
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.instant.partial_cmp(&other.instant)
    }
}

/// Since PartialOrd is provably infallible
/// if we base the comparison on the instant,
/// we can safely unwrap it to get Ord for free.
impl<K, V> Ord for CommitLog<K, V> 
    where
        K: Clone + Hash + Eq
{
    fn cmp(&self, other: &Self) -> std::cmp::Ordering { 
        self.partial_cmp(other).unwrap()
    }
}


pub trait KeyValueTrait<K, V> 
    where
        K: Clone + Hash + Eq
{
    fn set(&self, key: K, value: V) -> Option<V>;
    fn get(&self, key: K) -> Option<V>;
    fn delete(&self, key: K) -> Option<V>;
    fn list(&self) -> Option<Vec<(K, V)>>;
}

impl<K, V> KeyValueStorage<K, V> 
    where
        K: Clone + Hash + Eq
{
    pub fn apply(&mut self, ) {}
}



impl<K, V, E> Commit<E> for KeyValueStorage<K, V> 
    where
        K: Clone + Hash + Eq
{
    fn commit(&self) -> Result<(), E> {
        todo!()
    }
}

// #[derive(Clone, Debug)]
// pub enum KeyValueStoreCommand<K, V> {
//     GET(GetCommand<K>),
//     ALL(AllCommand)
// }

// #[derive(Debug, Clone)]
// pub enum KeyValueStoreQuery<K: Key + Clone + Hash + Eq> {
//     GET(GetCommand<K>),
//     LIST(ListCommand)
// }

// #[derive(Clone, Debug)]
// pub enum KeyValueStoreMutation<K: Key + Clone + Hash + Eq, V> {
//     SET(SetCommand<K, V>),
// }


// #[derive(Debug, Clone)]
// pub enum KeyValueStoreCommand<K: Key + Clone + Hash + Eq, V> {
//     KeyValueStoreMutation(KeyValueStoreMutation<K, V>),
//     KeyValueStoreQuery(KeyValueStoreQuery<K>),
// }

// #[derive(Clone, Debug)]
// pub struct SetCommand<K: Key + Clone + Hash + Eq, V> {
//     pub key: K,
//     pub value: V
// }

// #[derive(Clone, Debug)]
// pub struct GetCommand<K: Key + Clone> {
//     pub key: K
// }

// pub trait Key{}

// impl<K: Hash + Eq + Clone> Key for K {}
// // pub trait SetCommand<K, V> {
// //     pub type Output = Option<>
// // }

// #[derive(Clone, Debug)]
// pub struct ListCommand;

// #[derive(Clone, Debug)]
// pub struct KeyValueStorage<K: Key + Clone, V>(HashMap<K, V>);

// impl<K: Key + Clone + Eq + Hash, V> KeyValueStorage<K, V> {
//     pub fn transition(&mut self, mutation: KeyValueStoreMutation<K, V>) {
//         match mutation {
//             KeyValueStoreMutation::SET(set) => {
//                 self.0.insert(set.key, set.value).unwrap();
//             },
//             _ => {}
//         }
//     }
// }

// #[derive(Clone, Debug)]
// pub struct KeyValueStateMachine<K: Key + Clone + Hash + Eq, V> {
//    pub uncommitted: Arc<Mutex<Vec<KeyValueStoreCommand<K, V>>>>,
//    pub commit_log: Arc<Mutex<Vec<KeyValueStoreCommand<K, V>>>>,
//    pub state: Arc<Mutex<KeyValueStorage<K, V>>>
// }


// pub trait KeyValueStorageImpl<K: Key + Hash + Eq, V> {
//     type STATE;

//     fn set(&self, key: K, value: V) -> Option<V>;
//     fn get(&self, key: K) -> Option<V>;
//     fn list(&self) -> Option<Self::STATE>;
// }


// impl<K: Key + Clone + Hash + Eq, V: Clone> KeyValueStateMachine<K, V> {
//     // pub fn process(&self, command: KeyValueStoreCommand<K, V>) {
//     //     match command {
//     //         KeyValueStoreCommand::GET(get) => {

//     //             // old_state.insert(set.key.clone(), set.value.clone());
//     //         },
//     //         KeyValueStoreCommand::ALL(all) => {
//     //             // old_state.insert(set.key.clone(), set.value.clone());
//     //         },
//     //         other_command => {
//     //             let mut uncommitted = self.uncommitted.lock().unwrap();
//     //             uncommitted.push(other_command);
//     //         }
//     //     }
//     // }
//     // pub fn apply(&self, command: &KeyValueStoreCommand<K, V>) {
//     //     match command {
//     //         KeyValueStoreCommand::KeyValueStoreMutation(mutation) => {
//     //             match mutation {
//     //                 KeyValueStoreMutation::SET(set) => {
//     //                     self.state.lock().unwrap().0.insert(set.key.clone(), set.value.clone());
//     //                     self.commit_log.lock().unwrap().push(command.clone());
//     //                 },
//     //                 _ => {}
//     //             }
//     //         },
//     //         KeyValueStoreCommand::KeyValueStoreQuery(query) => {
//     //             match query  {
//     //                 KeyValueStoreQuery::GET(get) => {
//     //                     self.state.lock().unwrap().0.insert(set.key.clone(), set.value.clone());
//     //                     self.commit_log.lock().unwrap().push(command.clone());

//     //                 },
//     //                 KeyValueStoreQuery::LIST(list) => {

//     //                 },

//     //             }
//     //         }

//     //         _ => {}
//     //     }
//     // }

//     // pub fn apply_all(&self, commands: &[KeyValueStoreCommand<K, V>]) -> KeyValueStorage<K, V> {
//     //     let mut old_state = self.state.lock().unwrap().clone();

//     //     commands
//     //     .iter()
//     //     .for_each( |command|  {
//     //         self.transition(command)
//     //     });
//     //     old_state
//     // }
// }

// impl<K: Key + Clone + Hash + Eq, V> KeyValueStorageImpl<K, V> for KeyValueStateMachine<K, V> {
//     type STATE = KeyValueStorage<K, V>;

//     fn set(&self, key: K, value: V) -> Option<V> {
//         todo!()
//     }

//     fn get(&self, key: K) -> Option<V> {
        
//         todo!()
//     }

//     fn list(&self) -> Option<Self::STATE> {
//         todo!()
//     }
// }

// impl<K: Key + Clone + Hash + Eq, V: Clone, E> Commit<E> for KeyValueStateMachine<K, V> {
//     fn commit(&self) -> Result<(), E> {
//         let uncommitted = self.uncommitted.lock().unwrap().clone();
//         let new_state = self.apply_all(&uncommitted);
//         *self.state.lock().unwrap() = new_state;

//         Ok(())
//     }
// }

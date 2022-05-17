#[cfg(feature = "hashbrown")]
use hashbrown::HashMap;

#[cfg(not(feature = "hashbrown"))]
use std::collections::HashMap;

use std::hash::Hash;
use std::str::FromStr;
use std::sync::{Arc, Mutex};
use serde_derive::{Deserialize, Serialize};
use std::fmt::{Debug, Display};
use log::{debug};

use anyhow::Result;

use crate::StateMachine;


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyValueStore<K: Hash + Eq + PartialEq, V> {
    pub(crate) inner: Arc<Mutex<HashMap<K, V>>>
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct GetCommand<K> {
    pub key: K
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum MutationCommand<K, V> {
    PUT(PutCommand<K, V>),
    DELETE(DeleteCommand<K>)
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum QueryCommand<K> {
    GET(GetCommand<K>)
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PutCommand<K, V> {
    pub key: K,
    pub value: V
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DeleteCommand<K> {
    pub key: K
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Response<V> {
    pub value: V
}

/// A command is something that the state machine receives and processes.
#[derive(Debug, Clone, serde_derive::Serialize, serde_derive::Deserialize)]
pub enum Command<K, V> {
    /// A command to retrieve the value corresponding to a given key.
    GET(GetCommand<K>),
    /// A command to assign a given value to a given key. If the key previously exists,
    /// it returns the previously assigned value.
    PUT(PutCommand<K, V>),
    /// A command to delete a key.
    DELETE(DeleteCommand<K>)
}


pub mod pretty {
    use std::fmt::Write;
    use core::fmt::Display;

    use super::{
        Command,
        QueryCommand,
        MutationCommand,
        GetCommand,
        PutCommand,
        DeleteCommand
    };

    // impl<K, V> Display for Command<K, V> 
    // where
    //     K: Display,
    //     V: Display
    // {
    //     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    //         write!(f, "{}", self).unwrap();
    //         Ok(())
    //     }
    // }

    impl<K> Display for GetCommand<K> 
    where
        K: Display
    {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "GET {}", self.key).unwrap();
            Ok(())
        }
    }
    impl<K, V> Display for PutCommand<K, V> 
    where
        K:Display,
        V:Display
    {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "PUT {} {}", self.key, self.value).unwrap();
            Ok(())
        }
    }
    impl<K> Display for DeleteCommand<K> 
    where
        K: Display
    {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {

            write!(f, "DELETE {}", self.key).unwrap();
            Ok(())
        }
    }
    impl<K> Display for QueryCommand<K> 
    where
        K: Display
    {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            match self {
                QueryCommand::GET(get) => {
                    write!(f, "QUERY({})", get).unwrap();
                },
            }
            Ok(())
        }
    }
    impl<K, V> core::fmt::Display for MutationCommand<K, V> 
    where
        K: Display,
        V: Display
    {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            match self {
                MutationCommand::PUT(put) => {
                    write!(f, "MUTATION({})", put).unwrap();
                },
                MutationCommand::DELETE(delete) => {
                    write!(f, "MUTATION({})", delete).unwrap();
                }
            }
            Ok(())
        }
    }
}




pub mod parse { 
    use std::str::FromStr;


    use super::{
        PutCommand, 
        GetCommand,
        DeleteCommand, MutationCommand, QueryCommand,
        Command
    };

    #[derive(Debug)]
    pub struct ParseError<T>
    where
        T: std::fmt::Debug
    {
        pub error: T
    }

    #[derive(Debug)]
    pub enum KeyValueError<'a, K, V> 
    where
        K: std::fmt::Debug,
        V: std::fmt::Debug,
    {
        KeyError(ParseError<K>),
        ValueError(ParseError<V>),
        CommandError(ParseError<&'a str>)
    }


    impl<K, V> TryFrom<&str> for PutCommand<K, V>
    where
        K: FromStr,
        V: FromStr,
        K::Err: std::fmt::Debug,
        V::Err: std::fmt::Debug,
    {

        type Error = KeyValueError<'static, K::Err, V::Err>;

        fn try_from(value: &str) -> Result<Self, Self::Error> {

            let tokens = value.split_whitespace().collect::<Vec<&str>>();

            if tokens.len() != 3 {
                Err(KeyValueError::CommandError(ParseError { error: "Invalid command. A PUT command should look like: PUT <key> <value>" }))
            } else {
                let mut iter = tokens.into_iter();
                let command = iter.next().unwrap();
                if command != "PUT" {
                    Err(KeyValueError::CommandError(ParseError { error: "Invalid command. A PUT command must start with: PUT" }))
                } else {

                    let key = iter.next().unwrap();
                    let val = iter.next().unwrap();

                    match (K::from_str(key), V::from_str(val)) {
                        (Ok(k), Ok(v)) => {
                            Ok(PutCommand { key: k, value: v })
                        },
                        (_, Err(err)) => {
                            Err(KeyValueError::ValueError(ParseError{error: err} ))
                        },
                        (Err(err), _) => {
                            Err(KeyValueError::KeyError(ParseError {error: err} ))
                        },
                    }
                }
            }
        }
    }


    #[derive(Debug)]
    pub enum KeyError<'a, K> 
    where
        K: std::fmt::Debug,
    {
        KeyError(ParseError<K>),
        CommandError(ParseError<&'a str>)
    }

    impl<K> TryFrom<&str> for GetCommand<K>
    where
        K: FromStr,
        K::Err: std::fmt::Debug
    {

        type Error = KeyError<'static, K::Err>;

        fn try_from(value: &str) -> Result<Self, Self::Error> {

            let tokens = value.split_whitespace().collect::<Vec<&str>>();

            if tokens.len() != 2 {
                Err(KeyError::CommandError(ParseError {error: "Invalid command. A GET command should look like: GET <key>"}))
            } else {
                let mut iter = tokens.into_iter();
                let command = iter.next().unwrap();
                if command != "GET" {
                    Err(KeyError::CommandError(ParseError {error: "Invalid command. A GET command must start with: GET"}))
                } else {

                    let key = iter.next().unwrap();

                    match K::from_str(key) {
                        Ok(k) => {
                            Ok(GetCommand { key: k })
                        },
                        Err(err) => {
                            Err(KeyError::KeyError(ParseError { error: err }))
                        }
                    }
                }
            }
        }
    }


    impl<K> TryFrom<&str> for DeleteCommand<K>
    where
        K: FromStr,
        K::Err: std::fmt::Debug
    {

        type Error = KeyError<'static, K::Err>;

        fn try_from(value: &str) -> Result<Self, Self::Error> {

            let tokens = value.split_whitespace().collect::<Vec<&str>>();

            if tokens.len() != 2 {
                Err(KeyError::CommandError(ParseError {error: "Invalid command. A DELETE command should look like: DELETE <key>"}))
            } else {
                let mut iter = tokens.into_iter();
                let command = iter.next().unwrap();
                if command != "DELETE" {
                    Err(KeyError::CommandError(ParseError {error: "Invalid command. A DELETE command must start with: DELETE"}))
                } else {

                    let key = iter.next().unwrap();

                    match K::from_str(key) {
                        Ok(k) => {
                            Ok(DeleteCommand { key: k })
                        },
                        Err(err) => {
                            Err(KeyError::KeyError(ParseError { error: err }))
                        }
                    }
                }
            }
        }
    }

    impl<K, V> TryFrom<&str> for MutationCommand<K, V> 
    where
        K: FromStr,
        K::Err: std::fmt::Debug,
        V: FromStr,
        V::Err: std::fmt::Debug,
    {
        type Error = KeyValueError<'static, K::Err, V::Err>;
        fn try_from(value: &str) -> Result<Self, Self::Error> {
            if let Ok(PutCommand { key, value: val }) = value.try_into() {
                Ok(MutationCommand::PUT(PutCommand {key, value: val}))
            } else if let Ok(DeleteCommand { key }) = value.try_into() {
                Ok(MutationCommand::DELETE(DeleteCommand { key }))
            } else {
                Err(KeyValueError::CommandError(ParseError { error: "Could not parse mutation command." }))
            }
        }
    }


    impl<K> TryFrom<&str> for QueryCommand<K> 
    where
        K: FromStr,
        K::Err: std::fmt::Debug,
    {
        type Error = KeyError<'static, K::Err>;
        fn try_from(value: &str) -> Result<Self, Self::Error> {
            if let Ok(GetCommand { key }) = value.try_into() {
                Ok(QueryCommand::GET(GetCommand { key }))
            } else {
                Err(KeyError::CommandError(ParseError { error: "Could not parse query command." }))
            }
        }
    }

    impl<K, V> TryFrom<&str> for Command<K, V>
    where
        K: FromStr,
        K::Err: std::fmt::Debug,
        V: FromStr,
        V::Err: std::fmt::Debug
    {
        type Error = KeyValueError<'static, K::Err, V::Err>;

        fn try_from(value: &str) -> Result<Self, Self::Error> {
            if let Ok(PutCommand { key, value: val }) = value.try_into() {
                Ok(Command::PUT(PutCommand {key, value: val}))
            } else if let Ok(DeleteCommand { key }) = value.try_into() {
                Ok(Command::DELETE(DeleteCommand { key }))
            } else if let Ok(GetCommand { key }) = value.try_into() {
                Ok(Command::GET(GetCommand { key }))
            } else {
                Err(Self::Error::CommandError(ParseError { error: "Could not parse given value into a valid command." }))
            }
        }
    }

    impl<K, V> From<Vec<u8>> for MutationCommand<K, V> 
    where
        K: serde::de::DeserializeOwned,
        V: serde::de::DeserializeOwned
    {
        fn from(serialized: Vec<u8>) -> Self {
            #[cfg(not(features = "msgpack"))]
            let data: MutationCommand<K, V> = serde_json::from_slice(&serialized).expect("Couldn't deserialize into a MutationCommand.");
            
            #[cfg(features = "msgpack")]
            let data: MutationCommand<K, V> = rmp_serde::from_slice(&serialized).expect("Couldn't deserialize msgpack into a MutationCommand.");

            data
        }
    }

    impl<K, V> Into<Vec<u8>> for MutationCommand<K, V> 
    where
        K: serde::ser::Serialize,
        V: serde::ser::Serialize

    {
        fn into(self) -> Vec<u8> {
            #[cfg(not(features = "msgpack"))]
            {
                let serialized = serde_json::to_string(&self).expect("Couldn't serialize MutationCommand.");
                serialized.into_bytes()
            }

            #[cfg(features = "msgpack")]
            {
                let serialized = rmp_serde::to_vec(&self).expect("Couldn't serialize MutationCommand.");
                serialized
            }
        }
    }


    impl<K, V> From<Vec<u8>> for Command<K, V> 
    where
        K: serde::de::DeserializeOwned,
        V: serde::de::DeserializeOwned
    {

        fn from(serialized: Vec<u8>) -> Self {
            #[cfg(not(features = "msgpack"))]
            let data: Command<K, V> = serde_json::from_slice(&serialized).expect("Couldn't deserialize into a Command.");

            #[cfg(features = "msgpack")]
            let data: Command<K, V> = rmp_serde::from_slice(&serialized).expect("Couldn't deserialize msgpack into a Command.");

            data
        }
    }


    impl<K, V> Into<Vec<u8>> for Command<K, V> 
    where
        K: serde::ser::Serialize,
        V: serde::ser::Serialize
    {

        fn into(self) -> Vec<u8> {
            #[cfg(not(features = "msgpack"))]
            {
                let serialized = serde_json::to_string(&self).expect("Couldn't serialize Command.");
                serialized.into_bytes()
            }

            #[cfg(features = "msgpack")]
            {
                let serialized = rmp_serde::to_vec(&self).expect("Couldn't serialize Command into bytes using msgpack.");
                serialized
            }
        }
    }
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
    K: Hash + serde::Serialize + serde::de::DeserializeOwned + PartialEq + Eq + Clone + Debug,
    V: serde::Serialize + serde::de::DeserializeOwned + Clone
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


#[cfg(feature = "random")]
pub mod random {
    use rand::Rng;
    use rand::distributions::{Standard, Distribution};

    use super::{MutationCommand, PutCommand, DeleteCommand};

    impl<K, V> Distribution<MutationCommand<K, V>> for Standard 
    where
        Standard: Distribution<K> + Distribution<V>
    {
        fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> MutationCommand<K, V> {
            match rng.gen_range(0..=1) {
                0 => {
                    MutationCommand::PUT(
                        PutCommand {
                            key: rng.gen::<K>(),
                            value: rng.gen::<V>()
                        }
                    )
                },
                _ => {
                    MutationCommand::DELETE(
                        DeleteCommand {
                            key: rng.gen::<K>(),
                        }
                    )
                }
            }
        }
    }
}

#[cfg(test)]
pub mod tests {

    use serde_derive::{Serialize, Deserialize};
    use std::hash::Hash;
    use std::fmt::Debug;

    use super::*;
    use super::parse::*;
    use crate::StateMachine;
    use log;
    use log::debug;
    use serde_json;
    use simple_logger;

    #[cfg(feature = "hashbrown")]
    use hashbrown::HashMap;

    #[cfg(not(feature = "hashbrown"))]
    use std::collections::HashMap;
    
    use serde::Serialize;
    use serde::de::DeserializeOwned;
    use std::sync::Once;

    static INIT: Once = Once::new();

    /// Set up the SimpleLogger.
    /// No matter how many times this function is invoked,
    /// the actual logger is only set up once.
    pub fn set_up_logging() {
        INIT.call_once(|| {
            simple_logger::SimpleLogger::new().with_level(log::LevelFilter::Debug).init().unwrap();
        });
    }

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

    #[test]
    pub fn test_put_from_str() {
        set_up_logging();
        let expected = PutCommand { key: 1usize, value: "cat".to_owned()};
        let observed: PutCommand<usize, String> = "PUT 1 cat".try_into().expect("Could not understand PUT");
        assert_eq!(expected, observed);
    }

    #[test]
    pub fn test_get_from_str() {
        set_up_logging();
        let expected = GetCommand { key: 1usize };
        let observed: GetCommand<usize> = "GET 1".try_into().expect("Could not understand GET");
        assert_eq!(expected, observed);
    }

    #[test]
    pub fn test_delete_from_str() {
        set_up_logging();
        let expected = DeleteCommand { key: 1usize };
        let observed: DeleteCommand<usize> = "DELETE 1".try_into().expect("Could not understand DELETE");
        assert_eq!(expected, observed);
    }

    #[test]
    pub fn test_mutation_delete_from_str() {
        set_up_logging();
        let expected: MutationCommand<usize, usize> = MutationCommand::DELETE(DeleteCommand { key: 1usize });
        let observed: MutationCommand<usize, usize> = "DELETE 1".try_into().expect("Could not understand Mutation DELETE.");
        assert_eq!(expected, observed);
    }

    #[test]
    pub fn test_mutation_put_from_str() {
        set_up_logging();
        let expected: MutationCommand<usize, usize> = MutationCommand::PUT(PutCommand { key: 1usize, value: 1 });
        let observed: MutationCommand<usize, usize> = "PUT 1 1".try_into().expect("Could not understand Mutation PUT.");
        assert_eq!(expected, observed);
    }

    #[test]
    pub fn test_mutation_from_str() {
        set_up_logging();
        let commands = vec![
            "PUT x 1",
            "DELETE x",
            "PUT y 2",
            "PUT z 10000"
        ];
        let observed: Vec<MutationCommand<String, serde_json::Value>> = 
            commands
            .iter()
            .map(
                |&cmd| cmd.try_into().expect("Couldnt parse cmd.")
            )
            .collect();

        let expected: Vec<MutationCommand<String, serde_json::Value>> = vec![
            MutationCommand::PUT( PutCommand { key: "x".to_owned(), value: serde_json::json!(1)}),
            MutationCommand::DELETE( DeleteCommand { key: "x".to_owned() }),
            MutationCommand::PUT( PutCommand { key: "y".to_owned(), value: serde_json::json!(2)}),
            MutationCommand::PUT( PutCommand { key: "z".to_owned(), value: serde_json::json!(10000)}),
        ];
        assert_eq!(observed, expected);
    }



    #[test]
    pub fn test_query_from_str() {
        set_up_logging();
        let commands = vec![
            "GET x",
            "GET y",
        ];
        let observed: Vec<QueryCommand<String>> = 
            commands
            .iter()
            .map(
                |&cmd| cmd.try_into().expect("Couldnt parse cmd.")
            )
            .collect();

        let expected: Vec<QueryCommand<String>> = vec![
            QueryCommand::GET( GetCommand { key: "x".to_owned() }),
            QueryCommand::GET( GetCommand { key: "y".to_owned() }),
        ];
        
        assert_eq!(observed, expected);
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
    fn test_create_new_key_value_store() {
        set_up_logging();

        let store: KeyValueStore<String, serde_json::Value> =  KeyValueStore::new();
        let snapshot = store.snapshot().expect("Couldn't capture snapshot.");
        assert_eq!(&snapshot, &HashMap::new());

    }

    #[test]
    fn test_create_empty_key_value_store() {
        set_up_logging();

        let store: KeyValueStore<String, serde_json::Value> =  KeyValueStore::empty();
        let snapshot = store.snapshot().expect("Couldn't capture snapshot.");
        assert_eq!(&snapshot, &HashMap::new());
        
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
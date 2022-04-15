use std::time::{Duration, SystemTime, Instant};
// use rmp_serde as rmps;
// use serde;
pub use serde::{Serialize, Deserialize};
use serde_derive::{Serialize, Deserialize};

// use serde::{Deserialize, Serialize};


#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct FizzBuzzCounter {
    
    #[serde(with = "serde_millis")]
    pub time_stamp: SystemTime,

    pub value: u64
}

impl FizzBuzzCounter {
    pub fn print(&self) {
        if self.value % 15 == 0 {
            println!("Fizz Buzz");
        }
        else if self.value % 5 == 0 {
            println!("Buzz");
        }
        else if self.value % 3 == 0 {
            println!("Fizz");
        }
        else {
            println!("{}", self.value);
        }
    }
}

// pub struct FizzBuzzAtomic {

// }
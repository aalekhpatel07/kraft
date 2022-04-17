// use tokio::{net::{TcpStream}, io::{AsyncWriteExt, AsyncReadExt}};
// use std::{time::{SystemTime}};
// use kraft::election::*;
// use rmp_serde::{Serializer};
// use serde::Serialize;


// // pub struct Client {

// // }


#[tokio::main]
async fn main() {
    // let mut stream = TcpStream::connect("0.0.0.0:9000").await.unwrap();

}

//     // let mut remote_nodes: Vec<TcpNode> = vec![

//     //     TcpNode {
//     //         id: 1,
//     //         addr: "192.168.1.113:9000".parse().unwrap()
//     //     }

//     // ];

//     // let node = remote_nodes.get(0).unwrap();

//     // for node in &remote_nodes {
//     // let addr: String = format!("{}:{}", node.addr.host, node.addr.port);

//     let mut stream = TcpStream::connect("192.168.1.113:9000").await.unwrap();
    
//     stream.writable().await.unwrap();

//     // let counter = FizzBuzzCounter {
//     //     time_stamp: SystemTime::now(),
//     //     value: 0
//     // };
//     let request_vote = RequestVoteRPCRequest {
//         participant_type: Election::Follower,
//         term: 0,
//         candidate_id: 0,
//         last_log_index: 0,
//         last_log_term: 0
//     };

//     let mut buf = Vec::new();
//     request_vote.serialize(&mut Serializer::new(&mut buf)).unwrap();

//     stream.writable().await.unwrap();

//     stream.write_all(&buf).await.unwrap();

//     let mut buffer: Vec<u8> = vec![];

//     while let Ok(size) = stream.read_buf(&mut buffer).await {
//         if size == 0 {
//             println!("Bye exiting...");
//             return;
//         }

//         // let mut fizz_buzz_counter: FizzBuzzCounter = rmp_serde::decode::from_slice(&buffer).unwrap();
        
//         // fizz_buzz_counter.print();

//         // fizz_buzz_counter.value += 1;
//         // fizz_buzz_counter.time_stamp = SystemTime::now();

//         // let mut buf = Vec::new();
//         // fizz_buzz_counter.serialize(&mut Serializer::new(&mut buf)).unwrap();

//         // stream.write_all(&buf).await.unwrap();

//         buffer.clear();
//     }

//     // loop {
//     // tokio::spawn(async move {
//     //     process(&mut stream).await
//     // });
//     // }
// }


// async fn process(stream: &mut TcpStream) {
//     let mut buffer: Vec<u8> = vec![];

//     while let Ok(size) = stream.read_buf(&mut buffer).await {
//         if size == 0 {
//             return;
//         }

//         // let mut fizz_buzz_counter: FizzBuzzCounter = rmp_serde::decode::from_slice(&buffer).unwrap();

//         // println!("Fizz buzz counter: {:?}", fizz_buzz_counter);

//         // let mut buf = Vec::new();
//         // fizz_buzz_counter.serialize(&mut Serializer::new(&mut buf)).unwrap();

//         // stream.write_all(&buf).await.unwrap();

//         buffer.clear();
//     }

// }

use std::net::TcpStream;
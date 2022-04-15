use kraft::network::node::*;
use tokio::{net::{TcpSocket, TcpStream, TcpListener}, io::AsyncReadExt, io::AsyncWriteExt};
use kraft::data::*;
use std::time::{Instant, SystemTime};
use rmp_serde::{Deserializer, Serializer};


#[tokio::main]
async fn main() {

    let node = TcpNode {
        id: 1,
        addr: "0.0.0.0:9000".parse().unwrap()
    };

    let listener = 
        TcpListener::bind(&node.addr)
        .await
        .unwrap();

    dbg!("TCP Server listening on: {}", &node.addr);

    loop {
        let (mut stream, _) = listener.accept().await.unwrap();
        tokio::spawn(async move {

            process(&mut stream).await

        });
    }
    
}

async fn process(stream: &mut TcpStream) {
    let mut buffer: Vec<u8> = vec![];

    while let Ok(size) = stream.read_buf(&mut buffer).await {

        if size == 0 {
            return;
        }
        
        let mut fizz_buzz_counter: FizzBuzzCounter = rmp_serde::decode::from_slice(&buffer).unwrap();
        fizz_buzz_counter.print();

        buffer.clear();

        fizz_buzz_counter.value += 1;
        fizz_buzz_counter.time_stamp = SystemTime::now();

        let mut buf = Vec::new();
        fizz_buzz_counter.serialize(&mut Serializer::new(&mut buf)).unwrap();

        stream.write_all(&buf).await.unwrap();
        // let mut deserializer = Deserializer::new();

        // println!("Read {} bytes: {:?}", size, &buffer);
    }

}



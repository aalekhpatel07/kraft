use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpStream, TcpListener};
use tokio::time::{sleep, Duration};
use std::fmt::Debug;


pub async fn log<M: Debug>(message: M) -> Option<Vec<u8>> {
    sleep(Duration::from_millis(10)).await;
    dbg!("Logging {:?} after {} milliseconds", &message, 10);
    Some(format!("{:?}", message).as_bytes().to_owned())
}


#[derive(Debug, Clone)]
pub struct TcpNode {
    pub id: usize,
    pub addr: Addr
}


#[derive(Debug, Clone)]
pub struct Addr {
    pub host: String,
    pub port: u16
}

impl Addr {
    pub fn new(host: impl Into<String>, port: impl Into<u16>) -> Self {
        Self {
            host: host.into(),
            port: port.into()
        }
    }
}


impl From<Addr> for String {
    fn from(addr: Addr) -> Self {
        format!("{}:{}", addr.host, addr.port)
    }
}

impl From<&Addr> for String {
    fn from(addr: &Addr) -> Self {
        format!("{}:{}", addr.host, addr.port)
    }
}

impl TcpNode {
    pub fn new(id: usize, host: &str, port: u16) -> Self {
        Self {
            id,
            addr: Addr::new(host, port)
        }
    }

    pub async fn connect(&self, addr: &Addr) {
        dbg!("Attempting to connect to {}:{}", &addr.host, &addr.port);
        let addr: String = addr.into();

        let mut stream = TcpStream::connect(addr).await.expect("Connection to be successful.");

        let msg_to_send = format!("{}", self.id);

        stream.write_all(msg_to_send.as_bytes()).await.expect("Write to be successful.");

    }


    pub async fn listen(&self) {

        let addr: String = (&self.addr).into();

        let listener = {

            TcpListener::bind(&addr)
            .await
            .expect(format!("Should be able to listen at {}", &addr).as_str())
        };
        
        dbg!("Listening on {}", &addr);

        loop {
            let (socket, _) = listener.accept().await.expect("Should be able to accept the connection.");
            dbg!("Connection received from {:?}", &socket.local_addr());
            self.handle_connection_established(socket).await;
        }
    }

    async fn handle_connection_established(&self, mut socket: TcpStream) {
        let mut buffer: Vec<u8> = vec![];


        while let Ok(size) = socket.read_buf(&mut buffer).await {
            dbg!("Size read: {}", size);
            if size == 0 {
                break;
            } else {
                println!("{:?}", buffer);
                self.handle_message(&buffer, &mut socket).await;
                buffer.clear();
            }
        }
    }

    async fn handle_message(&self, message: &[u8], socket: &mut TcpStream) {
        let ascii = 
            message
                .iter()
                .map(|c: &u8| (*c as char).to_string())
                .collect::<Vec<String>>();
        log(
            ascii.join("")
        ).await;
    }
}

#[cfg(test)]
mod tests {
    use crate::network::node::*;

    #[ignore = "Requires netcat (or another external TCPServer). Run `nc 127.0.0.1 9000 -l` before this test."]
    #[tokio::test]
    async fn test_connect() {

        dbg!("Run `nc 127.0.0.1 9000 -l` and receive messages from the TCPClient.");
        let node = TcpNode::new(1, "127.0.0.1", 9000);
        node.connect(&node.addr).await;
    }

    #[ignore = "Requires netcat (or another external TCPClient). Run `nc 127.0.0.1 9001` before this test."]
    #[tokio::test]
    async fn test_listen() {
        dbg!("Run `nc 127.0.0.1 9001` and send messages to see the TCPServer logging the messages.");
        let node = TcpNode::new(1, "127.0.0.1", 9001);
        node.listen().await;
    }
}
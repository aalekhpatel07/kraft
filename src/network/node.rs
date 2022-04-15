use std::fmt::Debug;
use std::net::SocketAddr;

#[derive(Debug, Clone)]
pub struct TcpNode {
    pub id: usize,
    pub addr: SocketAddr,
}


// #[derive(Debug, Clone)]
// pub struct Addr {
//     pub host: String,
//     pub port: u16
// }

// impl Default for Addr {
//     fn default() -> Self {
//         Addr {
//             host: "127.0.0.1".to_string(),
//             port: 0
//         }
//     }
// }

// impl Addr {
//     pub fn with_port(port: impl Into<u16>) -> Self {
//         Self {
//             host: "127.0.0.1".to_string(),
//             port: port.into()
//         }
//     }

//     pub fn new(host: impl Into<String>, port: impl Into<u16>) -> Self {
//         Self {
//             host: host.into(),
//             port: port.into()
//         }
//     }
// }


// impl From<Addr> for String {
//     fn from(addr: Addr) -> Self {
//         format!("{}:{}", addr.host, addr.port)
//     }
// }

// impl From<&Addr> for String {
//     fn from(addr: &Addr) -> Self {
//         format!("{}:{}", addr.host, addr.port)
//     }
// }

// impl TcpNode {
//     pub fn new(id: usize, host: &str, port: u16) -> Self {
//         Self {
//             id,
//             addr: Addr::new(host, port)
//         }
//     }
//     pub async fn process(&self, socket: TcpStream) {

//     }

//     pub async fn listen(&'static self) {

//         let addr: String = (&self.addr).into();

//         let listener = {

//             TcpListener::bind(&addr)
//             .await
//             .expect(format!("Should be able to listen at {}", &addr).as_str())
//         };
        
//         dbg!("Listening on {}", &addr);

//         loop {
//             let (socket, _) = listener.accept().await.expect("Should be able to accept the connection.");
//             dbg!("Connection received from {:?}", &socket.local_addr());
//             tokio::spawn(async move {
//                 self.process(socket).await;
//             });
//         }
//     }
// }



#[cfg(test)]
mod tests {
    // use crate::network::node::*;

    // #[ignore = "Requires netcat (or another external TCPServer). Run `nc 127.0.0.1 9000 -l` before this test."]
    // #[tokio::test]
    // async fn test_connect() {

    //     dbg!("Run `nc 127.0.0.1 9000 -l` and receive messages from the TCPClient.");
    //     let node = TcpNode::new(1, "127.0.0.1", 9000);
    //     node.connect(&node.addr).await;
    // }

    // #[ignore = "Requires netcat (or another external TCPClient). Run `nc 127.0.0.1 9001` before this test."]
    // #[tokio::test]
    // async fn test_listen() {
    //     dbg!("Run `nc 127.0.0.1 9001` and send messages to see the TCPServer logging the messages.");
    //     let node = TcpNode::new(1, "127.0.0.1", 9001);
    //     node.listen().await;
    // }
}
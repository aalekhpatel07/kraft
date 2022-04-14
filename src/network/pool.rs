use crate::network::node::*;


#[derive(Debug, Clone)]
pub struct TcpNodePool {
    pub nodes: Vec<TcpNode>
}


impl TcpNodePool {
    pub fn new(nodes: Vec<TcpNode>) -> Self {
        Self {
            nodes
        }
    }

    pub async fn start(&self) {
        let handles = 
            self
                .nodes
                .iter()
                .map(|node| {
                    node.listen()
                });

        let mut servers = Vec::new();

        for handle in handles {
            servers.push(handle.await);
        }
        println!("servers: {:?}", servers);
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    fn sample_tcp_nodes() -> Vec<TcpNode> {
        (1..6)
        .map(|idx| {
            TcpNode::new(idx as usize, "127.0.0.1", (9000 + (idx as u16)) as u16)
        })
        .collect()
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 5)]
    async fn test_new_pool() {

        let nodes = sample_tcp_nodes();
        let pool = TcpNodePool::new(nodes);
        dbg!("{:?}", pool.clone());
        pool.start().await;
    }
}
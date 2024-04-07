use std::net::SocketAddr;

use super::io::BiStream;
use tokio::net::{TcpSocket, TcpStream};

use super::spawner::Spawner;

#[derive(Clone, Copy)]
pub struct TcpOutStream {
    raddr: Option<SocketAddr>,
}

impl TcpOutStream {
    pub fn new(raddr: Option<SocketAddr>) -> Self {
        Self { raddr }
    }
}

impl Spawner<TcpStream> for TcpOutStream {
    async fn spawn(self) -> std::io::Result<BiStream<TcpStream>> {
        let raddr = self.raddr.unwrap();
        let stream = TcpSocket::new_v4()?.connect(raddr).await?;
        return Ok(BiStream::new(stream));
    }

    async fn spawn_target(self, target: SocketAddr) -> std::io::Result<BiStream<TcpStream>> {
        let stream = TcpSocket::new_v4()?.connect(target).await?;
        return Ok(BiStream::new(stream));
    }
}

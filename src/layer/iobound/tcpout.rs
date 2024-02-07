use std::net::SocketAddr;

use super::io::BiStream;
use tokio::net::{TcpSocket, TcpStream};

use super::spawner::Spawner;

#[derive(Clone, Copy)]
pub struct TcpOutStream {
    raddr: SocketAddr,
}

impl TcpOutStream {
    pub fn new(raddr: SocketAddr) -> Self {
        Self { raddr }
    }
}

impl Spawner<TcpStream> for TcpOutStream {
    async fn spawn(self) -> std::io::Result<BiStream<TcpStream>> {
        let stream = TcpSocket::new_v4()?.connect(self.raddr).await?;
        return Ok(BiStream::new(stream));
    }
}

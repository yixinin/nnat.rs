use std::net::SocketAddr;

use super::io::BiStream;
use s2n_quic::{stream::BidirectionalStream, Connection};

use super::spawner::Spawner;

pub struct QuicOutStream {
    conn: Connection,
}

impl QuicOutStream {
    pub fn new(conn: Connection) -> Self {
        Self { conn: conn }
    }
}

impl Spawner<BidirectionalStream> for QuicOutStream {
    async fn spawn(mut self) -> std::io::Result<BiStream<BidirectionalStream>> {
        let stream = self.conn.open_bidirectional_stream().await?;
        return Ok(BiStream::new(stream));
    }
    async fn spawn_target(
        mut self,
        _target: SocketAddr,
    ) -> std::io::Result<BiStream<BidirectionalStream>> {
        let stream = self.conn.open_bidirectional_stream().await?;
        return Ok(BiStream::new(stream));
    }
}

use r2d2;

use crate::error::Result;

use super::Forwarder;
pub struct QuicConnectionManager {
    idx: usize,
    conns: Vec<s2n_quic::Connection>,
}

impl QuicConnectionManager {
    pub fn new(conns: Vec<s2n_quic::Connection>) -> QuicConnectionManager {
        QuicConnectionManager { idx: 0, conns }
    }
}

impl r2d2::ManageConnection for QuicConnectionManager {
    fn connect(&self) -> std::result::Result<Self::Connection, Self::Error> {
        let mut conn = self.conns[self.idx % self.conns.len()];
        let handle = tokio::runtime::Handle::current();
        let stream = handle.block_on(conn.open_bidirectional_stream())?;
        self.idx += 1;
        Ok(stream)
    }

    fn is_valid(&self, conn: &mut Self::Connection) -> std::result::Result<(), Self::Error> {
        Ok(())
    }

    fn has_broken(&self, conn: &mut Self::Connection) -> bool {
        false
    }

    type Connection = s2n_quic::stream::BidirectionalStream;

    type Error = s2n_quic::stream::Error;
}

pub struct QuicStreamConnection {
    rx: s2n_quic::stream::ReceiveStream,
    tx: s2n_quic::stream::SendStream,
}
pub struct QuicUpstream {
    rx: s2n_quic::stream::ReceiveStream,
    tx: s2n_quic::stream::SendStream,
    conn_pool: r2d2::Pool<QuicConnectionManager>,
}

impl QuicUpstream {
    pub fn new(conns: Vec<s2n_quic::Connection>) -> Result<Self> {
        let manager = QuicConnectionManager::new(conns);
        let pool = r2d2::Pool::builder().max_size(10).build(manager)?;
        let upstream = QuicUpstream { conn_pool: pool };
        Ok(upstream)
    }
}

impl Forwarder for QuicUpstream {
    fn forward(&mut self, data: &[u8]) -> Result<usize> {
        let pool = self.conn_pool.clone();
        let mut conn = pool.get()?;
        let data = bytes::Bytes::from(data);
        _ = self.tx.send_data(data)?;
        Ok(data.len())
    }
}

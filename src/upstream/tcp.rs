use std::net::SocketAddr;

use tokio::net::TcpStream;

use r2d2;

use crate::error::Result;

use super::tokioio;
use super::Forwarder;
pub struct Http1ConnectionManager {
    idx: usize,
    addrs: Vec<SocketAddr>,
}

impl Http1ConnectionManager {
    pub fn new(addrs: Vec<SocketAddr>) -> Http1ConnectionManager {
        Http1ConnectionManager { idx: 0, addrs }
    }
}

impl r2d2::ManageConnection for Http1ConnectionManager {
    fn connect(&self) -> std::result::Result<Self::Connection, Self::Error> {
        let addr = self.addrs[self.idx % self.addrs.len()];
        let handle = tokio::runtime::Handle::current();
        let stream = handle.block_on(TcpStream::connect(addr.clone()))?;
        let io = tokioio::TokioIo::new(stream);
        let hs = hyper::client::conn::http1::handshake(io);
        let (sender, conn) = handle.block_on(hs)?;

        self.idx += 1;
        Ok(Http1Connection::new(sender, conn))
    }

    fn is_valid(&self, conn: &mut Self::Connection) -> std::result::Result<(), Self::Error> {
        Ok(())
    }

    fn has_broken(&self, conn: &mut Self::Connection) -> bool {
        false
    }

    type Connection = TcpStream;

    type Error = std::io::Error;
}

pub struct TcpUpstream {
    rx: tokio::net::tcp::OwnedReadHalf,
    tx: tokio::net::tcp::OwnedWriteHalf,
    conn_pool: r2d2::Pool<Http1ConnectionManager>,
}

impl TcpUpstream {
    pub async fn new(addrs: Vec<SocketAddr>) -> Result<Self> {
        let manager = Http1ConnectionManager::new(addrs);
        let pool = r2d2::Pool::builder().max_size(10).build(manager)?;
        let upstream = TcpUpstream { conn_pool: pool };
        Ok(upstream)
    }
}

impl Forwarder for TcpUpstream {
    fn forward(&mut self, data: &[u8]) -> Result<usize> {
        let n = self.tx.try_write(data)?;
        Ok(n)
    }
}

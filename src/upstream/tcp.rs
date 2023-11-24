use std::net::SocketAddr;

use futures::TryFutureExt;
use tokio::net::TcpStream;

use r2d2;

use crate::error::Result;
pub struct TcpsStream(SocketAddr);

impl r2d2::ManageConnection for TcpsStream {
    fn connect(&self) -> std::result::Result<Self::Connection, Self::Error> {
        let stream = TcpStream::connect(self.0.clone());

        let stream = stream.and_then(|stream| async move {
            return stream;
        });
        Ok(stream as Self::Connection);
    }

    fn is_valid(&self, conn: &mut Self::Connection) -> std::result::Result<(), Self::Error> {
        Ok(())
    }

    fn has_broken(&self, conn: &mut Self::Connection) -> bool {
        false
    }

    type Connection = tokio::net::TcpStream;

    type Error = std::io::Error;
}

pub struct TcpUpstream {
    addr: SocketAddr,
    conn_pool: r2d2::Pool<TcpsStream>,
}

impl TcpUpstream {
    pub async fn new(addr: SocketAddr) -> Result<Self> {
        let manager = TcpsStream(addr.clone());

        let upstream = TcpUpstream {
            addr: addr,
            conn_pool: r2d2::Pool::new(manager)?,
        };
        Ok(upstream)
    }

    pub async fn forward(&mut self, data: &[u8]) -> Result<usize> {
        let stream = self.conn_pool.get()?;
        return stream.try_write(data)?;
    }
}

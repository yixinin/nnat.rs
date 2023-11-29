use bytes::Bytes;
use h2::client::{Connection, SendRequest};
use http::{HeaderMap, Request};
use tokio::net::TcpStream;

use crate::error::Result;

pub struct Http2Upstream {
    sender: SendRequest<Bytes>,
    conn: Connection<TcpStream>,
}

impl Http2Upstream {
    pub fn new(sender: SendRequest<Bytes>, conn: Connection<TcpStream>) -> Self {
        Http2Upstream {
            sender: sender,
            conn: conn,
        }
    }

    pub async fn forward(
        &self,
        req: Request<()>,
        headers: HeaderMap,
        body: tokio::io::AsyncRead,
    ) -> Result<()> {
        let (f, mut stream) = self.sender.send_request(req, false)?;

        stream.send_data(body, false)?;
        let rx = f.await?;
        Ok(())
    }
}

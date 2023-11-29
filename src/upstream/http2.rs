use std::{net::SocketAddr, rc};

use bytes::Bytes;
use h2::client::{Connection, SendRequest};
use http::{HeaderMap, Request};
use tokio::net::TcpStream;

use super::http::HttpForward;
use crate::error::Result;

pub struct Http2Upstream {
    raddr: String,
    sender: SendRequest<Bytes>,
    conn: Connection<TcpStream>,
}

impl Http2Upstream {
    pub async fn new(raddr: String) -> Result<Self> {
        let addr: SocketAddr = raddr.parse().unwrap();
        let stream = TcpStream::connect(addr).await?;
        let (sender, conn) = h2::client::handshake(stream)?;
        let s = Http2Upstream {
            raddr: raddr,
            sender: sender,
            conn: conn,
        };
        Ok(s)
    }

    pub async fn forward<R>(&self, req: Request<()>, rx: R) -> Result<()>
    where
        R: tokio::io::AsyncRead,
    {
        let (f, mut stream) = self.sender.send_request(req, false)?;

        std::io::copy(&mut rx, &mut stream);
        let rx = f.await?;
        let rx = rx.into_body();

        Ok(())
    }
}

impl<R, W> HttpForward<R, W> for Http2Upstream
where
    R: std::io::Read,
    W: std::io::Write,
{
    fn forward(&self, req: Request<()>, body: R, writer: W) -> Request<()> {
        let (f, mut stream) = self.sender.send_request(req, false)?;
        tokio::spawn(async move {
            let rx = f.await?.into_body();
            std::io::copy(&mut rx, &mut writer);
        });
        std::io::copy(&mut body, &mut stream);
    }
}

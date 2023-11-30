use std::net::SocketAddr;

use super::http::HttpForward;
use bytes::Bytes;
use http::{HeaderMap, Request};
use http_body_util::Full;
use hyper::body::{Body, Incoming};
use hyper::client::conn::http1::{Connection, SendRequest};
use tokio::net::TcpStream;

use crate::error::Result;
use crate::TcpStreamIo;

pub struct Http1Upstream<B>
where
    B: Body + 'static,
{
    raddr: String,
    sender: SendRequest<B>,
    conn: Connection<TcpStreamIo, B>,
}

impl<B> Http1Upstream<B>
where
    B: Body + 'static,
{
    pub async fn new(raddr: String) -> Result<Self> {
        let addr: SocketAddr = raddr.parse().unwrap();
        let stream = TcpStream::connect(addr).await?;
        let io = TcpStreamIo(stream);
        let (sender, conn) = hyper::client::conn::http1::handshake(io).await?;
        let s = Http1Upstream {
            raddr: raddr,
            sender: sender,
            conn: conn,
        };
        Ok(s)
    }
}
impl<R, W, B> HttpForward<R, W> for Http1Upstream<B>
where
    R: std::io::Read,
    W: std::io::Write,
    B: Body + 'static,
{
    fn forward(&self, req: Request<()>, mut body: R, mut writer: W) -> Result<()> {
        let mut buf = Vec::new();
        let n = body.read_to_end(&mut buf)?;
        let mut req = Request::builder()
            .uri(req.uri())
            .body(Full::new(Bytes::from(buf.as_slice())))?
            .into()?;
        let mut resp = futures::executor::block_on(self.sender.send_request(req))?;

        std::io::copy(&mut resp, &mut writer);
        Ok(())
    }
}

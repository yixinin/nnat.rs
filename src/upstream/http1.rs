use std::net::SocketAddr;

use super::HttpForwarder;
use bytes::Bytes;
use http::{HeaderMap, Request, Response};
use http_body_util::{BodyExt, Full};
use hyper::body::{Body, Incoming};
use hyper::client::conn::http1::{Connection, SendRequest};
use tokio::net::TcpStream;

use crate::error::Result;
use crate::TcpStreamIo;

pub struct Http1Upstream<B>
where
    B: 'static + Body + Unpin,
{
    raddr: String,
    sender: SendRequest<B>,
    conn: Connection<TcpStreamIo, B>,
}

impl<B> Http1Upstream<B>
where
    B: Body + Unpin,
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
impl<B> HttpForwarder<B> for Http1Upstream<B>
where
    B: Body + Unpin,
{
    fn forward(&mut self, req: Request<()>, mut body_in: B, body_out: B) -> Result<Response<()>> {
        let mut buf = Vec::new();

        let f = body_in.frame();
        if let Some(Ok(f)) = futures::executor::block_on(f) {
            if f.is_data() {
                if let Some(data) = f.data_ref() {
                    let mut req = Request::builder().uri(req.uri()).body(data)?;
                    let mut resp = futures::executor::block_on(self.sender.send_request(req))?;
                }
            }
        }

        let ret = Response::builder().body(())?;
        Ok(ret)
    }
}

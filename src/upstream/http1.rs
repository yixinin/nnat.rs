use bytes::Bytes;
use http::{HeaderMap, Request};
use hyper::body::{Body, Incoming};
use hyper::client::conn::http1::{Connection, SendRequest};
use tokio::net::TcpStream;

use crate::error::Result;
use crate::tokioio::TokioIo;

pub struct Http1Upstream<B>
where
    B: Body + 'static,
{
    sender: SendRequest<B>,
    conn: Connection<TokioIo<TcpStream>, B>,
}

impl<B> Http1Upstream<B>
where
    B: Body + 'static,
{
    pub fn new(sender: SendRequest<B>, conn: Connection<TokioIo<TcpStream>, B>) -> Self {
        Http1Upstream {
            sender: sender,
            conn: conn,
        }
    }

    pub fn forward(&self, req: Request<()>, headers: HeaderMap, body: B) -> Result<()> {
        let mut b = Request::builder().uri(req.uri()); //.body(body)?;
        for (k, v) in headers {
            if let Some(name) = k {
                b = b.header(name, v);
            }
        }
        let req = b.body(body)?;
        self.sender.send_request(req);
        Ok(())
    }
}

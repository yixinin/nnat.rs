use crate::error::Result;
use bytes::Bytes;
use futures::Future;
use http::{Request, Response};

pub trait HttpForward<R, W>
where
    R: std::io::Read,
    W: std::io::Write,
{
    fn forward(&self, req: Request<()>, body: R, writer: W) -> Request<()>;
}
pub trait StreamForward<R, W>
where
    R: std::io::Read,
    W: std::io::Write,
{
    fn forward(&self, reader: R, writer: W) -> Result<()>;
}

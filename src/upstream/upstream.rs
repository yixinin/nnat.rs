use crate::error::Result;

use super::tokioio;

pub trait Forwarder {
    fn forward(&mut self, data: &[u8]) -> Result<usize>;
}

pub struct Upstream<F>
where
    F: Forwarder + Clone + Sync + Send + 'static,
{
    pub u: F,
}

pub async fn forward() {
    let req = hyper::Request::builder().uri("uri").body("body").unwrap();
    let stream = tokio::net::TcpStream::connect("addr").await?;
    let io = tokioio::TokioIo::new(stream);
    let (sender, conn) = hyper::client::conn::http1::Builder::new().handshake(stream);
}

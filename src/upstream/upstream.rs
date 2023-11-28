use tokio::net::TcpSocket;

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

pub async fn forward_h1() -> Result<usize> {
    let req = hyper::Request::builder()
        .uri("uri")
        .body(bytes::Bytes::new())
        .unwrap();

    let stream = tokio::net::TcpStream::connect("addr").await?;

    let io = tokioio::TokioIo::new(stream);
    let (mut sender, conn) = hyper::client::conn::http1::handshake(io).await?;
    let resp = sender.send_request(req).await?;

    Ok(0)
}

pub async fn forward_h2() -> Result<usize> {
    let req = hyper::Request::builder().uri("uri").body(()).unwrap();

    let stream = tokio::net::TcpStream::connect("addr").await?;

    // let io = tokioio::TokioIo::new(stream);

    // let (sender, conn) = hyper::client::handshake(io).await?;
    let (mut sender, conn) = h2::client::handshake(stream).await?;
    let (resp, mut stream) = sender.send_request(req, false)?;

    Ok(0)
}
pub async fn forward_h3(conn: s2n_quic::Connection) -> Result<usize> {
    let req = http::Request::builder().uri("uri").body(()).unwrap();
    let (conn, sender) = h3::client::new(s2n_quic_h3::Connection::new(conn)).await?;
    let stream = sender.send_request(req).await?;
    stream.finish();
    stream.recv_data();

    Ok(0)
}

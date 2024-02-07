use tokio::net::TcpSocket;

use std::io;
use std::net::SocketAddr;

use super::io::BiStream;
use super::spawner::Spawner;
use axum::{
    body::Body,
    extract::Request,
    http::{Method, StatusCode},
    response::{IntoResponse, Response},
    routing::get,
    Router,
};

use hyper::body::Incoming;
use hyper::server::conn::http1;
use hyper::upgrade::Upgraded;
use tokio::net::{TcpListener, TcpStream};
use tower::Service;
use tower::ServiceExt;

use hyper_util::rt::TokioIo;

pub struct TcpProxy<T, S>
where
    T: tokio::io::AsyncRead + tokio::io::AsyncWrite + std::marker::Unpin + Send + 'static,
    S: Spawner<T> + Copy + Send,
{
    laddr: SocketAddr,
    out: S,
    _t: Option<T>,
}

impl<T, S> TcpProxy<T, S>
where
    T: tokio::io::AsyncRead + tokio::io::AsyncWrite + std::marker::Unpin + Send + 'static,
    S: Spawner<T> + Copy + Send,
{
    pub fn new(out: S, laddr: SocketAddr) -> std::io::Result<TcpProxy<T, S>> {
        let tower_service = tower::service_fn(move |req: Request<_>| {
            let router_svc = router_svc.clone();
            let req = req.map(Body::new);
            async move {
                if req.method() == Method::CONNECT {
                    proxy(req).await
                } else {
                    router_svc.oneshot(req).await.map_err(|err| match err {})
                }
            }
        });
        let p = TcpProxy {
            laddr: laddr,
            out: out,
            _t: None,
        };
        Ok(p)
    }
    pub async fn run(self) -> io::Result<()> {
        let socket = TcpSocket::new_v4()?;
        socket.bind(self.laddr)?;
        let lis = socket.listen(1024)?;

        loop {
            let (stream_in, raddr) = lis.accept().await?;
            println!("accept new conn: {}", raddr);

            tokio::task::spawn(async move {});
        }
    }
}

async fn proxy(req: Request) -> Result<Response, hyper::Error> {
    if let Some(host_addr) = req.uri().authority().map(|auth| auth.to_string()) {
        tokio::task::spawn(async move {
            match hyper::upgrade::on(req).await {
                Ok(upgraded) => {
                    if let Err(e) = tunnel(upgraded, host_addr).await {
                        println!("server io error: {}", e);
                    };
                }
                Err(e) => println!("upgrade error: {}", e),
            }
        });

        Ok(Response::new(Body::empty()))
    } else {
        println!("CONNECT host is not socket addr: {:?}", req.uri());
        Ok((
            StatusCode::BAD_REQUEST,
            "CONNECT must be to a socket address",
        )
            .into_response())
    }
}

async fn tunnel(upgraded: Upgraded, addr: String) -> std::io::Result<()> {
    let mut server = TcpStream::connect(addr).await?;
    let mut upgraded = TokioIo::new(upgraded);

    let (from_client, from_server) =
        tokio::io::copy_bidirectional(&mut upgraded, &mut server).await?;

    println!(
        "client wrote {} bytes and received {} bytes",
        from_client, from_server
    );

    Ok(())
}

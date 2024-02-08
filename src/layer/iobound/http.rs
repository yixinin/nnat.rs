use clap::error;
use tokio::net::TcpSocket;
use tower::BoxError;
use tower::{util::ServiceFn, Service};

use std::net::SocketAddr;
use std::{future::Future, io};

use super::spawner::Spawner;
use axum::{
    body::Body,
    extract::Request,
    http::{Method, StatusCode},
    response::{IntoResponse, Response},
};
use hyper_util::{client::legacy::connect::HttpConnector, rt::TokioExecutor};

use hyper::upgrade::Upgraded;
use hyper::{body::Incoming, server::conn::http1};

use hyper_util::rt::TokioIo;

pub struct TcpProxy<T, S>
where
    T: tokio::io::AsyncRead + tokio::io::AsyncWrite + std::marker::Unpin + Send + 'static,
    S: Spawner<T> + Copy + Send + 'static,
{
    laddr: SocketAddr,
    out: S,
    _t: Option<T>,
}

impl<T, S> TcpProxy<T, S>
where
    T: tokio::io::AsyncRead + tokio::io::AsyncWrite + std::marker::Unpin + Send + 'static,
    S: Spawner<T> + Copy + Send + 'static,
{
    pub fn new(out: S, laddr: SocketAddr) -> std::io::Result<TcpProxy<T, S>> {
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
            let (stream, raddr) = lis.accept().await?;
            println!("accept new conn: {}", raddr);
            let io = TokioIo::new(stream);

            let tower_service = tower::service_fn(move |req: Request<_>| {
                let proxy = ProxyService::new(self.out.clone());
                let req = req.map(Body::new);
                async move {
                    if req.method() == Method::CONNECT {
                        proxy.proxy(req).await
                    } else {
                        proxy.request(req).await
                    }
                }
            });

            let hyper_service = hyper::service::service_fn(move |request: Request<Incoming>| {
                tower_service.clone().call(request)
            });

            tokio::task::spawn(async move {
                if let Err(err) = http1::Builder::new()
                    .preserve_header_case(true)
                    .title_case_headers(true)
                    .serve_connection(io, hyper_service)
                    .with_upgrades()
                    .await
                {
                    println!("Failed to serve connection: {:?}", err);
                }
            });
        }
    }
}

pub struct ProxyService<T, S>
where
    T: tokio::io::AsyncRead + tokio::io::AsyncWrite + std::marker::Unpin + Send + 'static,
    S: Spawner<T> + Copy + Send + 'static,
{
    out: S,
    _t: Option<T>,
}

impl<T, S> ProxyService<T, S>
where
    T: tokio::io::AsyncRead + tokio::io::AsyncWrite + std::marker::Unpin + Send + 'static,
    S: Spawner<T> + Copy + Send + 'static,
{
    pub fn new(out: S) -> ProxyService<T, S> {
        ProxyService { out: out, _t: None }
    }

    async fn proxy(self, req: Request) -> Result<Response, hyper::Error> {
        if let Some(host_addr) = req.uri().authority().map(|auth| auth.to_string()) {
            tokio::task::spawn(async move {
                match hyper::upgrade::on(req).await {
                    Ok(upgraded) => {
                        if let Err(e) = self.tunnel(upgraded, host_addr).await {
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

    async fn tunnel(self, upgraded: Upgraded, addr: String) -> std::io::Result<()> {
        let mut upgraded = TokioIo::new(upgraded);
        let raddr = addr.parse().unwrap();
        let mut server = self.out.spawn_target(raddr).await?;
        let (from_client, from_server) =
            tokio::io::copy_bidirectional(&mut upgraded, &mut server).await?;

        println!(
            "client wrote {} bytes and received {} bytes",
            from_client, from_server
        );

        Ok(())
    }
    async fn request(self, req: Request) -> Result<Response, hyper::Error> {
        let client = hyper::client::conn::http1::Builder::new();
        if let Some(host_addr) = req.uri().authority().map(|auth| auth.to_string()) {
            match host_addr.parse() {
                Ok(raddr) => {
                    println!("connect to {}", raddr);
                    let stream = TcpSocket::new_v4().unwrap().connect(raddr).await.unwrap();
                    let (mut send, _) = client.handshake(TokioIo::new(stream)).await?;

                    let resp = send.send_request(req).await?;
                    return Ok(resp.into_response());
                }
                Err(e) => {
                    panic!("{} err: {}", host_addr, e)
                }
            }
        } else {
            panic!("addr parse error");
        }
    }
}

impl<T, S> Clone for ProxyService<T, S>
where
    T: tokio::io::AsyncRead + tokio::io::AsyncWrite + std::marker::Unpin + Send + 'static,
    S: Spawner<T> + Copy + Send + 'static,
{
    fn clone(&self) -> Self {
        Self {
            out: self.out.clone(),
            _t: None,
        }
    }
}

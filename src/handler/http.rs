use std::net::SocketAddr;
use std::sync::Arc;

use super::Acceptor;
use crate::error::Result;
use tokio::net::TcpListener;

use hyper::service::service_fn;
use hyper::{
    body::{Bytes, Incoming},
    Request,
};

use hyper::rt::bounds::Http2ClientConnExec;

pub struct HttpHandler {
    laddr: SocketAddr,
    tls_on: bool,
}

impl HttpHandler {
    pub fn new(laddr: SocketAddr, tls: bool) -> HttpHandler {
        HttpHandler {
            laddr: laddr,
            tls_on: tls,
        }
    }

    pub async fn handle_request() {}

    async fn service(
        handler: Arc<HttpHandler>,
        req: Request<Incoming>,
        client_addr: SocketAddr,
        listen_addr: SocketAddr,
        tls_on: bool,
        server_name: String,
    ) -> Result<hyper::Response<Bytes>> {
        handler
            .handle_request(req, client_addr, listen_addr, tls_on, server_name)
            .await
    }

    pub async fn serve_tcp_http1(self, server_name: String) -> Result<()> {
        let b = hyper::server::conn::http1::Builder::new();
        let lis = TcpListener::bind(self.laddr).await?;
        while let Ok((stream, raddr)) = lis.accept().await {
            b.serve_connection(
                stream,
                service_fn(move |req: Request<Incoming>| {
                    Self::service(
                        self.handler.clone(),
                        req,
                        raddr,
                        self.laddr,
                        self.tls_on,
                        server_name.clone(),
                    )
                }),
            )
        }
    }

    pub async fn serve_quic(self, server_name: String) -> Result<()> {
        let lis = TcpListener::bind(self.laddr).await?;
        while let Ok((stream, raddr)) = lis.accept().await {
            b.serve_connection(
                stream,
                service_fn(move |req: Request<Incoming>| {
                    Self::service(
                        self.handler.clone(),
                        req,
                        raddr,
                        self.laddr,
                        self.tls_on,
                        server_name.clone(),
                    )
                }),
            )
        }
    }
}

impl Acceptor for HttpHandler {
    fn accept(self) {}
}

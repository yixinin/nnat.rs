use std::net::SocketAddr;
use std::sync::Arc;

use super::Acceptor;
use crate::error::Result;
use h2;
use h3;
use h3::server::RequestStream;
use hyper::service::service_fn;
use hyper::{
    body::{Bytes, Incoming},
    Request,
};
use s2n_quic::provider::io::tokio::Builder as IOBuilder;
use s2n_quic::stream::BidirectionalStream;
use s2n_quic::Server;
use std::error::Error;
use std::path::Path;
use std::time::Duration;
use tokio::net::TcpListener;
use tokio::net::{TcpSocket, UdpSocket};

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
            );
        }
        Ok(())
    }

    pub async fn serve_quic(self, server_name: String) -> Result<()> {
        let socket = UdpSocket::bind(self.laddr).await?;
        let tx = socket.into_std()?;
        let rx = tx.try_clone()?;

        let tls = s2n_quic::provider::tls::default::Server::builder()
            .with_certificate(Path::new("quic.crt"), Path::new("quic.key"))?
            .build()?;
        let socket_io = IOBuilder::default()
            .with_tx_socket(tx)?
            .with_rx_socket(rx)?
            .build()?;
        println!("recv conn from frontend, start listen quic ...");
        let mut server = Server::builder()
            .with_tls(tls)?
            .with_io(socket_io)?
            .start()?;

        println!("quic server started, accept msg ...");
        while let Some(mut conn) = server.accept().await {
            // let mut builder = h3::server::builder()
            //     .enable_datagram(true)
            //     .enable_connect(true)
            //     .enable_webtransport(true);
            let conn = s2n_quic_h3::Connection::new(conn);
            let mut b: h3::server::Connection<s2n_quic_h3::Connection, bytes::Bytes> =
                h3::server::Connection::new(conn).await.unwrap();

            if let Ok(Some((req, stream))) = b.accept().await {
                println!("new request: {:#?}", req);
            }
        }
        Ok(())
    }
}

impl Acceptor for HttpHandler {
    fn accept(self) {}
}

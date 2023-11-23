use s2n_quic::provider::io::tokio::Builder as IOBuilder;
use s2n_quic::stream::BidirectionalStream;
use s2n_quic::Server;
use std::error::Error;
use std::net::SocketAddr;
use std::path::Path;
use std::time::Duration;
use tokio::net::{TcpSocket, UdpSocket};

use crate::error::Result;

pub struct StreamHandler {
    laddr: SocketAddr,
    tls_on: bool,
}
impl StreamHandler {
    pub fn new(laddr: SocketAddr, tls_on: bool) -> StreamHandler {
        StreamHandler {
            laddr: laddr,
            tls_on: tls_on,
        }
    }

    pub async fn serve_quic(self) -> Result<()> {
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
        while let Some(mut connection) = server.accept().await {  
            tokio::spawn(async move {
                while let Some(stream) = connection.accept_bidirectional_stream().await? {
                    let (mut rx, mut tx) = stream.split();
                }
            })
        }
    }
}

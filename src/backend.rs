use s2n_quic::provider::io::tokio::Builder as IOBuilder;
use s2n_quic::Server;
use std::error::Error;
use std::net::SocketAddr;
use std::path::Path;
use std::time::Duration;
use tokio::net::UdpSocket;

use crate::endpoint::Kind;
use crate::message::{self, ConnMessage, Message, StunMessage};
use crate::tunnel;

pub struct Backend {
    fqdn: String,
    laddr: SocketAddr,
    stun_addr: String,
}

impl Backend {
    pub fn new(fqdn: &str, laddr: SocketAddr, stun_addr: &str) -> Self {
        return Backend {
            fqdn: fqdn.to_string(),
            laddr: laddr,
            stun_addr: stun_addr.to_string(),
        };
    }

    pub async fn run(self) -> Result<(), Box<dyn Error>> {
        let laddr = self.laddr.clone();
        let stun_addr = self.stun_addr.clone();
        let fqdn = self.fqdn.clone();
        while let Ok(socket) = Self::fetch(stun_addr.clone(), fqdn.clone()).await {
            tokio::spawn(async move {
                _ = Self::handle(socket, laddr.clone()).await;
            });
        }

        Ok(())
    }

    pub async fn fetch(stun_addr: String, fqdn: String) -> Result<UdpSocket, Box<dyn Error>> {
        let socket = UdpSocket::bind("0.0.0.0:0").await?;
        let stun_addr = stun_addr;
        let fqdn = fqdn;

        let msg = StunMessage::new(Kind::Backend, fqdn);
        let data = msg.encode()?;
        let mut buf = [0; 1500];
        loop {
            let data = data.clone();
            _ = socket.send_to(&data, stun_addr.clone()).await?;

            tokio::time::sleep(Duration::from_secs(2)).await;

            if let Ok((n, raddr)) = socket.try_recv_from(&mut buf) {
                let msg = message::decode(&buf[..n]);
                match msg {
                    Message::Stun(msg) => {
                        println!("recv stun message {} from: {}", msg, raddr.to_string(),);
                    }
                    Message::Conn(msg) => {
                        println!("recv conn message {} from {}", msg, raddr.to_string());

                        let target_addr = msg.raddr.clone();
                        let msg = ConnMessage::new(Kind::Backend, socket.local_addr()?, msg.fqdn);
                        let data = msg.encode()?;
                        _ = socket.send_to(&data, target_addr).await?;
                        return Ok(socket);
                    }
                    Message::Unknown(data) => {
                        println!("reccv unknown msg {:?}", data);
                    }
                }
            }
        }
    }
    pub async fn handle(socket: UdpSocket, laddr: SocketAddr) -> Result<(), Box<dyn Error>> {
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
            // spawn a new task for the connection
            _ = tokio::spawn(async move {
                println!("Connection accepted from {:?}", connection.remote_addr());

                while let Ok(Some(stream)) = connection.accept_bidirectional_stream().await {
                    _ = tokio::spawn(async move {
                        _ = tunnel::backward_tunnel(laddr.clone(), stream).await;
                    });
                }
            });
        }
        Ok(())
    }
}

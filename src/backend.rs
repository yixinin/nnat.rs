use s2n_quic::provider::io::tokio::Builder as IOBuilder;
use s2n_quic::Server;
use std::error::Error;
use std::path::Path;
use std::time::Duration;
use tokio::net::{TcpListener, TcpSocket, UdpSocket};

use crate::endpoint::Kind;
use crate::message::{self, ConnMessage, Message, StunMessage};

pub struct Backend {
    fqdn: String,
    laddr: String,
    stun_addr: String,
}

impl Backend {
    pub fn new(fqdn: &str, laddr: &str, stun_addr: &str) -> Self {
        return Backend {
            fqdn: fqdn.to_string(),
            laddr: laddr.to_string(),
            stun_addr: stun_addr.to_string(),
        };
    }

    pub async fn run(self) -> Result<(), Box<dyn Error>> {
        let laddr = self.laddr.clone();
        let stun_addr = self.stun_addr.clone();
        let fqdn = self.fqdn.clone();
        let socket = self.fetch(laddr.clone(), stun_addr, fqdn).await?;

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
            let t: tokio::task::JoinHandle<Result<(), std::io::Error>> = tokio::spawn(async move {
                eprintln!("Connection accepted from {:?}", connection.remote_addr());

                while let Ok(Some(mut stream)) = connection.accept_bidirectional_stream().await {
                    // // spawn a new task for the stream
                    // tokio::spawn(async move {
                    //     eprintln!("Stream opened from {:?}", stream.connection().remote_addr());

                    //     // echo any data back to the stream
                    //     while let Ok(Some(data)) = stream.receive().await {
                    //         stream.send(data).await.expect("stream should be open");
                    //     }
                    // });
                    // let (quic_rx, quic_tx) = stream.split();
                    // let tcp_socket = TcpSocket::new_v4()?;
                    // let addr = "127.0.0.1:8080".parse().unwrap();
                    // let tcp_stream = tcp_socket.connect(addr).await?; 
                    // let (tcp_rx, tcp_tx) = tcp_stream.split();

                    // let rx = tokio::spawn(async move {
                    //     tokio::io::copy(&mut tcp_rx, &mut quic_tx).await?;
                    // });
                    // let tx = tokio::spawn(async move {
                    //     tokio::io::copy(&mut quic_rx, &mut tcp_tx).await?;
                    // });
                }

                Ok(())
            });
        }

        Ok(())
    }

    pub async fn fetch(
        self,
        laddr: String,
        stun_addr: String,
        fqdn: String,
    ) -> Result<UdpSocket, Box<dyn Error>> {
        let socket = UdpSocket::bind(laddr).await?;
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
}

use s2n_quic::provider::io::tokio::Builder as IOBuilder;
use s2n_quic::Server;
use std::error::Error;
use std::net::SocketAddr;
use std::path::Path;
use std::time::Duration;
use tokio::net::UdpSocket;
use tokio::time;

use crate::endpoint::Kind;
use crate::message::StunMessage;

pub struct Backend {
    fqdn: String,
    stun_addr: SocketAddr,
    laddr: SocketAddr,
}

impl Backend {
    pub fn new(fqdn: &str, stun_addr: SocketAddr, laddr: SocketAddr) -> Self {
        return Backend {
            fqdn: fqdn.to_string(),
            stun_addr: stun_addr,
            laddr: laddr,
        };
    }
    pub async fn run(self) -> Result<(), Box<dyn Error>> {
        let socket = UdpSocket::bind(self.laddr).await?;
        let std_socket = socket.into_std()?;

        let socket = UdpSocket::from_std(std_socket.try_clone()?)?;
        tokio::spawn(async move {
            _ = self.stun_send(socket).await;
        })
        .await?;

        let socket_io = IOBuilder::default().with_rx_socket(std_socket)?.build()?;
        let mut server = Server::builder()
            .with_tls((Path::new("quic.crt"), Path::new("quic.key")))?
            .with_io(socket_io)?
            .start()?;

        while let Some(mut connection) = server.accept().await {
            // spawn a new task for the connection
            tokio::spawn(async move {
                eprintln!("Connection accepted from {:?}", connection.remote_addr());

                while let Ok(Some(mut stream)) = connection.accept_bidirectional_stream().await {
                    // spawn a new task for the stream
                    tokio::spawn(async move {
                        eprintln!("Stream opened from {:?}", stream.connection().remote_addr());

                        // echo any data back to the stream
                        while let Ok(Some(data)) = stream.receive().await {
                            stream.send(data).await.expect("stream should be open");
                        }
                    });
                }
            });
        }

        Ok(())
    }

    pub async fn stun_send(self, socket: UdpSocket) -> Result<(), Box<dyn Error>> {
        let stun_addr = self.stun_addr.clone();
        let fqdn = self.fqdn.clone();

        let mut ticker = time::interval(Duration::from_secs(10));
        let msg = StunMessage::new(Kind::Backend, fqdn);
        let data = msg.encode()?;
        let mut buf = [0; 1500];
        loop {
            _ = tokio::select! {
                 _= ticker.tick() => {
                    let data = data.clone();
                    _ = socket.send_to(&data, stun_addr.clone()).await?;
                },
                r = socket.recv_from(&mut buf)=>{
                    let (n, raddr)= r?;
                    let mut msg = StunMessage::default();
                    _ = msg.decode(&buf[..n])?;
                    println!(
                        "recv stun message from: {} raddr: {}, fqdn: {}",
                        msg.kind,
                        raddr.to_string(),
                        msg.fqdn,
                    )
                },
            };
        }
    }
}

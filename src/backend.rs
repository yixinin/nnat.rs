use s2n_quic::provider::io::tokio::Builder as IOBuilder;
use s2n_quic::Server;
use std::error::Error;
use std::net::SocketAddr;
use std::net::UdpSocket;
use std::path::Path;
use tokio::time;

use crate::endpoint::Kind;
use crate::message;

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
        let socket = UdpSocket::bind(self.laddr)?;
        let socket_clone = socket.try_clone()?;

        tokio::spawn(async move {
            _ = self.keepalive(socket).await;
        });
        let socket_io: s2n_quic::provider::io::Default =
            IOBuilder::default().with_rx_socket(socket_clone)?.build()?;
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

    pub async fn keepalive(self, udp: UdpSocket) -> Result<(), Box<dyn Error>> {
        let mut ticker = time::interval(time::Duration::from_secs(10));
        let msg = message::StunMessage::new(Kind::Backend, self.fqdn);
        let buf = msg.encode()?;
        loop {
            ticker.tick().await;
            let buf = buf.clone();
            udp.send_to(&buf, self.stun_addr)?;
        }
    }
}

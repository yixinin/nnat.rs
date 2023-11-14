use s2n_quic::client::Connect;
use s2n_quic::provider::io::tokio::Builder as IOBuilder;
use s2n_quic::{Client, Server};
use std::error::Error;
use std::net::SocketAddr;
use std::net::UdpSocket;
use std::os::windows::io::AsRawSocket;
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
        let udp = socket.try_clone()?;
        tokio::spawn(async move {
            _ = self.keepalive(socket).await;
        });
        let p = IOBuilder::default().with_tx_socket(udp)?.build()?;
        let mut server = Server::builder()
            .with_tls((Path::new("quic.crt"), Path::new("quic.key")))?
            .with_io(p)?
            .start()?;

        loop {
            match server.accept().await {
                Some(mut rconn) => {
                    let stream = rconn.open_bidirectional_stream().await?;
                    let (mut receive_stream, mut send_stream) = stream.split();
                    tokio::spawn(async move {
                        let mut stdout = tokio::io::stdout();
                        let _ = tokio::io::copy(&mut receive_stream, &mut stdout).await;
                    });

                    // copy data from stdin and send it to the server
                    let mut stdin = tokio::io::stdin();
                    tokio::io::copy(&mut stdin, &mut send_stream).await?;
                }
                None => {}
            }
        }
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

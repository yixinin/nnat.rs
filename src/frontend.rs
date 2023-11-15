use s2n_quic::provider::io::tokio::Builder as IOBuilder;
use s2n_quic::{client::Connect, Client};
use std::error::Error;
use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4, UdpSocket};
use std::path::Path;

use crate::{endpoint, message};
use endpoint::Kind;
use message::{ConnMessage, StunMessage};

pub struct Frontend {
    fqdn: String,
    stun_addr: SocketAddr,
}

impl Frontend {
    pub fn new(fqdn: &str, stun_addr: SocketAddr) -> Self {
        Frontend {
            fqdn: fqdn.to_string(),
            stun_addr: stun_addr,
        }
    }

    pub async fn run(self) -> Result<(), Box<dyn Error>> {
        let laddr = SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::new(0, 0, 0, 0), 0));
        let socket = UdpSocket::bind(laddr)?;
        let msg = StunMessage::new(Kind::Frontend, self.fqdn.clone());
        let data = msg.encode()?;

        let mut buf = [0; 1500];

        let data = data.clone();
        _ = socket.send_to(&data, self.stun_addr)?;

        let (n, _) = socket.recv_from(&mut buf)?;

        let mut msg = ConnMessage::default();
        _ = msg.decode(&buf[..n])?;

        println!(
            "recv conn message, fqdn:{} raddr:{}",
            msg.fqdn,
            msg.raddr.to_string()
        );

        socket.send_to(&buf[..n], msg.raddr)?;

        let socket_io = IOBuilder::default().with_rx_socket(socket)?.build()?;
        let client = Client::builder()
            .with_tls(Path::new("quic.crt"))?
            .with_io(socket_io)?
            .start()?;

        // try send raw message to raddr

        let connect = Connect::new(msg.raddr).with_server_name(self.fqdn.as_str());

        let mut connection = client.connect(connect).await?;
        connection.keep_alive(true)?;
        let stream = connection.open_bidirectional_stream().await?;
        let (mut receive_stream, mut send_stream) = stream.split();

        // ensure the connection doesn't time out with inactivity
        connection.keep_alive(true)?;

        tokio::spawn(async move {
            let mut stdout = tokio::io::stdout();
            let _ = tokio::io::copy(&mut receive_stream, &mut stdout).await;
        });

        // copy data from stdin and send it to the server
        let mut stdin = tokio::io::stdin();
        tokio::io::copy(&mut stdin, &mut send_stream).await?;

        Ok(())
    }
}

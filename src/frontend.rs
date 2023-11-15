use s2n_quic::provider::io::tokio::Builder as IOBuilder;
use s2n_quic::{client::Connect, Client};
use std::error::Error;
use std::net::{SocketAddr, UdpSocket};
use std::path::Path;
use std::time::Duration;

use crate::message::MessageKind;
use crate::{endpoint, message};
use endpoint::Kind;
use message::{ConnMessage, StunMessage};

pub struct Frontend {
    fqdn: String,
    laddr: String,
    stun_addr: String,
}

impl Frontend {
    pub fn new(fqdn: &str, laddr: &str, stun_addr: &str) -> Self {
        Frontend {
            fqdn: fqdn.to_string(),
            laddr: laddr.to_string(),
            stun_addr: stun_addr.to_string(),
        }
    }

    pub async fn run(self) -> Result<(), Box<dyn Error>> {
        let laddr = self.laddr.clone();
        let stun_addr = self.stun_addr.clone();
        let fqdn = self.fqdn.clone();
        let raddr = self.fetch(laddr.clone(), stun_addr, fqdn.clone()).await?;
        // let socket_io = IOBuilder::default().with_tx_socket(socket)?.build()?;
        let client = Client::builder()
            .with_tls(Path::new("quic.crt"))?
            .with_io(laddr.as_str())?
            .start()?;

        // try send raw message to raddr

        let connect = Connect::new(raddr.clone()).with_server_name(fqdn.as_str());

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
    pub async fn fetch(
        self,
        laddr: String,
        stun_addr: String,
        fqdn: String,
    ) -> Result<SocketAddr, Box<dyn Error>> {
        let socket = UdpSocket::bind(laddr)?;
        let msg = StunMessage::new(Kind::Frontend, fqdn);
        let data = msg.encode()?;

        let mut buf = [0; 1500];

        let data = data.clone();
        _ = socket.send_to(&data, stun_addr)?;

        let (n, _) = socket.recv_from(&mut buf)?;

        let mut msg = ConnMessage::default();
        _ = msg.decode(&buf[..n])?;

        println!(
            "recv conn message, fqdn:{} raddr:{}",
            msg.fqdn,
            msg.raddr.to_string()
        );
        let raddr = msg.raddr.clone();
        let msg = msg.clone();
        let mut data = Vec::with_capacity(n + 1);
        data.push(MessageKind::Conn as u8);
        data.append(&mut msg.encode()?);

        _ = socket.send_to(&data, raddr.clone())?;

        // // recv conn from backend
        // let mut buf = [0; 1500];
        // let (n, raddr) = socket.recv_from(&mut buf)?;
        // let mut msg = ConnMessage::default();
        // _ = msg.decode(&buf[..n])?;

        // wait backend quic start
        tokio::time::sleep(Duration::from_secs(5)).await;
        Ok(raddr)
    }
}

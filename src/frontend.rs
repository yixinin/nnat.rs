use crate::message::MessageKind;
use crate::tls::NoCertVerifier;
use crate::{endpoint, message};
use endpoint::Kind;
use message::{ConnMessage, StunMessage};
use rustls::client::ClientConfig;
use s2n_quic::provider::io::tokio::Builder as IOBuilder;
use s2n_quic::{client::Connect, Client};
use std::error::Error;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;
use tokio::net::UdpSocket;

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

        let socket: UdpSocket = UdpSocket::bind(laddr).await?;
        let socket = socket.into_std()?;
        let tx_udp = socket.try_clone()?;
        let rx_udp = tx_udp.try_clone()?;

        let msg = StunMessage::new(Kind::Frontend, fqdn.clone());
        let data = msg.encode()?;
        _ = tx_udp.send_to(&data, stun_addr)?;

        let mut buf = [0; 1500];

        let (n, _) = rx_udp.recv_from(&mut buf)?;
        let mut msg = ConnMessage::default();
        _ = msg.decode(&buf[..n])?;

        let target_addr = msg.raddr.clone();
        let fqdn_clone = fqdn.clone();
        let task = tokio::spawn(async move {
            if let Ok(laddr) = tx_udp.local_addr() {
                let msg = ConnMessage::new(Kind::Frontend, laddr, fqdn_clone);
                if let Ok(data) = msg.encode() {
                    let mut ticker = tokio::time::interval(Duration::from_secs(2));
                    loop {
                        if let Err(err) = tx_udp.send_to(&data, target_addr) {
                            println!("{}", err);
                            return;
                        }
                        ticker.tick().await;
                    }
                }
            }
        });

        let mut buf = [0; 1500];
        loop {
            let (n, _) = rx_udp.recv_from(&mut buf)?;
            let mut msg = ConnMessage::default();
            msg.decode(&mut buf[..n])?;
            match msg.kind {
                Kind::Backend => {
                    task.abort();
                    break;
                }
                _ => {
                    println!(
                        "recv msg {} {} {}",
                        msg.kind,
                        msg.raddr.to_string(),
                        msg.fqdn
                    );
                }
            }
        }

        let verifier = Arc::new(NoCertVerifier {});
        let mut cb = ClientConfig::builder()
            .with_safe_default_cipher_suites()
            .with_safe_default_kx_groups()
            .with_safe_default_protocol_versions()?
            .with_custom_certificate_verifier(verifier.clone())
            .with_no_client_auth();

        cb.dangerous().set_certificate_verifier(verifier);

        let tls = s2n_quic_rustls::Client::new(cb);

        let socket_io = IOBuilder::default()
            .with_tx_socket(socket)?
            .with_rx_socket(rx_udp)?
            .build()?;
        let client = Client::builder()
            .with_tls(tls)?
            .with_io(socket_io)?
            .start()?;

        let connect = Connect::new(target_addr).with_server_name(fqdn.clone().as_str());
        let mut connection = client.connect(connect).await?;
        connection.keep_alive(true)?;
        let stream = connection.open_bidirectional_stream().await?;
        let (mut receive_stream, mut send_stream) = stream.split();

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
        let socket = UdpSocket::bind(laddr).await?;
        let msg = StunMessage::new(Kind::Frontend, fqdn);
        let data = msg.encode()?;

        let mut buf = [0; 1500];

        let data = data.clone();
        _ = socket.send_to(&data, stun_addr).await?;

        let (n, _) = socket.recv_from(&mut buf).await?;

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

        let mut ticker = tokio::time::interval(Duration::from_secs(2));
        let mut buf = [0 as u8; 1500];
        loop {
            _ = tokio::select! {
                _= ticker.tick() =>{
                    _ = socket.send_to(&data, raddr.clone()).await?;
                },
                r = socket.recv_from(&mut buf) =>{
                    let (n, raddr) = r?;
                    let mut msg = ConnMessage::default();
                    _ = msg.decode(&buf[..n])?;
                    return Ok(raddr)
                }
            }
        }
    }
}

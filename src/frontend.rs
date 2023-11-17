use crate::tls::NoCertVerifier;
use crate::{endpoint, message};
use endpoint::Kind;
use message::{ConnMessage, StunMessage};
use rustls::client::ClientConfig;
use s2n_quic::provider::io::tokio::Builder as IOBuilder;
use s2n_quic::{client::Connect, Client};
use std::error::Error;
use std::sync::Arc;
use std::time::Duration;
use tokio::net::UdpSocket;
use tokio::task::JoinHandle;

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

        let msg = StunMessage::new(Kind::Frontend, fqdn.clone());
        let data = msg.encode()?;
        _ = socket.send_to(&data, stun_addr).await?;

        let mut buf = [0; 1500];

        let (n, _) = socket.recv_from(&mut buf).await?;
        let mut msg = ConnMessage::default();
        _ = msg.decode(&buf[..n])?;

        let target_addr = msg.raddr.clone();
        let fqdn_clone = fqdn.clone();
        let task: JoinHandle<Result<UdpSocket, std::io::Error>> = tokio::spawn(async move {
            let laddr = socket.local_addr()?;
            let msg = ConnMessage::new(Kind::Frontend, laddr, fqdn_clone);
            let data = msg.encode().unwrap();
            let mut buf = [0; 1500];
            loop {
                _ = socket.send_to(&data, target_addr).await?;
                tokio::time::sleep(Duration::from_secs(2)).await;
                match socket.try_recv_from(&mut buf) {
                    Ok((n, _raddr)) => {
                        let mut msg = ConnMessage::default();
                        if let Err(err)=  msg.decode(&buf[..n]){
                            return  Err(std::io::Error::new(std::io::ErrorKind::InvalidData,err));
                        }
                        match msg.kind {
                            Kind::Backend => {
                                return  Ok(socket);
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
                    Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                        continue;
                    }
                    Err(e) => {
                        return Err(e);
                    }
                }
            }
        });

        let socket = task.await??;

        let tx_udp = socket.into_std()?;
        let rx_udp = tx_udp.try_clone()?;

        let mut buf = [0; 1500];
        loop {
            let (n, _) = rx_udp.recv_from(&mut buf)?;
            let mut msg = ConnMessage::default();
            msg.decode(&mut buf[..n])?;
            match msg.kind {
                Kind::Backend => {
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

        let tls = s2n_quic::provider::tls::rustls::Client::new(cb);

        let socket_io = IOBuilder::default()
            .with_tx_socket(tx_udp)?
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
        _ = tokio::io::copy(&mut stdin, &mut send_stream).await?;

        Ok(())
    }
}

use crate::message::Message;
use crate::{endpoint, message, tunnel};
use endpoint::Kind;
use message::{ConnMessage, StunMessage};
use s2n_quic::connection::Connection;
use s2n_quic::provider::io::tokio::Builder as IOBuilder;
use s2n_quic::{client::Connect, Client};
use std::error::Error;

use std::time::Duration;
use tokio::net::{TcpListener, UdpSocket};
use tokio::task::JoinHandle;

#[cfg(target_family = "windows")]
pub use crate::tls::rustls::insecure_client_tls;
#[cfg(target_family = "unix")]
pub use crate::tls::s2ntls::insecure_client_tls;

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

    pub async fn listen(self, mut quic_conn: Connection) -> Result<(), Box<dyn Error>> {
        let lis = TcpListener::bind(self.laddr).await?;
        loop {
            let (tcp_stream, _raddr) = lis.accept().await?;
            let quic_stream = quic_conn.open_bidirectional_stream().await?;
            tunnel::forward_tunnel(tcp_stream, quic_stream).await?;
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
        println!("send connect msg, wait stun connection info");

        let mut buf = [0; 1500];
        let (n, raddr) = socket.recv_from(&mut buf).await?;
        let mut msg = ConnMessage::default();
        _ = msg.decode(&buf[..n])?;
        println!("recv connect msg {} from {}", msg, raddr);

        let target_addr = msg.raddr.clone();
        let fqdn_clone = fqdn.clone();
        let task: JoinHandle<Result<UdpSocket, std::io::Error>> = tokio::spawn(async move {
            let laddr = socket.local_addr()?;
            let msg = ConnMessage::new(Kind::Frontend, laddr, fqdn_clone);
            let data = msg.encode().unwrap();
            let mut buf = [0; 1500];
            loop {
                _ = socket.send_to(&data, target_addr).await?;
                println!("send connect msg to {}", target_addr.to_string());
                tokio::time::sleep(Duration::from_secs(2)).await;
                match socket.try_recv_from(&mut buf) {
                    Ok((n, raddr)) => match message::decode(&buf[..n]) {
                        Message::Conn(msg) => match msg.kind {
                            Kind::Backend => {
                                return Ok(socket);
                            }
                            _ => {
                                println!("recv msg {} from {}", msg, raddr,);
                            }
                        },
                        Message::Stun(msg) => {
                            println!("recv unexpected stun msg {}", msg);
                        }
                        Message::Unknown(data) => {
                            println!("recv unknown msg {:?}", data);
                        }
                    },
                    Err(e) => {
                        if e.kind() as u8 == std::io::ErrorKind::WouldBlock as u8 {
                            continue;
                        }
                        println!("recv msg error {}", e.kind());
                        return Err(e);
                    }
                }
            }
        });

        let socket = task.await??;

        println!("start quic conn");

        let tx_udp = socket.into_std()?;
        let rx_udp = tx_udp.try_clone()?;

        let socket_io = IOBuilder::default()
            .with_tx_socket(tx_udp)?
            .with_rx_socket(rx_udp)?
            .build()?;
        let tls = insecure_client_tls("quic.crt")?;
        let client = Client::builder()
            .with_tls(tls)?
            .with_io(socket_io)?
            .start()?;

        let connect = Connect::new(target_addr).with_server_name(fqdn.clone().as_str());
        let mut connection = client.connect(connect).await?;
        connection.keep_alive(true)?;

        self.listen(connection).await?;
        Ok(())
    }
}

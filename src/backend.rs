use s2n_quic::provider::io::tokio::Builder as IOBuilder;
use s2n_quic::Server;
use std::error::Error;
use std::path::Path;
use std::time::Duration;
use tokio::net::UdpSocket;
use tokio::time;

use crate::endpoint::Kind;
use crate::message::{ConnMessage, MessageKind, StunMessage};

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
        _ = self.fetch(laddr.clone(), stun_addr, fqdn).await?;

        println!("recv conn from frontend, start listen quic ...");
        let mut server = Server::builder()
            .with_tls((Path::new("quic.crt"), Path::new("quic.key")))?
            .with_io(laddr.as_str())?
            .start()?;

        println!("quic server started, accept msg ...");
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

    pub async fn fetch(
        self,
        laddr: String,
        stun_addr: String,
        fqdn: String,
    ) -> Result<(), Box<dyn Error>> {
        let socket = UdpSocket::bind(laddr).await?;
        let stun_addr = stun_addr;
        let fqdn = fqdn;

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
                    let kind =  MessageKind::from(buf[0]);
                    let buf = buf[1..n].to_vec();
                    match kind{
                        MessageKind::Stun=>{
                            let mut msg = StunMessage::default();
                            _ = msg.decode(&buf[..])?;
                            println!(
                                "recv stun message from: {} raddr: {}, fqdn: {}",
                                msg.kind.to_string(),
                                raddr.to_string(),
                                msg.fqdn,
                            );
                        },
                        MessageKind::Conn=>{
                            let mut msg = ConnMessage::default();
                            _ = msg.decode(&buf[..])?;
                            println!(
                                "recv conn message raddr: {}, fqdn: {}",
                                raddr.to_string(),
                                msg.fqdn,
                            );
                            return Ok(());
                        },
                        _=>{
                            println!("reccv unknown msg");
                        }
                    }


                },
            };
        }
    }
}

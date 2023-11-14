use std::net::SocketAddr;
use std::rc;

use s2n_quic::client::Connect;
use s2n_quic::{Client, Server};
use std::path::Path;

use std::error::Error;

pub struct Backend {
    stun_addr: SocketAddr,
    laddr: SocketAddr,
}

impl Backend {
    pub fn new(stun_addr: SocketAddr, laddr: SocketAddr) -> Self {
        return Backend {
            stun_addr: stun_addr,
            laddr: laddr,
        };
    }
    pub async fn run(self) -> Result<(), Box<dyn Error>> {
        let mut server = Server::builder()
            .with_tls(Path::new("cert.pem"))?
            .with_io(self.laddr)?
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
    pub async fn keepalive(self) -> Result<(), Box<dyn Error>> {
        let client = Client::builder()
            .with_tls(Path::new("cert.pem"))?
            .with_io(self.laddr)?
            .start()?;
        let connect = Connect::new(self.stun_addr);
        let mut rconn = client.connect(connect).await?;
        rconn.keep_alive(true)?;

        let stream = rconn.open_bidirectional_stream().await?;
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
}

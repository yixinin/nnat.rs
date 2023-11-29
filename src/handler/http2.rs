use std::net::SocketAddr;

use bytes::Bytes;
use h2::server::SendResponse;
use h2::RecvStream;
use http::Request;
use tokio::net::{TcpListener, TcpStream};

use crate::error::Result;
use crate::upstream;
pub struct Http2Handler {
    listener: TcpListener,
    upstream: upstream::http2::Http2Upstream,
}

impl Http2Handler {
    pub async fn new(addr: String) -> Result<Self> {
        let addr: SocketAddr = addr.parse().unwrap();
        let listener = TcpListener::bind(&addr).await?;
        let h = Http2Handler { listener: listener };
        Ok(h)
    }
    async fn run(self) -> Result<()> {
        loop {
            if let Ok((socket, _peer_addr)) = self.listener.accept().await {
                tokio::spawn(async move {
                    if let Err(e) = self.serve(socket).await {
                        println!("  -> err={:?}", e);
                    }
                });
            }
        }
    }

    async fn serve(&self, socket: TcpStream) -> Result<()> {
        let mut connection = h2::server::handshake(socket).await?;
        println!("H2 connection bound");

        while let Some(result) = connection.accept().await {
            let (request, respond) = result?;
            tokio::spawn(async move {
                if let Err(e) = self.handle_request(request, respond).await {
                    println!("error while handling request: {}", e);
                }
            });
        }

        println!("~~~~~~~~~~~ H2 connection CLOSE !!!!!! ~~~~~~~~~~~");
        Ok(())
    }

    async fn handle_request(
        self,
        mut request: Request<RecvStream>,
        mut respond: SendResponse<Bytes>,
    ) -> Result<()> {
        self.upstream.forward(req, headers, body);
        println!("GOT request: {:?}", request);

        let body = request.body_mut();
        while let Some(data) = body.data().await {
            let data = data?;
            println!("<<<< recv {:?}", data);
            let _ = body.flow_control().release_capacity(data.len());
        }

        Ok(())
    }
}

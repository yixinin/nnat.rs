use std::net::SocketAddr;

use bytes::Bytes;
use h2::server::SendResponse;
use h2::RecvStream;
use http::Request;
use tokio::net::{TcpListener, TcpStream};

use crate::error::Result;
use crate::upstream::http::HttpForward;
use crate::upstream::http2::Http2Upstream;
pub struct Http2Handler {
    raddr: String,
    listener: TcpListener,
    // upstream: upstream::http2::Http2Upstream,
}

impl Http2Handler {
    pub async fn new(addr: String, raddr: String) -> Result<Self> {
        let addr: SocketAddr = addr.parse().unwrap();
        let listener = TcpListener::bind(&addr).await?;
        let h = Http2Handler {
            raddr: raddr,
            listener: listener,
        };
        Ok(h)
    }
}

async fn run(
    listener: TcpListener,
) -> std::result::Result<(), Box<dyn std::error::Error + Send + Sync>> {
    loop {
        if let Ok((socket, _peer_addr)) = listener.accept().await {
            tokio::spawn(async move {
                if let Err(e) = serve(socket).await {
                    println!("  -> err={:?}", e);
                }
            });
        }
    }
    Ok(())
}

async fn serve(
    socket: TcpStream,
) -> std::result::Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let mut connection = h2::server::handshake(socket).await?;
    println!("H2 connection bound");

    while let Some(result) = connection.accept().await {
        let (request, respond) = result?;
        tokio::spawn(async move {
            if let Err(e) = handle_request(request, respond).await {
                println!("error while handling request: {}", e);
            }
        });
    }

    println!("~~~~~~~~~~~ H2 connection CLOSE !!!!!! ~~~~~~~~~~~");
    Ok(())
}

async fn handle_request(
    mut request: Request<RecvStream>,
    mut respond: SendResponse<Bytes>,
) -> std::result::Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let b = request.into_body();
    let req = Request::builder()
        .uri(request.uri())
        .method(request.method())
        .body(())?;
    let data = b.poll_data(cx);

    req.headers_mut().clone_from(request.headers());
    let ups = Http2Upstream::new(self.raddr).await?;
    ups.forward(req, b, respond);
    println!("GOT request: {:?}", request);

    let body = request.body_mut();
    while let Some(data) = body.data().await {
        let data = data?;
        println!("<<<< recv {:?}", data);
        let _ = body.flow_control().release_capacity(data.len());
    }

    Ok(())
}

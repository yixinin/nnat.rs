use bytes::Bytes;
use http_body_util::{combinators::BoxBody, BodyExt, Empty, Full};
use hyper::body::{Body, Incoming};
use hyper::service::service_fn;
use std::net::SocketAddr;
use tokio::net::TcpListener;

use crate::tokioio::TokioIo;

use crate::{error::Result, upstream};
use hyper::{Method, Request, Response};

pub struct Http1Handler {
    listener: TcpListener,
    upstream: upstream::http1::Http1Upstream,
}

impl Http1Handler {
    pub async fn new(addr: String) -> Result<Self> {
        let addr: SocketAddr = addr.parse().unwrap();
        let listener = TcpListener::bind(&addr).await?;
        let h = Http1Handler { listener: listener };
        Ok(h)
    }

    pub async fn serve(&self) -> Result<()> {
        loop {
            let (stream, _) = self.listener.accept().await?;
            let io = TokioIo::new(stream);

            tokio::task::spawn(async move {
                if let Err(err) = hyper::server::conn::http1::Builder::new()
                    .preserve_header_case(true)
                    .title_case_headers(true)
                    .serve_connection(
                        io,
                        service_fn(|req: Request<hyper::body::Incoming>| async move {
                            // self.Ok(Response::new(Full::<Bytes>::from("Hello World")))

                            self.upstream.forward(
                                req.into(),
                                req.headers().to_owned(),
                                req.into_body(),
                            )
                        }),
                    )
                    .with_upgrades()
                    .await
                {
                    println!("Failed to serve connection: {:?}", err);
                }
            });
        }
    }
    async fn handle_request(
        &self,
        req: Request<hyper::body::Incoming>,
    ) -> std::result::Result<Response<BoxBody<Bytes, hyper::Error>>, hyper::Error> {
        let uri = req.uri().to_owned();
        let headers = req.headers().to_owned();
        let body = req.into_body();

        let req = Request::builder().uri(uri).body(())?;
        self.upstream.forward(req, headers, body);
        Ok(())
    }
}

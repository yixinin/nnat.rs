use bytes::Bytes;
use futures::Future;
use http_body_util::{combinators::BoxBody, BodyExt, Empty, Full};
use hyper::body::{Body, Incoming};
use hyper::service::service_fn;
use std::net::SocketAddr;
use tokio::net::TcpListener;

use crate::tokioio::TokioIo;

use crate::upstream::http::HttpForward;
use crate::{error::Result, upstream};
use hyper::{Method, Request, Response};

use crate::{error, TcpStreamIo};

pub struct Http1Handler {
    listener: TcpListener,
    upstream: upstream::http1::Http1Upstream<Incoming>,
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
            let io = TcpStreamIo(stream);

            tokio::task::spawn(async move {
                if let Err(err) = hyper::server::conn::http1::Builder::new()
                    .preserve_header_case(true)
                    .title_case_headers(true)
                    .serve_connection(
                        io,
                        service_fn(|req: Request<hyper::body::Incoming>| async move {
                            // self.Ok(Response::new(Full::<Bytes>::from("Hello World")))

                            self.upstream.forward(req.into(), req.into_body(), writer)
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

pub struct Http1Service {}

impl hyper::service::Service<Request<Incoming>> for Http1Service {
    type Response = Response<Bytes>;
    type Error = hyper::Error;
    type Future = Future<Output = Result<Self::Response, Box<dyn Self::Error>>>;

    fn call(&self, req: Request<Incoming>) -> Self::Future {}
}

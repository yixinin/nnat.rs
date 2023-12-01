use bytes::Bytes;
use futures::Future;
use http_body_util::{combinators::BoxBody, BodyExt, Empty, Full};
use hyper::body::{Body, Incoming};
use hyper::service::service_fn;
use std::net::SocketAddr;
use tokio::net::TcpListener;

use crate::tokioio::TokioIo;

use crate::upstream::http::HttpForward;
use crate::upstream::Forwarder;
use crate::{error::Result, upstream};
use hyper::{Method, Request, Response};

use crate::{error, TcpStreamIo};

use super::http1_rw::{ReqBody, ResBody};

pub struct Http1Handler<T>
where
    T: Forwarder,
{
    listener: TcpListener,
    upstream: T,
}

impl<T> Http1Handler<T>
where
    T: Forwarder,
{
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
                            let r = Request::builder()
                                .method(req.method())
                                .uri(req.uri())
                                .body(())?;
                            r.headers_mut().clone_from(req.headers());
                            r.extensions_mut().clone_from(req.extensions());
                            let body = ReqBody(req.into_body());
                            let res_body = ResBody::new();
                            self.upstream.forward(r, body, res_body);

                            Ok(Response::new(res_body))
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

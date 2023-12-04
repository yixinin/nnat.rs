use bytes::Bytes;
use futures::Future;
use http_body_util::{combinators::BoxBody, BodyExt, Empty, Full};
use hyper::body::{Body, Incoming};
use hyper::rt::Read;
use hyper::service::service_fn;
use std::io::Write;
use std::net::SocketAddr;
use tokio::net::TcpListener;

use crate::upstream::http::HttpForward;
use crate::upstream::http1::Http1Upstream;
use crate::upstream::HttpForwarder;
use crate::{error::Result, upstream};
use hyper::{Method, Request, Response};

use crate::{error, TcpStreamIo};

use super::http1_rw::{ReqBody, ResBody};

pub struct Http1Handler {
    raddr: String,
    listener: TcpListener,
}

impl Http1Handler {
    pub async fn new(addr: String, raddr: String) -> Result<Self> {
        let addr: SocketAddr = addr.parse().unwrap();
        let listener = TcpListener::bind(&addr).await?;
        let h = Http1Handler {
            raddr: raddr,
            listener: listener,
        };
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
                            let mut f = req.into_body();
                            let f = f.poll_frame();

                            let res_body = ResBody::new();
                            let ups: Http1Upstream<Incoming> =
                                Http1Upstream::new(self.raddr).await?;
                            ups.forward(r, body, res_body);

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
}

use s2n_quic::Connection;

use std::io;

use super::io::BiStream;
use super::spawner::Spawner;
use super::tunnel;

pub struct QuicProxy<T, S>
where
    T: tokio::io::AsyncRead + tokio::io::AsyncWrite + std::marker::Unpin + Send + 'static,
    S: Spawner<T> + Copy + Send,
{
    conn: Connection,
    out: S,
    _t: Option<T>,
}

impl<T, S> QuicProxy<T, S>
where
    T: tokio::io::AsyncRead + tokio::io::AsyncWrite + std::marker::Unpin + Send + 'static,
    S: Spawner<T> + Copy + Send,
{
    pub fn new(out: S, conn: Connection) -> std::io::Result<QuicProxy<T, S>> {
        let p = QuicProxy {
            conn: conn,
            out: out,
            _t: None,
        };
        Ok(p)
    }
    pub async fn run(self) -> io::Result<()> {
        let mut conn = self.conn;
        loop {
            if let Some(stream_in) = conn.accept().await? {
                println!("accept new conn: {}", stream_in.id());
                let stream_out = self.out.spawn().await?;
                let stream_in = BiStream::new(stream_in);
                let mut conn = tunnel::Tunnel::new(stream_in, stream_out);
                tokio::task::spawn(async move {
                    match conn.copy().await {
                        Ok((a, b)) => {
                            println!("copy {}:{}", a, b)
                        }
                        Err(err) => {
                            println!("{}", err)
                        }
                    }
                });
            }
        }
    }
}

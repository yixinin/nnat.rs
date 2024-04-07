use tokio::net::TcpSocket;

use std::io;
use std::net::SocketAddr;

use super::io::BiStream;
use super::spawner::Spawner;
use super::tunnel;

pub struct TcpProxy<T, S>
where
    T: tokio::io::AsyncRead + tokio::io::AsyncWrite + std::marker::Unpin + Send + 'static,
    S: Spawner<T> + Copy + Send,
{
    laddr: SocketAddr,
    out: S,
    _t: Option<T>,
}

impl<T, S> TcpProxy<T, S>
where
    T: tokio::io::AsyncRead + tokio::io::AsyncWrite + std::marker::Unpin + Send + 'static,
    S: Spawner<T> + Copy + Send,
{
    pub fn new(out: S, laddr: SocketAddr) -> std::io::Result<TcpProxy<T, S>> {
        let p = TcpProxy {
            laddr: laddr,
            out: out,
            _t: None,
        };
        Ok(p)
    }
    pub async fn run(self) -> io::Result<()> {
        let socket = TcpSocket::new_v4()?;
        socket.bind(self.laddr)?;
        let lis = socket.listen(1024)?;

        loop {
            let (stream_in, raddr) = lis.accept().await?;
            println!("accept new conn: {}", raddr);
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

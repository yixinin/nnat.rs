use std::task::Poll;

use tokio::{io::AsyncWriteExt, net::TcpStream};

#[derive(Debug)]
pub struct TcpStreamIo(pub TcpStream);

impl hyper::rt::Read for TcpStreamIo {
    fn poll_read(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        mut buf: hyper::rt::ReadBufCursor<'_>,
    ) -> std::task::Poll<Result<(), std::io::Error>> {
        let ready = self.0.poll_read_ready(cx);
        match ready {
            Poll::Ready(Ok(())) => {
                unsafe {
                    let mut tbuf = tokio::io::ReadBuf::uninit(buf.as_mut());
                    let n = self.0.try_read_buf(&mut tbuf)?;
                    buf.advance(n)
                }
                return Poll::Ready(Ok(()));
            }
            other => return Poll::Pending,
        }
    }
}
impl hyper::rt::Write for TcpStreamIo {
    fn poll_write(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &[u8],
    ) -> std::task::Poll<Result<usize, std::io::Error>> {
        let ready = self.0.poll_write_ready(cx);
        match ready {
            Poll::Ready(Ok(())) => {
                let n = self.0.try_write(buf)?;
                return Poll::Ready(Ok(n));
            }
            other => return Poll::Pending,
        };
    }

    fn poll_flush(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), std::io::Error>> {
        if let Err(err) = futures::executor::block_on(self.get_mut().0.flush()) {
            return Poll::Ready(Err(err));
        }
        return Poll::Ready(Ok(()));
    }

    fn poll_shutdown(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), std::io::Error>> {
        if let Err(err) = futures::executor::block_on(self.get_mut().0.shutdown()) {
            return Poll::Ready(Err(err));
        }
        return Poll::Ready(Ok(()));
    }
}

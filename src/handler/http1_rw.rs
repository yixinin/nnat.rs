use bytes::Bytes;
use hyper::body::{Body, Incoming};

pub struct ReqBody(pub Incoming);

impl std::io::Read for ReqBody {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        todo!()
    }
}

pub struct ResBody {
    buf: tokio::io::BufStream<Bytes>,
}

impl ResBody {
    pub fn new() -> Self {
        ResBody {
            buf: tokio::io::BufStream::new(Bytes::new()),
        }
    }
}

impl Body for ResBody {
    type Data = Bytes;

    type Error = std::io::Error;

    fn poll_frame(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Result<hyper::body::Frame<Self::Data>, Self::Error>>> {
    }
}

impl std::io::Write for ResBody {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        todo!()
    }

    fn flush(&mut self) -> std::io::Result<()> {
        todo!()
    }
}

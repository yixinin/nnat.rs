use super::http::HttpForward;
use crate::error::Result;

use bytes::Bytes;
use h3::client::{Connection, SendRequest};
use http::Request;

pub struct Http3Upstream {
    sender: SendRequest<s2n_quic_h3::OpenStreams, Bytes>,
    conn: Connection<s2n_quic_h3::Connection, Bytes>,
}

impl Http3Upstream {
    pub fn new(
        sender: SendRequest<s2n_quic_h3::OpenStreams, Bytes>,
        conn: Connection<s2n_quic_h3::Connection, Bytes>,
    ) -> Self {
        Http3Upstream {
            sender: sender,
            conn: conn,
        }
    }
}

impl<R, W> HttpForward<R, W> for Http3Upstream
where
    R: std::io::Read,
    W: std::io::Write,
{
    fn forward(&self, req: Request<()>, mut body: R, mut writer: W) -> Result<()> {
        let (mut stream) = self.sender.send_request(req).await?;
        let (mut tx, mut rx) = stream.split();

        std::io::copy(&mut body, &mut tx);
        std::io::copy(&mut rx, &mut writer);
        Ok(())
    }
}

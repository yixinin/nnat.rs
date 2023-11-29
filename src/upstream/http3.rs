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

    pub fn forward(&self, req: Request<()>) -> Result<()> {
        self.sender.send_request(req);
        Ok(())
    }
}

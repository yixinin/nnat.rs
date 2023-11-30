use std::net::SocketAddr;

pub struct QuicConnection {
    conn: s2n_quic::Connection,
}

impl QuicConnection {
    pub fn connect(raddr: SocketAddr) -> Self {}
}

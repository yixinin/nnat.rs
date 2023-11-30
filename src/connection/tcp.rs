use tokio::net::TcpStream;

pub struct TcpConnection {
    conn: TcpStream,
}

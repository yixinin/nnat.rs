use std::{error::Error, net::SocketAddr};

use s2n_quic::stream::BidirectionalStream;
use tokio::net::{TcpSocket, TcpStream};

pub async fn forward_tunnel(
    mut tcp_stream: TcpStream,
    mut quic_stream: BidirectionalStream,
) -> Result<(), Box<dyn Error>> {
    // let mut quic_stream = quic_conn.open_bidirectional_stream().await?;
    let (from_client, from_server) =
        tokio::io::copy_bidirectional(&mut tcp_stream, &mut quic_stream).await?;

    println!(
        "client wrote {} bytes and received {} bytes",
        from_client, from_server
    );
    Ok(())
}

pub async fn backward_tunnel(
    raddr: SocketAddr,
    mut quic_stream: BidirectionalStream,
) -> Result<(), Box<dyn Error>> {
    let mut tcp_stream: TcpStream;
    match raddr {
        SocketAddr::V4(raddr) => {
            let tcp_socket = TcpSocket::new_v4()?;
            tcp_stream = tcp_socket.connect(raddr.into()).await?;
        }
        SocketAddr::V6(raddr) => {
            let tcp_socket = TcpSocket::new_v6()?;
            tcp_stream = tcp_socket.connect(raddr.into()).await?;
        }
    }

    let (from_client, from_server) =
        tokio::io::copy_bidirectional(&mut quic_stream, &mut tcp_stream).await?;

    println!(
        "client wrote {} bytes and received {} bytes",
        from_client, from_server
    );
    Ok(())
}

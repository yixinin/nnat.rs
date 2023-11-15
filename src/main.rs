pub mod backend;
pub mod endpoint;
pub mod frontend;
pub mod message;
pub mod server;

pub use server::StunServer;

use std::{
    error::Error,
    net::{Ipv4Addr, SocketAddr, SocketAddrV4},
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // let laddr = SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::new(127, 0, 0, 1), 8080));
    // let mut s = StunServer::new(laddr);
    // s.run()?;

    let stun_addr = SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::new(127, 0, 0, 1), 8080));
    let laddr = SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::new(127, 0, 0, 1), 8081));
    let be = backend::Backend::new("rust.iakl.top", stun_addr, laddr);
    be.run().await?;

    Ok(())
}

pub mod backend;
pub mod endpoint;
pub mod frontend;
pub mod message;
pub mod server;

use clap::Parser;

pub use backend::Backend;
pub use frontend::Frontend;
pub use server::StunServer;

use std::{
    error::Error,
    net::{Ipv4Addr, SocketAddr, SocketAddrV4},
};

#[derive(Parser)]
pub struct Cli {
    pub debug: Option<bool>,
    pub b: Option<bool>,
    pub f: Option<bool>,
    pub stun: Option<bool>,
}

pub enum CliKind {
    Unknown = 0,
    Backend = 1,
    Frontend = 2,
    StunServer = 3,
}

impl Cli {
    pub fn kind(self) -> CliKind {
        if let Some(b) = self.b {
            if b {
                return CliKind::Backend;
            }
        }
        if let Some(f) = self.f {
            if f {
                return CliKind::Frontend;
            }
        }
        if let Some(stun) = self.stun {
            if stun {
                return CliKind::StunServer;
            }
        }
        return CliKind::Unknown;
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let args = Cli::parse();
    let stun_addr = SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::new(127, 0, 0, 1), 8080));
    match args.kind() {
        CliKind::Backend => {
            let laddr = SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::new(0, 0, 0, 0), 8081));
            let be = Backend::new("rust.iakl.top", stun_addr, laddr);
            be.run().await?;
        }
        CliKind::StunServer => {
            let laddr = SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::new(0, 0, 0, 0), 8080));
            let mut s = StunServer::new(laddr);
            s.run()?;
        }
        CliKind::Frontend => {
            let fb = Frontend::new("rust.iakl.top", stun_addr);
            fb.run().await?;
        }
        CliKind::Unknown => {
            println!("nothing run")
        }
    }
    Ok(())
}

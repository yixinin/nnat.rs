pub mod backend;
pub mod endpoint;
pub mod frontend;
pub mod layer;
pub mod message;
pub mod pool;
pub mod server;
pub mod tls;
pub mod tunnel;
use clap::Parser;

pub use backend::Backend;
pub use frontend::Frontend;
use layer::iobound::{http, tcpin, tcpout};
pub use server::StunServer;

use std::error::Error;

#[derive(Parser)]
pub struct Cli {
    #[arg(long)]
    pub debug: bool,
    #[arg(short)]
    pub b: bool,
    #[arg(short)]
    pub f: bool,
    #[arg(long)]
    pub stun: bool,
}

pub enum CliKind {
    Unknown = 0,
    Backend = 1,
    Frontend = 2,
    StunServer = 3,
}

impl Cli {
    pub fn kind(self) -> CliKind {
        if self.b {
            return CliKind::Backend;
        }

        if self.f {
            return CliKind::Frontend;
        }

        if self.stun {
            return CliKind::StunServer;
        }

        return CliKind::Unknown;
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // let args = Cli::parse();
    // let stun_addr = "114.115.218.1:3440";
    // let fqdn = "localhost";
    // match args.kind() {
    //     CliKind::StunServer => {
    //         let mut s = StunServer::new("0.0.0.0:3440");
    //         s.run()?;
    //     }
    //     CliKind::Backend => {
    //         let laddr = "0.0.0.0:3441".parse().unwrap();
    //         let be = Backend::new(fqdn, laddr, stun_addr);
    //         be.run().await?;
    //     }
    //     CliKind::Frontend => loop {
    //         let fb = Frontend::new(fqdn, "0.0.0.0:3442", stun_addr);
    //         match fb.run().await {
    //             Ok(_) => return Ok(()),
    //             Err(err) => {
    //                 println!("proxy error: {}", err)
    //             }
    //         }
    //     },
    //     CliKind::Unknown => {
    //         println!("nothing run")
    //     }
    // }
    let raddr = "127.0.0.1:8111".parse()?;
    let out = tcpout::TcpOutStream::new(Some(raddr));
    let laddr = "0.0.0.0:8110".parse()?;
    let ht = http::TcpProxy::new(out, laddr)?;
    // let tcp = tcpin::TcpProxy::new(out, laddr)?;
    ht.run().await?;
    Ok(())
}

pub mod backend;
pub mod endpoint;
pub mod frontend;
pub mod message;
pub mod server;
pub mod tls;

use clap::Parser;

pub use backend::Backend;
pub use frontend::Frontend;
pub use server::StunServer;

use std::error::Error;

#[derive(Parser)]
pub struct Cli {
    #[arg(long)]
    pub debug: bool,
    #[arg(long)]
    pub b: bool,
    #[arg(long)]
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
    let args = Cli::parse();
    let stun_addr = "114.115.218.1:3440";
    let fqdn = "localhost";
    match args.kind() {
        CliKind::StunServer => {
            let mut s = StunServer::new("0.0.0.0:3440");
            s.run()?;
        }
        CliKind::Backend => {
            let be = Backend::new(fqdn, "0.0.0.0:3441", stun_addr);
            be.run().await?;
        }
        CliKind::Frontend => {
            let fb = Frontend::new(fqdn, "0.0.0.0:3442", stun_addr);
            fb.run().await?;
        }
        CliKind::Unknown => {
            println!("nothing run")
        }
    }
    Ok(())
}

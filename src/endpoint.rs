use std::net::SocketAddr;

#[derive(PartialEq)]
pub enum Kind {
    Unknown = 0,
    Frontend = 1,
    Backend = 2,
}

impl Kind {
    pub fn from(i: u8) -> Kind {
        return match i {
            1 => Kind::Frontend,
            2 => Kind::Backend,
            _ => Kind::Unknown,
        };
    }
}

pub struct Endpoint {
    kind: Kind,
    addr: SocketAddr,
    fqdn: String,
}

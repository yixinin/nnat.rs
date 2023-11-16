use crate::endpoint::Kind;
use std::fmt;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr};
pub struct FmtError(Box<dyn 'static + fmt::Display + Send + Sync>);

impl std::error::Error for FmtError {}

impl FmtError {
    pub(crate) fn new<T: 'static + fmt::Display + Send + Sync>(msg: T) -> Self {
        Self(Box::new(msg))
    }
}

impl fmt::Debug for FmtError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_tuple("FmtError")
            .field(&format_args!("{}", self.0))
            .finish()
    }
}

impl fmt::Display for FmtError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.0.fmt(f)
    }
}
#[derive(PartialEq)]
pub enum MessageKind {
    Unknown = 0,
    Stun = 1,
    Conn = 2,
}

impl MessageKind {
    pub fn from(b: u8) -> Self {
        return match b {
            1 => MessageKind::Stun,
            2 => MessageKind::Conn,
            _ => MessageKind::Unknown,
        };
    }
}

#[derive(Clone)]
pub struct StunMessage {
    pub kind: Kind,
    pub fqdn: String,
}

impl StunMessage {
    pub fn default() -> Self {
        return StunMessage {
            kind: Kind::Unknown,
            fqdn: String::default(),
        };
    }
    pub fn new(kind: Kind, fqdn: String) -> Self {
        return StunMessage {
            kind: kind,
            fqdn: fqdn,
        };
    }

    pub fn encode(self) -> Result<Vec<u8>, FmtError> {
        if self.kind == Kind::Unknown {
            return Err(FmtError::new("kind error"));
        }
        let mut buf = Vec::with_capacity(self.fqdn.len() + 1);
        buf.push(self.kind as u8);
        buf.extend(self.fqdn.as_bytes());
        return Ok(buf);
    }
    pub fn decode(&mut self, buf: &[u8]) -> Result<(), FmtError> {
        if buf.len() <= 1 {
            return Err(FmtError::new("size error"));
        }
        let fqdn = String::from_utf8_lossy(&buf[1..]).to_string();
        self.kind = Kind::from(buf[0]);
        self.fqdn = fqdn;
        return Ok(());
    }
}

#[derive(Clone, Debug)]
pub struct ConnMessage {
    pub kind: Kind,
    pub fqdn: String,
    pub raddr: SocketAddr,
}

impl ConnMessage {
    pub fn default() -> Self {
        ConnMessage {
            kind: Kind::Unknown,
            fqdn: String::default(),
            raddr: SocketAddr::new(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)), 0),
        }
    }
    pub fn new(kind: Kind, raddr: SocketAddr, fqdn: String) -> Self {
        ConnMessage {
            kind: kind,
            fqdn: fqdn,
            raddr: raddr,
        }
    }
    pub fn encode(self) -> Result<Vec<u8>, FmtError> {
        let mut ip_size = 4;
        if self.raddr.is_ipv6() {
            ip_size = 16;
        } else if !self.raddr.is_ipv4() {
            return Err(FmtError::new("ip type error"));
        }

        let mut buf = Vec::with_capacity(1 + ip_size + 2 + self.fqdn.len());
        buf.push(self.kind as u8);
        buf.push(ip_size as u8);
        match self.raddr.ip() {
            std::net::IpAddr::V4(ip) => {
                buf.extend(&ip.octets()[..]);
            }
            std::net::IpAddr::V6(ip) => {
                buf.extend(&ip.octets()[..]);
            }
        }
        buf.extend(self.raddr.port().to_be_bytes());
        buf.extend(self.fqdn.as_bytes());
        return Ok(buf);
    }
    pub fn decode(&mut self, buf: &[u8]) -> Result<(), FmtError> {
        let kind = Kind::from(buf[0]);
        let ip_size = buf[1] as usize;

        let ip: IpAddr = match ip_size {
            4 => IpAddr::V4(Ipv4Addr::new(buf[2], buf[3], buf[4], buf[5])),
            16 => {
                let mut a = [0, 2];
                a[0] = buf[2];
                a[1] = buf[3];
                let mut b = [0, 2];
                b[0] = buf[4];
                b[1] = buf[5];
                let mut c = [0, 2];
                c[0] = buf[6];
                c[1] = buf[7];
                let mut d = [0, 2];
                d[0] = buf[8];
                d[1] = buf[9];
                let mut e = [0, 2];
                e[0] = buf[10];
                e[1] = buf[11];
                let mut f = [0, 2];
                f[0] = buf[12];
                f[1] = buf[13];
                let mut g = [0, 2];
                g[0] = buf[14];
                g[1] = buf[15];
                let mut h = [0, 2];
                h[0] = buf[16];
                h[1] = buf[17];
                IpAddr::V6(Ipv6Addr::new(
                    u16::from_be_bytes(a),
                    u16::from_be_bytes(b),
                    u16::from_be_bytes(c),
                    u16::from_be_bytes(d),
                    u16::from_be_bytes(e),
                    u16::from_be_bytes(f),
                    u16::from_be_bytes(g),
                    u16::from_be_bytes(h),
                ))
            }
            _ => return Err(FmtError::new("ip format error")),
        };
        let mut pb = [0; 2];
        pb[0] = buf[1+ip_size + 1];
        pb[1] = buf[1+ip_size + 2];
        let port = u16::from_be_bytes(pb);
        self.kind = kind;
        self.raddr = SocketAddr::new(ip, port);
        self.fqdn = String::from_utf8_lossy(&buf[(1+ip_size + 3)..]).to_string();
        return Ok(());
    }
}

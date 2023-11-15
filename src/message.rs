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
        let mut buf = Vec::with_capacity(1 + self.fqdn.len() + 1);
        buf.push(MessageKind::Stun as u8);
        buf.push(self.kind as u8);
        buf.extend(self.fqdn.as_bytes());
        return Ok(buf);
    }
    pub fn decode(&mut self, buf: &[u8]) -> Result<(), FmtError> {
        if buf.len() <= 2 {
            return Err(FmtError::new("size error"));
        }
        if MessageKind::from(buf[0]) != MessageKind::Stun {
            return Err(FmtError::new("not stun message"));
        }
        self.kind = Kind::from(buf[1]);
        let fqdn = String::from_utf8_lossy(&buf[2..]).to_string();
        self.fqdn = fqdn;
        return Ok(());
    }
}

impl std::fmt::Display for StunMessage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{{ {} {} }}", self.kind.to_string(), self.fqdn)
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
        buf.push(MessageKind::Conn as u8);
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
        if buf.len() <= 2 {
            return Err(FmtError::new("size error"));
        }
        if MessageKind::from(buf[0]) != MessageKind::Conn {
            return Err(FmtError::new("not conn message"));
        }
        let mut cur = 1;
        let kind = Kind::from(buf[cur]);
        cur += 1;
        let ip_size = buf[cur] as usize;

        let ip: IpAddr = match ip_size {
            4 => IpAddr::V4(Ipv4Addr::new(
                buf[cur + 1],
                buf[cur + 2],
                buf[cur + 3],
                buf[cur + 4],
            )),
            16 => {
                let mut a = [0, 2];
                a[0] = buf[cur + 1];
                a[1] = buf[cur + 2];
                let mut b = [0, 2];
                b[0] = buf[cur + 3];
                b[1] = buf[cur + 4];
                let mut c = [0, 2];
                c[0] = buf[cur + 5];
                c[1] = buf[cur + 6];
                let mut d = [0, 2];
                d[0] = buf[cur + 7];
                d[1] = buf[cur + 8];
                let mut e = [0, 2];
                e[0] = buf[cur + 9];
                e[1] = buf[cur + 10];
                let mut f = [0, 2];
                f[0] = buf[cur + 11];
                f[1] = buf[cur + 12];
                let mut g = [0, 2];
                g[0] = buf[cur + 13];
                g[1] = buf[cur + 14];
                let mut h = [0, 2];
                h[0] = buf[cur + 15];
                h[1] = buf[cur + 16];
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
        cur += ip_size;
        let mut pb = [0; 2];
        pb[0] = buf[cur + 1];
        pb[1] = buf[cur + 2];
        cur += 2;
        let port = u16::from_be_bytes(pb);
        self.kind = kind;
        self.raddr = SocketAddr::new(ip, port);
        self.fqdn = String::from_utf8_lossy(&buf[cur + 1..]).to_string();
        return Ok(());
    }
}

impl std::fmt::Display for ConnMessage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{{{} {} {} }}",
            self.kind.to_string(),
            self.fqdn,
            self.raddr.to_string()
        )
    }
}
pub enum Message {
    Stun(StunMessage),
    Conn(ConnMessage),
    Unknown(Vec<u8>),
}

pub fn decode(buf: &[u8]) -> Message {
    match MessageKind::from(buf[0]) {
        MessageKind::Conn => {
            let mut msg = ConnMessage::default();
            if let Err(err) = msg.decode(&buf) {
                print!("decode conn msg error: {}", err);
                return Message::Unknown(buf.to_vec());
            }
            return Message::Conn(msg);
        }
        MessageKind::Stun => {
            let mut msg = StunMessage::default();
            if let Err(err) = msg.decode(&buf) {
                println!("decode stun msg error: {}", err);
                return Message::Unknown(buf.to_vec());
            }

            return Message::Stun(msg);
        }
        _ => {
            return Message::Unknown(buf.to_vec());
        }
    }
}

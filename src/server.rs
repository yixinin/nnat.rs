use std::collections::HashMap;
use std::net::SocketAddr;

use std::ops::Sub;
use std::str::FromStr;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use std::net::UdpSocket;

use std::error::Error;

use crate::endpoint::Kind;
use crate::message::{ConnMessage, StunMessage};

pub struct StunServer {
    laddr: String,
    backends: HashMap<String, HashMap<String, Duration>>,
}

impl StunServer {
    pub fn new(laddr: &str) -> Self {
        return StunServer {
            laddr: laddr.to_string(),
            backends: HashMap::new(),
        };
    }

    fn add_backend(&mut self, fqdn: String, raddr: SocketAddr, now: Duration) -> bool {
        let key = fqdn.as_str();
        if !self.backends.contains_key(fqdn.clone().as_str()) {
            self.backends.insert(fqdn.clone(), HashMap::new());
        }
        if let Some(addrs) = self.backends.get_mut(key) {
            addrs.insert(raddr.to_string(), now);
            return true;
        }
        return false;
    }
    fn get_backend(&mut self, fqdn: String, now: Duration) -> String {
        let mut key = String::default();
        let mut rm_keys = Vec::new();
        if let Some(bs) = self.backends.get_mut(fqdn.clone().as_str()) {
            for (k, v) in bs {
                if now.sub(*v) > Duration::from_secs(120) {
                    rm_keys.push(k.clone());
                    continue;
                }

                key = k.into();
                break;
            }
        }
        if let Some(bs) = self.backends.get_mut(fqdn.clone().as_str()) {
            bs.remove(key.as_str());
            for k in rm_keys {
                bs.remove(k.as_str());
            }
        }

        return key;
    }

    pub fn run(&mut self) -> Result<(), Box<dyn Error>> {
        let laddr = self.laddr.clone();
        let socket = UdpSocket::bind(laddr)?;

        let mut buf = [0u8; 1500];
        loop {
            let (n, raddr) = socket.recv_from(&mut buf)?;
            if n > 0 {
                let mut msg = StunMessage::default();
                _ = msg.decode(&buf[..n])?;
                let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
                match msg.kind {
                    Kind::Unknown => {}
                    Kind::Stun => {}
                    Kind::Frontend => {
                        let fqdn = msg.fqdn.clone();
                        println!("recv from fontend: {}, fqdn: {}", raddr.to_string(), fqdn);
                        let baddr = self.get_backend(fqdn.clone(), now);
                        if baddr == "" {
                            println!("{} have no backend", fqdn);
                            continue;
                        }
                        let baddr = SocketAddr::from_str(baddr.as_str())?;
                        let msg = ConnMessage::new(Kind::Backend, baddr, fqdn);
                        let data = msg.clone().encode()?;
                        if let Err(err) = socket.send_to(&data, raddr.clone()) {
                            println!("send udp conn message err:{}", err)
                        }
                        let mut msg = msg.clone();
                        msg.raddr = raddr.clone();
                        let data = msg.encode()?;
                        if let Err(err) = socket.send_to(&data, baddr) {
                            println!("send udp conn message err:{}", err)
                        }
                    }
                    Kind::Backend => {
                        let fqdn = msg.fqdn.clone();
                        println!("recv from backend: {}, fqdn: {}", raddr.to_string(), fqdn);
                        if self.add_backend(fqdn, raddr, now) {
                            let msg = StunMessage::new(Kind::Stun, msg.fqdn.clone());
                            let data = msg.encode()?;
                            socket.send_to(&data, raddr)?;
                        }
                    }
                }
            }
        }
    }
}

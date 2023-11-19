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
                        let mut rm_keys = Vec::new();
                        let fqdn = msg.fqdn.clone();
                        if let Some(bs) = self.backends.get_mut(fqdn.clone().as_str()) {
                            for (k, v) in bs {
                                if now.sub(*v) > Duration::from_secs(120) {
                                    rm_keys.push(k.clone());
                                    continue;
                                }
                                let baddr = SocketAddr::from_str(k.as_str())?;
                                let msg = ConnMessage::new(Kind::Backend, baddr, fqdn.clone());
                                let data = msg.clone().encode()?;
                                if let Err(err) = socket.send_to(&data, raddr.clone()) {
                                    println!("send udp conn message err:{}", err)
                                }
                                let mut msg = msg.clone();
                                msg.raddr = raddr.clone();
                                let data = msg.encode()?;
                                if let Err(err) = socket.send_to(&data, k) {
                                    println!("send udp conn message err:{}", err)
                                }
                                break;
                            }
                        }
                        if let Some(bs) = self.backends.get_mut(fqdn.clone().as_str()) {
                            for k in rm_keys {
                                bs.remove(k.as_str());
                            }
                        }
                    }
                    Kind::Backend => {
                        let fqdn = msg.fqdn.clone();
                        if !self.backends.contains_key(fqdn.clone().as_str()) {
                            self.backends.insert(fqdn.clone(), HashMap::new());
                        }
                        if let Some(bs) = self.backends.get_mut(fqdn.clone().as_str()) {
                            let key = raddr.to_string();
                            bs.insert(key, now);
                            println!("recv from backend: {}, fqdn: {}", raddr.to_string(), fqdn);
                            // send back to endpoint
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

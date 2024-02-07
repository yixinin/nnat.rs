pub mod iobound;

pub trait InBound {
    fn recv();
}

pub trait OutBound {
    fn send();
}

pub trait Proxy: InBound + OutBound {}

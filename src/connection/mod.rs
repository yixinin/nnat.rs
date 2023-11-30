pub mod quic;
pub mod tcp;

use crate::upstream;

pub trait Connection<T, F>
where
    T: upstream::Forwarder,
{
    fn upstream(&self) -> T;
}

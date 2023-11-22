pub trait Forwarder {
    fn forward(self);
}

pub struct Upstream<F>
where
    F: Forwarder + Clone + Sync + Send + 'static,
{
    pub u: F,
}

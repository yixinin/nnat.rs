pub trait Acceptor {
    fn accept(self);
}

pub struct Handler<T>
where
    T: Acceptor + Clone + Sync + Send + 'static,
{
    pub h: T,
}

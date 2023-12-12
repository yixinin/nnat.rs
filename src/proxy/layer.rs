use crate::{
    handler::{Acceptor, Handler},
    upstream::{Forwarder, Upstream},
};

use std::sync::Arc;
#[derive(Clone)]
pub struct Layer<T, F>
where
    T: Acceptor + Clone + Sync + Send + 'static,
    F: Forwarder + Clone + Sync + Send + 'static,
{
    handler: Arc<Handler<T>>,
    upstream: Arc<Upstream<F>>,
}

use std::net::SocketAddr;

use std::sync::Arc;

use super::layer::Layer;

pub struct Proxy {
    layer: Layer<T, F>,
}

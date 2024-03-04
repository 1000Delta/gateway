use pingora::prelude::*;
use std::sync::Arc;

pub struct ServerConf<S> {
    pub upstream: Arc<LoadBalancer<S>>,
    pub host: &'static [u8],
}

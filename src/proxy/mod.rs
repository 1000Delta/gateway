pub mod server;

use std::str::from_utf8;

use async_trait::async_trait;
use pingora::{
    lb::selection::{BackendIter, BackendSelection},
    prelude::*,
};
use radix_trie::Trie;

pub struct Ctx {}

pub struct Proxy<S>
where
    S: BackendSelection + 'static,
    S::Iter: BackendIter,
{
    v_server: Trie<&'static [u8], server::ServerConf<S>>,
    global_server: Option<server::ServerConf<S>>,
}

impl<S> Proxy<S>
where
    S: BackendSelection,
    S::Iter: BackendIter,
{
    pub fn new() -> Self {
        Self {
            v_server: Trie::new(),
            global_server: None,
        }
    }

    pub fn add_server(&mut self, host: &'static [u8], conf: server::ServerConf<S>) {
        self.v_server.insert(host, conf);
    }

    pub fn set_global_server(&mut self, conf: server::ServerConf<S>) {
        self.global_server = Some(conf);
    }

    pub fn get_server(&self, host: &[u8]) -> Option<&server::ServerConf<S>> {
        self.v_server.get(host)
    }

    pub fn remove_server(&mut self, host: &[u8]) {
        self.v_server.remove(host);
    }

    fn global_upstream(&self, session: &mut Session, _ctx: &mut Ctx) -> Result<Box<HttpPeer>> {
        // let is_tls = session.
        let upstream = match self.global_server {
            Some(ref conf) => conf.upstream.select(b"", 256).unwrap(),
            None => return Err(Error::explain(InternalError, "No global server")),
        };
        let peer = Box::new(HttpPeer::new(upstream, true, "".to_string()));
        Ok(peer)
    }
}

#[async_trait]
impl<S> ProxyHttp for Proxy<S>
where
    S: BackendSelection + 'static + Send + Sync,
    S::Iter: BackendIter,
{
    type CTX = Ctx;

    fn new_ctx(&self) -> Self::CTX {
        Ctx {}
    }

    async fn upstream_peer(
        &self,
        session: &mut Session,
        ctx: &mut Self::CTX,
    ) -> Result<Box<HttpPeer>> {
        let host = session.get_header_bytes("Host");
        if host.len() == 0 {
            return self.global_upstream(session, ctx);
        }

        let conf = match self.get_server(&host) {
            Some(conf) => conf,
            None => return self.global_upstream(session, ctx),
        };

        let upstream = conf.upstream.select(b"", 256).unwrap();
        let peer = Box::new(HttpPeer::new(
            upstream,
            true,
            match from_utf8(host) {
                Ok(s) => s.to_string(),
                Err(_) => "".to_string(),
            },
        ));

        Ok(peer)
    }
}

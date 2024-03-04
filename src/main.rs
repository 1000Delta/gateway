mod proxy;

use std::sync::Arc;

mod lb;
use lb::*;

use pingora::prelude::*;

fn main() {
    let mut my_server = Server::new(Some(Opt::default())).unwrap();
    my_server.bootstrap();

    let upstreams = LoadBalancer::try_from_iter(["1.1.1.1:443", "1.0.0.1:443"]).unwrap();
    let mut lb = http_proxy_service(&my_server.configuration, LB(Arc::new(upstreams)));
    lb.add_tcp("0.0.0.0:6188");

    my_server.add_service(lb);

    my_server.run_forever();
}

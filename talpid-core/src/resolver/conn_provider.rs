use std::{future::Future, pin::Pin, time::Duration, net::SocketAddr};
use trust_dns_proto::{
    xfer::{DnsRequest, DnsResponse},
    DnsHandle,
    TokioTime,
};
use trust_dns_resolver::{
    config::{NameServerConfig, ResolverConfig, ResolverOpts},
    error::ResolveError,
    lookup::Lookup,
    name_server::{
        self, ConnectionProvider, GenericConnection, GenericConnectionProvider, TokioHandle,
    },
    AsyncResolver,
};

#[derive(Clone)]
struct ConnProvider {
    lmao: String,
}

#[derive(Clone)]
struct ConnHandle {
    probs_a_socket: String,
}

struct UdpClientStream {
    name_server: SocketAddr,
    timeout: Duration,
    is_shutdown: bool,
}

struct TcpClientStream {

}

impl ConnectionProvider for ConnProvider {
    type Conn = ConnHandle;
    type FutureConn = Box<dyn Future<Output = Result<Self::Conn, ResolveError>> + Send + 'static + Unpin>;

    type Time = TokioTime;

    fn new_connection(&self, config: &NameServerConfig, options: &ResolverOpts) -> Self::FutureConn {
        unimplemented!()
    }
}

impl DnsHandle for ConnHandle {
    type Response = Pin<Box<dyn Future<Output = Result<DnsResponse, ResolveError>> + Send + Unpin>>;
    type Error = ResolveError;

    fn send<R: Into<DnsRequest>>(&mut self, _: R) -> Self::Response {
        unimplemented!()
    }
}

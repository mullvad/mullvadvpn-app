use std::net::SocketAddr;

#[derive(Debug, Clone)]
pub enum RpcProxySettings {
    Socks5(Socks5Settings),
}

#[derive(Debug, Clone, Copy)]
pub struct Socks5Settings {
    pub address: SocketAddr,
}

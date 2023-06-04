#[derive(Debug, Clone)]
pub enum RpcProxySettings {
    LocalSocks5Settings(LocalSocks5Settings),
}

#[derive(Debug, Clone, Copy)]
pub struct LocalSocks5Settings {
    pub port: u16,
}

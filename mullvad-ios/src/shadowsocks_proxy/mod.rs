use super::mullvad_ios_runtime;
use shadowsocks_service::{
    config::{
        Config, ConfigType, LocalConfig, LocalInstanceConfig, ProtocolType, ServerInstanceConfig,
    },
    local::Server,
    shadowsocks::{config::ServerConfig, crypto::CipherKind},
};
use std::{
    io,
    net::{IpAddr, Ipv4Addr, SocketAddr, TcpListener},
    str::FromStr,
};
use tokio::task::JoinHandle;
mod ffi;

pub fn run_forwarding_proxy(
    forward_socket_addr: SocketAddr,
    bridge_socket_addr: SocketAddr,
    password: &str,
    cipher: &str,
) -> io::Result<(u16, ShadowsocksHandle)> {
    let runtime =
        ShadowsocksService::new(forward_socket_addr, bridge_socket_addr, password, cipher)?;
    let port = runtime.port();
    let handle = runtime.run()?;

    Ok((port, handle))
}

struct ShadowsocksService {
    config: Config,
    local_port: u16,
}

pub struct ShadowsocksHandle {
    abort_handle: JoinHandle<()>,
}

impl ShadowsocksHandle {
    pub fn stop(self) {
        self.abort_handle.abort();
    }
}

impl ShadowsocksService {
    pub fn new(
        forward_socket_addr: SocketAddr,
        bridge_socket_addr: SocketAddr,
        password: &str,
        cipher: &str,
    ) -> io::Result<Self> {
        let (config, local_port) =
            Self::create_config(forward_socket_addr, bridge_socket_addr, password, cipher)?;
        Ok(Self { config, local_port })
    }

    pub fn port(&self) -> u16 {
        self.local_port
    }

    pub fn run(self) -> io::Result<ShadowsocksHandle> {
        let runtime = mullvad_ios_runtime().map_err(io::Error::other)?;

        let abort_handle = runtime.spawn(async move {
            self.run_service_inner().await;
        });

        Ok(ShadowsocksHandle { abort_handle })
    }

    async fn run_service_inner(self) {
        let Self { config, .. } = self;

        let _ = Server::new(config)
            .await
            .map_err(io::Error::from)
            .expect("Could not create Shadowsocks server")
            .run()
            .await;
    }

    pub fn create_config(
        forward_socket_addr: SocketAddr,
        bridge_socket_addr: SocketAddr,
        password: &str,
        cipher: &str,
    ) -> io::Result<(Config, u16)> {
        let mut cfg = Config::new(ConfigType::Local);
        let free_port = get_free_port()?;
        let bind_addr = SocketAddr::new(Ipv4Addr::UNSPECIFIED.into(), free_port);

        let mut local_config = LocalConfig::new_with_addr(bind_addr.into(), ProtocolType::Tunnel);
        local_config.forward_addr = Some(forward_socket_addr.into());
        cfg.local = vec![LocalInstanceConfig::with_local_config(local_config)];

        let cipher = match CipherKind::from_str(cipher) {
            Ok(cipher) => cipher,
            Err(err) => {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    format!("Invalid cipher specified: {}", err),
                ));
            }
        };
        let server_config = ServerInstanceConfig::with_server_config(ServerConfig::new(
            bridge_socket_addr,
            password,
            cipher,
        ));

        cfg.server = vec![server_config];

        Ok((cfg, free_port))
    }
}

fn get_free_port() -> io::Result<u16> {
    let bind_addr: SocketAddr = SocketAddr::new(IpAddr::V4(Ipv4Addr::UNSPECIFIED), 0);
    let port = TcpListener::bind(bind_addr)?.local_addr()?.port();
    Ok(port)
}

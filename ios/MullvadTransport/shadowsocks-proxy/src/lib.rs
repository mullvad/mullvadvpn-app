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
use tokio::sync::oneshot;

mod ffi;
pub use ffi::{start_shadowsocks_proxy, stop_shadowsocks_proxy, ProxyHandle};

pub fn run_forwarding_proxy(
    forward_socket_addr: SocketAddr,
    bridge_socket_addr: SocketAddr,
    password: &str,
    cipher: &str,
) -> io::Result<(u16, ShadowsocksHandle)> {
    let runtime =
        ShadowsocksRuntime::new(forward_socket_addr, bridge_socket_addr, password, cipher)?;
    let port = runtime.port();
    let handle = runtime.run()?;

    Ok((port, handle))
}

struct ShadowsocksRuntime {
    runtime: tokio::runtime::Runtime,
    config: Config,
    local_port: u16,
}

pub struct ShadowsocksHandle {
    tx: oneshot::Sender<oneshot::Sender<()>>,
}

impl ShadowsocksHandle {
    pub fn stop(self) {
        let (shutdown_tx, shutdown_rx) = oneshot::channel();
        let _ = self.tx.send(shutdown_tx);
        let _ = shutdown_rx.blocking_recv();
    }
}

impl ShadowsocksRuntime {
    pub fn new(
        forward_socket_addr: SocketAddr,
        bridge_socket_addr: SocketAddr,
        password: &str,
        cipher: &str,
    ) -> io::Result<Self> {
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()?;

        let (config, local_port) =
            Self::create_config(forward_socket_addr, bridge_socket_addr, password, cipher)?;
        Ok(Self {
            runtime,
            config,
            local_port,
        })
    }

    pub fn port(&self) -> u16 {
        self.local_port
    }

    pub fn run(self) -> io::Result<ShadowsocksHandle> {
        let (tx, rx) = oneshot::channel();
        let (shutdown_tx, shutdown_rx) = oneshot::channel();
        let (startup_tx, startup_rx) = oneshot::channel();
        std::thread::spawn(move || {
            self.run_service_inner(rx, startup_tx);
        });

        match startup_rx.blocking_recv() {
            Ok(Ok(_)) => Ok(ShadowsocksHandle { tx }),
            Ok(Err(err)) => {
                let _ = tx.send(shutdown_tx);
                let _ = shutdown_rx.blocking_recv();
                Err(err)
            }
            Err(_) => {
                let _ = tx.send(shutdown_tx);
                let _ = shutdown_rx.blocking_recv();
                Err(io::Error::new(
                    io::ErrorKind::Other,
                    "Tokio runtime crashed",
                ))
            }
        }
    }

    fn run_service_inner(
        self,
        shutdown_rx: oneshot::Receiver<oneshot::Sender<()>>,
        startup_done_tx: oneshot::Sender<io::Result<()>>,
    ) {
        let Self {
            config, runtime, ..
        } = self;

        std::thread::spawn(move || {
            runtime.spawn(async move {
                match Server::new(config).await {
                    Ok(server) => {
                        let _ = startup_done_tx.send(Ok(()));
                        let _ = server.run().await;
                    }
                    Err(err) => {
                        let _ = startup_done_tx.send(Err(err));
                    }
                }
            });
            if let Ok(shutdown_tx) = runtime.block_on(shutdown_rx) {
                std::mem::drop(runtime);
                let _ = shutdown_tx.send(());
            }
        });
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

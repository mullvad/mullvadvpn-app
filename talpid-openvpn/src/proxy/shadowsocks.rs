pub use std::io;

use async_trait::async_trait;
use futures::future::{abortable, AbortHandle, Aborted};
use std::net::{Ipv4Addr, SocketAddr};
use tokio::task::JoinHandle;

use shadowsocks_service::{
    config::{
        Config, ConfigType, LocalConfig, LocalInstanceConfig, ProtocolType, ServerInstanceConfig,
    },
    local,
    shadowsocks::{
        config::{Mode, ServerConfig},
        ServerAddr,
    },
};

use super::{Error, ProxyMonitor, ProxyMonitorCloseHandle, ProxyResourceData};
use talpid_types::{net::openvpn::ShadowsocksProxySettings, ErrorExt};

pub struct ShadowsocksProxyMonitor {
    port: u16,
    server_join_handle: Option<JoinHandle<Result<io::Result<()>, Aborted>>>,
    server_abort_handle: AbortHandle,
}

impl ShadowsocksProxyMonitor {
    pub async fn start(
        settings: &ShadowsocksProxySettings,
        _resource_data: &ProxyResourceData,
    ) -> super::Result<Self> {
        Self::start_inner(settings).await.map_err(Error::Io)
    }

    async fn start_inner(settings: &ShadowsocksProxySettings) -> io::Result<Self> {
        let mut config = Config::new(ConfigType::Local);

        config.fast_open = true;

        let mut local = LocalConfig::new(ProtocolType::Socks);
        local.mode = Mode::TcpOnly;
        local.addr = Some(ServerAddr::SocketAddr(SocketAddr::from((
            Ipv4Addr::LOCALHOST,
            0,
        ))));

        config
            .local
            .push(LocalInstanceConfig::with_local_config(local));

        let server = ServerConfig::new(
            settings.peer,
            settings.password.clone(),
            settings.cipher.parse().map_err(|_| {
                io::Error::new(
                    io::ErrorKind::Other,
                    format!("Invalid cipher: {}", settings.cipher),
                )
            })?,
        );

        config
            .server
            .push(ServerInstanceConfig::with_server_config(server));

        #[cfg(target_os = "linux")]
        {
            config.outbound_fwmark = settings.fwmark;
        }

        let srv = local::Server::new(config).await?;
        let listener_addr = Self::get_listener_addr(&srv)?;

        let (fut, server_abort_handle) = abortable(async move {
            let result = srv.run().await;
            if let Err(error) = &result {
                log::error!(
                    "{}",
                    error.display_chain_with_msg("sslocal stopped with an error")
                );
            }
            result
        });
        let server_join_handle = tokio::spawn(fut);

        Ok(Self {
            port: listener_addr.port(),
            server_join_handle: Some(server_join_handle),
            server_abort_handle,
        })
    }

    fn get_listener_addr(srv: &local::Server) -> io::Result<SocketAddr> {
        let no_addr_err = || io::Error::new(io::ErrorKind::Other, "Missing listener address");
        let socks_server = srv.socks_servers().first().ok_or_else(no_addr_err)?;
        socks_server
            .tcp_server()
            .ok_or_else(no_addr_err)?
            .local_addr()
    }
}

impl Drop for ShadowsocksProxyMonitor {
    fn drop(&mut self) {
        self.server_abort_handle.abort();
    }
}

#[async_trait]
impl ProxyMonitor for ShadowsocksProxyMonitor {
    fn close_handle(&mut self) -> Box<dyn ProxyMonitorCloseHandle> {
        Box::new(ShadowsocksProxyMonitorCloseHandle {
            server_abort_handle: self.server_abort_handle.clone(),
        })
    }

    async fn wait(mut self: Box<Self>) -> super::Result<()> {
        if let Some(join_handle) = self.server_join_handle.take() {
            match join_handle.await {
                Ok(Err(Aborted)) => Ok(()),

                Err(join_err) if join_err.is_cancelled() => Ok(()),
                Err(_) => Err(Error::UnexpectedExit(
                    "Shadowsocks task panicked".to_string(),
                )),

                Ok(Ok(result)) => match result {
                    Ok(()) => Err(Error::UnexpectedExit("Exited without error".to_string())),
                    Err(error) => Err(Error::UnexpectedExit(format!(
                        "Error: {}",
                        error.display_chain()
                    ))),
                },
            }
        } else {
            Ok(())
        }
    }

    fn port(&self) -> u16 {
        self.port
    }
}

struct ShadowsocksProxyMonitorCloseHandle {
    server_abort_handle: AbortHandle,
}

impl ProxyMonitorCloseHandle for ShadowsocksProxyMonitorCloseHandle {
    fn close(self: Box<Self>) -> super::Result<()> {
        self.server_abort_handle.abort();
        Ok(())
    }
}

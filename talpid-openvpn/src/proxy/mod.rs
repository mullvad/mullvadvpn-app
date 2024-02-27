mod noop;
mod shadowsocks;

use self::shadowsocks::ShadowsocksProxyMonitor;
use async_trait::async_trait;
use std::{fmt, io};
use talpid_types::net::proxy::CustomProxy;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Monitor exited unexpectedly: {0}")]
    UnexpectedExit(String),

    #[error("I/O error")]
    Io(io::Error),
}

pub type Result<T> = std::result::Result<T, Error>;

#[async_trait]
pub trait ProxyMonitor: Send {
    /// Create a handle than can be used to ask the proxy service to shut down.
    fn close_handle(&mut self) -> Box<dyn ProxyMonitorCloseHandle>;

    /// Consume monitor and wait for proxy service to shut down.
    async fn wait(self: Box<Self>) -> Result<()>;

    /// The port bound to.
    fn port(&self) -> u16;
}

impl fmt::Debug for dyn ProxyMonitor {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "ProxyMonitor {{ port: {} }}", self.port())
    }
}

pub trait ProxyMonitorCloseHandle: Send {
    fn close(self: Box<Self>) -> Result<()>;
}

pub async fn start_proxy(
    settings: &CustomProxy,
    #[cfg(target_os = "linux")] fwmark: u32,
) -> Result<Box<dyn ProxyMonitor>> {
    match settings {
        CustomProxy::Socks5Local(local_settings) => {
            // These are generic proxy settings with the proxy client not managed by us.
            Ok(Box::new(noop::NoopProxyMonitor::start(
                local_settings.local_port,
            )?))
        }
        CustomProxy::Socks5Remote(remote_settings) => {
            // These are generic proxy settings with the proxy client not managed by us.
            Ok(Box::new(noop::NoopProxyMonitor::start(
                remote_settings.endpoint.port(),
            )?))
        }
        CustomProxy::Shadowsocks(ss_settings) => Ok(Box::new(
            ShadowsocksProxyMonitor::start(
                ss_settings,
                #[cfg(target_os = "linux")]
                fwmark,
            )
            .await?,
        )),
    }
}

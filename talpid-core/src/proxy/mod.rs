mod noop;
mod shadowsocks;

pub use std::io::Result;

use self::shadowsocks::ShadowsocksProxyMonitor;
use std::{fmt, path::PathBuf};
use talpid_types::net::openvpn;

pub enum WaitResult {
    UnexpectedExit(String),
    ProperShutdown,
}

pub trait ProxyMonitor: Send {
    /// Create a handle than can be used to ask the proxy service to shut down.
    fn close_handle(&mut self) -> Box<dyn ProxyMonitorCloseHandle>;

    /// Consume monitor and wait for proxy service to shut down.
    fn wait(self: Box<Self>) -> Result<WaitResult>;

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

/// Variables that define the environment to help
/// proxy implementations find their way around.
/// TODO: Move struct to wider scope and use more generic name.
pub struct ProxyResourceData {
    pub resource_dir: PathBuf,
    pub log_dir: Option<PathBuf>,
}

pub fn start_proxy(
    settings: &openvpn::ProxySettings,
    resource_data: &ProxyResourceData,
) -> Result<Box<dyn ProxyMonitor>> {
    match settings {
        openvpn::ProxySettings::Local(local_settings) => {
            // These are generic proxy settings with the proxy client not managed by us.
            Ok(Box::new(noop::NoopProxyMonitor::start(
                local_settings.port,
            )?))
        }
        openvpn::ProxySettings::Remote(remote_settings) => {
            // These are generic proxy settings with the proxy client not managed by us.
            Ok(Box::new(noop::NoopProxyMonitor::start(
                remote_settings.address.port(),
            )?))
        }
        openvpn::ProxySettings::Shadowsocks(ss_settings) => Ok(Box::new(
            ShadowsocksProxyMonitor::start(ss_settings, resource_data)?,
        )),
    }
}

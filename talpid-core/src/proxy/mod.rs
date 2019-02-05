mod shadowsocks;

pub use std::io::Result;

use self::shadowsocks::ShadowsocksProxyMonitor;
use std::{fmt, path::PathBuf, sync::mpsc};
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
}

pub trait ProxyMonitorCloseHandle: Send {
    fn close(self: Box<Self>) -> Result<()>;
}

struct NoopProxyMonitor {
    tx: mpsc::Sender<()>,
    rx: mpsc::Receiver<()>,
}

impl NoopProxyMonitor {
    fn start() -> Result<Self> {
        let (tx, rx) = mpsc::channel();
        Ok(NoopProxyMonitor { tx, rx })
    }
}

impl ProxyMonitor for NoopProxyMonitor {
    fn close_handle(&mut self) -> Box<dyn ProxyMonitorCloseHandle> {
        Box::new(NoopProxyMonitorCloseHandle {
            tx: self.tx.clone(),
        })
    }

    fn wait(self: Box<Self>) -> Result<WaitResult> {
        let _ = self.rx.recv();
        Ok(WaitResult::ProperShutdown)
    }
}

struct NoopProxyMonitorCloseHandle {
    tx: mpsc::Sender<()>,
}

impl ProxyMonitorCloseHandle for NoopProxyMonitorCloseHandle {
    fn close(self: Box<Self>) -> Result<()> {
        let _ = self.tx.send(());
        Ok(())
    }
}

/// Variables that define the environment to help
/// proxy implementations find their way around.
pub struct ProxyResourceData {
    pub binaries_dir: PathBuf,
    pub logs_dir: PathBuf,
}

pub struct ProxyRuntimeData {
    pub monitor: Box<dyn ProxyMonitor>,
    // Port bound to at runtime.
    pub port: u16,
}

impl fmt::Debug for ProxyRuntimeData {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "ProxyRuntimeData {{ monitor: present, port: {} }}",
            self.port
        )
    }
}

impl ProxyRuntimeData {
    pub fn new(monitor: Box<dyn ProxyMonitor>, port: u16) -> Self {
        ProxyRuntimeData { monitor, port }
    }
}

pub fn start_proxy(
    settings: &openvpn::ProxySettings,
    resource_data: &ProxyResourceData,
) -> Result<ProxyRuntimeData> {
    match settings {
        openvpn::ProxySettings::Local(local_settings) => {
            // These are generic proxy settings with the proxy client not managed by us.
            Ok(ProxyRuntimeData::new(
                Box::new(NoopProxyMonitor::start()?),
                local_settings.port,
            ))
        }
        openvpn::ProxySettings::Remote(remote_settings) => {
            // These are generic proxy settings with the proxy client not managed by us.
            Ok(ProxyRuntimeData::new(
                Box::new(NoopProxyMonitor::start()?),
                remote_settings.address.port(),
            ))
        }
        openvpn::ProxySettings::Shadowsocks(ss_settings) => {
            match ShadowsocksProxyMonitor::start(ss_settings, resource_data) {
                Ok((monitor, port)) => Ok(ProxyRuntimeData::new(Box::new(monitor), port)),
                Err(err) => Err(err),
            }
        }
    }
}

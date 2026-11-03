mod interface;
mod network_manager;
mod resolvconf;
mod static_resolv_conf;
mod systemd_resolved;

use std::env;
use std::fmt;
use std::net::IpAddr;
use talpid_error::ErrorExt;

use self::network_manager::NetworkManager;
use self::resolvconf::Resolvconf;
use self::static_resolv_conf::StaticResolvConf;
use self::systemd_resolved::SystemdResolved;
use crate::ResolvedDnsConfig;

pub type Result<T> = std::result::Result<T, Error>;

/// Errors that can happen in the Linux DNS monitor
#[derive(thiserror::Error, Debug)]
pub enum Error {
    /// Error in systemd-resolved DNS monitor
    #[error("Error in systemd-resolved DNS monitor")]
    SystemdResolved(#[from] systemd_resolved::Error),

    /// Error in NetworkManager DNS monitor
    #[error("Error in NetworkManager DNS monitor")]
    NetworkManager(#[from] network_manager::Error),

    /// Error in resolvconf DNS monitor
    #[error("Error in resolvconf DNS monitor")]
    Resolvconf(#[from] resolvconf::Error),

    /// Error in static /etc/resolv.conf DNS monitor
    #[error("Error in static /etc/resolv.conf DNS monitor")]
    StaticResolvConf(#[from] static_resolv_conf::Error),

    /// No suitable DNS monitor implementation detected
    #[error("No suitable DNS monitor implementation detected")]
    NoDnsMonitor,
}

pub struct DnsMonitor {
    handle: tokio::runtime::Handle,
    inner: Option<DnsMonitorHolder>,
}

impl super::DnsMonitorT for DnsMonitor {
    type Error = Error;

    fn new(handle: tokio::runtime::Handle) -> Result<Self> {
        Ok(DnsMonitor {
            handle,
            inner: None,
        })
    }

    fn set(&mut self, interface: &str, config: ResolvedDnsConfig) -> Result<()> {
        let servers = config.tunnel_config();
        self.reset()?;
        // Creating a new DNS monitor for each set, in case the system changed how it manages DNS.
        let mut inner = self.handle.block_on(DnsMonitorHolder::new())?;
        if !servers.is_empty() {
            self.handle.block_on(inner.set(interface, servers))?;
            self.inner = Some(inner);
        }
        Ok(())
    }

    fn reset(&mut self) -> Result<()> {
        if let Some(mut inner) = self.inner.take() {
            self.handle.block_on(inner.reset())?;
        }
        Ok(())
    }
}

pub enum DnsMonitorHolder {
    SystemdResolved(SystemdResolved),
    NetworkManager(NetworkManager),
    Resolvconf(Resolvconf),
    StaticResolvConf(StaticResolvConf),
}

impl fmt::Display for DnsMonitorHolder {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use self::DnsMonitorHolder::*;
        let name = match self {
            Resolvconf(..) => "resolvconf",
            StaticResolvConf(..) => "/etc/resolv.conf",
            SystemdResolved(..) => "systemd-resolved",
            NetworkManager(..) => "NetworkManager",
        };
        f.write_str(name)
    }
}

impl DnsMonitorHolder {
    async fn new() -> Result<Self> {
        let dns_module = env::var_os("TALPID_DNS_MODULE");

        let manager = match dns_module.as_ref().and_then(|value| value.to_str()) {
            Some("static-file") => DnsMonitorHolder::StaticResolvConf(StaticResolvConf::new()?),
            Some("resolvconf") => DnsMonitorHolder::Resolvconf(Resolvconf::new()?),
            Some("systemd") => DnsMonitorHolder::SystemdResolved(SystemdResolved::new().await?),
            Some("network-manager") => {
                DnsMonitorHolder::NetworkManager(NetworkManager::new().await?)
            }
            Some(_) | None => Self::with_detected_dns_manager().await?,
        };
        log::debug!("Managing DNS via {}", manager);
        Ok(manager)
    }

    async fn with_detected_dns_manager() -> Result<Self> {
        fn log_err<E: std::error::Error>(method: &'static str) -> impl Fn(&E) {
            move |err: &E| {
                log::debug!(
                    "{}",
                    err.display_chain_with_msg(&format!("Can't manage DNS using {method}"))
                );
            }
        }
        match SystemdResolved::new().await {
            Ok(systemd) => return Ok(DnsMonitorHolder::SystemdResolved(systemd)),
            Err(err) => log_err("systemd-resolved")(&err),
        };

        match NetworkManager::new().await {
            Ok(systemd) => return Ok(DnsMonitorHolder::NetworkManager(systemd)),
            Err(err) => log_err("NetworkManager")(&err),
        };

        match Resolvconf::new() {
            Ok(systemd) => return Ok(DnsMonitorHolder::Resolvconf(systemd)),
            Err(err) => log_err("resolveconf")(&err),
        };

        match StaticResolvConf::new() {
            Ok(systemd) => return Ok(DnsMonitorHolder::StaticResolvConf(systemd)),
            Err(err) => log_err("/etc/resolv.conf")(&err),
        };

        Err(Error::NoDnsMonitor)
    }

    async fn set(&mut self, interface: &str, servers: &[IpAddr]) -> Result<()> {
        use self::DnsMonitorHolder::*;
        match self {
            Resolvconf(resolvconf) => resolvconf.set_dns(interface, servers)?,
            StaticResolvConf(static_resolv_conf) => static_resolv_conf.set_dns(servers.to_vec())?,
            SystemdResolved(systemd_resolved) => {
                systemd_resolved.set_dns(interface, servers).await?
            }
            NetworkManager(network_manager) => network_manager.set_dns(interface, servers).await?,
        }
        Ok(())
    }

    async fn reset(&mut self) -> Result<()> {
        use self::DnsMonitorHolder::*;
        match self {
            Resolvconf(resolvconf) => resolvconf.reset()?,
            StaticResolvConf(static_resolv_conf) => static_resolv_conf.reset()?,
            SystemdResolved(systemd_resolved) => systemd_resolved.reset().await?,
            NetworkManager(network_manager) => network_manager.reset().await?,
        }
        Ok(())
    }
}

/// Returns true if DnsMonitor will use NetworkManager to manage DNS.
pub async fn will_use_nm() -> bool {
    SystemdResolved::new().await.is_err() && NetworkManager::new().await.is_ok()
}

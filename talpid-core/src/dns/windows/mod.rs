use std::{env, fmt, net::IpAddr};

use super::DnsMonitorT;

mod dnsapi;
mod iphlpapi;
mod netsh;
mod tcpip;

/// Errors that can happen when configuring DNS on Windows.
#[derive(err_derive::Error, Debug)]
pub enum Error {
    /// Failed to set DNS config using the iphlpapi module.
    #[error(display = "Error in iphlpapi module")]
    Iphlpapi(#[error(source)] iphlpapi::Error),

    /// Failed to set DNS config using the netsh module.
    #[error(display = "Error in netsh module")]
    Netsh(#[error(source)] netsh::Error),

    /// Failed to set DNS config using the tcpip module.
    #[error(display = "Error in tcpip module")]
    Tcpip(#[error(source)] tcpip::Error),
}

pub struct DnsMonitor {
    inner: DnsMonitorHolder,
}

impl DnsMonitorT for DnsMonitor {
    type Error = Error;

    fn new() -> Result<Self, Error> {
        let dns_module = env::var_os("TALPID_DNS_MODULE");

        let inner = match dns_module.as_ref().and_then(|value| value.to_str()) {
            Some("iphlpapi") => DnsMonitorHolder::Iphlpapi(iphlpapi::DnsMonitor::new()?),
            Some("tcpip") => DnsMonitorHolder::Tcpip(tcpip::DnsMonitor::new()?),
            Some("netsh") => DnsMonitorHolder::Netsh(netsh::DnsMonitor::new()?),
            Some(_) | None => Self::detect_appropriate_method()?,
        };

        log::debug!("DNS monitor: {}", inner);

        Ok(DnsMonitor { inner })
    }

    fn set(&mut self, interface: &str, servers: &[IpAddr]) -> Result<(), Error> {
        match self.inner {
            DnsMonitorHolder::Iphlpapi(ref mut inner) => inner.set(interface, servers)?,
            DnsMonitorHolder::Netsh(ref mut inner) => inner.set(interface, servers)?,
            DnsMonitorHolder::Tcpip(ref mut inner) => inner.set(interface, servers)?,
        }
        Ok(())
    }

    fn reset(&mut self) -> Result<(), Error> {
        match self.inner {
            DnsMonitorHolder::Iphlpapi(ref mut inner) => inner.reset()?,
            DnsMonitorHolder::Netsh(ref mut inner) => inner.reset()?,
            DnsMonitorHolder::Tcpip(ref mut inner) => inner.reset()?,
        }
        Ok(())
    }

    fn reset_before_interface_removal(&mut self) -> Result<(), Error> {
        match self.inner {
            DnsMonitorHolder::Iphlpapi(ref mut inner) => inner.reset_before_interface_removal()?,
            DnsMonitorHolder::Netsh(ref mut inner) => inner.reset_before_interface_removal()?,
            DnsMonitorHolder::Tcpip(ref mut inner) => inner.reset_before_interface_removal()?,
        }
        Ok(())
    }
}

impl DnsMonitor {
    fn detect_appropriate_method() -> Result<DnsMonitorHolder, Error> {
        Ok(DnsMonitorHolder::Iphlpapi(iphlpapi::DnsMonitor::new()?))
    }
}

enum DnsMonitorHolder {
    Iphlpapi(iphlpapi::DnsMonitor),
    Netsh(netsh::DnsMonitor),
    Tcpip(tcpip::DnsMonitor),
}

impl fmt::Display for DnsMonitorHolder {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DnsMonitorHolder::Iphlpapi(_) => f.write_str("SetInterfaceDnsSettings (iphlpapi)"),
            DnsMonitorHolder::Netsh(_) => f.write_str("netsh"),
            DnsMonitorHolder::Tcpip(_) => f.write_str("TCP/IP registry parameter"),
        }
    }
}

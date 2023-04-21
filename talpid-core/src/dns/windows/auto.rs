use super::{iphlpapi, netsh, tcpip};
use crate::dns::DnsMonitorT;
use windows_sys::Win32::System::Rpc::RPC_S_SERVER_UNAVAILABLE;

pub struct DnsMonitor {
    current_monitor: InnerMonitor,
}
enum InnerMonitor {
    Iphlpapi(iphlpapi::DnsMonitor),
    Netsh(netsh::DnsMonitor),
    Tcpip(tcpip::DnsMonitor),
}

impl InnerMonitor {
    fn set(&mut self, interface: &str, servers: &[std::net::IpAddr]) -> Result<(), super::Error> {
        match self {
            InnerMonitor::Iphlpapi(monitor) => monitor.set(interface, servers)?,
            InnerMonitor::Netsh(monitor) => monitor.set(interface, servers)?,
            InnerMonitor::Tcpip(monitor) => monitor.set(interface, servers)?,
        }
        Ok(())
    }

    fn reset(&mut self) -> Result<(), super::Error> {
        match self {
            InnerMonitor::Iphlpapi(monitor) => monitor.reset()?,
            InnerMonitor::Netsh(monitor) => monitor.reset()?,
            InnerMonitor::Tcpip(monitor) => monitor.reset()?,
        }
        Ok(())
    }

    fn reset_before_interface_removal(&mut self) -> Result<(), super::Error> {
        match self {
            InnerMonitor::Iphlpapi(monitor) => monitor.reset_before_interface_removal()?,
            InnerMonitor::Netsh(monitor) => monitor.reset_before_interface_removal()?,
            InnerMonitor::Tcpip(monitor) => monitor.reset_before_interface_removal()?,
        }
        Ok(())
    }
}

impl DnsMonitorT for DnsMonitor {
    type Error = super::Error;

    fn new() -> Result<Self, Self::Error> {
        let current_monitor = if iphlpapi::DnsMonitor::is_supported() {
            InnerMonitor::Iphlpapi(iphlpapi::DnsMonitor::new()?)
        } else {
            InnerMonitor::Netsh(netsh::DnsMonitor::new()?)
        };

        Ok(Self { current_monitor })
    }

    fn set(&mut self, interface: &str, servers: &[std::net::IpAddr]) -> Result<(), Self::Error> {
        let result = self.current_monitor.set(interface, servers);
        if self.fallback_due_to_dnscache(&result) {
            return self.set(interface, servers);
        }
        result
    }

    fn reset(&mut self) -> Result<(), Self::Error> {
        let result = self.current_monitor.reset();
        if self.fallback_due_to_dnscache(&result) {
            return self.reset();
        }
        result
    }

    fn reset_before_interface_removal(&mut self) -> Result<(), Self::Error> {
        let result = self.current_monitor.reset_before_interface_removal();
        if self.fallback_due_to_dnscache(&result) {
            return self.reset_before_interface_removal();
        }
        result
    }
}

impl DnsMonitor {
    fn fallback_due_to_dnscache(&mut self, result: &Result<(), super::Error>) -> bool {
        let is_dnscache_error = match result {
            Err(super::Error::Iphlpapi(iphlpapi::Error::SetInterfaceDnsSettings(error))) => {
                *error == RPC_S_SERVER_UNAVAILABLE
            }
            Err(super::Error::Netsh(netsh::Error::Netsh(Some(1)))) => true,
            _ => false,
        };
        if is_dnscache_error {
            log::warn!("dnscache is not running? Falling back on tcpip method");

            match tcpip::DnsMonitor::new() {
                Ok(mut tcpip) => {
                    // We need to disable flushing here since it may fail.
                    // Because dnscache is disabled, there's nothing to flush anyhow.
                    tcpip.disable_flushing();
                    self.current_monitor = InnerMonitor::Tcpip(tcpip);
                    true
                }
                Err(error) => {
                    log::error!("Failed to init tcpip DNS module: {error}");
                    false
                }
            }
        } else {
            false
        }
    }
}

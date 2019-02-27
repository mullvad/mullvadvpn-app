use std::{net::IpAddr, path::Path};

error_chain! {}

pub struct DnsMonitor;

impl super::DnsMonitorT for DnsMonitor {
    type Error = Error;

    fn new(_cache_dir: impl AsRef<Path>) -> Result<Self> {
        Ok(DnsMonitor)
    }

    fn set(&mut self, _interface: &str, _servers: &[IpAddr]) -> Result<()> {
        Ok(())
    }

    fn reset(&mut self) -> Result<()> {
        Ok(())
    }
}

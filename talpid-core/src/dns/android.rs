use std::{net::IpAddr, path::Path};

/// Stub error type for DNS errors on Android.
#[derive(Debug, err_derive::Error)]
#[error(display = "Unknown Android DNS error")]
pub struct Error;

pub struct DnsMonitor;

impl super::DnsMonitorT for DnsMonitor {
    type Error = Error;

    fn new(
        _handle: tokio::runtime::Handle,
        _cache_dir: impl AsRef<Path>,
    ) -> Result<Self, Self::Error> {
        Ok(DnsMonitor)
    }

    fn set(&mut self, _interface: &str, _servers: &[IpAddr]) -> Result<(), Self::Error> {
        Ok(())
    }

    fn reset(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }
}

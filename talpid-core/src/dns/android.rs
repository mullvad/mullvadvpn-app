use std::net::IpAddr;

/// Stub error type for DNS errors on Android.
#[derive(Debug, thiserror::Error)]
#[error("Unknown Android DNS error")]
pub struct Error;

pub struct DnsMonitor;

impl super::DnsMonitorT for DnsMonitor {
    type Error = Error;

    fn new() -> Result<Self, Self::Error> {
        Ok(DnsMonitor)
    }

    fn set(&mut self, _interface: &str, _servers: &[IpAddr]) -> Result<(), Self::Error> {
        Ok(())
    }

    fn reset(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }
}

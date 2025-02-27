mod icmp;

pub use icmp::Error;

/// Trait for sending ICMP requests to get some traffic from a remote server
#[async_trait::async_trait]
pub trait Pinger: Send {
    /// Sends an ICMP packet
    async fn send_icmp(&mut self) -> Result<(), Error>;
    /// Clears all resources used by the pinger.
    async fn reset(&mut self) {}
}

/// Create a new pinger
pub fn new_pinger(
    addr: std::net::Ipv4Addr,
    #[cfg(any(target_os = "linux", target_os = "macos"))] interface_name: String,
) -> Result<Box<dyn Pinger>, Error> {
    Ok(Box::new(icmp::Pinger::new(
        addr,
        #[cfg(any(target_os = "linux", target_os = "macos"))]
        interface_name,
    )?))
}

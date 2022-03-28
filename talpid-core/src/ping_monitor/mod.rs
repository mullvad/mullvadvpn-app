#[cfg(any(target_os = "android"))]
#[path = "android.rs"]
mod imp;

#[cfg(any(target_os = "windows", target_os = "linux", target_os = "macos"))]
#[path = "icmp.rs"]
mod imp;

pub use imp::Error;

/// Trait for sending ICMP requests to get some traffic from a remote server
pub trait Pinger: Send {
    /// Sends an ICMP packet
    fn send_icmp(&mut self) -> Result<(), Error>;
    /// Clears all resources used by the pinger.
    fn reset(&mut self) {}
}

/// Create a new pinger
pub fn new_pinger(
    addr: std::net::Ipv4Addr,
    #[cfg(any(target_os = "linux", target_os = "macos"))] interface_name: String,
) -> Result<Box<dyn Pinger>, Error> {
    Ok(Box::new(imp::Pinger::new(
        addr,
        #[cfg(any(target_os = "linux", target_os = "macos"))]
        interface_name,
    )?))
}

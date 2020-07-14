#[cfg(any(target_os = "android", target_os = "macos", target_os = "linux"))]
#[path = "unix.rs"]
mod imp;


#[cfg(target_os = "windows")]
#[path = "win.rs"]
mod imp;

pub use imp::Error;

pub trait Pinger: Send {
    fn send_icmp(&mut self) -> Result<(), Error>;
    #[cfg(unix)]
    fn reset(&mut self);
}

pub fn new_pinger(
    addr: std::net::Ipv4Addr,
    interface_name: String,
) -> Result<Box<dyn Pinger>, Error> {
    Ok(Box::new(imp::Pinger::new(addr, interface_name)?))
}

use super::TunConfig;
use nix::fcntl;
use std::{
    io,
    net::IpAddr,
    ops::Deref,
    os::unix::io::{AsRawFd, IntoRawFd, RawFd},
};
use tun::{platform, Configuration, Device};

/// Errors that can occur while setting up a tunnel device.
#[derive(Debug, err_derive::Error)]
#[error(no_from)]
pub enum Error {
    /// Failure to create a tunnel device.
    #[error(display = "Failed to create a tunnel device")]
    CreateTunnelDevice(#[cause] NetworkInterfaceError),

    /// Failure to set a tunnel device IP address.
    #[error(display = "Failed to set tunnel IP address: {}", _0)]
    SetIpAddr(IpAddr, #[cause] NetworkInterfaceError),

    /// Failure to set the tunnel device as up.
    #[error(display = "Failed to set the tunnel device as up")]
    SetUp(#[cause] NetworkInterfaceError),
}

/// Factory of tunnel devices on Unix systems.
pub struct UnixTunProvider;

impl Default for UnixTunProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl UnixTunProvider {
    pub fn new() -> Self {
        UnixTunProvider
    }

    pub fn get_tun(&mut self, config: TunConfig) -> Result<UnixTun, Error> {
        let mut tunnel_device = TunnelDevice::new().map_err(Error::CreateTunnelDevice)?;

        for ip in config.addresses.iter() {
            tunnel_device
                .set_ip(*ip)
                .map_err(|cause| Error::SetIpAddr(*ip, cause))?;
        }

        tunnel_device.set_up(true).map_err(Error::SetUp)?;

        Ok(UnixTun(tunnel_device))
    }
}

/// Generic tunnel device.
///
/// Contains the file descriptor representing the device.
pub struct UnixTun(TunnelDevice);

impl UnixTun {
    /// Retrieve the tunnel interface name.
    pub fn interface_name(&self) -> &str {
        self.get_name()
    }
}

impl Deref for UnixTun {
    type Target = TunnelDevice;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// Errors that can happen when working with *nix tunnel interfaces.
#[derive(err_derive::Error, Debug)]
#[error(no_from)]
pub enum NetworkInterfaceError {
    /// Failed to set IP address
    #[error(display = "Failed to set IPv4 address")]
    SetIpv4(#[error(source)] tun::Error),

    /// Failed to set IP address
    #[error(display = "Failed to set IPv6 address")]
    SetIpv6(#[error(source)] io::Error),

    /// Unable to open a tunnel device
    #[error(display = "Unable to open a tunnel device")]
    CreateDevice(#[error(source)] tun::Error),

    /// Failed to apply async flags to tunnel device
    #[error(display = "Failed to apply async flags to tunnel device")]
    SetDeviceAsync(#[error(source)] nix::Error),

    /// Failed to enable/disable link device
    #[error(display = "Failed to enable/disable link device")]
    ToggleDevice(#[error(source)] tun::Error),
}

/// A trait for managing link devices
pub trait NetworkInterface: Sized {
    /// Bring a given interface up or down
    fn set_up(&mut self, up: bool) -> Result<(), NetworkInterfaceError>;

    /// Set host IPs for interface
    fn set_ip(&mut self, ip: IpAddr) -> Result<(), NetworkInterfaceError>;

    /// Set MTU for interface
    fn set_mtu(&mut self, mtu: u16) -> Result<(), NetworkInterfaceError>;

    /// Get name of interface
    fn get_name(&self) -> &str;
}

fn apply_async_flags(fd: RawFd) -> Result<(), nix::Error> {
    fcntl::fcntl(fd, fcntl::FcntlArg::F_GETFL)?;
    let arg = fcntl::FcntlArg::F_SETFL(fcntl::OFlag::O_RDWR | fcntl::OFlag::O_NONBLOCK);
    fcntl::fcntl(fd, arg)?;
    Ok(())
}

/// A tunnel device
pub struct TunnelDevice {
    dev: platform::Device,
}

impl TunnelDevice {
    /// Creates a new Tunnel device
    #[allow(unused_mut)]
    pub fn new() -> Result<Self, NetworkInterfaceError> {
        let mut config = Configuration::default();

        #[cfg(target_os = "linux")]
        config.platform(|config| {
            config.packet_information(true);
        });
        let mut dev = platform::create(&config).map_err(NetworkInterfaceError::CreateDevice)?;
        apply_async_flags(dev.as_raw_fd()).map_err(NetworkInterfaceError::SetDeviceAsync)?;
        Ok(Self { dev })
    }
}

impl AsRawFd for TunnelDevice {
    fn as_raw_fd(&self) -> RawFd {
        self.dev.as_raw_fd()
    }
}

impl IntoRawFd for TunnelDevice {
    fn into_raw_fd(self) -> RawFd {
        self.dev.into_raw_fd()
    }
}

impl NetworkInterface for TunnelDevice {
    fn set_ip(&mut self, ip: IpAddr) -> Result<(), NetworkInterfaceError> {
        match ip {
            IpAddr::V4(ipv4) => self
                .dev
                .set_address(ipv4)
                .map_err(NetworkInterfaceError::SetIpv4),
            IpAddr::V6(ipv6) => {
                #[cfg(target_os = "linux")]
                {
                    duct::cmd!(
                        "ip",
                        "-6",
                        "addr",
                        "add",
                        ipv6.to_string(),
                        "dev",
                        self.dev.name()
                    )
                    .run()
                    .map(|_| ())
                    .map_err(NetworkInterfaceError::SetIpv6)
                }
                #[cfg(target_os = "macos")]
                {
                    duct::cmd!(
                        "ifconfig",
                        self.dev.name(),
                        "inet6",
                        ipv6.to_string(),
                        "alias"
                    )
                    .run()
                    .map(|_| ())
                    .map_err(NetworkInterfaceError::SetIpv6)
                }
            }
        }
    }

    fn set_up(&mut self, up: bool) -> Result<(), NetworkInterfaceError> {
        self.dev
            .enabled(up)
            .map_err(NetworkInterfaceError::ToggleDevice)
    }

    fn set_mtu(&mut self, mtu: u16) -> Result<(), NetworkInterfaceError> {
        self.dev
            .set_mtu(i32::from(mtu))
            .map_err(NetworkInterfaceError::ToggleDevice)
    }

    fn get_name(&self) -> &str {
        self.dev.name()
    }
}

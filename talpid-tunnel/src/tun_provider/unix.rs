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
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// Failed to set IP address
    #[error("Failed to set IPv4 address")]
    SetIpv4(#[source] tun::Error),

    /// Failed to set IP address
    #[error("Failed to set IPv6 address")]
    SetIpv6(#[source] io::Error),

    /// Unable to open a tunnel device
    #[error("Unable to open a tunnel device")]
    CreateDevice(#[source] tun::Error),

    /// Failed to apply async flags to tunnel device
    #[error("Failed to apply async flags to tunnel device")]
    SetDeviceAsync(#[source] nix::Error),

    /// Failed to enable/disable link device
    #[error("Failed to enable/disable link device")]
    ToggleDevice(#[source] tun::Error),
}

/// Factory of tunnel devices on Unix systems.
pub struct UnixTunProvider {
    config: TunConfig,
}

impl UnixTunProvider {
    pub const fn new(config: TunConfig) -> Self {
        UnixTunProvider { config }
    }

    /// Get the current tunnel config. Note that the tunnel must be recreated for any changes to
    /// take effect.
    pub fn config_mut(&mut self) -> &mut TunConfig {
        &mut self.config
    }

    /// Open a tunnel using the current tunnel config.
    pub fn open_tun(&mut self) -> Result<UnixTun, Error> {
        let mut tunnel_device = {
            let mut builder = TunnelDeviceBuilder::default();
            #[cfg(target_os = "linux")]
            builder.enable_packet_information();
            #[cfg(target_os = "linux")]
            if let Some(ref name) = self.config.name {
                builder.name(name);
            }
            builder.create()?
        };

        for ip in self.config.addresses.iter() {
            tunnel_device.set_ip(*ip)?;
        }

        tunnel_device.set_up(true)?;

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

/// A tunnel device
pub struct TunnelDevice {
    dev: platform::Device,
}

/// A tunnel device builder.
///
/// Call [`Self::create`] to create [`TunnelDevice`] from the config.
pub struct TunnelDeviceBuilder {
    config: Configuration,
}

impl TunnelDeviceBuilder {
    /// Create a [`TunnelDevice`] from this builder.
    ///
    /// Note: this function may fail if <TODO>
    pub fn create(self) -> Result<TunnelDevice, Error> {
        fn apply_async_flags(fd: RawFd) -> Result<(), nix::Error> {
            fcntl::fcntl(fd, fcntl::FcntlArg::F_GETFL)?;
            let arg = fcntl::FcntlArg::F_SETFL(fcntl::OFlag::O_RDWR | fcntl::OFlag::O_NONBLOCK);
            fcntl::fcntl(fd, arg)?;
            Ok(())
        }

        let dev = platform::create(&self.config).map_err(Error::CreateDevice)?;
        apply_async_flags(dev.as_raw_fd()).map_err(Error::SetDeviceAsync)?;
        Ok(TunnelDevice { dev })
    }

    /// Set a custom name for this tunnel device.
    #[cfg(target_os = "linux")]
    pub fn name(&mut self, name: &str) -> &mut Self {
        self.config.name(name);
        self
    }

    /// Enable packet information.
    /// When enabled the first 4 bytes of each packet is a header with flags and protocol type.
    #[cfg(target_os = "linux")]
    pub fn enable_packet_information(&mut self) -> &mut Self {
        self.config.platform(|platform_config| {
            platform_config.packet_information(true);
        });
        self
    }
}

impl Default for TunnelDeviceBuilder {
    fn default() -> Self {
        let config = Configuration::default();
        Self { config }
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

impl TunnelDevice {
    fn set_ip(&mut self, ip: IpAddr) -> Result<(), Error> {
        match ip {
            IpAddr::V4(ipv4) => self.dev.set_address(ipv4).map_err(Error::SetIpv4),
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
                    .map_err(Error::SetIpv6)
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
                    .map_err(Error::SetIpv6)
                }
            }
        }
    }

    fn set_up(&mut self, up: bool) -> Result<(), Error> {
        self.dev.enabled(up).map_err(Error::ToggleDevice)
    }

    fn get_name(&self) -> &str {
        self.dev.name()
    }
}

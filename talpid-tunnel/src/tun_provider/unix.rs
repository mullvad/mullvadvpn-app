use super::TunConfig;
use nix::fcntl;
#[cfg(target_os = "macos")]
use std::io;
use std::{
    net::IpAddr,
    ops::Deref,
    os::unix::io::{AsRawFd, IntoRawFd, RawFd},
};
use tun::{AbstractDevice, Configuration};

/// Errors that can occur while setting up a tunnel device.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// Failed to set IP address on tunnel device
    #[cfg(target_os = "linux")]
    #[error("Failed to set IP address on tunnel device")]
    SetIp(#[source] tun::Error),

    /// Failed to set IPv4 address on tunnel device
    #[cfg(target_os = "macos")]
    #[error("Failed to set IPv4 address")]
    SetIpv4(#[source] tun::Error),

    /// Failed to set IPv6 address on tunnel device
    #[cfg(target_os = "macos")]
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

    /// Failed to get device name
    #[error("Failed to get tunnel device name")]
    GetDeviceName(#[source] tun::Error),
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
            #[allow(unused_mut)]
            let mut builder = TunnelDeviceBuilder::default();
            #[cfg(target_os = "linux")]
            {
                builder.enable_packet_information();
                if let Some(ref name) = self.config.name {
                    builder.name(name);
                }
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
    pub fn interface_name(&self) -> Result<String, Error> {
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
    dev: tun::Device,
}

/// A tunnel device builder.
///
/// Call [`Self::create`] to create [`TunnelDevice`] from the config.
#[derive(Default)]
pub struct TunnelDeviceBuilder {
    config: Configuration,
}

impl TunnelDeviceBuilder {
    /// Create a [`TunnelDevice`] from this builder.
    pub fn create(self) -> Result<TunnelDevice, Error> {
        fn apply_async_flags(fd: RawFd) -> Result<(), nix::Error> {
            fcntl::fcntl(fd, fcntl::FcntlArg::F_GETFL)?;
            let arg = fcntl::FcntlArg::F_SETFL(fcntl::OFlag::O_RDWR | fcntl::OFlag::O_NONBLOCK);
            fcntl::fcntl(fd, arg)?;
            Ok(())
        }

        let dev = tun::create(&self.config).map_err(Error::CreateDevice)?;
        apply_async_flags(dev.as_raw_fd()).map_err(Error::SetDeviceAsync)?;
        Ok(TunnelDevice { dev })
    }

    /// Set a custom name for this tunnel device.
    #[cfg(target_os = "linux")]
    pub fn name(&mut self, name: &str) -> &mut Self {
        self.config.tun_name(name);
        self
    }

    /// Enable packet information.
    /// When enabled the first 4 bytes of each packet is a header with flags and protocol type.
    #[cfg(target_os = "linux")]
    pub fn enable_packet_information(&mut self) -> &mut Self {
        self.config.platform_config(|config| {
            #[allow(deprecated)]
            // NOTE: This function does seemingly have an effect on Linux, despite what the deprecation
            // warning says.
            config.packet_information(true);
        });
        self
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
    #[cfg(target_os = "linux")]
    fn set_ip(&mut self, ip: IpAddr) -> Result<(), Error> {
        self.dev.set_address(ip).map_err(Error::SetIp)?;
        Ok(())
    }

    #[cfg(target_os = "macos")]
    fn set_ip(&mut self, ip: IpAddr) -> Result<(), Error> {
        match ip {
            // NOTE: As of `tun 0.7`, `Device::set_address` accepts an `IpAddr` but
            // only supports the `IpAddr::V4` address kind and panics if you pass it an
            // `IpAddr::V6` value..
            IpAddr::V4(ipv4) => {
                self.dev.set_address(ipv4.into()).map_err(Error::SetIpv4)?;
            }
            IpAddr::V6(ipv6) => {
                use std::process::Command;
                // ifconfig <device> inet6 <ipv6 address> alias
                let address = ipv6.to_string();
                let device = self.dev.tun_name().unwrap(); // TODO: Do not unwrap!
                let mut ifconfig = Command::new("ifconfig");
                ifconfig.args([&device, "inet6", &address, "alias"]);
                ifconfig.output().map_err(Error::SetIpv6)?;
            }
        }
        Ok(())
    }

    fn set_up(&mut self, up: bool) -> Result<(), Error> {
        self.dev.enabled(up).map_err(Error::ToggleDevice)
    }

    fn get_name(&self) -> Result<String, Error> {
        self.dev.tun_name().map_err(Error::GetDeviceName)
    }
}

use super::TunConfig;
use std::{io, net::IpAddr, ops::Deref};
use tun::{AbstractDevice, AsyncDevice, Configuration, Device};

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

    /// Failed to enable/disable link device
    #[error("Failed to enable/disable link device")]
    ToggleDevice(#[source] tun::Error),
}

/// Factory of tunnel devices on Unix systems.
pub struct WindowsTunProvider {
    config: TunConfig,
}

impl WindowsTunProvider {
    pub const fn new(config: TunConfig) -> Self {
        WindowsTunProvider { config }
    }

    /// Get the current tunnel config. Note that the tunnel must be recreated for any changes to
    /// take effect.
    pub fn config_mut(&mut self) -> &mut TunConfig {
        &mut self.config
    }

    /// Open a tunnel using the current tunnel config.
    pub fn open_tun(&mut self) -> Result<WindowsTun, Error> {
        let mut tunnel_device = {
            #[allow(unused_mut)]
            let mut builder = TunnelDeviceBuilder::default();
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

        Ok(WindowsTun(tunnel_device))
    }
}

/// Generic tunnel device.
///
/// Contains the file descriptor representing the device.
pub struct WindowsTun(TunnelDevice);

impl WindowsTun {
    /// Retrieve the tunnel interface name.
    pub fn interface_name(&self) -> String {
        self.get_name()
    }

    pub fn into_tun_lol(self) -> AsyncDevice {
        AsyncDevice::new(self.0.dev).unwrap()
    }
}

impl Deref for WindowsTun {
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
pub struct TunnelDeviceBuilder {
    config: Configuration,
}

impl TunnelDeviceBuilder {
    /// Create a [`TunnelDevice`] from this builder.
    pub fn create(self) -> Result<TunnelDevice, Error> {
        /*fn apply_async_flags(fd: RawFd) -> Result<(), nix::Error> {
            fcntl::fcntl(fd, fcntl::FcntlArg::F_GETFL)?;
            //let arg = fcntl::FcntlArg::F_SETFL(fcntl::OFlag::O_RDWR | fcntl::OFlag::O_NONBLOCK);
            // FIXME
            let arg = fcntl::FcntlArg::F_SETFL(fcntl::OFlag::O_RDWR);
            fcntl::fcntl(fd, arg)?;
            Ok(())
        }*/

        let dev = tun::create(&self.config).map_err(Error::CreateDevice)?;
        //apply_async_flags(dev.as_raw_fd()).map_err(Error::SetDeviceAsync)?;
        Ok(TunnelDevice { dev })
    }

    /// Set a custom name for this tunnel device.
    #[cfg(target_os = "linux")]
    pub fn name(&mut self, name: &str) -> &mut Self {
        self.config.name(name);
        self
    }

    /*/// Enable packet information.
    /// When enabled the first 4 bytes of each packet is a header with flags and protocol type.
    #[cfg(target_os = "linux")]
    pub fn enable_packet_information(&mut self) -> &mut Self {
        self.config.platform(|platform_config| {
            platform_config.packet_information(true);
        });
        self
    }*/
}

impl Default for TunnelDeviceBuilder {
    fn default() -> Self {
        let config = Configuration::default();
        Self { config }
    }
}

impl TunnelDevice {
    fn set_ip(&mut self, ip: IpAddr) -> Result<(), Error> {
        match ip {
            IpAddr::V4(ipv4) => self.dev.set_address(ipv4.into()).map_err(Error::SetIpv4),
            IpAddr::V6(ipv6) => {
                // TODO
                Ok(())
            }
        }
    }

    fn set_up(&mut self, up: bool) -> Result<(), Error> {
        self.dev.enabled(up).map_err(Error::ToggleDevice)
    }

    fn get_name(&self) -> String {
        self.dev.tun_name().unwrap()
    }
}

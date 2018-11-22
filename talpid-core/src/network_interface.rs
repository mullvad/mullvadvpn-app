use nix::fcntl;
use std::net::IpAddr;
use std::os::unix::io::{AsRawFd, IntoRawFd, RawFd};
use tun::{platform, Configuration, Device};

error_chain! {
    errors {
        /// Unable to open a tunnel device
        SetupDeviceError { description("Failed to setup a device") }
        /// Unable to get the name of a tunnel device
        GetNameError { description("Failed to get a name for the device") }
        /// Failed to set IP address
        SetIpError{ description( "Failed to set IP address" ) }
        /// Failed to toggle device state
        ToggleDeviceError{ description( "Failed to enable/disable link device" ) }
    }
}


/// A trait for managing link devices
pub trait NetworkInterface: Sized {
    /// Bring a given interface up or down
    fn set_up(&mut self, up: bool) -> Result<()>;

    /// Set host IPs for interface
    fn set_ip(&mut self, ip: IpAddr) -> Result<()>;

    /// Set MTU for interface
    fn set_mtu(&mut self, mtu: u16) -> Result<()>;

    /// Get name of interface
    fn get_name(&self) -> &str;
}


trait WireguardLink: AsRawFd + IntoRawFd {}

fn apply_async_flags(fd: RawFd) -> Result<()> {
    fcntl::fcntl(fd, fcntl::FcntlArg::F_GETFL).chain_err(|| ErrorKind::SetupDeviceError)?;
    let arg = fcntl::FcntlArg::F_SETFL(fcntl::OFlag::O_RDWR | fcntl::OFlag::O_NONBLOCK);
    fcntl::fcntl(fd, arg).chain_err(|| ErrorKind::SetupDeviceError)?;
    Ok(())
}

/// A tunnel devie
pub struct TunnelDevice {
    dev: platform::Device,
}

impl TunnelDevice {
    /// Creates a new Tunnel device
    #[allow(unused_mut)]
    pub fn new() -> Result<Self> {
        let mut config = Configuration::default();

        #[cfg(target_os = "linux")]
        config.platform(|config| {
            config.packet_information(true);
        });
        let mut dev = platform::create(&config).chain_err(|| ErrorKind::SetupDeviceError)?;
        apply_async_flags(dev.as_raw_fd())?;
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
    fn set_ip(&mut self, ip: IpAddr) -> Result<()> {
        match ip {
            IpAddr::V4(ipv4) => self
                .dev
                .set_address(ipv4)
                .chain_err(|| ErrorKind::SetIpError),
            IpAddr::V6(ipv6) => {
                #[cfg(target_os = "linux")]
                {
                    duct::cmd!(
                        "ip",
                        "addr",
                        "add",
                        ipv6.to_string(),
                        "dev",
                        self.dev.name()
                    )
                    .run()
                    .map(|_| ())
                    .chain_err(|| ErrorKind::SetIpError)
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
                    .chain_err(|| ErrorKind::SetIpError)
                }
            }
        }
    }

    fn set_up(&mut self, up: bool) -> Result<()> {
        self.dev
            .enabled(up)
            .chain_err(|| ErrorKind::ToggleDeviceError)
    }


    fn set_mtu(&mut self, mtu: u16) -> Result<()> {
        self.dev
            .set_mtu(mtu as i32)
            .chain_err(|| ErrorKind::ToggleDeviceError)
    }

    fn get_name(&self) -> &str {
        self.dev.name()
    }
}

use std::net::IpAddr;

#[cfg(windows)]
use std::os::win::io::{AsRawHandle, IntoRawHandle};

#[cfg(unix)]
use std::os::unix::io::{AsRawFd, IntoRawFd, RawFd};

error_chain! {
    errors {
        /// Unable to open a tunnel device
        FailedToSetupDevice { description("Failed to setup a device") }
        /// Unable to create a default device name
        GetDefaultDeviceName { description("Failed to get defualt device name") }
        /// Unable to gen name of a tunnel device
        FailedToGetName { description("Failed to get a name for the device") }
        /// Failed to set IP address
        FailedToSetIp{ description( "Failed to set IP address" ) }
        /// Failed to set IP address
        FailedToToggleDevice{ description( "Failed to enable/disable link device" ) }
    }
}


#[cfg(unix)]
use self::unix as imp;

/// Link is a tunnel device
#[cfg(unix)]
pub struct Link {
    link: imp::Link,
}

impl AsRawFd for Link {
    fn as_raw_fd(&self) -> RawFd {
        self.link.as_raw_fd()
    }
}

impl IntoRawFd for Link {
    fn into_raw_fd(self) -> RawFd {
        self.link.into_raw_fd()
    }
}

#[cfg(unix)]
impl Link {
    /// Creates a new interface with the given link
    pub fn new() -> Result<Self> {
        Ok(Link {
            link: imp::Link::new()?,
        })
    }

    /// Bring a given interface up or down
    pub fn set_up(&mut self, up: bool) -> Result<()> {
        self.link.set_up(up)
    }

    /// Set host IPs for interface
    pub fn set_ip(&mut self, ip: IpAddr) -> Result<()> {
        self.link.set_ip(ip)
    }

    /// Set MTU for interface
    pub fn set_mtu(&mut self, mtu: u16) -> Result<()> {
        self.link.set_mtu(mtu)
    }

    /// Get name of interface
    pub fn get_name(&self) -> &str {
        self.link.get_name()
    }
}


/// A trait for managing link devices
pub trait LinkT: Sized {
    /// Creates a new link
    fn new() -> Result<Self>;

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

#[cfg(unix)]
mod unix {
    use super::LinkT;
    use super::{ErrorKind, Result, ResultExt};
    use nix::fcntl;
    use std::net::IpAddr;
    use std::os::unix::io::{AsRawFd, IntoRawFd, RawFd};
    use tun::{platform, Configuration, Device};

    fn apply_async_flags(fd: RawFd) -> Result<()> {
        fcntl::fcntl(fd, fcntl::FcntlArg::F_GETFL).chain_err(|| ErrorKind::FailedToSetupDevice)?;
        let arg = fcntl::FcntlArg::F_SETFL(fcntl::OFlag::O_RDWR | fcntl::OFlag::O_NONBLOCK);
        fcntl::fcntl(fd, arg).chain_err(|| ErrorKind::FailedToSetupDevice)?;
        Ok(())
    }

    pub struct Link {
        dev: platform::Device,
    }

    impl AsRawFd for Link {
        fn as_raw_fd(&self) -> RawFd {
            self.dev.as_raw_fd()
        }
    }

    impl IntoRawFd for Link {
        fn into_raw_fd(self) -> RawFd {
            self.dev.into_raw_fd()
        }
    }

    impl LinkT for Link {
        #[allow(unused_mut)]
        fn new() -> Result<Self> {
            let mut config = Configuration::default();

            #[cfg(target_os = "linux")]
            config.platform(|config| {
                config.packet_information(true);
            });
            let mut dev = platform::create(&config).chain_err(|| ErrorKind::FailedToSetupDevice)?;
            apply_async_flags(dev.as_raw_fd())?;
            Ok(Self { dev })
        }

        fn set_ip(&mut self, ip: IpAddr) -> Result<()> {
            match ip {
                IpAddr::V4(ipv4) => self
                    .dev
                    .set_address(ipv4)
                    .chain_err(|| ErrorKind::FailedToSetIp),
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
                        .chain_err(|| ErrorKind::FailedToSetIp)
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
                        .chain_err(|| ErrorKind::FailedToSetIp)
                    }
                }
            }
        }

        fn set_up(&mut self, up: bool) -> Result<()> {
            self.dev
                .enabled(up)
                .chain_err(|| ErrorKind::FailedToToggleDevice)
        }


        fn set_mtu(&mut self, mtu: u16) -> Result<()> {
            self.dev
                .set_mtu(mtu as i32)
                .chain_err(|| ErrorKind::FailedToToggleDevice)
        }

        fn get_name(&self) -> &str {
            self.dev.name()
        }
    }
}

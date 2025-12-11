#[cfg(feature = "wireguard-go")]
pub use tun05_imp::{Error, UnixTun, UnixTunProvider};
#[cfg(not(feature = "wireguard-go"))]
pub use tun07_imp::{Error, UnixTun, UnixTunProvider};
#[cfg(feature = "wireguard-go")]
mod tun05_imp {
    use std::{
        net::IpAddr,
        ops::Deref,
        os::{
            fd::AsFd,
            unix::{
                io::{AsRawFd, RawFd},
                prelude::BorrowedFd,
            },
        },
        process::Command,
    };
    use tun::{Configuration, Device};

    use crate::tun_provider::TunConfig;

    /// Errors that can occur while setting up a tunnel device.
    #[derive(Debug, thiserror::Error)]
    pub enum Error {
        /// Failed to set IPv4 address on tunnel device
        #[error("Failed to set IPv4 address")]
        SetIpv4(#[source] tun::Error),

        /// Failed to set IPv6 address on tunnel device
        #[error("Failed to set IPv6 address")]
        SetIpv6(#[source] std::io::Error),

        /// Unable to open a tunnel device
        #[error("Unable to open a tunnel device")]
        CreateDevice(#[source] tun::Error),

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
                #[expect(unused_mut)]
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

    impl AsFd for UnixTun {
        fn as_fd(&self) -> BorrowedFd<'_> {
            self.deref().as_fd()
        }
    }

    /// A tunnel device
    pub struct TunnelDevice {
        dev: tun::AsyncDevice,
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
            let dev = tun::create_as_async(&self.config).map_err(Error::CreateDevice)?;
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
            self.config.platform(|config| {
                #[expect(deprecated)]
                // NOTE: This function does seemingly have an effect on Linux, despite what the deprecation
                // warning says.
                config.packet_information(true);
            });
            self
        }
    }

    impl AsFd for TunnelDevice {
        fn as_fd(&self) -> BorrowedFd<'_> {
            // TODO: make sure we uphold safety requirements of BorrowedFd
            #[expect(clippy::undocumented_unsafe_blocks)]
            unsafe {
                BorrowedFd::borrow_raw(self.as_raw_fd())
            }
        }
    }

    impl AsRawFd for TunnelDevice {
        fn as_raw_fd(&self) -> RawFd {
            self.dev.get_ref().as_raw_fd()
        }
    }

    impl TunnelDevice {
        fn set_ip(&mut self, ip: IpAddr) -> Result<(), Error> {
            match ip {
                IpAddr::V4(ipv4) => {
                    self.dev
                        .get_mut()
                        .set_address(ipv4)
                        .map_err(Error::SetIpv4)?;
                }

                // NOTE: On MacOs, As of `tun 0.7`, `Device::set_address` accepts an `IpAddr` but
                // only supports the `IpAddr::V4` address kind and panics if you pass it an
                // `IpAddr::V6` value.
                #[cfg(target_os = "macos")]
                IpAddr::V6(ipv6) => {
                    // ifconfig <device> inet6 <ipv6 address> alias
                    let ipv6 = ipv6.to_string();
                    let device = self.dev.get_ref().name();
                    Command::new("ifconfig")
                        .args([device, "inet6", &ipv6, "alias"])
                        .output()
                        .map_err(Error::SetIpv6)?;
                }

                // NOTE: On Linux, As of `tun 0.7`, `Device::set_address` throws an I/O error if you
                // pass it an IPv6-address.
                #[cfg(target_os = "linux")]
                IpAddr::V6(ipv6) => {
                    // ip -6 addr add <ipv6 address> dev <device>
                    let ipv6 = ipv6.to_string();
                    let device = self.dev.get_ref().name();
                    Command::new("ip")
                        .args(["-6", "addr", "add", &ipv6, "dev", device])
                        .output()
                        .map_err(Error::SetIpv6)?;
                }
            }
            Ok(())
        }

        fn set_up(&mut self, up: bool) -> Result<(), Error> {
            self.dev.get_mut().enabled(up).map_err(Error::ToggleDevice)
        }

        fn get_name(&self) -> Result<String, Error> {
            Ok(self.dev.get_ref().name().to_owned())
        }
    }
}

#[cfg(not(feature = "wireguard-go"))]
mod tun07_imp {
    use std::net::IpAddr;
    use std::os::fd::{AsRawFd, RawFd};
    use std::process::Command;

    use std::ops::Deref;

    use tun07::{AbstractDevice, AsyncDevice};

    use crate::tun_provider::TunConfig;

    /// Errors that can occur while setting up a tunnel device.
    #[derive(Debug, thiserror::Error)]
    pub enum Error {
        /// Failed to set IPv4 address on tunnel device
        #[error("Failed to set IPv4 address")]
        SetIpv4(#[source] tun07::Error),

        /// Failed to set IPv6 address on tunnel device
        #[error("Failed to set IPv6 address")]
        SetIpv6(#[source] std::io::Error),

        /// Unable to open a tunnel device
        #[error("Unable to open a tunnel device")]
        CreateDevice(#[source] tun07::Error),

        /// Failed to enable/disable link device
        #[error("Failed to enable/disable link device")]
        ToggleDevice(#[source] tun07::Error),

        /// Failed to get device name
        #[error("Failed to get tunnel device name")]
        GetDeviceName(#[source] tun07::Error),
    }

    /// Factory of tunnel devices on Unix systems.
    pub struct UnixTunProvider {
        pub(crate) config: TunConfig,
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
                {
                    if self.config.packet_information {
                        builder.enable_packet_information();
                    }

                    if let Some(ref name) = self.config.name {
                        builder.name(name);
                    }
                }
                builder.mtu(self.config.mtu);
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

        pub fn into_inner(self) -> TunnelDevice {
            self.0
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
        pub(crate) dev: tun07::AsyncDevice,
    }

    /// A tunnel device builder.
    ///
    /// Call [`Self::create`] to create [`TunnelDevice`] from the config.
    #[derive(Default)]
    pub struct TunnelDeviceBuilder {
        pub(crate) config: tun07::Configuration,
    }

    impl TunnelDeviceBuilder {
        /// Create a [`TunnelDevice`] from this builder.
        pub fn create(self) -> Result<TunnelDevice, Error> {
            let dev = tun07::create_as_async(&self.config).map_err(Error::CreateDevice)?;
            Ok(TunnelDevice { dev })
        }

        /// Set a custom name for this tunnel device.
        #[cfg(target_os = "linux")]
        pub fn name(&mut self, name: &str) -> &mut Self {
            self.config.tun_name(name);
            self
        }

        /// Set tunnel device MTU.
        pub fn mtu(&mut self, mtu: u16) -> &mut Self {
            self.config.mtu(mtu);
            self
        }

        /// Enable packet information.
        /// When enabled the first 4 bytes of each packet is a header with flags and protocol type.
        #[cfg(target_os = "linux")]
        pub fn enable_packet_information(&mut self) -> &mut Self {
            self.config.platform_config(|config| {
                #[expect(deprecated)]
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

    impl TunnelDevice {
        pub(crate) fn set_ip(&mut self, ip: IpAddr) -> Result<(), Error> {
            match ip {
                IpAddr::V4(ipv4) => {
                    self.dev.set_address(ipv4.into()).map_err(Error::SetIpv4)?;
                }

                // NOTE: As of `tun 0.7`, `Device::set_address` accepts an `IpAddr` but
                // only supports the `IpAddr::V4`.
                // On MacOs, `Device::set_address` panics if you pass it an `IpAddr::V6` value.
                // On Linux, `Device::set_address` throws an I/O error if you pass it an IPv6-address.
                IpAddr::V6(ipv6) => {
                    let ipv6 = ipv6.to_string();
                    let device = self.get_name()?;
                    // ifconfig <device> inet6 <ipv6 address> alias
                    #[cfg(target_os = "macos")]
                    Command::new("ifconfig")
                        .args([&device, "inet6", &ipv6, "alias"])
                        .output()
                        .map_err(Error::SetIpv6)?;
                    // ip -6 addr add <ipv6 address> dev <device>
                    #[cfg(target_os = "linux")]
                    Command::new("ip")
                        .args(["-6", "addr", "add", &ipv6, "dev", &device])
                        .output()
                        .map_err(Error::SetIpv6)?;
                }
            }
            Ok(())
        }

        pub(crate) fn set_up(&mut self, up: bool) -> Result<(), Error> {
            self.dev.enabled(up).map_err(Error::ToggleDevice)
        }

        pub(crate) fn get_name(&self) -> Result<String, Error> {
            self.dev.tun_name().map_err(Error::GetDeviceName)
        }

        pub fn into_inner(self) -> AsyncDevice {
            self.dev
        }
    }
}

use super::TunConfig;
use crate::network_interface::{self, NetworkInterface, TunnelDevice};
use std::{net::IpAddr, ops::Deref};

/// Errors that can occur while setting up a tunnel device.
#[derive(Debug, err_derive::Error)]
#[error(no_from)]
pub enum Error {
    /// Failure to create a tunnel device.
    #[error(display = "Failed to create a tunnel device")]
    CreateTunnelDevice(#[cause] network_interface::Error),

    /// Failure to set a tunnel device IP address.
    #[error(display = "Failed to set tunnel IP address: {}", _0)]
    SetIpAddr(IpAddr, #[cause] network_interface::Error),

    /// Failure to set the tunnel device as up.
    #[error(display = "Failed to set the tunnel device as up")]
    SetUp(#[cause] network_interface::Error),
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

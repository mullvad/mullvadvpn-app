use super::{Tun, TunConfig, TunProvider};
use crate::network_interface::{self, NetworkInterface, TunnelDevice};
use std::net::IpAddr;
use talpid_types::BoxedError;

/// Errors that can occur while setting up a tunnel device.
#[derive(Debug, err_derive::Error)]
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

impl TunProvider for UnixTunProvider {
    fn create_tun(&self, config: TunConfig) -> Result<Box<dyn Tun>, BoxedError> {
        let mut tunnel_device = TunnelDevice::new()
            .map_err(|cause| BoxedError::new(Error::CreateTunnelDevice(cause)))?;

        for ip in config.addresses.iter() {
            tunnel_device
                .set_ip(*ip)
                .map_err(|cause| BoxedError::new(Error::SetIpAddr(*ip, cause)))?;
        }

        tunnel_device
            .set_up(true)
            .map_err(|cause| BoxedError::new(Error::SetUp(cause)))?;

        Ok(Box::new(tunnel_device))
    }
}

impl Tun for TunnelDevice {
    fn interface_name(&self) -> &str {
        self.get_name()
    }
}

use super::TunConfig;
use std::{io, net::IpAddr, ops::Deref};
use tun08::{self as tun, AbstractDeviceExt};
use tun08::{AbstractDevice, AsyncDevice, Configuration};
use windows_sys::Win32::NetworkManagement::Ndis::NET_LUID_LH;

/// Errors that can occur while setting up a tunnel device.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// Failed to set IP address
    #[error("Failed to set IPv6 address")]
    SetIp(#[source] talpid_windows::net::Error),

    /// Unable to open a tunnel device
    #[error("Unable to open a tunnel device")]
    CreateDevice(#[source] tun::Error),

    /// Failed to enable/disable link device
    #[error("Failed to enable/disable link device")]
    ToggleDevice(#[source] tun::Error),

    /// Failed to get device luid
    #[error("Failed to get tunnel device luid")]
    GetDeviceLuid(#[source] io::Error),

    /// Failed to get device name
    #[error("Failed to get tunnel device name")]
    GetDeviceName(#[source] tun::Error),

    /// IO error
    #[error("IO error")]
    Io(#[from] io::Error),
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
        let (first_addr, remaining_addrs) = self
            .config
            .addresses
            .split_first()
            .map(|(first, rest)| (Some(first), rest))
            .unwrap_or((None, &[]));

        let mut tunnel_device = {
            #[allow(unused_mut)]
            let mut builder = TunnelDeviceBuilder::default();

            // When routing, metric of a route is equal to the sum of the interface metric and
            // metric value set for the route itself. Setting the interface metric to 1 gives tunnel
            // routes the highest possible priority.
            builder.config.metric(1);

            // TODO: have tun either not use netsh or not set any default address at all
            // TODO: tun can only set a single address
            if let Some(addr) = first_addr {
                builder.config.address(*addr);
            }

            /// Tunnel adapter name
            const ADAPTER_NAME: &str = "Mullvad";
            /// Tunnel adapter GUID.
            /// Reuse the same ID, if possible. This prevents Windows from thinking it's a
            /// "new network".
            // {AFE43773-E1F8-4EBB-8536-576AB86AFE9A}
            const ADAPTER_GUID: u128 = 0xAFE4_3773_E1F8_4EBB_8536_576A_B86A_FE9A;

            builder.config.tun_name(ADAPTER_NAME);
            builder
                .config
                .platform_config(|cfg: &mut tun08::PlatformConfig| {
                    cfg.device_guid(ADAPTER_GUID);

                    let wintun_path = self.config.resource_dir.join("wintun.dll");
                    cfg.wintun_file(wintun_path);
                });

            builder.create()?
        };

        for ip in remaining_addrs {
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
    pub fn interface_name(&self) -> Result<String, Error> {
        self.get_name()
    }

    pub fn into_inner(self) -> AsyncDevice {
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
        let dev = tun::create(&self.config).map_err(Error::CreateDevice)?;
        Ok(TunnelDevice { dev })
    }
}

impl Default for TunnelDeviceBuilder {
    fn default() -> Self {
        let config = Configuration::default();
        Self { config }
    }
}

impl TunnelDevice {
    fn set_ip(&mut self, ip: IpAddr) -> Result<(), Error> {
        let luid = self.dev.tun_luid();
        let luid = NET_LUID_LH { Value: luid };
        talpid_windows::net::add_ip_address_for_interface(luid, ip).map_err(Error::SetIp)
    }

    fn set_up(&mut self, up: bool) -> Result<(), Error> {
        self.dev.enabled(up).map_err(Error::ToggleDevice)
    }

    fn get_name(&self) -> Result<String, Error> {
        self.dev.tun_name().map_err(Error::GetDeviceName)
    }
}

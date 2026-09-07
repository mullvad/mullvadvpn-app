use super::TunConfig;
use std::sync::Arc;
use std::time::Duration;
use std::{io, net::IpAddr, ops::Deref};
use tun::{self, AbstractDeviceExt};
use tun::{AbstractDevice, AsyncDevice, Configuration};
use windows_sys::Win32::NetworkManagement::Ndis::NET_LUID_LH;
use wintun_bindings::Adapter;

/// Tunnel adapter name.
///
/// This must differ from the name of the wireguard-nt adapter, since both adapters are kept alive
/// between connections and Windows requires interface names to be unique.
const ADAPTER_NAME: &str = "Mullvad GotaTun";
/// Tunnel adapter type. Unlike the name, this does not have to be unique. It ends up in the
/// description of the network adapter.
const ADAPTER_TYPE: &str = "Mullvad";
/// Tunnel adapter GUID.
/// Reuse the same ID, if possible. This prevents Windows from thinking it's a
/// "new network".
// {AFE43773-E1F8-4EBB-8536-576AB86AFE9A}
const ADAPTER_GUID: u128 = 0xAFE4_3773_E1F8_4EBB_8536_576A_B86A_FE9A;

/// Maximum time to wait for the tunnel IP addresses to complete duplicate address detection.
const WAIT_FOR_ADDRESSES_TIMEOUT: Duration = Duration::from_secs(5);

/// Errors that can occur while setting up a tunnel device.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// Failed to set IP address
    #[error("Failed to set IPv6 address")]
    SetIp(#[source] talpid_windows::net::Error),

    /// Failed to remove an IP address that is no longer in use
    #[error("Failed to remove a stale IP address")]
    RemoveIp(#[source] talpid_windows::net::Error),

    /// Failed to list the IP addresses of the tunnel device
    #[error("Failed to list the addresses of the tunnel device")]
    ListIps(#[source] talpid_windows::net::Error),

    /// Failed to load wintun.dll
    #[error("Failed to load wintun.dll")]
    LoadWintun(#[source] wintun_bindings::Error),

    /// Failed to create the wintun adapter
    #[error("Failed to create the wintun adapter")]
    CreateAdapter(#[source] wintun_bindings::Error),

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

    /// Timeout waiting for IP interfaces
    #[error("Error waiting for IP interfaces")]
    WaitForInterfaces(#[source] talpid_windows::net::Error),

    /// Timeout waiting for the tunnel IP addresses to become usable
    #[error("Error waiting for tunnel IP addresses")]
    WaitForAddresses(#[source] talpid_windows::net::Error),

    /// Failed to configure IP interfaces
    #[error("Failed to configure MTU and metric")]
    InitializeInterfaces(#[source] std::io::Error),

    /// IO error
    #[error("IO error")]
    Io(#[from] io::Error),
}

/// Factory of tunnel devices on Unix systems.
pub struct WindowsTunProvider {
    config: TunConfig,
    /// Keeps the wintun adapter alive between connections.
    ///
    /// Creating the adapter takes on the order of 200 ms, whereas opening an existing one takes
    /// about 20 ms. Without this handle the adapter is destroyed as soon as the tunnel device is
    /// dropped, and has to be created again for the next connection.
    adapter: Option<Arc<Adapter>>,
    /// The addresses that have been added to the adapter, so that they can be removed again.
    ///
    /// Only addresses added here are ever removed. Addresses that Windows assigns by itself, such
    /// as link-local ones, are left alone.
    configured_addresses: Vec<IpAddr>,
}

impl WindowsTunProvider {
    pub const fn new(config: TunConfig) -> Self {
        WindowsTunProvider {
            config,
            adapter: None,
            configured_addresses: vec![],
        }
    }

    /// Get the current tunnel config. Note that the tunnel must be recreated for any changes to
    /// take effect.
    pub fn config_mut(&mut self) -> &mut TunConfig {
        &mut self.config
    }

    /// Open a tunnel using the current tunnel config.
    ///
    /// The wintun adapter is reused if one is already open, and created otherwise. If anything
    /// goes wrong the adapter is discarded, so that the next attempt starts from a new one.
    pub fn open_tun(&mut self) -> Result<WindowsTun, Error> {
        self.open_tun_inner().inspect_err(|error| {
            log::warn!("Discarding the tunnel adapter after a failure: {error}");
            self.close_adapter();
        })
    }

    /// Destroy the wintun adapter, if there is one. It is created again by the next
    /// [`Self::open_tun`].
    pub fn close_adapter(&mut self) {
        self.adapter = None;
        self.configured_addresses.clear();
    }

    fn open_tun_inner(&mut self) -> Result<WindowsTun, Error> {
        let has_ipv4 = self.config.addresses.iter().any(|addr| addr.is_ipv4());
        let has_ipv6 = self.config.addresses.iter().any(|addr| addr.is_ipv6());

        self.open_adapter()?;

        let mut tunnel_device = {
            let mut builder = TunnelDeviceBuilder::default();

            // When routing, the metric of a route is equal to the sum of the interface metric and
            // metric value set for the route itself. Setting the interface metric to 1 gives tunnel
            // routes the highest possible priority.
            builder.config.metric(1);
            builder.config.mtu(self.config.mtu);

            builder.config.tun_name(ADAPTER_NAME);
            builder
                .config
                .platform_config(|cfg: &mut tun::PlatformConfig| {
                    cfg.device_guid(ADAPTER_GUID);

                    // TODO: This isn't cancellable by the user or TSM, which means tunnel state machine can
                    // become unresponsive if it takes a long time for the interfaces to appear. Seems to happen
                    // for some users.
                    cfg.wait_for_interfaces(has_ipv4, has_ipv6, Duration::from_secs(30));

                    let wintun_path = self.config.resource_dir.join("wintun.dll");
                    cfg.wintun_file(wintun_path);
                });

            builder.create()?
        };

        // TODO: `tun` currently cannot handle IPv6 without using netsh,
        // so we add IPs ourselves.
        self.configure_addresses(tunnel_device.luid())?;

        tunnel_device.set_up(true)?;

        // Wait for the addresses to become usable. Until they are, they cannot be selected as
        // source addresses, so connections made this early may be routed out another interface and
        // blocked (WSAEACCES). Suspected, not confirmed: they are normally usable right away.
        // TODO: Like the interface wait above, this isn't cancellable by the user or TSM.
        let luid = NET_LUID_LH {
            Value: tunnel_device.dev.tun_luid(),
        };
        talpid_windows::net::wait_for_addresses_sync(
            luid,
            &self.config.addresses,
            WAIT_FOR_ADDRESSES_TIMEOUT,
        )
        .map_err(Error::WaitForAddresses)?;

        Ok(WindowsTun(tunnel_device))
    }

    /// Create the wintun adapter, unless one is already open.
    ///
    /// Holding on to the adapter is what keeps it from being destroyed when the tunnel device is
    /// dropped. `tun` opens the existing adapter by name rather than creating a new one.
    fn open_adapter(&mut self) -> Result<(), Error> {
        if self.adapter.is_some() {
            return Ok(());
        }

        let wintun_path = self.config.resource_dir.join("wintun.dll");
        // SAFETY: `wintun_path` refers to the wintun.dll shipped with the app.
        let wintun =
            unsafe { wintun_bindings::load_from_path(&wintun_path) }.map_err(Error::LoadWintun)?;
        let adapter = Adapter::create(&wintun, ADAPTER_NAME, ADAPTER_TYPE, Some(ADAPTER_GUID))
            .map_err(Error::CreateAdapter)?;

        self.adapter = Some(adapter);
        self.configured_addresses.clear();

        Ok(())
    }

    /// Give the tunnel device exactly the addresses in the current config.
    ///
    /// A reused adapter still has the addresses of the previous connection, and adding an address
    /// that is already configured fails, so the addresses that are no longer wanted have to be
    /// removed first.
    fn configure_addresses(&mut self, luid: NET_LUID_LH) -> Result<(), Error> {
        let addresses = self.config.addresses.clone();

        for address in &self.configured_addresses {
            if !addresses.contains(address) {
                talpid_windows::net::delete_ip_address_for_interface(luid, *address)
                    .map_err(Error::RemoveIp)?;
            }
        }

        // The adapter may also have addresses that we did not add ourselves, either because it
        // outlived a previous daemon or because Windows assigned them.
        let existing =
            talpid_windows::net::get_ip_addresses_for_interface(luid).map_err(Error::ListIps)?;

        for address in &addresses {
            if !existing.contains(address) {
                talpid_windows::net::add_ip_address_for_interface(luid, *address)
                    .map_err(Error::SetIp)?;
            }
        }

        self.configured_addresses = addresses;

        Ok(())
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
    fn luid(&self) -> NET_LUID_LH {
        NET_LUID_LH {
            Value: self.dev.tun_luid(),
        }
    }

    fn set_up(&mut self, up: bool) -> Result<(), Error> {
        self.dev.enabled(up).map_err(Error::ToggleDevice)
    }

    fn get_name(&self) -> Result<String, Error> {
        self.dev.tun_name().map_err(Error::GetDeviceName)
    }
}

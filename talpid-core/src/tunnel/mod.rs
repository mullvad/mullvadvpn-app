use std::path;
#[cfg(target_os = "android")]
use talpid_tunnel::tun_provider;
pub use talpid_tunnel::{TunnelArgs, TunnelEvent, TunnelMetadata};

use talpid_types::{
    net::{wireguard as wireguard_types, wireguard::TunnelParameters},
    tunnel::ErrorStateCause,
};
use talpid_wireguard::WireguardMonitor;

const WIREGUARD_LOG_FILENAME: &str = "wireguard.log";

/// Results from operations in the tunnel module.
pub type Result<T> = std::result::Result<T, Error>;

/// Errors that can occur in the [`TunnelMonitor`].
#[derive(thiserror::Error, Debug)]
pub enum Error {
    /// Tunnel can't have IPv6 enabled because the system has disabled IPv6 support.
    #[error("Can't enable IPv6 on tunnel interface because IPv6 is disabled")]
    EnableIpv6Error,

    /// Running on an operating system which is not supported yet.
    #[error("Tunnel type not supported on this operating system")]
    UnsupportedPlatform,

    /// Failed to rotate tunnel log file
    #[error("Failed to rotate tunnel log file")]
    RotateLogError(#[from] crate::logging::RotateLogError),

    /// There was an error listening for events from the Wireguard tunnel
    #[error("Failed while listening for events from the Wireguard tunnel")]
    WireguardTunnelMonitoringError(#[from] talpid_wireguard::Error),
}

impl From<Error> for ErrorStateCause {
    fn from(error: Error) -> ErrorStateCause {
        match error {
            Error::EnableIpv6Error => ErrorStateCause::Ipv6Unavailable,

            #[cfg(target_os = "android")]
            Error::WireguardTunnelMonitoringError(talpid_wireguard::Error::TunnelError(
                talpid_wireguard::TunnelError::SetupTunnelDevice(
                    tun_provider::Error::OtherLegacyAlwaysOnVpn,
                ),
            )) => ErrorStateCause::OtherLegacyAlwaysOnVpn,

            #[cfg(target_os = "android")]
            Error::WireguardTunnelMonitoringError(talpid_wireguard::Error::TunnelError(
                talpid_wireguard::TunnelError::SetupTunnelDevice(
                    tun_provider::Error::OtherAlwaysOnApp { app_name },
                ),
            )) => ErrorStateCause::OtherAlwaysOnApp { app_name },

            #[cfg(target_os = "android")]
            Error::WireguardTunnelMonitoringError(talpid_wireguard::Error::TunnelError(
                talpid_wireguard::TunnelError::SetupTunnelDevice(tun_provider::Error::NotPrepared),
            )) => ErrorStateCause::NotPrepared,

            #[cfg(target_os = "android")]
            Error::WireguardTunnelMonitoringError(talpid_wireguard::Error::TunnelError(
                talpid_wireguard::TunnelError::SetupTunnelDevice(
                    tun_provider::Error::InvalidDnsServers(addresses),
                ),
            )) => ErrorStateCause::InvalidDnsServers(addresses),
            #[cfg(target_os = "windows")]
            error => match error.get_tunnel_device_error() {
                Some(error) => ErrorStateCause::CreateTunnelDevice {
                    os_error: error.raw_os_error(),
                },
                None => ErrorStateCause::StartTunnelError,
            },
            #[cfg(not(target_os = "windows"))]
            _ => ErrorStateCause::StartTunnelError,
        }
    }
}

impl Error {
    /// Return whether retrying the operation that caused this error is likely to succeed.
    pub fn is_recoverable(&self) -> bool {
        match self {
            Error::WireguardTunnelMonitoringError(error) => error.is_recoverable(),
            _ => false,
        }
    }

    /// Get the inner tunnel device error, if there is one
    #[cfg(target_os = "windows")]
    pub fn get_tunnel_device_error(&self) -> Option<&std::io::Error> {
        match self {
            Error::WireguardTunnelMonitoringError(error) => error.get_tunnel_device_error(),
            _ => None,
        }
    }
}

/// Abstraction for monitoring a generic VPN tunnel.
pub struct TunnelMonitor {
    monitor: WireguardMonitor,
}

impl TunnelMonitor {
    /// Creates a new `TunnelMonitor` that connects to the given remote and notifies `on_event`
    /// on tunnel state changes.
    #[cfg_attr(any(target_os = "android", windows), allow(unused_variables))]
    pub fn start(
        tunnel_parameters: &TunnelParameters,
        log_dir: &Option<path::PathBuf>,
        args: TunnelArgs<'_>,
    ) -> Result<Self> {
        Self::ensure_ipv6_can_be_used_if_enabled(tunnel_parameters)?;
        let log_file = Self::prepare_tunnel_log_file(log_dir.as_ref())?;

        Self::start_wireguard_tunnel(tunnel_parameters, log_file, args)
    }

    fn start_wireguard_tunnel(
        params: &wireguard_types::TunnelParameters,
        log: Option<path::PathBuf>,
        args: TunnelArgs<'_>,
    ) -> Result<Self> {
        let monitor = talpid_wireguard::WireguardMonitor::start(params, args, log.as_deref())?;
        Ok(TunnelMonitor { monitor })
    }

    fn ensure_ipv6_can_be_used_if_enabled(tunnel_parameters: &TunnelParameters) -> Result<()> {
        let options = &tunnel_parameters.generic_options;
        if options.enable_ipv6 {
            if is_ipv6_enabled_in_os() {
                Ok(())
            } else {
                Err(Error::EnableIpv6Error)
            }
        } else {
            Ok(())
        }
    }

    #[cfg(not(target_os = "windows"))]
    fn prepare_tunnel_log_file(log_dir: Option<&path::PathBuf>) -> Result<Option<path::PathBuf>> {
        Ok(log_dir.map(|dir| dir.join(WIREGUARD_LOG_FILENAME)))
    }

    #[cfg(target_os = "windows")]
    fn prepare_tunnel_log_file(log_dir: Option<&path::PathBuf>) -> Result<Option<path::PathBuf>> {
        if let Some(log_dir) = log_dir {
            let filename = WIREGUARD_LOG_FILENAME;
            let tunnel_log = log_dir.join(filename);
            crate::logging::rotate_log(&tunnel_log)?;
            Ok(Some(tunnel_log))
        } else {
            Ok(None)
        }
    }

    /// Consumes the monitor and blocks until the tunnel exits or there is an error.
    pub fn wait(self) -> Result<()> {
        self.monitor.wait().map_err(Error::from)
    }
}

#[cfg(target_os = "windows")]
fn is_ipv6_enabled_in_os() -> bool {
    use winreg::{RegKey, enums::*};

    const IPV6_DISABLED_ON_TUNNELS_MASK: u32 = 0x01;

    // Check registry if IPv6 is disabled on tunnel interfaces, as documented in
    // https://support.microsoft.com/en-us/help/929852/guidance-for-configuring-ipv6-in-windows-for-advanced-users
    let globally_enabled = RegKey::predef(HKEY_LOCAL_MACHINE)
        .open_subkey(r"SYSTEM\CurrentControlSet\Services\Tcpip6\Parameters")
        .and_then(|ipv6_config| ipv6_config.get_value("DisabledComponents"))
        .map(|ipv6_disabled_bits: u32| (ipv6_disabled_bits & IPV6_DISABLED_ON_TUNNELS_MASK) == 0)
        .unwrap_or(true);

    if globally_enabled {
        true
    } else {
        log::debug!("IPv6 disabled in all tunnel interfaces");
        false
    }
}

#[cfg(not(target_os = "windows"))]
fn is_ipv6_enabled_in_os() -> bool {
    #[cfg(target_os = "linux")]
    {
        std::fs::read_to_string("/proc/sys/net/ipv6/conf/all/disable_ipv6")
            .map(|disable_ipv6| disable_ipv6.trim() == "0")
            .unwrap_or(false)
    }
    #[cfg(any(target_os = "macos", target_os = "android"))]
    {
        true
    }
}

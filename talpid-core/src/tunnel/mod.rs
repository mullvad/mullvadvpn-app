use crate::logging;
#[cfg(not(target_os = "android"))]
use futures::channel::oneshot;
use std::path;
#[cfg(not(target_os = "android"))]
use talpid_openvpn;
#[cfg(not(target_os = "android"))]
use talpid_routing::RouteManagerHandle;
pub use talpid_tunnel::{TunnelArgs, TunnelEvent, TunnelMetadata};
#[cfg(not(target_os = "android"))]
use talpid_types::net::openvpn as openvpn_types;
use talpid_types::net::{wireguard as wireguard_types, TunnelParameters};

/// A module for all WireGuard related tunnel management.
use talpid_wireguard;

const OPENVPN_LOG_FILENAME: &str = "openvpn.log";
const WIREGUARD_LOG_FILENAME: &str = "wireguard.log";

/// Set the MTU to the lowest possible whilst still allowing for IPv6 to help with wireless
/// carriers that do a lot of encapsulation.
const DEFAULT_MTU: u16 = if cfg!(target_os = "android") {
    1280
} else {
    1380
};

/// Results from operations in the tunnel module.
pub type Result<T> = std::result::Result<T, Error>;

/// Errors that can occur in the [`TunnelMonitor`].
#[derive(err_derive::Error, Debug)]
pub enum Error {
    /// Tunnel can't have IPv6 enabled because the system has disabled IPv6 support.
    #[error(display = "Can't enable IPv6 on tunnel interface because IPv6 is disabled")]
    EnableIpv6Error,

    /// Running on an operating system which is not supported yet.
    #[error(display = "Tunnel type not supported on this operating system")]
    UnsupportedPlatform,

    /// Failed to rotate tunnel log file
    #[error(display = "Failed to rotate tunnel log file")]
    RotateLogError(#[error(source)] crate::logging::RotateLogError),

    /// Failure to build Wireguard configuration.
    #[error(display = "Failed to configure Wireguard with the given parameters")]
    WireguardConfigError(#[error(source)] talpid_wireguard::config::Error),

    /// There was an error listening for events from the OpenVPN tunnel
    #[cfg(not(target_os = "android"))]
    #[error(display = "Failed while listening for events from the OpenVPN tunnel")]
    OpenVpnTunnelMonitoringError(#[error(source)] talpid_openvpn::Error),

    /// There was an error listening for events from the Wireguard tunnel
    #[error(display = "Failed while listening for events from the Wireguard tunnel")]
    WireguardTunnelMonitoringError(#[error(source)] talpid_wireguard::Error),

    /// Could not detect and assign the correct mtu
    #[error(display = "Could not detect and assign a correct MTU for the Wireguard tunnel")]
    AssignMtuError,
}

impl Error {
    /// Return whether retrying the operation that caused this error is likely to succeed.
    pub fn is_recoverable(&self) -> bool {
        match self {
            Error::WireguardTunnelMonitoringError(error) => error.is_recoverable(),
            #[cfg(not(target_os = "android"))]
            Error::OpenVpnTunnelMonitoringError(error) => error.is_recoverable(),
            _ => false,
        }
    }

    /// Get the inner tunnel device error, if there is one
    #[cfg(target_os = "windows")]
    pub fn get_tunnel_device_error(&self) -> Option<&std::io::Error> {
        match self {
            Error::WireguardTunnelMonitoringError(error) => error.get_tunnel_device_error(),
            Error::OpenVpnTunnelMonitoringError(error) => error.get_tunnel_device_error(),
            _ => None,
        }
    }
}

/// Abstraction for monitoring a generic VPN tunnel.
pub struct TunnelMonitor {
    monitor: InternalTunnelMonitor,
}

// TODO(emilsp) move most of the openvpn tunnel details to OpenVpnTunnelMonitor
impl TunnelMonitor {
    /// Creates a new `TunnelMonitor` that connects to the given remote and notifies `on_event`
    /// on tunnel state changes.
    #[cfg_attr(any(target_os = "android", windows), allow(unused_variables))]
    pub fn start<L>(
        tunnel_parameters: &mut TunnelParameters,
        log_dir: &Option<path::PathBuf>,
        args: TunnelArgs<'_, L>,
    ) -> Result<Self>
    where
        L: (Fn(TunnelEvent) -> std::pin::Pin<Box<dyn std::future::Future<Output = ()> + Send>>)
            + Send
            + Clone
            + Sync
            + 'static,
    {
        Self::ensure_ipv6_can_be_used_if_enabled(tunnel_parameters)?;
        let log_file = Self::prepare_tunnel_log_file(tunnel_parameters, log_dir)?;

        match tunnel_parameters {
            #[cfg(not(target_os = "android"))]
            TunnelParameters::OpenVpn(config) => args.runtime.block_on(Self::start_openvpn_tunnel(
                config,
                log_file,
                args.resource_dir,
                args.on_event,
                args.tunnel_close_rx,
                args.route_manager,
            )),
            #[cfg(target_os = "android")]
            TunnelParameters::OpenVpn(_) => Err(Error::UnsupportedPlatform),

            TunnelParameters::Wireguard(ref mut config) => {
                Self::start_wireguard_tunnel(config, log_file, args)
            }
        }
    }

    /// Returns a path to an executable that communicates with relay servers.
    /// Returns `None` if the executable is unknown.
    #[cfg(windows)]
    pub fn get_relay_client(
        resource_dir: &path::Path,
        params: &TunnelParameters,
    ) -> Option<path::PathBuf> {
        use talpid_types::net::proxy::CustomProxy;

        let resource_dir = resource_dir.to_path_buf();
        match params {
            TunnelParameters::OpenVpn(params) => match &params.proxy {
                Some(CustomProxy::Shadowsocks(_)) => Some(std::env::current_exe().unwrap()),
                Some(CustomProxy::Socks5Local(_)) => None,
                Some(CustomProxy::Socks5Remote(_)) | None => Some(resource_dir.join("openvpn.exe")),
            },
            _ => Some(std::env::current_exe().unwrap()),
        }
    }

    fn start_wireguard_tunnel<L>(
        #[cfg(not(any(target_os = "linux", target_os = "windows")))]
        params: &wireguard_types::TunnelParameters,
        #[cfg(any(target_os = "linux", target_os = "windows"))]
        params: &mut wireguard_types::TunnelParameters,
        log: Option<path::PathBuf>,
        args: TunnelArgs<'_, L>,
    ) -> Result<Self>
    where
        L: (Fn(TunnelEvent) -> std::pin::Pin<Box<dyn std::future::Future<Output = ()> + Send>>)
            + Send
            + Sync
            + Clone
            + 'static,
    {
        let default_mtu = DEFAULT_MTU;

        #[cfg(any(target_os = "linux", target_os = "windows"))]
        // Detects the MTU of the device and sets the default tunnel MTU to that minus headers and
        // the safety margin
        let default_mtu = args
            .runtime
            .block_on(
                args.route_manager
                    .get_mtu_for_route(params.connection.peer.endpoint.ip()),
            )
            .map(|mtu| Self::clamp_mtu(params, mtu))
            .unwrap_or(default_mtu);

        #[cfg(any(target_os = "linux", windows))]
        let detect_mtu = params.options.mtu.is_none();

        let config = talpid_wireguard::config::Config::from_parameters(params, default_mtu)?;
        let monitor = talpid_wireguard::WireguardMonitor::start(
            config,
            params.options.quantum_resistant,
            #[cfg(any(target_os = "linux", windows))]
            detect_mtu,
            log.as_deref(),
            args,
        )?;
        Ok(TunnelMonitor {
            monitor: InternalTunnelMonitor::Wireguard(monitor),
        })
    }

    /// Calculates and appropriate tunnel MTU based on the given peer MTU minus header sizes
    #[cfg(any(target_os = "linux", target_os = "windows"))]
    fn clamp_mtu(params: &wireguard_types::TunnelParameters, peer_mtu: u16) -> u16 {
        use talpid_tunnel::{
            IPV4_HEADER_SIZE, IPV6_HEADER_SIZE, MIN_IPV4_MTU, MIN_IPV6_MTU, WIREGUARD_HEADER_SIZE,
        };
        // Some users experience fragmentation issues even when we take the interface MTU and
        // subtract the header sizes. This is likely due to some program that they use which does
        // not change the interface MTU but adds its own header onto the outgoing packets. For this
        // reason we subtract some extra bytes from our MTU in order to give other programs some
        // safety margin.
        const MTU_SAFETY_MARGIN: u16 = 60;

        let total_header_size = WIREGUARD_HEADER_SIZE
            + match params.connection.peer.endpoint.is_ipv6() {
                false => IPV4_HEADER_SIZE,
                true => IPV6_HEADER_SIZE,
            };

        // The largest peer MTU that we allow
        let max_peer_mtu: u16 = 1500 - MTU_SAFETY_MARGIN - total_header_size;

        let min_mtu = match params.generic_options.enable_ipv6 {
            false => MIN_IPV4_MTU,
            true => MIN_IPV6_MTU,
        };

        peer_mtu
            .saturating_sub(total_header_size)
            .clamp(min_mtu, max_peer_mtu)
    }

    #[cfg(not(target_os = "android"))]
    async fn start_openvpn_tunnel<L>(
        config: &openvpn_types::TunnelParameters,
        log: Option<path::PathBuf>,
        resource_dir: &path::Path,
        on_event: L,
        tunnel_close_rx: oneshot::Receiver<()>,
        route_manager: RouteManagerHandle,
    ) -> Result<Self>
    where
        L: (Fn(TunnelEvent) -> std::pin::Pin<Box<dyn std::future::Future<Output = ()> + Send>>)
            + Send
            + Sync
            + 'static,
    {
        let monitor = talpid_openvpn::OpenVpnMonitor::start(
            on_event,
            config,
            log,
            resource_dir,
            route_manager,
        )
        .await?;

        let close_handle = monitor.close_handle();
        tokio::spawn(async move {
            if tunnel_close_rx.await.is_ok() {
                close_handle.close();
            }
        });

        Ok(TunnelMonitor {
            monitor: InternalTunnelMonitor::OpenVpn(monitor),
        })
    }

    fn ensure_ipv6_can_be_used_if_enabled(tunnel_parameters: &TunnelParameters) -> Result<()> {
        let options = tunnel_parameters.get_generic_options();
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
    fn prepare_tunnel_log_file(
        parameters: &TunnelParameters,
        log_dir: &Option<path::PathBuf>,
    ) -> Result<Option<path::PathBuf>> {
        if let Some(ref log_dir) = log_dir {
            match parameters {
                TunnelParameters::OpenVpn(_) => {
                    let tunnel_log = log_dir.join(OPENVPN_LOG_FILENAME);
                    logging::rotate_log(&tunnel_log)?;
                    Ok(Some(tunnel_log))
                }
                TunnelParameters::Wireguard(_) => Ok(Some(log_dir.join(WIREGUARD_LOG_FILENAME))),
            }
        } else {
            Ok(None)
        }
    }

    #[cfg(target_os = "windows")]
    fn prepare_tunnel_log_file(
        parameters: &TunnelParameters,
        log_dir: &Option<path::PathBuf>,
    ) -> Result<Option<path::PathBuf>> {
        if let Some(ref log_dir) = log_dir {
            let filename = match parameters {
                TunnelParameters::OpenVpn(_) => OPENVPN_LOG_FILENAME,
                TunnelParameters::Wireguard(_) => WIREGUARD_LOG_FILENAME,
            };
            let tunnel_log = log_dir.join(filename);
            logging::rotate_log(&tunnel_log)?;
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

enum InternalTunnelMonitor {
    #[cfg(not(target_os = "android"))]
    OpenVpn(talpid_openvpn::OpenVpnMonitor),
    Wireguard(talpid_wireguard::WireguardMonitor),
}

impl InternalTunnelMonitor {
    fn wait(self) -> Result<()> {
        #[cfg(not(target_os = "android"))]
        let handle = tokio::runtime::Handle::current();

        match self {
            #[cfg(not(target_os = "android"))]
            InternalTunnelMonitor::OpenVpn(tun) => handle.block_on(tun.wait())?,
            InternalTunnelMonitor::Wireguard(tun) => tun.wait()?,
        }

        Ok(())
    }
}

#[cfg(target_os = "windows")]
fn is_ipv6_enabled_in_os() -> bool {
    use winreg::{enums::*, RegKey};

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

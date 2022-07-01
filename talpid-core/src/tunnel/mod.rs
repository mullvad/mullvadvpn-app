use self::tun_provider::TunProvider;
use crate::{logging, routing::RouteManagerHandle};
use futures::{channel::oneshot, future::BoxFuture};
use std::{
    net::{IpAddr, Ipv4Addr, Ipv6Addr},
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
};
#[cfg(not(target_os = "android"))]
use talpid_types::net::openvpn as openvpn_types;
use talpid_types::net::{wireguard as wireguard_types, AllowedTunnelTraffic, TunnelParameters};

#[cfg(target_os = "android")]
pub use self::tun_provider::TunConfig;

#[cfg(target_os = "windows")]
mod windows;

/// A module for all OpenVPN related tunnel management.
#[cfg(not(target_os = "android"))]
pub mod openvpn;

/// A module for all WireGuard related tunnel management.
pub mod wireguard;

/// A module for low level platform specific tunnel device management.
pub(crate) mod tun_provider;

const OPENVPN_LOG_FILENAME: &str = "openvpn.log";
const WIREGUARD_LOG_FILENAME: &str = "wireguard.log";

/// Results from operations in the tunnel module.
pub type Result<T> = std::result::Result<T, Error>;

/// Errors that can occur in the [`TunnelMonitor`].
#[derive(err_derive::Error, Debug)]
pub enum Error {
    /// Tunnel can't have IPv6 enabled because the system has disabled IPv6 support.
    #[error(display = "Can't enable IPv6 on tunnel interface because IPv6 is disabled")]
    EnableIpv6Error,

    /// Failure in Windows syscall.
    #[cfg(windows)]
    #[error(display = "Failure in Windows syscall")]
    WinnetError(#[error(source)] crate::winnet::Error),

    /// Running on an operating system which is not supported yet.
    #[error(display = "Tunnel type not supported on this operating system")]
    UnsupportedPlatform,

    /// Failed to rotate tunnel log file
    #[error(display = "Failed to rotate tunnel log file")]
    RotateLogError(#[error(source)] crate::logging::RotateLogError),

    /// Failure to build Wireguard configuration.
    #[error(display = "Failed to configure Wireguard with the given parameters")]
    WireguardConfigError(#[error(source)] self::wireguard::config::Error),

    /// There was an error listening for events from the OpenVPN tunnel
    #[cfg(not(target_os = "android"))]
    #[error(display = "Failed while listening for events from the OpenVPN tunnel")]
    OpenVpnTunnelMonitoringError(#[error(source)] openvpn::Error),

    /// There was an error listening for events from the Wireguard tunnel
    #[error(display = "Failed while listening for events from the Wireguard tunnel")]
    WireguardTunnelMonitoringError(#[error(source)] wireguard::Error),

    /// Could not detect and assign the correct mtu
    #[error(display = "Could not detect and assign a correct MTU for the Wireguard tunnel")]
    AssignMtuError,
}

/// Possible events from the VPN tunnel and the child process managing it.
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum TunnelEvent {
    /// Sent when the tunnel fails to connect due to an authentication error.
    AuthFailed(Option<String>),
    /// Sent when the tunnel interface has been created, before routes are set up.
    InterfaceUp(TunnelMetadata, AllowedTunnelTraffic),
    /// Sent when the tunnel comes up and is ready for traffic.
    Up(TunnelMetadata),
    /// Sent when the tunnel goes down.
    Down,
}

/// Information about a VPN tunnel.
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct TunnelMetadata {
    /// The name of the device which the tunnel is running on.
    pub interface: String,
    /// The local IPs on the tunnel interface.
    pub ips: Vec<IpAddr>,
    /// The IP to the default gateway on the tunnel interface.
    pub ipv4_gateway: Ipv4Addr,
    /// The IP to the IPv6 default gateway on the tunnel interface.
    pub ipv6_gateway: Option<Ipv6Addr>,
}

/// Abstraction for monitoring a generic VPN tunnel.
pub struct TunnelMonitor {
    monitor: InternalTunnelMonitor,
}

/// Arguments for creating a tunnel.
pub struct TunnelArgs<'a, L>
where
    // L: (Fn(TunnelEvent) -> std::pin::Pin<Box<dyn std::future::Future<Output = ()> + Send>>)
    L: (Fn(TunnelEvent) -> BoxFuture<'static, ()>) + Send + Clone + Sync + 'static,
{
    /// Toktio runtime handle.
    pub runtime: tokio::runtime::Handle,
    /// Resource directory path.
    pub resource_dir: &'a Path,
    /// Callback function called when an event happens.
    pub on_event: L,
    /// Receiver oneshot channel for closing the tunnel.
    pub tunnel_close_rx: oneshot::Receiver<()>,
    /// Mutex to tunnel provider.
    pub tun_provider: Arc<Mutex<TunProvider>>,
    /// Connection retry attempts.
    pub retry_attempt: u32,
    /// Route manager handle.
    pub route_manager: RouteManagerHandle,
}

// TODO(emilsp) move most of the openvpn tunnel details to OpenVpnTunnelMonitor
impl TunnelMonitor {
    /// Creates a new `TunnelMonitor` that connects to the given remote and notifies `on_event`
    /// on tunnel state changes.
    #[cfg_attr(any(target_os = "android", windows), allow(unused_variables))]
    pub fn start<L>(
        tunnel_parameters: &mut TunnelParameters,
        log_dir: &Option<PathBuf>,
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
                #[cfg(target_os = "linux")]
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
    #[cfg(windows)]
    pub fn get_relay_client(resource_dir: &Path, params: &TunnelParameters) -> PathBuf {
        let resource_dir = resource_dir.to_path_buf();
        let process_string = match params {
            TunnelParameters::OpenVpn(params) => {
                if let Some(proxy) = &params.proxy {
                    match proxy {
                        openvpn_types::ProxySettings::Shadowsocks(..) => {
                            return std::env::current_exe().unwrap()
                        }
                        _ => "openvpn.exe",
                    }
                } else {
                    "openvpn.exe"
                }
            }
            _ => return std::env::current_exe().unwrap(),
        };
        resource_dir.join(process_string)
    }

    fn start_wireguard_tunnel<L>(
        params: &mut wireguard_types::TunnelParameters,
        log: Option<PathBuf>,
        args: TunnelArgs<'_, L>,
    ) -> Result<Self>
    where
        L: (Fn(TunnelEvent) -> std::pin::Pin<Box<dyn std::future::Future<Output = ()> + Send>>)
            + Send
            + Sync
            + Clone
            + 'static,
    {
        #[cfg(target_os = "linux")]
        args.runtime
            .block_on(Self::assign_mtu(&args.route_manager, params));
        let config = wireguard::config::Config::from_parameters(params)?;
        let monitor = wireguard::WireguardMonitor::start(
            config,
            if params.options.use_pq_safe_psk {
                Some(
                    params
                        .connection
                        .exit_peer
                        .as_ref()
                        .map(|peer| peer.public_key.clone())
                        .unwrap_or_else(|| params.connection.peer.public_key.clone()),
                )
            } else {
                None
            },
            log.as_deref(),
            args,
        )?;
        Ok(TunnelMonitor {
            monitor: InternalTunnelMonitor::Wireguard(monitor),
        })
    }

    #[cfg(target_os = "linux")]
    fn set_mtu(params: &mut wireguard_types::TunnelParameters, peer_mtu: u16) {
        const IPV4_HEADER_SIZE: u16 = 20;
        const IPV6_HEADER_SIZE: u16 = 40;
        const WIREGUARD_HEADER_SIZE: u16 = 40;
        let total_header_size = WIREGUARD_HEADER_SIZE
            + match params.connection.peer.endpoint.is_ipv6() {
                false => IPV4_HEADER_SIZE,
                true => IPV6_HEADER_SIZE,
            };
        // The largest peer MTU that we allow
        const MAX_PEER_MTU: u16 = 1500;
        // The minimum allowed MTU size for our tunnel in IPv6 is 1280
        const MIN_IPV4_MTU: u16 = 576;
        const MIN_IPV6_MTU: u16 = 1280;
        let min_mtu = match params.generic_options.enable_ipv6 {
            false => MIN_IPV4_MTU,
            true => MIN_IPV6_MTU,
        };
        let tunnel_mtu = peer_mtu
            .saturating_sub(total_header_size)
            .clamp(min_mtu, MAX_PEER_MTU - total_header_size);
        params.options.mtu = Some(tunnel_mtu);
    }

    #[cfg(target_os = "linux")]
    async fn assign_mtu(
        route_manager: &RouteManagerHandle,
        params: &mut wireguard_types::TunnelParameters,
    ) {
        // Only calculate the mtu automatically if the user has not set any
        if params.options.mtu.is_none() {
            if let Ok(mtu) = route_manager
                .get_mtu_for_route(params.connection.peer.endpoint.ip())
                .await
            {
                Self::set_mtu(params, mtu);
            }
        }
    }

    #[cfg(not(target_os = "android"))]
    async fn start_openvpn_tunnel<L>(
        config: &openvpn_types::TunnelParameters,
        log: Option<PathBuf>,
        resource_dir: &Path,
        on_event: L,
        tunnel_close_rx: oneshot::Receiver<()>,
        #[cfg(target_os = "linux")] route_manager: RouteManagerHandle,
    ) -> Result<Self>
    where
        L: (Fn(TunnelEvent) -> std::pin::Pin<Box<dyn std::future::Future<Output = ()> + Send>>)
            + Send
            + Sync
            + 'static,
    {
        let monitor = openvpn::OpenVpnMonitor::start(
            on_event,
            config,
            log,
            resource_dir,
            tunnel_close_rx,
            #[cfg(target_os = "linux")]
            route_manager,
        )
        .await?;
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
        log_dir: &Option<PathBuf>,
    ) -> Result<Option<PathBuf>> {
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
        log_dir: &Option<PathBuf>,
    ) -> Result<Option<PathBuf>> {
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
    OpenVpn(openvpn::OpenVpnMonitor),
    Wireguard(wireguard::WireguardMonitor),
}

impl InternalTunnelMonitor {
    fn wait(self) -> Result<()> {
        match self {
            #[cfg(not(target_os = "android"))]
            InternalTunnelMonitor::OpenVpn(tun) => tun.wait()?,
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
        .open_subkey(r#"SYSTEM\CurrentControlSet\Services\Tcpip6\Parameters"#)
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

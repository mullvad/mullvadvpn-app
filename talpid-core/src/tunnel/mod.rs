use crate::logging;
use std::{
    collections::HashMap,
    ffi::OsString,
    io,
    net::{IpAddr, Ipv4Addr, Ipv6Addr},
    path::{Path, PathBuf},
};
#[cfg(unix)]
use talpid_types::net::wireguard as wireguard_types;
use talpid_types::net::{openvpn as openvpn_types, GenericTunnelOptions, TunnelParameters};

/// A module for all OpenVPN related tunnel management.
pub mod openvpn;

#[cfg(unix)]
mod wireguard;

const OPENVPN_LOG_FILENAME: &str = "openvpn.log";
const WIREGUARD_LOG_FILENAME: &str = "wireguard.log";


error_chain! {
    errors {
        /// Tunnel can't have IPv6 enabled because the system has disabled IPv6 support.
        EnableIpv6Error {
            description("Can't enable IPv6 on tunnel interface because IPv6 is disabled")
        }
        /// Running on an operating system which is not supported yet.
        UnsupportedPlatform {
            description("Tunnel type not supported on this operating system")
        }
        /// Failed to rotate tunnel log file
        RotateLogError {
            description("Failed to rotate tunnel log file")
        }
        /// Failure to build Wireguard configuration.
        WireguardConfigError {
            description("Failed to configure Wireguard with the given parameters")
        }
    }

    links {
        OpenVpnTunnelMonitoringError(openvpn::Error, openvpn::ErrorKind)
        /// There was an error listening for events from the OpenVPN tunnel
        ;
        WirguardTunnelMonitoringError(wireguard::Error, wireguard::ErrorKind)
        /// There was an error listening for events from the OpenVPN tunnel
        #[cfg(unix)]
        ;
    }
}


/// Possible events from the VPN tunnel and the child process managing it.
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum TunnelEvent {
    /// Sent when the tunnel fails to connect due to an authentication error.
    AuthFailed(Option<String>),
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

impl TunnelEvent {
    /// Converts an `openvpn_plugin::EventType` to a `TunnelEvent`.
    /// Returns `None` if there is no corresponding `TunnelEvent`.
    fn from_openvpn_event(
        event: openvpn_plugin::EventType,
        env: &HashMap<String, String>,
    ) -> Option<TunnelEvent> {
        match event {
            openvpn_plugin::EventType::AuthFailed => {
                let reason = env.get("auth_failed_reason").cloned();
                Some(TunnelEvent::AuthFailed(reason))
            }
            openvpn_plugin::EventType::RouteUp => {
                let interface = env
                    .get("dev")
                    .expect("No \"dev\" in tunnel up event")
                    .to_owned();
                let ips = vec![env
                    .get("ifconfig_local")
                    .expect("No \"ifconfig_local\" in tunnel up event")
                    .parse()
                    .expect("Tunnel IP not in valid format")];
                let ipv4_gateway = env
                    .get("route_vpn_gateway")
                    .expect("No \"route_vpn_gateway\" in tunnel up event")
                    .parse()
                    .expect("Tunnel gateway IP not in valid format");
                let ipv6_gateway = env.get("route_ipv6_gateway_1").map(|v6_str| {
                    v6_str
                        .parse()
                        .expect("V6 Tunnel gateway IP not in valid format")
                });
                Some(TunnelEvent::Up(TunnelMetadata {
                    interface,
                    ips,
                    ipv4_gateway,
                    ipv6_gateway,
                }))
            }
            openvpn_plugin::EventType::RoutePredown => Some(TunnelEvent::Down),
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
    pub fn start<L>(
        tunnel_parameters: &TunnelParameters,
        tunnel_alias: Option<OsString>,
        log_dir: &Option<PathBuf>,
        resource_dir: &Path,
        on_event: L,
    ) -> Result<Self>
    where
        L: Fn(TunnelEvent) + Send + Sync + 'static,
    {
        Self::ensure_ipv6_can_be_used_if_enabled(&tunnel_parameters.get_generic_options())?;
        let log_file = Self::prepare_tunnel_log_file(&tunnel_parameters, log_dir)?;

        match tunnel_parameters {
            TunnelParameters::OpenVpn(config) => {
                Self::start_openvpn_tunnel(&config, tunnel_alias, log_file, resource_dir, on_event)
            }
            #[cfg(unix)]
            TunnelParameters::Wireguard(config) => {
                Self::start_wireguard_tunnel(&config, log_file, on_event)
            }
            #[cfg(windows)]
            TunnelParameters::Wireguard(_) => bail!(ErrorKind::UnsupportedPlatform),
        }
    }

    #[cfg(unix)]
    fn start_wireguard_tunnel<L>(
        params: &wireguard_types::TunnelParameters,
        log: Option<PathBuf>,
        on_event: L,
    ) -> Result<Self>
    where
        L: Fn(TunnelEvent) + Send + Sync + 'static,
    {
        let config = wireguard::config::Config::from_parameters(&params)
            .chain_err(|| ErrorKind::WireguardConfigError)?;
        let monitor = wireguard::WireguardMonitor::start(
            &config,
            log.as_ref().map(|p| p.as_path()),
            on_event,
        )?;
        Ok(TunnelMonitor {
            monitor: InternalTunnelMonitor::Wireguard(monitor),
        })
    }

    fn start_openvpn_tunnel<L>(
        config: &openvpn_types::TunnelParameters,
        tunnel_alias: Option<OsString>,
        log: Option<PathBuf>,
        resource_dir: &Path,
        on_event: L,
    ) -> Result<Self>
    where
        L: Fn(TunnelEvent) + Send + Sync + 'static,
    {
        let monitor =
            openvpn::OpenVpnMonitor::start(on_event, config, tunnel_alias, log, resource_dir)?;
        Ok(TunnelMonitor {
            monitor: InternalTunnelMonitor::OpenVpn(monitor),
        })
    }

    fn ensure_ipv6_can_be_used_if_enabled(tunnel_options: &GenericTunnelOptions) -> Result<()> {
        if tunnel_options.enable_ipv6 && !is_ipv6_enabled_in_os() {
            bail!(ErrorKind::EnableIpv6Error);
        } else {
            Ok(())
        }
    }

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
            logging::rotate_log(&tunnel_log).chain_err(|| ErrorKind::RotateLogError)?;
            Ok(Some(tunnel_log))
        } else {
            Ok(None)
        }
    }


    /// Creates a handle to this monitor, allowing the tunnel to be closed while some other
    /// thread
    /// is blocked in `wait`.
    pub fn close_handle(&self) -> CloseHandle {
        self.monitor.close_handle()
    }

    /// Consumes the monitor and blocks until the tunnel exits or there is an error.
    pub fn wait(self) -> Result<()> {
        self.monitor.wait().map_err(Error::from)
    }
}


/// A handle to a `TunnelMonitor`
pub enum CloseHandle {
    /// OpenVpn close handle
    OpenVpn(openvpn::OpenVpnCloseHandle),
    #[cfg(unix)]
    /// Wireguard close handle
    Wireguard(wireguard::CloseHandle),
}

impl CloseHandle {
    /// Closes the underlying tunnel, making the `TunnelMonitor::wait` method return.
    pub fn close(self) -> io::Result<()> {
        match self {
            CloseHandle::OpenVpn(handle) => handle.close(),
            #[cfg(unix)]
            CloseHandle::Wireguard(mut handle) => {
                handle.close();
                Ok(())
            }
        }
    }
}

enum InternalTunnelMonitor {
    OpenVpn(openvpn::OpenVpnMonitor),
    #[cfg(unix)]
    Wireguard(wireguard::WireguardMonitor),
}

impl InternalTunnelMonitor {
    fn close_handle(&self) -> CloseHandle {
        match self {
            InternalTunnelMonitor::OpenVpn(tun) => CloseHandle::OpenVpn(tun.close_handle()),
            #[cfg(unix)]
            InternalTunnelMonitor::Wireguard(tun) => CloseHandle::Wireguard(tun.close_handle()),
        }
    }

    fn wait(self) -> Result<()> {
        match self {
            InternalTunnelMonitor::OpenVpn(tun) => tun.wait()?,
            #[cfg(unix)]
            InternalTunnelMonitor::Wireguard(tun) => tun.wait()?,
        }

        Ok(())
    }
}


fn is_ipv6_enabled_in_os() -> bool {
    #[cfg(windows)]
    {
        use winreg::{enums::*, RegKey};

        const IPV6_DISABLED_ON_TUNNELS_MASK: u32 = 0x01;

        // Check registry if IPv6 is disabled on tunnel interfaces, as documented in
        // https://support.microsoft.com/en-us/help/929852/guidance-for-configuring-ipv6-in-windows-for-advanced-users
        let globally_enabled = RegKey::predef(HKEY_LOCAL_MACHINE)
            .open_subkey(r#"SYSTEM\CurrentControlSet\Services\Tcpip6\Parameters"#)
            .and_then(|ipv6_config| ipv6_config.get_value("DisabledComponents"))
            .map(|ipv6_disabled_bits: u32| {
                (ipv6_disabled_bits & IPV6_DISABLED_ON_TUNNELS_MASK) == 0
            })
            .unwrap_or(true);
        let enabled_on_tap = crate::winnet::get_tap_interface_ipv6_status().unwrap_or(false);

        if !globally_enabled {
            log::debug!("IPv6 disabled in tunnel interfaces");
        }
        if !enabled_on_tap {
            log::debug!("IPv6 disabled in TAP adapter");
        }

        globally_enabled && enabled_on_tap
    }
    #[cfg(target_os = "linux")]
    {
        std::fs::read_to_string("/proc/sys/net/ipv6/conf/all/disable_ipv6")
            .map(|disable_ipv6| disable_ipv6.trim() == "0")
            .unwrap_or(false)
    }
    #[cfg(target_os = "macos")]
    {
        true
    }
}

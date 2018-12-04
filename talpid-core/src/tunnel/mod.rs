use crate::{mktemp, process::openvpn::OpenVpnCommand};

use std::{
    collections::HashMap,
    ffi::OsString,
    fs,
    io::{self, Write},
    net::IpAddr,
    path::{Path, PathBuf},
    result::Result as StdResult,
};

#[cfg(target_os = "linux")]
use failure::ResultExt as FailureResultExt;
#[cfg(target_os = "linux")]
use which;

use talpid_types::net::{
    Endpoint, OpenVpnProxySettings, TunnelEndpoint, TunnelEndpointData, TunnelOptions,
};

/// A module for all OpenVPN related tunnel management.
pub mod openvpn;

#[cfg(unix)]
mod wireguard;

use self::openvpn::{OpenVpnCloseHandle, OpenVpnMonitor};

#[cfg(target_os = "macos")]
const OPENVPN_PLUGIN_FILENAME: &str = "libtalpid_openvpn_plugin.dylib";
#[cfg(target_os = "linux")]
const OPENVPN_PLUGIN_FILENAME: &str = "libtalpid_openvpn_plugin.so";
#[cfg(windows)]
const OPENVPN_PLUGIN_FILENAME: &str = "talpid_openvpn_plugin.dll";

#[cfg(unix)]
const OPENVPN_BIN_FILENAME: &str = "openvpn";
#[cfg(windows)]
const OPENVPN_BIN_FILENAME: &str = "openvpn.exe";

error_chain! {
    errors {
        /// An error indicating there was an error listening for events from the VPN tunnel.
        TunnelMonitoringError {
            description("Error while setting up or processing events from the VPN tunnel")
        }
        /// The OpenVPN binary was not found.
        OpenVpnNotFound(path: PathBuf) {
            description("No OpenVPN binary found")
            display("No OpenVPN binary found at {}", path.display())
        }
        /// The IP routing program was not found.
        #[cfg(target_os = "linux")]
        IpRouteNotFound {
            description("The IP routing program `ip` was not found.")
        }
        /// The OpenVPN plugin was not found.
        PluginNotFound(path: PathBuf) {
            description("No OpenVPN plugin found")
            display("No OpenVPN plugin found at {}", path.display())
        }
        /// There was an error when writing authentication credentials to temporary file.
        CredentialsWriteError {
            description("Error while writing credentials to temporary file")
        }
        /// Tunnel can't have IPv6 enabled because the system has disabled IPv6 support.
        EnableIpv6Error {
            description("Can't enable IPv6 on tunnel interface because IPv6 is disabled")
        }
        /// Running on an operating system which is not supported yet.
        UnsupportedPlatform {
            description("Tunnel type not supported on this operating system")
        }
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
    /// The local IP on the tunnel interface.
    pub ip: Vec<IpAddr>,
    /// The IP to the default gateway on the tunnel interface.
    pub gateway: IpAddr,
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
                let ip = vec![env
                    .get("ifconfig_local")
                    .expect("No \"ifconfig_local\" in tunnel up event")
                    .parse()
                    .expect("Tunnel IP not in valid format")];
                let gateway = env
                    .get("route_vpn_gateway")
                    .expect("No \"route_vpn_gateway\" in tunnel up event")
                    .parse()
                    .expect("Tunnel gateway IP not in valid format");
                Some(TunnelEvent::Up(TunnelMetadata {
                    interface,
                    ip,
                    gateway,
                }))
            }
            openvpn_plugin::EventType::RoutePredown => Some(TunnelEvent::Down),
            _ => None,
        }
    }
}

enum Tunnel {
    OpenVpn(OpenVpnMonitor),
    #[cfg(unix)]
    Wireguard(wireguard::WireguardMonitor),
}

impl Tunnel {
    fn close_handle(&self) -> CloseHandle {
        match self {
            Tunnel::OpenVpn(tun) => CloseHandle::OpenVpn(tun.close_handle()),
            #[cfg(unix)]
            Tunnel::Wireguard(tun) => CloseHandle::Wireguard(tun.close_handle()),
        }
    }

    fn wait(self) -> Result<()> {
        match self {
            Tunnel::OpenVpn(tun) => tun.wait().chain_err(|| ErrorKind::TunnelMonitoringError),
            #[cfg(unix)]
            Tunnel::Wireguard(tun) => tun.wait().chain_err(|| ErrorKind::TunnelMonitoringError),
        }
    }
}


/// Abstraction for monitoring a generic VPN tunnel.
pub struct TunnelMonitor {
    monitor: Tunnel,
}

// TODO(emilsp) move most of the openvpn tunnel details to OpenVpnTunnelMonitor
impl TunnelMonitor {
    /// Creates a new `TunnelMonitor` that connects to the given remote and notifies `on_event`
    /// on tunnel state changes.
    pub fn start<L>(
        tunnel_endpoint: TunnelEndpoint,
        tunnel_options: &TunnelOptions,
        tunnel_alias: Option<OsString>,
        username: &str,
        log: Option<&Path>,
        resource_dir: &Path,
        on_event: L,
    ) -> Result<Self>
    where
        L: Fn(TunnelEvent) + Send + Sync + 'static,
    {
        Self::ensure_ipv6_can_be_used_if_enabled(tunnel_options)?;
        match &tunnel_endpoint.tunnel {
            TunnelEndpointData::OpenVpn(_) => Self::start_openvpn_tunnel(
                tunnel_endpoint,
                tunnel_options,
                tunnel_alias,
                username,
                log,
                resource_dir,
                on_event,
            ),
            #[cfg(unix)]
            TunnelEndpointData::Wireguard(_) => {
                Self::start_wireguard_tunnel(tunnel_endpoint, tunnel_options, log, on_event)
            }
            #[cfg(windows)]
            TunnelEndpointData::Wireguard(_) => bail!(ErrorKind::UnsupportedPlatform),
        }
    }

    #[cfg(unix)]
    fn start_wireguard_tunnel<L>(
        tunnel_endpoint: TunnelEndpoint,
        tunnel_options: &TunnelOptions,
        log: Option<&Path>,
        on_event: L,
    ) -> Result<Self>
    where
        L: Fn(TunnelEvent) + Send + Sync + 'static,
    {
        let TunnelEndpoint { address, tunnel } = tunnel_endpoint;
        let data = match tunnel {
            TunnelEndpointData::Wireguard(data) => data,
            _ => unreachable!("expected wireguard endpoint data"),
        };

        let monitor =
            wireguard::WireguardMonitor::start(address, data, tunnel_options, log, on_event)
                .chain_err(|| ErrorKind::TunnelMonitoringError)?;
        Ok(TunnelMonitor {
            monitor: Tunnel::Wireguard(monitor),
        })
    }

    fn start_openvpn_tunnel<L>(
        tunnel_endpoint: TunnelEndpoint,
        tunnel_options: &TunnelOptions,
        tunnel_alias: Option<OsString>,
        username: &str,
        log: Option<&Path>,
        resource_dir: &Path,
        on_event: L,
    ) -> Result<Self>
    where
        L: Fn(TunnelEvent) + Send + Sync + 'static,
    {
        let user_pass_file = Self::create_credentials_file(username, "-")
            .chain_err(|| ErrorKind::CredentialsWriteError)?;

        let proxy_auth_file = Self::create_proxy_auth_file(&tunnel_options.openvpn.proxy)
            .chain_err(|| ErrorKind::CredentialsWriteError)?;

        let cmd = Self::create_openvpn_cmd(
            tunnel_endpoint.to_endpoint(),
            tunnel_alias,
            &tunnel_options,
            user_pass_file.as_ref(),
            match proxy_auth_file {
                Some(ref file) => Some(file.as_ref()),
                _ => None,
            },
            log,
            resource_dir,
        )?;

        let user_pass_file_path = user_pass_file.to_path_buf();

        let proxy_auth_file_path = match proxy_auth_file {
            Some(ref file) => Some(file.to_path_buf()),
            _ => None,
        };

        let on_openvpn_event = move |event, env| {
            if event == openvpn_plugin::EventType::RouteUp {
                // The user-pass file has been read. Try to delete it early.
                let _ = fs::remove_file(&user_pass_file_path);

                // The proxy auth file has been read. Try to delete it early.
                if let Some(ref file_path) = &proxy_auth_file_path {
                    let _ = fs::remove_file(file_path);
                }
            }
            match TunnelEvent::from_openvpn_event(event, &env) {
                Some(tunnel_event) => on_event(tunnel_event),
                None => log::debug!("Ignoring OpenVpnEvent {:?}", event),
            }
        };

        let monitor = openvpn::OpenVpnMonitor::start(
            cmd,
            on_openvpn_event,
            Self::get_plugin_path(resource_dir)?,
            user_pass_file,
            proxy_auth_file,
        )
        .chain_err(|| ErrorKind::TunnelMonitoringError)?;
        Ok(TunnelMonitor {
            monitor: Tunnel::OpenVpn(monitor),
        })
    }

    fn ensure_ipv6_can_be_used_if_enabled(tunnel_options: &TunnelOptions) -> Result<()> {
        if tunnel_options.enable_ipv6 && !is_ipv6_enabled_in_os() {
            bail!(ErrorKind::EnableIpv6Error);
        } else {
            Ok(())
        }
    }

    fn create_openvpn_cmd(
        remote: Endpoint,
        tunnel_alias: Option<OsString>,
        options: &TunnelOptions,
        user_pass_file: &Path,
        proxy_auth_file: Option<&Path>,
        log: Option<&Path>,
        resource_dir: &Path,
    ) -> Result<OpenVpnCommand> {
        let mut cmd = OpenVpnCommand::new(Self::get_openvpn_bin(resource_dir)?);
        if let Some(config) = Self::get_config_path(resource_dir) {
            cmd.config(config);
        }
        #[cfg(target_os = "linux")]
        cmd.iproute_bin(
            which::which("ip")
                .compat()
                .chain_err(|| ErrorKind::IpRouteNotFound)?,
        );
        cmd.remote(remote)
            .user_pass(user_pass_file)
            .tunnel_options(&options.openvpn)
            .enable_ipv6(options.enable_ipv6)
            .tunnel_alias(tunnel_alias)
            .ca(resource_dir.join("ca.crt"));
        if let Some(log) = log {
            cmd.log(log);
        }
        if let Some(proxy_auth_file) = proxy_auth_file {
            cmd.proxy_auth(proxy_auth_file);
        }

        Ok(cmd)
    }

    fn get_openvpn_bin(resource_dir: &Path) -> Result<PathBuf> {
        let path = resource_dir.join(OPENVPN_BIN_FILENAME);
        if path.exists() {
            log::trace!("Using OpenVPN at {}", path.display());
            Ok(path)
        } else {
            bail!(ErrorKind::OpenVpnNotFound(path));
        }
    }

    fn get_plugin_path(resource_dir: &Path) -> Result<PathBuf> {
        let path = resource_dir.join(OPENVPN_PLUGIN_FILENAME);
        if path.exists() {
            log::trace!("Using OpenVPN plugin at {}", path.display());
            Ok(path)
        } else {
            bail!(ErrorKind::PluginNotFound(path));
        }
    }

    fn get_config_path(resource_dir: &Path) -> Option<PathBuf> {
        let path = resource_dir.join("openvpn.conf");
        if path.exists() {
            Some(path)
        } else {
            None
        }
    }

    fn create_credentials_file(username: &str, password: &str) -> io::Result<mktemp::TempFile> {
        let temp_file = mktemp::TempFile::new();
        log::debug!("Writing credentials to {}", temp_file.as_ref().display());
        let mut file = fs::File::create(&temp_file)?;
        Self::set_user_pass_file_permissions(&file)?;
        write!(file, "{}\n{}\n", username, password)?;
        Ok(temp_file)
    }

    fn create_proxy_auth_file(
        proxy: &Option<OpenVpnProxySettings>,
    ) -> StdResult<Option<mktemp::TempFile>, io::Error> {
        if let Some(OpenVpnProxySettings::Remote(ref remote_proxy)) = proxy {
            if let Some(ref proxy_auth) = remote_proxy.auth {
                return Ok(Some(Self::create_credentials_file(
                    &proxy_auth.username,
                    &proxy_auth.password,
                )?));
            }
        }
        Ok(None)
    }

    #[cfg(unix)]
    fn set_user_pass_file_permissions(file: &fs::File) -> io::Result<()> {
        use std::os::unix::fs::PermissionsExt;
        file.set_permissions(PermissionsExt::from_mode(0o400))
    }

    #[cfg(windows)]
    fn set_user_pass_file_permissions(_file: &fs::File) -> io::Result<()> {
        // TODO(linus): Lock permissions correctly on Windows.
        Ok(())
    }

    /// Creates a handle to this monitor, allowing the tunnel to be closed while some other
    /// thread
    /// is blocked in `wait`.
    pub fn close_handle(&self) -> CloseHandle {
        self.monitor.close_handle()
    }

    /// Consumes the monitor and blocks until the tunnel exits or there is an error.
    pub fn wait(self) -> Result<()> {
        self.monitor
            .wait()
            .chain_err(|| ErrorKind::TunnelMonitoringError)
    }
}


/// A handle to a `TunnelMonitor`
// pub struct CloseHandle(OpenVpnCloseHandle);
pub enum CloseHandle {
    /// OpenVpn close handle
    OpenVpn(OpenVpnCloseHandle),
    #[cfg(unix)]
    /// Wireguard close handle
    Wireguard(Box<wireguard::CloseHandle>),
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
        let enabled_on_tap = ::winnet::get_tap_interface_ipv6_status().unwrap_or(false);

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
        fs::read_to_string("/proc/sys/net/ipv6/conf/all/disable_ipv6")
            .map(|disable_ipv6| disable_ipv6.trim() == "0")
            .unwrap_or(false)
    }
    #[cfg(target_os = "macos")]
    {
        true
    }
}

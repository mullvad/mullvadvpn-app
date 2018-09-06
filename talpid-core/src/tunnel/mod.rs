use mktemp;

use openvpn_plugin::types::OpenVpnPluginEvent;

use process::openvpn::OpenVpnCommand;

use std::collections::HashMap;
use std::ffi::OsString;
use std::fs;
use std::io::{self, Write};
use std::net::Ipv4Addr;
use std::path::{Path, PathBuf};

#[cfg(target_os = "linux")]
use failure::ResultExt as FailureResultExt;
#[cfg(target_os = "linux")]
use which;

use talpid_types::net::{Endpoint, TunnelEndpoint, TunnelEndpointData, TunnelOptions};

/// A module for all OpenVPN related tunnel management.
pub mod openvpn;

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

error_chain!{
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
            description("Running on an unsupported operating system")
        }
        /// This type of VPN tunnel is not supported.
        UnsupportedTunnelProtocol {
            description("This tunnel protocol is not supported")
        }
    }
}


/// Possible events from the VPN tunnel and the child process managing it.
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum TunnelEvent {
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
    pub ip: Ipv4Addr,
    /// The IP to the default gateway on the tunnel interface.
    pub gateway: Ipv4Addr,
}

impl TunnelEvent {
    /// Converts an `OpenVpnPluginEvent` to a `TunnelEvent`.
    /// Returns `None` if there is no corresponding `TunnelEvent`.
    fn from_openvpn_event(
        event: &OpenVpnPluginEvent,
        env: &HashMap<String, String>,
    ) -> Option<TunnelEvent> {
        match *event {
            OpenVpnPluginEvent::Up => {
                let interface = env
                    .get("dev")
                    .expect("No \"dev\" in tunnel up event")
                    .to_owned();
                let ip = env
                    .get("ifconfig_local")
                    .expect("No \"ifconfig_local\" in tunnel up event")
                    .parse()
                    .expect("Tunnel IP not in valid format");
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
            OpenVpnPluginEvent::RoutePredown => Some(TunnelEvent::Down),
            _ => None,
        }
    }
}


/// Abstraction for monitoring a generic VPN tunnel.
pub struct TunnelMonitor {
    monitor: OpenVpnMonitor,
    /// Keep the `TempFile` for the user-pass file in the struct, so it's removed on drop.
    _user_pass_file: mktemp::TempFile,
}

impl TunnelMonitor {
    /// Creates a new `TunnelMonitor` that connects to the given remote and notifies `on_event`
    /// on tunnel state changes.
    pub fn new<L>(
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
        Self::ensure_endpoint_is_openvpn(&tunnel_endpoint)?;
        Self::ensure_ipv6_can_be_used_if_enabled(tunnel_options)?;

        let user_pass_file =
            Self::create_user_pass_file(username).chain_err(|| ErrorKind::CredentialsWriteError)?;
        let cmd = Self::create_openvpn_cmd(
            tunnel_endpoint.to_endpoint(),
            tunnel_alias,
            &tunnel_options,
            user_pass_file.as_ref(),
            log,
            resource_dir,
        )?;

        let user_pass_file_path = user_pass_file.to_path_buf();
        let on_openvpn_event = move |event, env| {
            if event == OpenVpnPluginEvent::Up {
                // The user-pass file has been read. Try to delete it early.
                let _ = fs::remove_file(&user_pass_file_path);
            }
            match TunnelEvent::from_openvpn_event(&event, &env) {
                Some(tunnel_event) => on_event(tunnel_event),
                None => debug!("Ignoring OpenVpnEvent {:?}", event),
            }
        };

        let monitor = openvpn::OpenVpnMonitor::new(
            cmd,
            on_openvpn_event,
            Self::get_plugin_path(resource_dir)?,
        ).chain_err(|| ErrorKind::TunnelMonitoringError)?;
        Ok(TunnelMonitor {
            monitor,
            _user_pass_file: user_pass_file,
        })
    }

    fn ensure_endpoint_is_openvpn(endpoint: &TunnelEndpoint) -> Result<()> {
        match endpoint.tunnel {
            TunnelEndpointData::OpenVpn(_) => Ok(()),
            TunnelEndpointData::Wireguard(_) => bail!(ErrorKind::UnsupportedTunnelProtocol),
        }
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
            .ca(resource_dir.join("ca.crt"))
            .crl(resource_dir.join("crl.pem"));
        if let Some(log) = log {
            cmd.log(log);
        }
        Ok(cmd)
    }

    fn get_openvpn_bin(resource_dir: &Path) -> Result<PathBuf> {
        let path = resource_dir.join(OPENVPN_BIN_FILENAME);
        if path.exists() {
            trace!("Using OpenVPN at {}", path.display());
            Ok(path)
        } else {
            bail!(ErrorKind::OpenVpnNotFound(path));
        }
    }

    fn get_plugin_path(resource_dir: &Path) -> Result<PathBuf> {
        let path = resource_dir.join(OPENVPN_PLUGIN_FILENAME);
        if path.exists() {
            trace!("Using OpenVPN plugin at {}", path.display());
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

    fn create_user_pass_file(username: &str) -> io::Result<mktemp::TempFile> {
        let temp_file = mktemp::TempFile::new();
        debug!(
            "Writing user-pass credentials to {}",
            temp_file.as_ref().display()
        );
        let mut file = fs::File::create(&temp_file)?;
        Self::set_user_pass_file_permissions(&file)?;
        write!(file, "{}\n-\n", username)?;
        Ok(temp_file)
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
        CloseHandle(self.monitor.close_handle())
    }

    /// Consumes the monitor and blocks until the tunnel exits or there is an error.
    pub fn wait(self) -> Result<()> {
        self.monitor
            .wait()
            .chain_err(|| ErrorKind::TunnelMonitoringError)
    }
}


/// A handle to a `TunnelMonitor`
pub struct CloseHandle(OpenVpnCloseHandle);

impl CloseHandle {
    /// Closes the underlying tunnel, making the `TunnelMonitor::wait` method return.
    pub fn close(self) -> io::Result<()> {
        self.0.close()
    }
}

fn is_ipv6_enabled_in_os() -> bool {
    #[cfg(windows)]
    {
        use winreg::enums::*;
        use winreg::RegKey;

        const IPV6_DISABLED: u8 = 0xFF;

        let globally_enabled = RegKey::predef(HKEY_LOCAL_MACHINE)
            .open_subkey(r#"SYSTEM\CurrentControlSet\Services\Tcpip6\Parameters"#)
            .and_then(|ipv6_config| ipv6_config.get_value("DisabledComponents"))
            .map(|ipv6_disabled_bits: u32| (ipv6_disabled_bits & 0xFF) == IPV6_DISABLED as u32)
            .unwrap_or(false);

        globally_enabled && ::ffi::route::get_tap_interface_ipv6_status().unwrap_or(false)
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

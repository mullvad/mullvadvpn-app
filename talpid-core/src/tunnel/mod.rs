use mktemp;

use openvpn_plugin::types::OpenVpnPluginEvent;

use process::openvpn::OpenVpnCommand;

use std::collections::HashMap;
use std::env;
use std::ffi::{OsStr, OsString};
use std::fs;
use std::io::{self, Write};
use std::net::Ipv4Addr;
use std::path::{Path, PathBuf};

use talpid_types::net::{Endpoint, TunnelEndpoint, TunnelParameters};

/// A module for all OpenVPN related tunnel management.
pub mod openvpn;

use self::openvpn::{OpenVpnCloseHandle, OpenVpnMonitor};

mod errors {
    error_chain!{
        errors {
            /// An error indicating there was an error listening for events from the VPN tunnel.
            TunnelMonitoringError {
                description("Error while setting up or processing events from the VPN tunnel")
            }
            /// Failed to get the current executable path.
            ExecutablePathInaccessible {
                description("Error while reading current executable path")
            }
            /// Obtained executable path doesn't have a parent directory.
            ExecutableHasNoParentDir {
                description("Executable path has no directories")
            }
            /// The OpenVPN plugin was not found.
            PluginNotFound {
                description("No OpenVPN plugin found")
            }
            /// There was an error when writing authentication credentials to temporary file.
            CredentialsWriteError {
                description("Error while writing credentials to temporary file")
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
}
pub use self::errors::*;


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
                let interface = env.get("dev")
                    .expect("No \"dev\" in tunnel up event")
                    .to_owned();
                let ip = env.get("ifconfig_local")
                    .expect("No \"ifconfig_local\" in tunnel up event")
                    .parse()
                    .expect("Tunnel IP not in valid format");
                let gateway = env.get("route_vpn_gateway")
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
        account_token: &str,
        log: Option<&Path>,
        resource_dir: &Path,
        on_event: L,
    ) -> Result<Self>
    where
        L: Fn(TunnelEvent) + Send + Sync + 'static,
    {
        match tunnel_endpoint.tunnel {
            TunnelParameters::OpenVpn(_) => (),
            TunnelParameters::Wireguard(_) => bail!(ErrorKind::UnsupportedTunnelProtocol),
        }
        let user_pass_file = Self::create_user_pass_file(account_token)
            .chain_err(|| ErrorKind::CredentialsWriteError)?;
        let cmd = Self::create_openvpn_cmd(
            tunnel_endpoint.to_endpoint(),
            user_pass_file.as_ref(),
            log,
            resource_dir,
        );

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

        let monitor = openvpn::OpenVpnMonitor::new(cmd, on_openvpn_event, Self::get_plugin_path()?)
            .chain_err(|| ErrorKind::TunnelMonitoringError)?;
        Ok(TunnelMonitor {
            monitor,
            _user_pass_file: user_pass_file,
        })
    }

    fn create_openvpn_cmd(
        remote: Endpoint,
        user_pass_file: &Path,
        log: Option<&Path>,
        resource_dir: &Path,
    ) -> OpenVpnCommand {
        let mut cmd = OpenVpnCommand::new(Self::get_openvpn_bin(resource_dir));
        if let Some(config) = Self::get_config_path(resource_dir) {
            cmd.config(config);
        }
        cmd.remote(remote)
            .user_pass(user_pass_file)
            .ca(resource_dir.join("ca.crt"))
            .crl(resource_dir.join("crl.pem"));
        if let Some(log) = log {
            cmd.log(log);
        }
        cmd
    }

    fn get_openvpn_bin(resource_dir: &Path) -> OsString {
        let bin = if cfg!(windows) {
            OsStr::new("openvpn.exe")
        } else {
            OsStr::new("openvpn")
        };
        let bundled_path = resource_dir.join("openvpn-binaries").join(bin);
        if bundled_path.exists() {
            bundled_path.into_os_string()
        } else {
            warn!("Did not find a bundled version of OpenVPN, will rely on the PATH instead");
            bin.to_os_string()
        }
    }

    fn get_plugin_path() -> Result<PathBuf> {
        let library = Self::get_library_name().chain_err(|| ErrorKind::PluginNotFound)?;
        let mut path = Self::get_executable_dir().chain_err(|| ErrorKind::PluginNotFound)?;

        path.push(library);

        if path.exists() {
            debug!("Using OpenVPN plugin at {}", path.to_string_lossy());
            Ok(path)
        } else {
            Err(ErrorKind::PluginNotFound.into())
        }
    }

    fn get_executable_dir() -> Result<PathBuf> {
        let exe_path = env::current_exe().chain_err(|| ErrorKind::ExecutablePathInaccessible)?;

        exe_path
            .parent()
            .map(Path::to_path_buf)
            .ok_or(ErrorKind::ExecutableHasNoParentDir.into())
    }

    fn get_library_name() -> Result<&'static str> {
        if cfg!(target_os = "macos") {
            Ok("libtalpid_openvpn_plugin.dylib")
        } else if cfg!(unix) {
            Ok("libtalpid_openvpn_plugin.so")
        } else if cfg!(windows) {
            Ok("talpid_openvpn_plugin.dll")
        } else {
            bail!(ErrorKind::UnsupportedPlatform);
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

    fn create_user_pass_file(account_token: &str) -> io::Result<mktemp::TempFile> {
        let temp_file = mktemp::TempFile::new();
        debug!(
            "Writing user-pass credentials to {}",
            temp_file.as_ref().to_string_lossy()
        );
        let mut file = fs::File::create(&temp_file)?;
        Self::set_user_pass_file_permissions(&file)?;
        write!(file, "{}\n-\n", account_token)?;
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

    /// Consumes the monitor and block until the tunnel exits or there is an error.
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

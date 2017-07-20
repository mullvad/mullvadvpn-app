use mktemp;
use net;
use openvpn_plugin::types::OpenVpnPluginEvent;
use process::openvpn::OpenVpnCommand;
use std::ffi::{OsStr, OsString};
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};

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
            /// The OpenVPN plugin was not found.
            PluginNotFound {
                description("No OpenVPN plugin found")
            }
            /// There was an error when writing authentication credentials to temporary file.
            CredentialsWriteError {
                description("Error while writing credentials to temporary file")
            }
        }
    }
}
pub use self::errors::*;


/// Possible events from the VPN tunnel and the child process managing it.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub enum TunnelEvent {
    /// Sent when the tunnel comes up and is ready for traffic.
    Up,
    /// Sent when the tunnel goes down.
    Down,
}

impl TunnelEvent {
    /// Converts an `OpenVpnPluginEvent` to a `TunnelEvent`.
    /// Returns `None` if there is no corresponding `TunnelEvent`.
    fn from_openvpn_event(event: &OpenVpnPluginEvent) -> Option<TunnelEvent> {
        match *event {
            OpenVpnPluginEvent::Up => Some(TunnelEvent::Up),
            OpenVpnPluginEvent::RoutePredown => Some(TunnelEvent::Down),
            _ => None,
        }
    }
}


/// Abstraction for monitoring a generic VPN tunnel.
pub struct TunnelMonitor {
    monitor: OpenVpnMonitor,
    _user_pass_file: mktemp::Temp,
}

impl TunnelMonitor {
    /// Creates a new `TunnelMonitor` that connects to the given remote and notifies `on_event`
    /// on tunnel state changes.
    pub fn new<L>(remote: net::Endpoint, account_token: &str, on_event: L) -> Result<Self>
        where L: Fn(TunnelEvent) + Send + Sync + 'static
    {
        let on_openvpn_event = move |event, _env| match TunnelEvent::from_openvpn_event(&event) {
            Some(tunnel_event) => on_event(tunnel_event),
            None => debug!("Ignoring OpenVpnEvent {:?}", event),
        };
        let user_pass_file = Self::create_user_pass_file(account_token)
            .chain_err(|| ErrorKind::CredentialsWriteError)?;
        let cmd = Self::create_openvpn_cmd(remote, user_pass_file.as_ref());
        let monitor = openvpn::OpenVpnMonitor::new(cmd, on_openvpn_event, get_plugin_path()?)
            .chain_err(|| ErrorKind::TunnelMonitoringError)?;
        Ok(
            TunnelMonitor {
                monitor,
                _user_pass_file: user_pass_file,
            },
        )
    }

    fn create_openvpn_cmd(remote: net::Endpoint, user_pass_file: &Path) -> OpenVpnCommand {
        let openvpn_binary = Self::find_path_to_openvpn_binary();

        let mut cmd = OpenVpnCommand::new(openvpn_binary);
        if let Some(config) = get_config_path() {
            cmd.config(config);
        }
        cmd.remote(remote).user_pass(user_pass_file).ca("ca.crt");
        cmd
    }

    fn find_path_to_openvpn_binary() -> OsString {
        match ::std::env::current_exe() {
            Ok(mut path) => {
                path.pop();

                path.push("openvpn-binaries");

                let openvpn_binary = path.join("openvpn");
                if openvpn_binary.exists() {
                    return openvpn_binary.into_os_string();
                }
            }
            Err(e) => warn!("Failed finding the directory of the executable, {}", e),
        }

        debug!("Did not find a bundled version of OpenVPN, will rely on the PATH instead");
        OsStr::new("openvpn").to_os_string()
    }

    fn create_user_pass_file(account_token: &str) -> io::Result<mktemp::Temp> {
        let path = mktemp::Temp::new_file()?;
        debug!(
            "Writing user-pass credentials to {}",
            path.as_ref().to_string_lossy()
        );
        let mut file = fs::File::create(&path)?;
        Self::set_user_pass_file_permissions(&file)?;
        write!(file, "{}\n-\n", account_token)?;
        Ok(path)
    }

    #[cfg(unix)]
    fn set_user_pass_file_permissions(file: &fs::File) -> io::Result<()> {
        use std::os::unix::fs::PermissionsExt;
        file.set_permissions(PermissionsExt::from_mode(0o400))
    }

    #[cfg(windows)]
    fn set_user_pass_file_permissions(file: &fs::File) -> io::Result<()> {
        // TODO(linus): Lock permissions correctly on Windows.
        Ok(())
    }

    /// Creates a handle to this monitor, allowing the tunnel to be closed while some other thread
    /// is blocked in `wait`.
    pub fn close_handle(&self) -> CloseHandle {
        CloseHandle(self.monitor.close_handle())
    }

    /// Consumes the monitor and block until the tunnel exits or there is an error.
    pub fn wait(self) -> Result<()> {
        self.monitor.wait().chain_err(|| ErrorKind::TunnelMonitoringError)
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


// TODO(linus): Temporary implementation for getting plugin path during development.
fn get_plugin_path() -> Result<PathBuf> {
    let dirs = &["./target/debug", "."];
    let filename = if cfg!(target_os = "macos") {
        "libtalpid_openvpn_plugin.dylib"
    } else if cfg!(unix) {
        "libtalpid_openvpn_plugin.so"
    } else if cfg!(windows) {
        "libtalpid_openvpn_plugin.dll"
    } else {
        bail!(ErrorKind::PluginNotFound);
    };

    for dir in dirs {
        let path = Path::new(dir).join(filename);
        if path.exists() {
            debug!("Using OpenVPN plugin at {}", path.to_string_lossy());
            return Ok(path);
        }
    }
    Err(ErrorKind::PluginNotFound.into())
}

// TODO(linus): Temporary implementation for getting hold of a config location.
// Manually place a working config here or change this string in order to test
fn get_config_path() -> Option<&'static Path> {
    let path = Path::new("./openvpn.conf");
    if path.exists() { Some(path) } else { None }
}

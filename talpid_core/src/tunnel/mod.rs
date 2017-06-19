use net;
use openvpn_ffi::OpenVpnPluginEvent;
use process::openvpn::OpenVpnCommand;
use std::io;
use std::path::PathBuf;

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
        }
    }
}
pub use self::errors::*;


/// Possible events from the VPN tunnel and the child process managing it.
#[derive(Debug)]
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
}

impl TunnelMonitor {
    /// Creates a new `TunnelMonitor` that connects to the given remote and notifies `on_event`
    /// on tunnel state changes.
    pub fn new<L>(remote: net::RemoteAddr, on_event: L) -> Result<Self>
        where L: Fn(TunnelEvent) + Send + Sync + 'static
    {
        let on_openvpn_event = move |event, _env| match TunnelEvent::from_openvpn_event(&event) {
            Some(tunnel_event) => on_event(tunnel_event),
            None => debug!("Ignoring OpenVpnEvent {:?}", event),
        };
        let cmd = Self::create_openvpn_cmd(remote);
        let monitor = openvpn::OpenVpnMonitor::new(cmd, on_openvpn_event, get_plugin_path()?)
            .chain_err(|| ErrorKind::TunnelMonitoringError)?;
        Ok(TunnelMonitor { monitor })
    }

    fn create_openvpn_cmd(remote: net::RemoteAddr) -> OpenVpnCommand {
        let mut cmd = OpenVpnCommand::new("openvpn");
        cmd.config(get_config_path())
            .remotes(remote)
            .unwrap();
        cmd
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
        let mut path = PathBuf::from(dir);
        path.push(filename);
        if path.exists() {
            debug!("Using OpenVPN plugin at {}", path.to_string_lossy());
            return Ok(path);
        }
    }
    Err(ErrorKind::PluginNotFound.into())
}

// TODO(linus): Temporary implementation for getting hold of a config location.
// Manually place a working config here or change this string in order to test
fn get_config_path() -> &'static str {
    "./openvpn.conf"
}

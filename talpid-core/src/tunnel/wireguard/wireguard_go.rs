use super::{Config, Error, Result, Tunnel};
use crate::tunnel::tun_provider::{Tun, TunConfig, TunProvider};
use std::{ffi::CString, fs, os::unix::io::AsRawFd, path::Path};

pub struct WgGoTunnel {
    interface_name: String,
    handle: Option<i32>,
    // holding on to the tunnel device and the log file ensures that the associated file handles
    // live long enough and get closed when the tunnel is stopped
    _tunnel_device: Box<dyn Tun>,
    _log_file: fs::File,
}

impl WgGoTunnel {
    pub fn start_tunnel(
        config: &Config,
        log_path: Option<&Path>,
        tun_provider: &dyn TunProvider,
    ) -> Result<Self> {
        let tunnel_config = TunConfig {
            addresses: config.tunnel.addresses.clone())
        };
        let tunnel_device = tun_provider
            .create_tun(tunnel_config)
            .map_err(Error::SetupTunnelDeviceError)?;

        let interface_name: String = tunnel_device.interface_name().to_string();
        let log_file = prepare_log_file(log_path)?;

        let wg_config_str = config.to_userspace_format();
        let iface_name =
            CString::new(interface_name.as_bytes()).map_err(Error::InterfaceNameError)?;

        let handle = unsafe {
            wgTurnOnWithFd(
                iface_name.as_ptr(),
                config.mtu as i64,
                wg_config_str.as_ptr(),
                tunnel_device.as_raw_fd(),
                log_file.as_raw_fd(),
                WG_GO_LOG_DEBUG,
            )
        };

        if handle < 0 {
            return Err(Error::StartWireguardError { status: handle });
        }

        Ok(WgGoTunnel {
            interface_name,
            handle: Some(handle),
            _tunnel_device: tunnel_device,
            _log_file: log_file,
        })
    }

    fn stop_tunnel(&mut self) -> Result<()> {
        if let Some(handle) = self.handle.take() {
            let status = unsafe { wgTurnOff(handle) };
            if status < 0 {
                return Err(Error::StopWireguardError { status });
            }
        }
        return Ok(());
    }
}

impl Drop for WgGoTunnel {
    fn drop(&mut self) {
        if let Err(e) = self.stop_tunnel() {
            log::error!("Failed to stop tunnel - {}", e);
        }
    }
}

fn prepare_log_file(log_path: Option<&Path>) -> Result<fs::File> {
    fs::File::create(log_path.unwrap_or("/dev/null".as_ref())).map_err(Error::PrepareLogFileError)
}

impl Tunnel for WgGoTunnel {
    fn get_interface_name(&self) -> &str {
        &self.interface_name
    }

    fn stop(mut self: Box<Self>) -> Result<()> {
        self.stop_tunnel()
    }
}

#[cfg(unix)]
pub type Fd = std::os::unix::io::RawFd;

#[cfg(windows)]
pub type Fd = std::os::windows::io::RawHandle;

type WgLogLevel = i32;
// wireguard-go supports log levels 0 through 3 with 3 being the most verbose
const WG_GO_LOG_DEBUG: WgLogLevel = 3;

#[link(name = "wg", kind = "static")]
extern "C" {
    // Creates a new wireguard tunnel, uses the specific interface name, MTU and file descriptors
    // for the tunnel device and logging.
    //
    // Positive return values are tunnel handles for this specific wireguard tunnel instance.
    // Negative return values signify errors. All error codes are opaque.
    fn wgTurnOnWithFd(
        iface_name: *const i8,
        mtu: i64,
        settings: *const i8,
        fd: Fd,
        log_fd: Fd,
        logLevel: WgLogLevel,
    ) -> i32;
    // Pass a handle that was created by wgTurnOnWithFd to stop a wireguard tunnel.
    fn wgTurnOff(handle: i32) -> i32;
}

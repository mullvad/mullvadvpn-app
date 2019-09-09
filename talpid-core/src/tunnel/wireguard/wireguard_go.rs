use super::{Config, Error, Result, Tunnel};
use crate::tunnel::tun_provider::{Tun, TunConfig, TunProvider};
use ipnetwork::IpNetwork;
use std::{ffi::CString, fs, net::IpAddr, os::unix::io::AsRawFd, path::Path};
#[cfg(target_os = "android")]
use talpid_types::BoxedError;

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
        tun_provider: &mut dyn TunProvider,
        routes: impl Iterator<Item = IpNetwork>,
    ) -> Result<Self> {
        #[cfg_attr(not(target_os = "android"), allow(unused_mut))]
        let mut tunnel_device = tun_provider
            .get_tun(Self::create_tunnel_config(config, routes))
            .map_err(Error::SetupTunnelDeviceError)?;

        let interface_name: String = tunnel_device.interface_name().to_string();
        let log_file = prepare_log_file(log_path)?;

        let wg_config_str = config.to_userspace_format();
        let iface_name =
            CString::new(interface_name.as_bytes()).map_err(Error::InterfaceNameError)?;

        let handle = unsafe {
            wgTurnOnWithFd(
                iface_name.as_ptr() as *const i8,
                config.mtu as isize,
                wg_config_str.as_ptr() as *const i8,
                tunnel_device.as_raw_fd(),
                log_file.as_raw_fd(),
                WG_GO_LOG_DEBUG,
            )
        };

        if handle < 0 {
            // Error values returned from the wireguard-go library
            return match handle {
                -1 => Err(Error::FatalStartWireguardError),
                -2 => Err(Error::RecoverableStartWireguardError),
                _ => unreachable!("Unknown status code returned from wireguard-go"),
            };
        }

        #[cfg(target_os = "android")]
        Self::bypass_tunnel_sockets(&mut tunnel_device, handle).map_err(Error::BypassError)?;

        Ok(WgGoTunnel {
            interface_name,
            handle: Some(handle),
            _tunnel_device: tunnel_device,
            _log_file: log_file,
        })
    }

    fn create_tunnel_config(config: &Config, routes: impl Iterator<Item = IpNetwork>) -> TunConfig {
        let mut dns_servers = vec![IpAddr::V4(config.ipv4_gateway)];
        dns_servers.extend(config.ipv6_gateway.map(IpAddr::V6));

        TunConfig {
            addresses: config.tunnel.addresses.clone(),
            dns_servers,
            routes: routes.collect(),
            mtu: config.mtu,
        }
    }

    #[cfg(target_os = "android")]
    fn bypass_tunnel_sockets(
        tunnel_device: &mut Box<dyn Tun>,
        handle: i32,
    ) -> std::result::Result<(), BoxedError> {
        let socket_v4 = unsafe { wgGetSocketV4(handle) };
        let socket_v6 = unsafe { wgGetSocketV6(handle) };

        tunnel_device.bypass(socket_v4)?;
        tunnel_device.bypass(socket_v6)?;

        Ok(())
    }

    fn stop_tunnel(&mut self) -> Result<()> {
        if let Some(handle) = self.handle.take() {
            let status = unsafe { wgTurnOff(handle) };
            if status < 0 {
                return Err(Error::StopWireguardError { status });
            }
        }
        Ok(())
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

#[cfg_attr(target_os = "android", link(name = "wg", kind = "dylib"))]
#[cfg_attr(not(target_os = "android"), link(name = "wg", kind = "static"))]
extern "C" {
    // Creates a new wireguard tunnel, uses the specific interface name, MTU and file descriptors
    // for the tunnel device and logging.
    //
    // Positive return values are tunnel handles for this specific wireguard tunnel instance.
    // Negative return values signify errors. All error codes are opaque.
    #[cfg_attr(target_os = "android", link_name = "wgTurnOnWithFdAndroid")]
    fn wgTurnOnWithFd(
        iface_name: *const i8,
        mtu: isize,
        settings: *const i8,
        fd: Fd,
        log_fd: Fd,
        logLevel: WgLogLevel,
    ) -> i32;

    // Pass a handle that was created by wgTurnOnWithFd to stop a wireguard tunnel.
    fn wgTurnOff(handle: i32) -> i32;

    // Returns the file descriptor of the tunnel IPv4 socket.
    #[cfg(target_os = "android")]
    fn wgGetSocketV4(handle: i32) -> Fd;

    // Returns the file descriptor of the tunnel IPv6 socket.
    #[cfg(target_os = "android")]
    fn wgGetSocketV6(handle: i32) -> Fd;
}

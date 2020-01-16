use super::{stats::Stats, Config, Error, Result, Tunnel};
use crate::tunnel::tun_provider::TunProvider;
use ipnetwork::IpNetwork;
use std::{
    ffi::{c_void, CStr, CString},
    path::Path,
};

#[cfg(target_os = "android")]
use crate::tunnel::tun_provider;

#[cfg(not(target_os = "windows"))]
use {
    crate::tunnel::tun_provider::{Tun, TunConfig},
    std::{
        net::IpAddr,
        os::unix::io::{AsRawFd, RawFd},
        ptr,
    },
};

#[cfg(target_os = "windows")]
use {
    crate::winnet::{self, add_device_ip_addresses},
    chrono,
    parking_lot::Mutex,
    std::{collections::HashMap, fs, io::Write},
};


#[cfg(target_os = "windows")]
lazy_static::lazy_static! {
    static ref LOG_MUTEX: Mutex<HashMap<u32, fs::File>> = Mutex::new(HashMap::new());
}

#[cfg(target_os = "windows")]
static mut LOG_CONTEXT_NEXT_ORDINAL: u32 = 0;

#[cfg(not(target_os = "windows"))]
const MAX_PREPARE_TUN_ATTEMPTS: usize = 4;


pub struct WgGoTunnel {
    interface_name: String,
    handle: Option<i32>,
    // holding on to the tunnel device and the log file ensures that the associated file handles
    // live long enough and get closed when the tunnel is stopped
    #[cfg(not(target_os = "windows"))]
    _tunnel_device: Tun,
    // ordinal that maps to fs::File instance, used with logging callback
    #[cfg(target_os = "windows")]
    log_context_ordinal: u32,
}

impl WgGoTunnel {
    #[cfg(not(target_os = "windows"))]
    pub fn start_tunnel(
        config: &Config,
        log_path: Option<&Path>,
        tun_provider: &mut TunProvider,
        routes: impl Iterator<Item = IpNetwork>,
    ) -> Result<Self> {
        #[cfg_attr(not(target_os = "android"), allow(unused_mut))]
        let (mut tunnel_device, tunnel_fd) = Self::get_tunnel(tun_provider, config, routes)?;
        let interface_name: String = tunnel_device.interface_name().to_string();

        let wg_config_str = config.to_userspace_format();
        let iface_name =
            CString::new(interface_name.as_bytes()).map_err(Error::InterfaceNameError)?;

        let log_path = log_path.and_then(|path| CString::new(path.to_string_lossy().as_ref()).ok());
        let log_path_ptr = log_path
            .as_ref()
            .map(|path| path.as_ptr())
            .unwrap_or_else(|| ptr::null());

        let handle = unsafe {
            wgTurnOnWithFd(
                iface_name.as_ptr() as *const i8,
                config.mtu as isize,
                wg_config_str.as_ptr() as *const i8,
                tunnel_fd,
                log_path_ptr as *const i8,
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
        })
    }

    #[cfg(target_os = "windows")]
    pub fn start_tunnel(
        config: &Config,
        log_path: Option<&Path>,
        _tun_provider: &mut TunProvider,
        _routes: impl Iterator<Item = IpNetwork>,
    ) -> Result<Self> {
        let log_file = prepare_log_file(log_path)?;

        let log_context_ordinal = unsafe {
            let mut map = LOG_MUTEX.lock();
            let ordinal = LOG_CONTEXT_NEXT_ORDINAL;
            LOG_CONTEXT_NEXT_ORDINAL += 1;
            map.insert(ordinal, log_file);
            ordinal
        };

        let wg_config_str = config.to_userspace_format();
        let iface_name: String = "wg-mullvad".to_string();
        let cstr_iface_name =
            CString::new(iface_name.as_bytes()).map_err(Error::InterfaceNameError)?;

        let handle = unsafe {
            wgTurnOn(
                cstr_iface_name.as_ptr(),
                config.mtu as i64,
                wg_config_str.as_ptr(),
                Some(Self::logging_callback),
                log_context_ordinal as *mut libc::c_void,
            )
        };

        if handle < 0 {
            clean_up_log_file(log_context_ordinal);
            return Err(Error::FatalStartWireguardError);
        }

        if !add_device_ip_addresses(&iface_name, &config.tunnel.addresses) {
            // Todo: what kind of clean-up is required?
            clean_up_log_file(log_context_ordinal);
            return Err(Error::SetIpAddressesError);
        }

        Ok(WgGoTunnel {
            interface_name: iface_name.clone(),
            handle: Some(handle),
            log_context_ordinal,
        })
    }

    // Callback to be used to rebind the tunnel sockets when the default route changes
    #[cfg(target_os = "windows")]
    pub unsafe extern "system" fn default_route_changed_callback(
        event_type: winnet::WinNetDefaultRouteChangeEventType,
        address_family: winnet::WinNetIpFamily,
        interface_luid: u64,
        _ctx: *mut libc::c_void,
    ) {
        use winapi::shared::{ifdef::NET_LUID, netioapi::ConvertInterfaceLuidToIndex};
        let iface_idx: u32 = match event_type {
            winnet::WinNetDefaultRouteChangeEventType::DefaultRouteChanged => {
                let mut iface_idx = 0u32;
                let iface_luid = NET_LUID {
                    Value: interface_luid,
                };
                let status =
                    ConvertInterfaceLuidToIndex(&iface_luid as *const _, &mut iface_idx as *mut _);
                if status != 0 {
                    log::error!(
                        "Failed to convert interface LUID to interface index - {} - {}",
                        status,
                        std::io::Error::last_os_error()
                    );
                    return;
                }
                iface_idx
            }
            // if there is no new default route, specify 0 as the interface index
            winnet::WinNetDefaultRouteChangeEventType::DefaultRouteRemoved => 0,
        };

        wgRebindTunnelSocket(address_family.to_windows_proto_enum(), iface_idx);
    }

    // Callback that receives messages from WireGuard
    #[cfg(target_os = "windows")]
    pub unsafe extern "system" fn logging_callback(
        level: WgLogLevel,
        msg: *const libc::c_char,
        context: *mut libc::c_void,
    ) {
        let map = LOG_MUTEX.lock();
        if let Some(mut logfile) = map.get(&(context as u32)) {
            let managed_msg = if !msg.is_null() {
                std::ffi::CStr::from_ptr(msg)
                    .to_string_lossy()
                    .to_string()
                    .replace("\n", "\r\n")
            } else {
                "Logging message from WireGuard is NULL".to_string()
            };

            let level_str = match level {
                WG_GO_LOG_DEBUG => "DEBUG",
                WG_GO_LOG_INFO => "INFO",
                WG_GO_LOG_ERROR | _ => "ERROR",
            };

            let _ = write!(
                logfile,
                "{}[{}][{}] {}",
                chrono::Local::now().format("[%Y-%m-%d %H:%M:%S%.3f]"),
                "wireguard-go",
                level_str,
                managed_msg
            );
        }
    }

    #[cfg(not(target_os = "windows"))]
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
        tunnel_device: &mut Tun,
        handle: i32,
    ) -> std::result::Result<(), tun_provider::Error> {
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

    #[cfg(not(target_os = "windows"))]
    fn get_tunnel(
        tun_provider: &mut TunProvider,
        config: &Config,
        routes: impl Iterator<Item = IpNetwork>,
    ) -> Result<(Tun, RawFd)> {
        let mut last_error = None;
        let tunnel_config = Self::create_tunnel_config(config, routes);

        for _ in 1..=MAX_PREPARE_TUN_ATTEMPTS {
            let tunnel_device = tun_provider
                .get_tun(tunnel_config.clone())
                .map_err(Error::SetupTunnelDeviceError)?;

            match nix::unistd::dup(tunnel_device.as_raw_fd()) {
                Ok(fd) => return Ok((tunnel_device, fd)),
                #[cfg(not(target_os = "macos"))]
                Err(error @ nix::Error::Sys(nix::errno::Errno::EBADFD)) => last_error = Some(error),
                Err(error @ nix::Error::Sys(nix::errno::Errno::EBADF)) => last_error = Some(error),
                Err(error) => return Err(Error::FdDuplicationError(error)),
            }
        }

        Err(Error::FdDuplicationError(
            last_error.expect("Should be collected in loop"),
        ))
    }
}

#[cfg(target_os = "windows")]
fn clean_up_log_file(ordinal: u32) {
    let mut map = LOG_MUTEX.lock();
    map.remove(&ordinal);
}

impl Drop for WgGoTunnel {
    fn drop(&mut self) {
        if let Err(e) = self.stop_tunnel() {
            log::error!("Failed to stop tunnel - {}", e);
        }
        #[cfg(target_os = "windows")]
        clean_up_log_file(self.log_context_ordinal);
    }
}

#[cfg(target_os = "windows")]
static NULL_DEVICE: &str = "NUL";

#[cfg(target_os = "windows")]
fn prepare_log_file(log_path: Option<&Path>) -> Result<fs::File> {
    fs::File::create(log_path.unwrap_or(NULL_DEVICE.as_ref())).map_err(Error::PrepareLogFileError)
}

impl Tunnel for WgGoTunnel {
    fn get_interface_name(&self) -> &str {
        &self.interface_name
    }

    fn get_config(&self) -> Result<Stats> {
        let (config_str, ptr) = unsafe {
            let ptr = wgGetConfig(self.handle.unwrap());
            if ptr.is_null() {
                log::error!("Failed to get config !");
                return Err(Error::GetConfigError);
            }

            (CStr::from_ptr(ptr), ptr)
        };

        let result =
            Stats::parse_config_str(config_str.to_str().expect("Go strings are always UTF-8"))
                .map_err(Error::StatsError);
        let len = config_str.to_bytes().len();
        unsafe {
            // Zerioing out config string to not leave private key in memory.
            for byte in std::slice::from_raw_parts_mut(ptr, len).iter_mut() {
                *byte = 0i8;
            }
            wgFreePtr(ptr as *mut c_void);
        }

        result
    }

    fn stop(mut self: Box<Self>) -> Result<()> {
        self.stop_tunnel()
    }
}

#[cfg(unix)]
pub type Fd = std::os::unix::io::RawFd;

#[cfg(windows)]
pub type Fd = std::os::windows::io::RawHandle;

type WgLogLevel = u32;
// wireguard-go supports log levels 0 through 3 with 3 being the most verbose
// const WG_GO_LOG_SILENT: WgLogLevel = 0;
#[cfg(target_os = "windows")]
const WG_GO_LOG_ERROR: WgLogLevel = 1;
#[cfg(target_os = "windows")]
const WG_GO_LOG_INFO: WgLogLevel = 2;
const WG_GO_LOG_DEBUG: WgLogLevel = 3;

#[cfg(target_os = "windows")]
pub type LoggingCallback = unsafe extern "system" fn(
    level: WgLogLevel,
    msg: *const libc::c_char,
    context: *mut libc::c_void,
);

extern "C" {
    // Creates a new wireguard tunnel, uses the specific interface name, MTU and file descriptors
    // for the tunnel device and logging.
    //
    // Positive return values are tunnel handles for this specific wireguard tunnel instance.
    // Negative return values signify errors. All error codes are opaque.
    #[cfg_attr(target_os = "android", link_name = "wgTurnOnWithFdAndroid")]
    #[cfg(not(target_os = "windows"))]
    fn wgTurnOnWithFd(
        iface_name: *const i8,
        mtu: isize,
        settings: *const i8,
        fd: Fd,
        log_path: *const i8,
        logLevel: WgLogLevel,
    ) -> i32;

    // Windows
    #[cfg(target_os = "windows")]
    fn wgTurnOn(
        iface_name: *const i8,
        mtu: i64,
        settings: *const i8,
        logging_callback: Option<LoggingCallback>,
        logging_context: *mut libc::c_void,
    ) -> i32;

    // Pass a handle that was created by wgTurnOnWithFd to stop a wireguard tunnel.
    fn wgTurnOff(handle: i32) -> i32;

    // Returns the file descriptor of the tunnel IPv4 socket.
    fn wgGetConfig(handle: i32) -> *mut std::os::raw::c_char;

    // Frees a pointer allocated by the go runtime - useful to free return value of wgGetConfig
    fn wgFreePtr(ptr: *mut c_void);

    // Returns the file descriptor of the tunnel IPv4 socket.
    #[cfg(target_os = "android")]
    fn wgGetSocketV4(handle: i32) -> Fd;

    // Returns the file descriptor of the tunnel IPv6 socket.
    #[cfg(target_os = "android")]
    fn wgGetSocketV6(handle: i32) -> Fd;

    // Rebind tunnel socket when network interfaces change
    #[cfg(target_os = "windows")]
    fn wgRebindTunnelSocket(family: u16, interfaceIndex: u32);
}

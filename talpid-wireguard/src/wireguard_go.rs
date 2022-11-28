use super::{
    stats::{Stats, StatsMap},
    Config, Tunnel, TunnelError,
};
use crate::logging::{clean_up_logging, initialize_logging, wg_go_logging_callback, WgLogLevel};
#[cfg(windows)]
use futures::SinkExt;
use std::{
    ffi::{c_void, CStr},
    future::Future,
    os::raw::c_char,
    path::Path,
    pin::Pin,
};
#[cfg(windows)]
use talpid_types::BoxedError;
use zeroize::Zeroize;
#[cfg(not(windows))]
use {ipnetwork::IpNetwork, talpid_tunnel::tun_provider::TunProvider};

#[cfg(target_os = "windows")]
use std::ffi::CString;

#[cfg(target_os = "android")]
use talpid_tunnel::tun_provider;

#[cfg(not(target_os = "windows"))]
use {
    std::{
        net::IpAddr,
        os::unix::io::{AsRawFd, RawFd},
    },
    talpid_tunnel::tun_provider::{Tun, TunConfig},
};

type Result<T> = std::result::Result<T, TunnelError>;

#[cfg(not(target_os = "windows"))]
use std::sync::{Arc, Mutex};

#[cfg(not(target_os = "windows"))]
const MAX_PREPARE_TUN_ATTEMPTS: usize = 4;

struct LoggingContext(u32);

impl Drop for LoggingContext {
    fn drop(&mut self) {
        clean_up_logging(self.0);
    }
}

pub struct WgGoTunnel {
    interface_name: String,
    handle: Option<i32>,
    // holding on to the tunnel device and the log file ensures that the associated file handles
    // live long enough and get closed when the tunnel is stopped
    #[cfg(not(target_os = "windows"))]
    _tunnel_device: Tun,
    // context that maps to fs::File instance, used with logging callback
    _logging_context: LoggingContext,
    #[cfg(target_os = "windows")]
    _route_callback_handle: Option<talpid_routing::CallbackHandle>,
    #[cfg(target_os = "windows")]
    setup_handle: tokio::task::JoinHandle<()>,
}

impl WgGoTunnel {
    #[cfg(not(target_os = "windows"))]
    pub fn start_tunnel(
        config: &Config,
        log_path: Option<&Path>,
        tun_provider: Arc<Mutex<TunProvider>>,
        routes: impl Iterator<Item = IpNetwork>,
    ) -> Result<Self> {
        #[cfg_attr(not(target_os = "android"), allow(unused_mut))]
        let (mut tunnel_device, tunnel_fd) = Self::get_tunnel(tun_provider, config, routes)?;
        let interface_name: String = tunnel_device.interface_name().to_string();
        let wg_config_str = config.to_userspace_format();
        let logging_context = initialize_logging(log_path)
            .map(LoggingContext)
            .map_err(TunnelError::LoggingError)?;

        #[cfg(not(target_os = "android"))]
        let mtu = config.mtu as isize;
        let handle = unsafe {
            wgTurnOn(
                #[cfg(not(target_os = "android"))]
                mtu,
                wg_config_str.as_ptr() as *const i8,
                tunnel_fd,
                Some(wg_go_logging_callback),
                logging_context.0 as *mut libc::c_void,
            )
        };
        check_wg_status(handle)?;

        #[cfg(target_os = "android")]
        Self::bypass_tunnel_sockets(&mut tunnel_device, handle)
            .map_err(TunnelError::BypassError)?;

        Ok(WgGoTunnel {
            interface_name,
            handle: Some(handle),
            _tunnel_device: tunnel_device,
            _logging_context: logging_context,
        })
    }

    #[cfg(target_os = "windows")]
    pub fn start_tunnel(
        config: &Config,
        log_path: Option<&Path>,
        route_manager_handle: talpid_routing::RouteManagerHandle,
        mut done_tx: futures::channel::mpsc::Sender<std::result::Result<(), BoxedError>>,
        runtime: &tokio::runtime::Handle,
    ) -> Result<Self> {
        use talpid_types::ErrorExt;

        let route_callback_handle = runtime
            .block_on(
                route_manager_handle.add_default_route_change_callback(Box::new(
                    WgGoTunnel::default_route_changed_callback,
                )),
            )
            .ok();
        if route_callback_handle.is_none() {
            log::warn!("Failed to register default route callback");
        }

        let wg_config_str = config.to_userspace_format();
        let iface_name: String = "Mullvad".to_string();
        let cstr_iface_name =
            CString::new(iface_name.as_bytes()).map_err(TunnelError::InterfaceNameError)?;
        let logging_context = initialize_logging(log_path)
            .map(LoggingContext)
            .map_err(TunnelError::LoggingError)?;

        let mut alias_ptr = std::ptr::null_mut();
        let mut interface_luid = 0u64;

        let handle = unsafe {
            wgTurnOn(
                cstr_iface_name.as_ptr(),
                config.mtu as i64,
                wg_config_str.as_ptr(),
                &mut alias_ptr,
                &mut interface_luid,
                Some(wg_go_logging_callback),
                logging_context.0 as *mut libc::c_void,
            )
        };
        check_wg_status(handle)?;

        let actual_iface_name = {
            let actual_iface_name_c = unsafe { CStr::from_ptr(alias_ptr) };
            let actual_iface_name = actual_iface_name_c
                .to_str()
                .map_err(|_| TunnelError::InvalidAlias)?
                .to_string();
            unsafe { wgFreePtr(alias_ptr as *mut c_void) };
            actual_iface_name
        };

        log::debug!("Adapter alias: {}", actual_iface_name);

        let has_ipv6 = config.tunnel.addresses.iter().any(|addr| addr.is_ipv6());
        let setup_handle = tokio::spawn(async move {
            use windows_sys::Win32::NetworkManagement::Ndis::NET_LUID_LH;
            let luid = NET_LUID_LH {
                Value: interface_luid,
            };
            log::debug!("Waiting for tunnel IP interfaces to arrive");

            let prepare_interfaces = async move {
                talpid_windows_net::wait_for_interfaces(luid, true, has_ipv6).await?;

                if let Err(error) =
                    talpid_tunnel::network_interface::initialize_interfaces(luid, None)
                {
                    log::error!(
                        "{}",
                        error.display_chain_with_msg("Failed to set tunnel interface metric"),
                    );
                }

                Ok(())
            };

            let _ = done_tx
                .send(
                    prepare_interfaces
                        .await
                        .map_err(|error| BoxedError::new(TunnelError::SetupIpInterfaces(error))),
                )
                .await;
            log::debug!("Waiting for tunnel IP interfaces: Done");
        });

        Ok(WgGoTunnel {
            interface_name: actual_iface_name,
            handle: Some(handle),
            setup_handle,
            _logging_context: logging_context,
            _route_callback_handle: route_callback_handle,
        })
    }

    // Callback to be used to rebind the tunnel sockets when the default route changes
    #[cfg(target_os = "windows")]
    pub fn default_route_changed_callback<'a>(
        event_type: crate::routing::EventType<'a>,
        address_family: talpid_windows_net::AddressFamily,
    ) {
        use crate::routing::EventType::*;
        use windows_sys::Win32::NetworkManagement::IpHelper::ConvertInterfaceLuidToIndex;

        let iface_idx: u32 = match event_type {
            Updated(default_route) => {
                let mut iface_idx = 0u32;
                let iface_luid = default_route.iface;
                let status = unsafe { ConvertInterfaceLuidToIndex(&iface_luid, &mut iface_idx) };
                if status != 0 {
                    log::error!(
                        "Failed to convert interface LUID to interface index: {}: {}",
                        status,
                        std::io::Error::last_os_error()
                    );
                    return;
                }
                iface_idx
            }
            // if there is no new default route, specify 0 as the interface index
            Removed => 0,
            // ignore interface updates that don't affect the interface to use
            UpdatedDetails(_) => return,
        };

        unsafe { wgRebindTunnelSocket(address_family.to_af_family(), iface_idx) };
    }

    #[cfg(not(target_os = "windows"))]
    fn create_tunnel_config(config: &Config, routes: impl Iterator<Item = IpNetwork>) -> TunConfig {
        let mut dns_servers = vec![IpAddr::V4(config.ipv4_gateway)];
        dns_servers.extend(config.ipv6_gateway.map(IpAddr::V6));

        TunConfig {
            addresses: config.tunnel.addresses.clone(),
            dns_servers,
            routes: routes.collect(),
            #[cfg(target_os = "android")]
            required_routes: Self::create_required_routes(config),
            mtu: config.mtu,
        }
    }

    #[cfg(target_os = "android")]
    fn create_required_routes(config: &Config) -> Vec<IpNetwork> {
        let mut required_routes = vec![IpNetwork::new(IpAddr::V4(config.ipv4_gateway), 32)
            .expect("Invalid IPv4 network prefix")];

        required_routes.extend(config.ipv6_gateway.map(|address| {
            IpNetwork::new(IpAddr::V6(address), 128).expect("Invalid IPv6 network prefix")
        }));

        required_routes
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
        #[cfg(windows)]
        self.setup_handle.abort();
        if let Some(handle) = self.handle.take() {
            let status = unsafe { wgTurnOff(handle) };
            if status < 0 {
                return Err(TunnelError::StopWireguardError { status });
            }
        }
        Ok(())
    }

    #[cfg(not(target_os = "windows"))]
    fn get_tunnel(
        tun_provider: Arc<Mutex<TunProvider>>,
        config: &Config,
        routes: impl Iterator<Item = IpNetwork>,
    ) -> Result<(Tun, RawFd)> {
        let mut last_error = None;
        let tunnel_config = Self::create_tunnel_config(config, routes);

        let mut tun_provider = tun_provider.lock().unwrap();

        for _ in 1..=MAX_PREPARE_TUN_ATTEMPTS {
            let tunnel_device = tun_provider
                .get_tun(tunnel_config.clone())
                .map_err(TunnelError::SetupTunnelDeviceError)?;

            match nix::unistd::dup(tunnel_device.as_raw_fd()) {
                Ok(fd) => return Ok((tunnel_device, fd)),
                #[cfg(not(target_os = "macos"))]
                Err(error @ nix::errno::Errno::EBADFD) => last_error = Some(error),
                Err(error @ nix::errno::Errno::EBADF) => last_error = Some(error),
                Err(error) => return Err(TunnelError::FdDuplicationError(error)),
            }
        }

        Err(TunnelError::FdDuplicationError(
            last_error.expect("Should be collected in loop"),
        ))
    }
}

impl Drop for WgGoTunnel {
    fn drop(&mut self) {
        if let Err(e) = self.stop_tunnel() {
            log::error!("Failed to stop tunnel: {}", e);
        }
    }
}

impl Tunnel for WgGoTunnel {
    fn get_interface_name(&self) -> String {
        self.interface_name.clone()
    }

    fn get_tunnel_stats(&self) -> Result<StatsMap> {
        let config_str = unsafe {
            let ptr = wgGetConfig(self.handle.unwrap());
            if ptr.is_null() {
                log::error!("Failed to get config !");
                return Err(TunnelError::GetConfigError);
            }

            CStr::from_ptr(ptr)
        };

        let result =
            Stats::parse_config_str(config_str.to_str().expect("Go strings are always UTF-8"))
                .map_err(TunnelError::StatsError);
        unsafe {
            // Zeroing out config string to not leave private key in memory.
            let slice = std::slice::from_raw_parts_mut(
                config_str.as_ptr() as *mut c_char,
                config_str.to_bytes().len(),
            );
            slice.zeroize();

            wgFreePtr(config_str.as_ptr() as *mut c_void);
        }

        result
    }

    fn stop(mut self: Box<Self>) -> Result<()> {
        self.stop_tunnel()
    }

    fn set_config(
        &self,
        config: Config,
    ) -> Pin<Box<dyn Future<Output = std::result::Result<(), super::TunnelError>> + Send>> {
        let wg_config_str = config.to_userspace_format();
        let handle = self.handle.unwrap();
        Box::pin(async move {
            let status = unsafe { wgSetConfig(handle, wg_config_str.as_ptr() as *const i8) };
            if status != 0 {
                return Err(TunnelError::SetConfigError);
            }
            Ok(())
        })
    }
}

fn check_wg_status(wg_code: i32) -> Result<()> {
    match wg_code {
        ERROR_GENERAL_FAILURE => Err(TunnelError::FatalStartWireguardError),
        ERROR_INTERMITTENT_FAILURE => Err(TunnelError::RecoverableStartWireguardError),
        0.. => Ok(()),
        _ => {
            log::error!("Unknown status code returned from wireguard-go");
            Err(TunnelError::FatalStartWireguardError)
        }
    }
}

#[cfg(unix)]
pub type Fd = std::os::unix::io::RawFd;

pub type LoggingCallback = unsafe extern "system" fn(
    level: WgLogLevel,
    msg: *const libc::c_char,
    context: *mut libc::c_void,
);

const ERROR_GENERAL_FAILURE: i32 = -1;
const ERROR_INTERMITTENT_FAILURE: i32 = -2;

extern "C" {
    /// Creates a new wireguard tunnel, uses the specific interface name, MTU and file descriptors
    /// for the tunnel device and logging.
    ///
    /// Positive return values are tunnel handles for this specific wireguard tunnel instance.
    /// Negative return values signify errors. All error codes are opaque.
    #[cfg(not(any(target_os = "android", target_os = "windows")))]
    fn wgTurnOn(
        mtu: isize,
        settings: *const i8,
        fd: Fd,
        logging_callback: Option<LoggingCallback>,
        logging_context: *mut libc::c_void,
    ) -> i32;

    // Android
    #[cfg(target_os = "android")]
    fn wgTurnOn(
        settings: *const i8,
        fd: Fd,
        logging_callback: Option<LoggingCallback>,
        logging_context: *mut libc::c_void,
    ) -> i32;

    // Windows
    #[cfg(target_os = "windows")]
    fn wgTurnOn(
        iface_name: *const i8,
        mtu: i64,
        settings: *const i8,
        iface_name_out: *const *mut std::os::raw::c_char,
        iface_luid_out: *mut u64,
        logging_callback: Option<LoggingCallback>,
        logging_context: *mut libc::c_void,
    ) -> i32;

    // Pass a handle that was created by wgTurnOn to stop a wireguard tunnel.
    fn wgTurnOff(handle: i32) -> i32;

    // Returns the file descriptor of the tunnel IPv4 socket.
    fn wgGetConfig(handle: i32) -> *mut std::os::raw::c_char;

    // Sets the config of the WireGuard interface.
    fn wgSetConfig(handle: i32, settings: *const i8) -> i32;

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

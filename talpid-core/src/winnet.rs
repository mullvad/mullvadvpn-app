use self::api::*;
pub use self::api::{
    LogSink, WinNet_ActivateConnectivityMonitor, WinNet_DeactivateConnectivityMonitor,
};
use crate::routing::Node;
use ipnetwork::IpNetwork;
use libc::{c_char, c_void, wchar_t};
use std::{
    ffi::{CStr, OsString},
    net::IpAddr,
    ptr,
};
use widestring::WideCString;

/// Errors that this module may produce.
#[derive(err_derive::Error, Debug)]
pub enum Error {
    /// Failed to set the metrics for a network interface.
    #[error(display = "Failed to set the metrics for a network interface")]
    MetricApplication,

    /// Supplied interface alias is invalid.
    #[error(display = "Supplied interface alias is invalid")]
    InvalidInterfaceAlias(#[error(source)] widestring::NulError<u16>),

    /// Failed to read IPv6 status on the TAP network interface.
    #[error(display = "Failed to read IPv6 status on the TAP network interface")]
    GetIpv6Status,

    /// Failed to determine alias of TAP adapter.
    #[error(display = "Failed to determine alias of TAP adapter")]
    GetTapAlias,

    /// Can't establish whether host is connected to a non-virtual network
    #[error(display = "Network connectivity undecideable")]
    ConnectivityUnkown,
}

#[allow(dead_code)]
#[repr(u8)]
pub enum LogSeverity {
    Error = 0,
    Warning,
    Info,
    Trace,
}

/// Logging callback used with `winnet.dll`.
pub extern "system" fn log_sink(severity: LogSeverity, msg: *const c_char, _ctx: *mut c_void) {
    if msg.is_null() {
        log::error!("Log message from FFI boundary is NULL");
    } else {
        let managed_msg = unsafe { CStr::from_ptr(msg).to_string_lossy() };
        match severity {
            LogSeverity::Warning => log::warn!("{}", managed_msg),
            LogSeverity::Info => log::info!("{}", managed_msg),
            LogSeverity::Trace => log::trace!("{}", managed_msg),
            _ => log::error!("{}", managed_msg),
        }
    }
}

/// Returns true if metrics were changed, false otherwise
pub fn ensure_top_metric_for_interface(interface_alias: &str) -> Result<bool, Error> {
    let interface_alias_ws =
        WideCString::from_str(interface_alias).map_err(Error::InvalidInterfaceAlias)?;

    let metric_result = unsafe {
        WinNet_EnsureTopMetric(interface_alias_ws.as_ptr(), Some(log_sink), ptr::null_mut())
    };

    match metric_result {
        // Metrics didn't change
        0 => Ok(false),
        // Metrics changed
        1 => Ok(true),
        // Failure
        2 => Err(Error::MetricApplication),
        // Unexpected value
        i => {
            log::error!("Unexpected return code from WinNet_EnsureTopMetric: {}", i);
            Err(Error::MetricApplication)
        }
    }
}

/// Checks if IPv6 is enabled for the TAP interface
pub fn get_tap_interface_ipv6_status() -> Result<bool, Error> {
    let tap_ipv6_status =
        unsafe { WinNet_GetTapInterfaceIpv6Status(Some(log_sink), ptr::null_mut()) };

    match tap_ipv6_status {
        // Enabled
        0 => Ok(true),
        // Disabled
        1 => Ok(false),
        // Failure
        2 => Err(Error::GetIpv6Status),
        // Unexpected value
        i => {
            log::error!(
                "Unexpected return code from WinNet_GetTapInterfaceIpv6Status: {}",
                i
            );
            Err(Error::GetIpv6Status)
        }
    }
}

/// Dynamically determines the alias of the TAP adapter.
pub fn get_tap_interface_alias() -> Result<OsString, Error> {
    let mut alias_ptr: *mut wchar_t = ptr::null_mut();
    let status = unsafe {
        WinNet_GetTapInterfaceAlias(&mut alias_ptr as *mut _, Some(log_sink), ptr::null_mut())
    };

    if !status {
        return Err(Error::GetTapAlias);
    }

    let alias = unsafe { WideCString::from_ptr_str(alias_ptr) };
    unsafe { WinNet_ReleaseString(alias_ptr) };

    Ok(alias.to_os_string())
}

#[repr(C)]
struct WinNetIpType(u32);

const WINNET_IPV4: u32 = 0;
const WINNET_IPV6: u32 = 1;

impl WinNetIpType {
    pub fn v4() -> Self {
        WinNetIpType(WINNET_IPV4)
    }

    pub fn v6() -> Self {
        WinNetIpType(WINNET_IPV6)
    }
}


#[repr(C)]
pub struct WinNetIpNetwork {
    ip_type: WinNetIpType,
    ip_bytes: [u8; 16],
    prefix: u8,
}

impl From<IpNetwork> for WinNetIpNetwork {
    fn from(network: IpNetwork) -> WinNetIpNetwork {
        let WinNetIp { ip_type, ip_bytes } = WinNetIp::from(network.ip());
        let prefix = network.prefix();
        WinNetIpNetwork {
            ip_type,
            ip_bytes,
            prefix,
        }
    }
}

#[repr(C)]
pub struct WinNetIp {
    ip_type: WinNetIpType,
    ip_bytes: [u8; 16],
}

impl From<IpAddr> for WinNetIp {
    fn from(addr: IpAddr) -> WinNetIp {
        let mut bytes = [0u8; 16];
        match addr {
            IpAddr::V4(v4_addr) => {
                bytes[..4].copy_from_slice(&v4_addr.octets());
                WinNetIp {
                    ip_type: WinNetIpType::v4(),
                    ip_bytes: bytes,
                }
            }
            IpAddr::V6(v6_addr) => {
                bytes.copy_from_slice(&v6_addr.octets());

                WinNetIp {
                    ip_type: WinNetIpType::v6(),
                    ip_bytes: bytes,
                }
            }
        }
    }
}

#[repr(C)]
pub struct WinNetNode {
    gateway: *mut WinNetIp,
    device_name: *mut u16,
}

impl WinNetNode {
    fn new(name: &str, ip: WinNetIp) -> Self {
        let device_name = WideCString::from_str(name)
            .expect("Failed to convert UTF-8 string to null terminated UCS string")
            .into_raw();
        let gateway = Box::into_raw(Box::new(ip));
        Self {
            gateway,
            device_name,
        }
    }

    fn from_gateway(ip: WinNetIp) -> Self {
        let gateway = Box::into_raw(Box::new(ip));
        Self {
            gateway,
            device_name: ptr::null_mut(),
        }
    }


    fn from_device(name: &str) -> Self {
        let device_name = WideCString::from_str(name)
            .expect("Failed to convert UTF-8 string to null terminated UCS string")
            .into_raw();
        Self {
            gateway: ptr::null_mut(),
            device_name,
        }
    }
}

impl From<&Node> for WinNetNode {
    fn from(node: &Node) -> Self {
        match (node.get_address(), node.get_device()) {
            (Some(gateway), None) => WinNetNode::from_gateway(gateway.into()),
            (None, Some(device)) => WinNetNode::from_device(device),
            (Some(gateway), Some(device)) => WinNetNode::new(device, gateway.into()),
            _ => unreachable!(),
        }
    }
}

impl Drop for WinNetNode {
    fn drop(&mut self) {
        if !self.gateway.is_null() {
            unsafe {
                let _ = Box::from_raw(self.gateway);
            }
        }
        if !self.device_name.is_null() {
            unsafe {
                let _ = WideCString::from_ptr_str(self.device_name);
            }
        }
    }
}


#[repr(C)]
pub struct WinNetRoute {
    gateway: WinNetIpNetwork,
    node: *mut WinNetNode,
}

impl WinNetRoute {
    pub fn through_default_node(gateway: WinNetIpNetwork) -> Self {
        Self {
            gateway,
            node: ptr::null_mut(),
        }
    }

    pub fn new(node: WinNetNode, gateway: WinNetIpNetwork) -> Self {
        let node = Box::into_raw(Box::new(node));
        WinNetRoute { gateway, node }
    }
}

impl Drop for WinNetRoute {
    fn drop(&mut self) {
        if !self.node.is_null() {
            unsafe {
                let _ = Box::from_raw(self.node);
            }
            self.node = ptr::null_mut();
        }
    }
}

pub fn activate_routing_manager(routes: &[WinNetRoute]) -> bool {
    unsafe { WinNet_ActivateRouteManager(Some(log_sink), ptr::null_mut()) };
    routing_manager_add_routes(routes)
}

pub struct WinNetCallbackHandle {
    handle: *mut libc::c_void,
    // allows us to keep the context pointer allive.
    _context: Box<dyn std::any::Any>,
}

unsafe impl Send for WinNetCallbackHandle {}

impl Drop for WinNetCallbackHandle {
    fn drop(&mut self) {
        unsafe { WinNet_UnregisterDefaultRouteChangedCallback(self.handle) };
    }
}

#[allow(dead_code)]
#[repr(u16)]
pub enum WinNetDefaultRouteChangeEventType {
    DefaultRouteChanged = 0,
    DefaultRouteRemoved = 1,
}

#[allow(dead_code)]
#[repr(u16)]
pub enum WinNetIpFamily {
    V4 = 0,
    V6 = 1,
}

impl WinNetIpFamily {
    pub fn to_windows_proto_enum(&self) -> u16 {
        match self {
            Self::V4 => 2,
            Self::V6 => 23,
        }
    }
}

pub type DefaultRouteChangedCallback = unsafe extern "system" fn(
    event_type: WinNetDefaultRouteChangeEventType,
    ip_family: WinNetIpFamily,
    interface_luid: u64,
    ctx: *mut c_void,
);

#[derive(err_derive::Error, Debug)]
#[error(display = "Failed to set callback for default route")]
pub struct DefaultRouteCallbackError;

pub fn set_default_route_change_callback<T: 'static>(
    callback: Option<DefaultRouteChangedCallback>,
    context: T,
) -> std::result::Result<WinNetCallbackHandle, DefaultRouteCallbackError> {
    let mut handle_ptr = ptr::null_mut();
    let mut context = Box::new(context);
    let ctx_ptr = &mut *context as *mut T as *mut libc::c_void;
    unsafe {
        if !WinNet_RegisterDefaultRouteChangedCallback(callback, ctx_ptr, &mut handle_ptr as *mut _)
        {
            return Err(DefaultRouteCallbackError);
        }


        Ok(WinNetCallbackHandle {
            handle: handle_ptr,
            _context: context,
        })
    }
}

pub fn routing_manager_add_routes(routes: &[WinNetRoute]) -> bool {
    let ptr = routes.as_ptr();
    let length: u32 = routes.len() as u32;
    unsafe { WinNet_AddRoutes(ptr, length) }
}

pub fn deactivate_routing_manager() -> bool {
    unsafe { WinNet_DeactivateRouteManager() }
}

pub fn add_device_ip_addresses(iface: &String, addresses: &Vec<IpAddr>) -> bool {
    let raw_iface = WideCString::from_str(iface)
        .expect("Failed to convert UTF-8 string to null terminated UCS string")
        .into_raw();
    let converted_addresses: Vec<_> = addresses.iter().map(|addr| WinNetIp::from(*addr)).collect();
    let ptr = converted_addresses.as_ptr();
    let length: u32 = converted_addresses.len() as u32;
    unsafe { WinNet_AddDeviceIpAddresses(raw_iface, ptr, length, Some(log_sink), ptr::null_mut()) }
}

#[allow(non_snake_case)]
mod api {
    use super::{DefaultRouteChangedCallback, LogSeverity};
    use libc::{c_char, c_void, wchar_t};

    /// logging callback type for use with `winnet.dll`.
    pub type LogSink =
        extern "system" fn(severity: LogSeverity, msg: *const c_char, ctx: *mut c_void);

    pub type ConnectivityCallback = unsafe extern "system" fn(is_connected: bool, ctx: *mut c_void);

    extern "system" {
        #[link_name = "WinNet_ActivateRouteManager"]
        pub fn WinNet_ActivateRouteManager(sink: Option<LogSink>, sink_context: *mut c_void);

        #[link_name = "WinNet_AddRoutes"]
        pub fn WinNet_AddRoutes(routes: *const super::WinNetRoute, num_routes: u32) -> bool;

        // #[link_name = "WinNet_AddRoute"]
        // pub fn WinNet_AddRoute(route: *const super::WinNetRoute) -> bool;

        // #[link_name = "WinNet_DeleteRoutes"]
        // pub fn WinNet_DeleteRoutes(routes: *const super::WinNetRoute, num_routes: u32) -> bool;

        // #[link_name = "WinNet_DeleteRoute"]
        // pub fn WinNet_DeleteRoute(route: *const super::WinNetRoute) -> bool;

        #[link_name = "WinNet_DeactivateRouteManager"]
        pub fn WinNet_DeactivateRouteManager() -> bool;

        #[link_name = "WinNet_EnsureTopMetric"]
        pub fn WinNet_EnsureTopMetric(
            tunnel_interface_alias: *const wchar_t,
            sink: Option<LogSink>,
            sink_context: *mut c_void,
        ) -> u32;

        #[link_name = "WinNet_GetTapInterfaceIpv6Status"]
        pub fn WinNet_GetTapInterfaceIpv6Status(
            sink: Option<LogSink>,
            sink_context: *mut c_void,
        ) -> u32;

        #[link_name = "WinNet_GetTapInterfaceAlias"]
        pub fn WinNet_GetTapInterfaceAlias(
            tunnel_interface_alias: *mut *mut wchar_t,
            sink: Option<LogSink>,
            sink_context: *mut c_void,
        ) -> bool;

        #[link_name = "WinNet_ReleaseString"]
        pub fn WinNet_ReleaseString(string: *mut wchar_t) -> u32;

        #[link_name = "WinNet_ActivateConnectivityMonitor"]
        pub fn WinNet_ActivateConnectivityMonitor(
            callback: Option<ConnectivityCallback>,
            callbackContext: *mut libc::c_void,
            sink: Option<LogSink>,
            sink_context: *mut c_void,
        ) -> bool;

        #[link_name = "WinNet_RegisterDefaultRouteChangedCallback"]
        pub fn WinNet_RegisterDefaultRouteChangedCallback(
            callback: Option<DefaultRouteChangedCallback>,
            callbackContext: *mut libc::c_void,
            registrationHandle: *mut *mut libc::c_void,
        ) -> bool;

        #[link_name = "WinNet_UnregisterDefaultRouteChangedCallback"]
        pub fn WinNet_UnregisterDefaultRouteChangedCallback(registrationHandle: *mut libc::c_void);

        #[link_name = "WinNet_DeactivateConnectivityMonitor"]
        pub fn WinNet_DeactivateConnectivityMonitor() -> bool;

        #[link_name = "WinNet_AddDeviceIpAddresses"]
        pub fn WinNet_AddDeviceIpAddresses(
            interface_alias: *const wchar_t,
            addresses: *const super::WinNetIp,
            num_addresses: u32,
            sink: Option<LogSink>,
            sink_context: *mut c_void,
        ) -> bool;
    }
}

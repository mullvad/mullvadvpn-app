use self::api::*;
pub use self::api::{WinNet_ActivateConnectivityMonitor, WinNet_DeactivateConnectivityMonitor};
use crate::{logging::windows::log_sink, routing::Node};
use ipnetwork::IpNetwork;
use libc::{c_void, wchar_t};
use std::{
    ffi::{OsStr, OsString},
    net::{IpAddr, Ipv4Addr, Ipv6Addr},
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

    /// Failed to enable IPv6 on the network interface.
    #[error(display = "Failed to enable IPv6 on the network interface")]
    EnableIpv6,

    /// Failed to enable IPv6 on the network interface.
    #[error(display = "Failed to obtain GUID for the network interface")]
    GetInterfaceGuid,

    /// Failed to get the current default route.
    #[error(display = "Failed to obtain default route")]
    GetDefaultRoute,

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

fn logging_context() -> *const u8 {
    b"WinNet\0".as_ptr()
}

/// Returns true if metrics were changed, false otherwise
pub fn ensure_best_metric_for_interface(interface_alias: &str) -> Result<bool, Error> {
    let interface_alias_ws =
        WideCString::from_str(interface_alias).map_err(Error::InvalidInterfaceAlias)?;

    let metric_result = unsafe {
        WinNet_EnsureBestMetric(
            interface_alias_ws.as_ptr(),
            Some(log_sink),
            logging_context(),
        )
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
            log::error!("Unexpected return code from WinNet_EnsureBestMetric: {}", i);
            Err(Error::MetricApplication)
        }
    }
}

/// Enables IPv6 for a given interface.
pub fn enable_ipv6_for_adapter(interface_guid: &str) -> Result<(), Error> {
    let interface_guid_ws =
        WideCString::from_str(interface_guid).map_err(Error::InvalidInterfaceAlias)?;

    let result = unsafe {
        WinNet_EnableIpv6ForAdapter(
            interface_guid_ws.as_ptr(),
            Some(log_sink),
            logging_context(),
        )
    };

    if result {
        Ok(())
    } else {
        Err(Error::EnableIpv6)
    }
}

/// Dynamically determines the alias of the TAP adapter.
pub fn get_tap_interface_alias() -> Result<OsString, Error> {
    let mut alias_ptr: *mut wchar_t = ptr::null_mut();
    let status = unsafe {
        WinNet_GetTapInterfaceAlias(&mut alias_ptr as *mut _, Some(log_sink), logging_context())
    };

    if !status {
        return Err(Error::GetTapAlias);
    }

    let alias = unsafe { WideCString::from_ptr_str(alias_ptr) };
    unsafe { WinNet_ReleaseString(alias_ptr) };

    Ok(alias.to_os_string())
}

/// Determines the interface guid for a given adapter alias.
pub fn interface_alias_to_guid(interface_alias: &OsStr) -> Result<String, Error> {
    let interface_alias =
        WideCString::from_os_str(interface_alias).map_err(Error::InvalidInterfaceAlias)?;
    let mut guid_ptr: *mut wchar_t = ptr::null_mut();
    let status = unsafe {
        WinNet_InterfaceAliasToGuid(
            interface_alias.as_ptr(),
            &mut guid_ptr as *mut _,
            Some(log_sink),
            logging_context(),
        )
    };

    if !status {
        return Err(Error::GetInterfaceGuid);
    }

    let guid = unsafe { WideCString::from_ptr_str(guid_ptr) };
    unsafe { WinNet_ReleaseString(guid_ptr) };

    Ok(guid.to_string_lossy())
}

#[allow(dead_code)]
#[repr(u32)]
pub enum WinNetAddrFamily {
    IPV4 = 0,
    IPV6 = 1,
}

impl Default for WinNetAddrFamily {
    fn default() -> Self {
        WinNetAddrFamily::IPV4
    }
}

impl WinNetAddrFamily {
    pub fn to_windows_proto_enum(&self) -> u16 {
        match self {
            Self::IPV4 => 2,
            Self::IPV6 => 23,
        }
    }
}

#[repr(C)]
#[derive(Default)]
pub struct WinNetIp {
    pub addr_family: WinNetAddrFamily,
    pub ip_bytes: [u8; 16],
}

#[repr(C)]
#[derive(Default)]
pub struct WinNetDefaultRoute {
    pub interface_luid: u64,
    pub gateway: WinNetIp,
}

impl From<WinNetIp> for IpAddr {
    fn from(addr: WinNetIp) -> IpAddr {
        match addr.addr_family {
            WinNetAddrFamily::IPV4 => {
                let mut bytes: [u8; 4] = Default::default();
                bytes.clone_from_slice(&addr.ip_bytes[..4]);
                IpAddr::V4(Ipv4Addr::from(bytes))
            }
            WinNetAddrFamily::IPV6 => IpAddr::V6(Ipv6Addr::from(addr.ip_bytes)),
        }
    }
}

impl From<IpAddr> for WinNetIp {
    fn from(addr: IpAddr) -> WinNetIp {
        let mut bytes = [0u8; 16];
        match addr {
            IpAddr::V4(v4_addr) => {
                bytes[..4].copy_from_slice(&v4_addr.octets());
                WinNetIp {
                    addr_family: WinNetAddrFamily::IPV4,
                    ip_bytes: bytes,
                }
            }
            IpAddr::V6(v6_addr) => {
                bytes.copy_from_slice(&v6_addr.octets());

                WinNetIp {
                    addr_family: WinNetAddrFamily::IPV6,
                    ip_bytes: bytes,
                }
            }
        }
    }
}

#[repr(C)]
pub struct WinNetIpNetwork {
    prefix: u8,
    ip: WinNetIp,
}

impl From<IpNetwork> for WinNetIpNetwork {
    fn from(network: IpNetwork) -> WinNetIpNetwork {
        WinNetIpNetwork {
            prefix: network.prefix(),
            ip: WinNetIp::from(network.ip()),
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
        Self { gateway, node }
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

pub fn activate_routing_manager() -> bool {
    unsafe { WinNet_ActivateRouteManager(Some(log_sink), logging_context()) }
}

pub struct WinNetCallbackHandle {
    handle: *mut libc::c_void,
    // Allows us to keep the context pointer alive.
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

pub type DefaultRouteChangedCallback = unsafe extern "system" fn(
    event_type: WinNetDefaultRouteChangeEventType,
    family: WinNetAddrFamily,
    default_route: WinNetDefaultRoute,
    ctx: *mut c_void,
);

#[derive(err_derive::Error, Debug)]
#[error(display = "Failed to set callback for default route")]
pub struct DefaultRouteCallbackError;

pub fn add_default_route_change_callback<T: 'static>(
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

pub fn routing_manager_delete_applied_routes() -> bool {
    unsafe { WinNet_DeleteAppliedRoutes() }
}

pub fn deactivate_routing_manager() {
    unsafe { WinNet_DeactivateRouteManager() }
}

// TODO: Remove attribute once this is in use.
#[allow(dead_code)]
pub fn get_best_default_route(
    family: WinNetAddrFamily,
) -> Result<Option<WinNetDefaultRoute>, Error> {
    let mut default_route = WinNetDefaultRoute::default();
    match unsafe {
        WinNet_GetBestDefaultRoute(
            family,
            &mut default_route as *mut _,
            Some(log_sink),
            logging_context(),
        )
    } {
        GetBestDefaultRouteStatus::Success => Ok(Some(default_route)),
        GetBestDefaultRouteStatus::NotFound => Ok(None),
        GetBestDefaultRouteStatus::Failure => Err(Error::GetDefaultRoute),
    }
}

pub fn add_device_ip_addresses(iface: &String, addresses: &Vec<IpAddr>) -> bool {
    let raw_iface = WideCString::from_str(iface)
        .expect("Failed to convert UTF-8 string to null terminated UCS string")
        .into_raw();
    let converted_addresses: Vec<_> = addresses.iter().map(|addr| WinNetIp::from(*addr)).collect();
    let ptr = converted_addresses.as_ptr();
    let length: u32 = converted_addresses.len() as u32;
    unsafe {
        WinNet_AddDeviceIpAddresses(raw_iface, ptr, length, Some(log_sink), logging_context())
    }
}

#[allow(non_snake_case)]
mod api {
    use super::DefaultRouteChangedCallback;
    use crate::logging::windows::LogSink;
    use libc::{c_void, wchar_t};

    pub type ConnectivityCallback = unsafe extern "system" fn(is_connected: bool, ctx: *mut c_void);

    #[allow(dead_code)]
    #[repr(u32)]
    pub enum FailableOptionalStatus {
        Success = 0,
        NotFound = 1,
        Failure = 2,
    }
    pub type GetBestDefaultRouteStatus = FailableOptionalStatus;

    extern "system" {
        #[link_name = "WinNet_ActivateRouteManager"]
        pub fn WinNet_ActivateRouteManager(sink: Option<LogSink>, sink_context: *const u8) -> bool;

        #[link_name = "WinNet_AddRoutes"]
        pub fn WinNet_AddRoutes(routes: *const super::WinNetRoute, num_routes: u32) -> bool;

        // #[link_name = "WinNet_AddRoute"]
        // pub fn WinNet_AddRoute(route: *const super::WinNetRoute) -> bool;

        // #[link_name = "WinNet_DeleteRoutes"]
        // pub fn WinNet_DeleteRoutes(routes: *const super::WinNetRoute, num_routes: u32) -> bool;

        // #[link_name = "WinNet_DeleteRoute"]
        // pub fn WinNet_DeleteRoute(route: *const super::WinNetRoute) -> bool;

        #[link_name = "WinNet_DeleteAppliedRoutes"]
        pub fn WinNet_DeleteAppliedRoutes() -> bool;

        #[link_name = "WinNet_DeactivateRouteManager"]
        pub fn WinNet_DeactivateRouteManager();

        #[link_name = "WinNet_EnsureBestMetric"]
        pub fn WinNet_EnsureBestMetric(
            tunnel_interface_alias: *const wchar_t,
            sink: Option<LogSink>,
            sink_context: *const u8,
        ) -> u32;

        #[link_name = "WinNet_EnableIpv6ForAdapter"]
        pub fn WinNet_EnableIpv6ForAdapter(
            tunnel_interface_alias: *const wchar_t,
            sink: Option<LogSink>,
            sink_context: *const u8,
        ) -> bool;

        // TODO: Remove "allow(dead_code)" this is in use.
        #[allow(dead_code)]
        #[link_name = "WinNet_GetBestDefaultRoute"]
        pub fn WinNet_GetBestDefaultRoute(
            family: super::WinNetAddrFamily,
            default_route: *mut super::WinNetDefaultRoute,
            sink: Option<LogSink>,
            sink_context: *const u8,
        ) -> GetBestDefaultRouteStatus;

        #[link_name = "WinNet_GetTapInterfaceAlias"]
        pub fn WinNet_GetTapInterfaceAlias(
            tunnel_interface_alias: *mut *mut wchar_t,
            sink: Option<LogSink>,
            sink_context: *const u8,
        ) -> bool;

        #[link_name = "WinNet_InterfaceAliasToGuid"]
        pub fn WinNet_InterfaceAliasToGuid(
            interface_alias: *const wchar_t,
            interface_guid: *mut *mut wchar_t,
            sink: Option<LogSink>,
            sink_context: *const u8,
        ) -> bool;

        #[link_name = "WinNet_ReleaseString"]
        pub fn WinNet_ReleaseString(string: *mut wchar_t);

        #[link_name = "WinNet_ActivateConnectivityMonitor"]
        pub fn WinNet_ActivateConnectivityMonitor(
            callback: Option<ConnectivityCallback>,
            callbackContext: *mut libc::c_void,
            sink: Option<LogSink>,
            sink_context: *const u8,
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
        pub fn WinNet_DeactivateConnectivityMonitor();

        #[link_name = "WinNet_AddDeviceIpAddresses"]
        pub fn WinNet_AddDeviceIpAddresses(
            interface_alias: *const wchar_t,
            addresses: *const super::WinNetIp,
            num_addresses: u32,
            sink: Option<LogSink>,
            sink_context: *const u8,
        ) -> bool;
    }
}

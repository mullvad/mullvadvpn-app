use self::api::*;
use crate::{logging::windows::log_sink, routing::Node};
use ipnetwork::IpNetwork;
use libc::c_void;
use std::{
    convert::TryFrom,
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

    /// Failed to get the current default route.
    #[error(display = "Failed to obtain default route")]
    GetDefaultRoute,

    /// Failed to get a network device.
    #[error(display = "Failed to obtain network interface by name")]
    GetDeviceByName,

    /// Failed to get a network device.
    #[error(display = "Failed to obtain network interface by gateway")]
    GetDeviceByGateway,

    /// Unexpected error while adding routes
    #[error(display = "Winnet returned an error while adding routes")]
    GeneralAddRoutesError,

    /// Failed to obtain an IP address given a LUID.
    #[error(display = "Failed to obtain IP address for the given interface")]
    GetIpAddressFromLuid,

    /// Failed to read IPv6 status on the TAP network interface.
    #[error(display = "Failed to read IPv6 status on the TAP network interface")]
    GetIpv6Status,
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

#[derive(Debug, Clone, Copy)]
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

#[derive(Debug)]
pub struct WrongIpFamilyError;

impl TryFrom<WinNetIp> for Ipv4Addr {
    type Error = WrongIpFamilyError;

    fn try_from(addr: WinNetIp) -> Result<Ipv4Addr, WrongIpFamilyError> {
        match addr.addr_family {
            WinNetAddrFamily::IPV4 => {
                let mut bytes: [u8; 4] = Default::default();
                bytes.clone_from_slice(&addr.ip_bytes[..4]);
                Ok(Ipv4Addr::from(bytes))
            }
            WinNetAddrFamily::IPV6 => Err(WrongIpFamilyError),
        }
    }
}

impl TryFrom<WinNetIp> for Ipv6Addr {
    type Error = WrongIpFamilyError;

    fn try_from(addr: WinNetIp) -> Result<Ipv6Addr, WrongIpFamilyError> {
        match addr.addr_family {
            WinNetAddrFamily::IPV4 => Err(WrongIpFamilyError),
            WinNetAddrFamily::IPV6 => Ok(Ipv6Addr::from(addr.ip_bytes)),
        }
    }
}

impl From<WinNetIp> for IpAddr {
    fn from(addr: WinNetIp) -> IpAddr {
        match addr.addr_family {
            WinNetAddrFamily::IPV4 => IpAddr::V4(Ipv4Addr::try_from(addr).unwrap()),
            WinNetAddrFamily::IPV6 => IpAddr::V6(Ipv6Addr::try_from(addr).unwrap()),
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

#[derive(Debug, Clone, Copy, PartialEq)]
#[allow(dead_code)]
#[repr(u16)]
pub enum WinNetDefaultRouteChangeEventType {
    DefaultRouteChanged = 0,
    DefaultRouteUpdatedDetails = 1,
    DefaultRouteRemoved = 2,
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

pub fn routing_manager_add_routes(routes: &[WinNetRoute]) -> Result<(), Error> {
    let ptr = routes.as_ptr();
    let length: u32 = routes.len() as u32;
    match unsafe { WinNet_AddRoutes(ptr, length) } {
        WinNetAddRouteStatus::Success => Ok(()),
        WinNetAddRouteStatus::GeneralError => Err(Error::GeneralAddRoutesError),
        WinNetAddRouteStatus::NoDefaultRoute => Err(Error::GetDefaultRoute),
        WinNetAddRouteStatus::NameNotFound => Err(Error::GetDeviceByName),
        WinNetAddRouteStatus::GatewayNotFound => Err(Error::GetDeviceByGateway),
    }
}

pub fn routing_manager_delete_applied_routes() -> bool {
    unsafe { WinNet_DeleteAppliedRoutes() }
}

pub fn deactivate_routing_manager() {
    unsafe { WinNet_DeactivateRouteManager() }
}

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
        WinNetStatus::Success => Ok(Some(default_route)),
        WinNetStatus::NotFound => Ok(None),
        WinNetStatus::Failure => Err(Error::GetDefaultRoute),
    }
}

pub fn interface_luid_to_ip(
    family: WinNetAddrFamily,
    luid: u64,
) -> Result<Option<WinNetIp>, Error> {
    let mut ip = WinNetIp::default();
    match unsafe {
        WinNet_InterfaceLuidToIpAddress(
            family,
            luid,
            &mut ip as *mut _,
            Some(log_sink),
            logging_context(),
        )
    } {
        WinNetStatus::Success => Ok(Some(ip)),
        WinNetStatus::NotFound => Ok(None),
        WinNetStatus::Failure => Err(Error::GetIpAddressFromLuid),
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
    use libc::wchar_t;

    #[allow(dead_code)]
    #[repr(u32)]
    pub enum WinNetStatus {
        Success = 0,
        NotFound = 1,
        Failure = 2,
    }

    #[allow(dead_code)]
    #[repr(u32)]
    pub enum WinNetAddRouteStatus {
        Success = 0,
        GeneralError = 1,
        NoDefaultRoute = 2,
        NameNotFound = 3,
        GatewayNotFound = 4,
    }

    extern "system" {
        #[link_name = "WinNet_ActivateRouteManager"]
        pub fn WinNet_ActivateRouteManager(sink: Option<LogSink>, sink_context: *const u8) -> bool;

        #[link_name = "WinNet_AddRoutes"]
        pub fn WinNet_AddRoutes(
            routes: *const super::WinNetRoute,
            num_routes: u32,
        ) -> WinNetAddRouteStatus;

        // #[link_name = "WinNet_AddRoute"]
        // pub fn WinNet_AddRoute(route: *const super::WinNetRoute) -> WinNetAddRouteStatus;

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

        #[link_name = "WinNet_GetBestDefaultRoute"]
        pub fn WinNet_GetBestDefaultRoute(
            family: super::WinNetAddrFamily,
            default_route: *mut super::WinNetDefaultRoute,
            sink: Option<LogSink>,
            sink_context: *const u8,
        ) -> WinNetStatus;

        #[link_name = "WinNet_InterfaceLuidToIpAddress"]
        pub fn WinNet_InterfaceLuidToIpAddress(
            family: super::WinNetAddrFamily,
            luid: u64,
            ip: *mut super::WinNetIp,
            sink: Option<LogSink>,
            sink_context: *const u8,
        ) -> WinNetStatus;

        #[link_name = "WinNet_RegisterDefaultRouteChangedCallback"]
        pub fn WinNet_RegisterDefaultRouteChangedCallback(
            callback: Option<DefaultRouteChangedCallback>,
            callbackContext: *mut libc::c_void,
            registrationHandle: *mut *mut libc::c_void,
        ) -> bool;

        #[link_name = "WinNet_UnregisterDefaultRouteChangedCallback"]
        pub fn WinNet_UnregisterDefaultRouteChangedCallback(registrationHandle: *mut libc::c_void);

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

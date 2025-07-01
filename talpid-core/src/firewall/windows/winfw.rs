//! Rust bindings for the WinFW library written in C++.

use super::{
    multibyte_to_wide, widestring_ip, AllowedEndpoint, AllowedTunnelTraffic, Error, WideCString,
};
use std::{
    ffi::{c_char, c_void, CStr},
    ptr,
};
use talpid_types::{net::TransportProtocol, tunnel::FirewallPolicyError};
use windows_sys::Win32::Globalization::CP_ACP;

/// Timeout for acquiring the WFP transaction lock
const WINFW_TIMEOUT_SECONDS: u32 = 5;

type LogSink = extern "system" fn(level: log::Level, msg: *const c_char, context: *mut c_void);

const LOGGING_CONTEXT: &CStr = c"WinFw";

pub struct WinFwAllowedEndpointContainer {
    _clients: Box<[WideCString]>,
    clients_ptrs: Box<[*const u16]>,
    ip: WideCString,
    port: u16,
    protocol: WinFwProt,
}

/// Initialize WinFw module. Returns an initialization error if called multiple times without
/// interleaving [Self::deinit].
pub(super) fn initialize() -> Result<(), Error> {
    // SAFETY: This function is always safe to call.
    let init = unsafe {
        WinFw_Initialize(
            WINFW_TIMEOUT_SECONDS,
            Some(log_sink),
            LOGGING_CONTEXT.as_ptr(),
        )
    };

    init.into_result()
}

/// Initialize WinFw module and apply blocking rules. Returns an initialization error if called
/// multiple times without interleaving [Self::deinit].
pub(super) fn initialize_blocked(
    allowed_endpoint: AllowedEndpoint,
    allow_lan: bool,
) -> Result<(), Error> {
    let cfg = WinFwSettings::new(allow_lan);
    let allowed_endpoint = WinFwAllowedEndpointContainer::from(allowed_endpoint);
    // SAFETY: This function is always safe to call.
    let init = unsafe {
        WinFw_InitializeBlocked(
            WINFW_TIMEOUT_SECONDS,
            &cfg,
            &allowed_endpoint.as_endpoint(),
            Some(log_sink),
            LOGGING_CONTEXT.as_ptr(),
        )
    };
    init.into_result()
}

/// Deinitialize WinFw module. Trying to use WinFw after calling deinit will result in an
/// error before [Self::initialize] is called.
pub(super) fn deinit(cleanup_policy: WinFwCleanupPolicy) -> Result<(), Error> {
    // SAFETY: WinFw_Deinitialize is always safe to call.
    // Will simply return false if WinFw already has been deinitialized.
    let deinit = unsafe { WinFw_Deinitialize(cleanup_policy) };
    deinit.into_result()
}

/// Reset all firewall policies applied by [winfw].
///
/// Sets the underlying active policy to None.
pub(super) fn reset() -> Result<(), FirewallPolicyError> {
    // SAFETY: WinFw_Reset is always safe to call, even before WinFW has been
    // initialized and after WinFW has been deinitialized.
    let reset = unsafe { WinFw_Reset() };
    reset.into_result()
}

/// Apply blocking firewall rules Sets the underlying active policy to Blocked. Exceptions
/// permitted through the firewall is defined by `winfw_settings` and `allowed_endpoint`. See
/// the BlockAll class for more information.
///
/// Returns an error if [winfw] is not initialized.
pub(super) fn apply_policy_blocked(
    winfw_settings: &WinFwSettings,
    allowed_endpoint: Option<WinFwAllowedEndpointContainer>,
) -> Result<(), FirewallPolicyError> {
    let allowed_endpoint = allowed_endpoint
        .as_ref()
        .map(WinFwAllowedEndpointContainer::as_endpoint)
        .as_ref()
        .map(ptr::from_ref)
        .unwrap_or(ptr::null());
    // SAFETY: This function is always safe to call
    let application = unsafe { WinFw_ApplyPolicyBlocked(winfw_settings, allowed_endpoint) };
    application.into_result()
}

pub(super) fn apply_policy_connecting(
    peer_endpoint: &AllowedEndpoint,
    winfw_settings: &WinFwSettings,
    tunnel_interface: Option<&str>,
    allowed_endpoint: AllowedEndpoint,
    allowed_tunnel_traffic: &AllowedTunnelTraffic,
) -> Result<(), FirewallPolicyError> {
    let ip_str = widestring_ip(peer_endpoint.endpoint.address.ip());
    let winfw_relay = WinFwEndpoint {
        ip: ip_str.as_ptr(),
        port: peer_endpoint.endpoint.address.port(),
        protocol: WinFwProt::from(peer_endpoint.endpoint.protocol),
    };

    // SAFETY: `endpoint1_ip`, `endpoint2_ip`, `endpoint1`, `endpoint2`, `relay_client_wstrs`
    // must not be dropped until `WinFw_ApplyPolicyConnecting` has returned.

    let relay_client_wstrs: Vec<_> = peer_endpoint
        .clients
        .iter()
        .map(WideCString::from_os_str_truncate)
        .collect();
    let relay_client_wstr_ptrs: Vec<*const u16> = relay_client_wstrs
        .iter()
        .map(|wstr| wstr.as_ptr())
        .collect();
    let relay_client_wstr_ptrs_len = relay_client_wstr_ptrs.len();

    let interface_wstr = tunnel_interface
        .as_ref()
        .map(WideCString::from_str_truncate);
    let interface_wstr_ptr = if let Some(ref wstr) = interface_wstr {
        wstr.as_ptr()
    } else {
        ptr::null()
    };

    let mut endpoint1_ip = WideCString::new();
    let mut endpoint2_ip = WideCString::new();
    let (endpoint1, endpoint2) = match allowed_tunnel_traffic {
        AllowedTunnelTraffic::One(endpoint) => {
            endpoint1_ip = widestring_ip(endpoint.address.ip());
            (
                Some(WinFwEndpoint {
                    ip: endpoint1_ip.as_ptr(),
                    port: endpoint.address.port(),
                    protocol: WinFwProt::from(endpoint.protocol),
                }),
                None,
            )
        }
        AllowedTunnelTraffic::Two(endpoint1, endpoint2) => {
            endpoint1_ip = widestring_ip(endpoint1.address.ip());
            let endpoint1 = Some(WinFwEndpoint {
                ip: endpoint1_ip.as_ptr(),
                port: endpoint1.address.port(),
                protocol: WinFwProt::from(endpoint1.protocol),
            });
            endpoint2_ip = widestring_ip(endpoint2.address.ip());
            let endpoint2 = Some(WinFwEndpoint {
                ip: endpoint2_ip.as_ptr(),
                port: endpoint2.address.port(),
                protocol: WinFwProt::from(endpoint2.protocol),
            });
            (endpoint1, endpoint2)
        }
        AllowedTunnelTraffic::None | AllowedTunnelTraffic::All => (None, None),
    };

    let allowed_endpoint = WinFwAllowedEndpointContainer::from(allowed_endpoint);
    let allowed_endpoint = allowed_endpoint.as_endpoint();

    let allowed_tunnel_traffic = WinFwAllowedTunnelTraffic {
        type_: WinFwAllowedTunnelTrafficType::from(allowed_tunnel_traffic),
        endpoint1: endpoint1
            .as_ref()
            .map(|ep| ep as *const _)
            .unwrap_or(ptr::null()),
        endpoint2: endpoint2
            .as_ref()
            .map(|ep| ep as *const _)
            .unwrap_or(ptr::null()),
    };

    #[allow(clippy::undocumented_unsafe_blocks)] // Remove me if you dare.
    let res = unsafe {
        WinFw_ApplyPolicyConnecting(
            winfw_settings,
            &winfw_relay,
            relay_client_wstr_ptrs.as_ptr(),
            relay_client_wstr_ptrs_len,
            interface_wstr_ptr,
            &allowed_endpoint,
            &allowed_tunnel_traffic,
        )
    };
    // SAFETY: All of these hold stack allocated memory which is pointed to by
    // `allowed_tunnel_traffic` and must remain allocated until `WinFw_ApplyPolicyConnecting`
    // has returned.
    drop(endpoint1_ip);
    drop(endpoint2_ip);
    #[allow(clippy::drop_non_drop)]
    drop(endpoint1);
    #[allow(clippy::drop_non_drop)]
    drop(endpoint2);
    drop(relay_client_wstrs);
    res.into_result()
}

pub(super) fn apply_policy_connected(
    endpoint: &AllowedEndpoint,
    winfw_settings: &WinFwSettings,
    tunnel_interface: &str,
    dns_config: &crate::dns::ResolvedDnsConfig,
) -> Result<(), FirewallPolicyError> {
    let ip_str = widestring_ip(endpoint.endpoint.address.ip());

    let tunnel_alias = WideCString::from_str_truncate(tunnel_interface);

    // ip_str, gateway_str and tunnel_alias have to outlive winfw_relay
    let winfw_relay = WinFwEndpoint {
        ip: ip_str.as_ptr(),
        port: endpoint.endpoint.address.port(),
        protocol: WinFwProt::from(endpoint.endpoint.protocol),
    };

    // SAFETY: `relay_client_wstrs` must not be dropped until `WinFw_ApplyPolicyConnected` has
    // returned.
    let relay_client_wstrs: Vec<_> = endpoint
        .clients
        .iter()
        .map(WideCString::from_os_str_truncate)
        .collect();
    let relay_client_wstr_ptrs: Vec<*const u16> = relay_client_wstrs
        .iter()
        .map(|wstr| wstr.as_ptr())
        .collect();
    let relay_client_wstr_ptrs_len = relay_client_wstr_ptrs.len();

    let tunnel_dns_servers: Vec<WideCString> = dns_config
        .tunnel_config()
        .iter()
        .cloned()
        .map(widestring_ip)
        .collect();
    let tunnel_dns_servers: Vec<*const u16> =
        tunnel_dns_servers.iter().map(|ip| ip.as_ptr()).collect();
    let non_tunnel_dns_servers: Vec<WideCString> = dns_config
        .non_tunnel_config()
        .iter()
        .cloned()
        .map(widestring_ip)
        .collect();
    let non_tunnel_dns_servers: Vec<*const u16> = non_tunnel_dns_servers
        .iter()
        .map(|ip| ip.as_ptr())
        .collect();

    #[allow(clippy::undocumented_unsafe_blocks)] // Remove me if you dare.
    let result = unsafe {
        WinFw_ApplyPolicyConnected(
            winfw_settings,
            &winfw_relay,
            relay_client_wstr_ptrs.as_ptr(),
            relay_client_wstr_ptrs_len,
            tunnel_alias.as_ptr(),
            tunnel_dns_servers.as_ptr(),
            tunnel_dns_servers.len(),
            non_tunnel_dns_servers.as_ptr(),
            non_tunnel_dns_servers.len(),
        )
    };

    // SAFETY: `relay_client_wstrs` holds memory pointed to by pointers used in C++ and must
    // not be dropped until after `WinFw_ApplyPolicyConnected` has returned.
    drop(relay_client_wstrs);
    result.into_result()
}

/// Logging callback implementation.
///
/// SAFETY:
/// - `msg` must point to a valid C string or be null.
/// - `context` must point to a valid C string or be null.
pub extern "system" fn log_sink(
    level: log::Level,
    msg: *const std::ffi::c_char,
    context: *mut std::ffi::c_void,
) {
    if msg.is_null() {
        log::error!("Log message from FFI boundary is NULL");
        return;
    }

    let target = if context.is_null() {
        "UNKNOWN".into()
    } else {
        // SAFETY: context is not null & caller promise that context is a valid C string.
        unsafe { CStr::from_ptr(context as *const _).to_string_lossy() }
    };

    // SAFETY: msg is not null & caller promise that msg is a valid C string.
    let mb_string = unsafe { CStr::from_ptr(msg) };

    let managed_msg = match multibyte_to_wide(mb_string, CP_ACP) {
        Ok(wide_str) => String::from_utf16_lossy(&wide_str),
        // Best effort:
        Err(_) => mb_string.to_string_lossy().into_owned(),
    };

    log::logger().log(
        &log::Record::builder()
            .level(level)
            .target(&target)
            .args(format_args!("{}", managed_msg))
            .build(),
    );
}

impl From<AllowedEndpoint> for WinFwAllowedEndpointContainer {
    fn from(endpoint: AllowedEndpoint) -> Self {
        let clients = endpoint
            .clients
            .iter()
            .map(WideCString::from_os_str_truncate)
            .collect::<Box<_>>();
        let clients_ptrs = clients
            .iter()
            .map(|client| client.as_ptr())
            .collect::<Box<_>>();
        let ip = widestring_ip(endpoint.endpoint.address.ip());

        WinFwAllowedEndpointContainer {
            _clients: clients,
            clients_ptrs,
            ip,
            port: endpoint.endpoint.address.port(),
            protocol: WinFwProt::from(endpoint.endpoint.protocol),
        }
    }
}

impl WinFwAllowedEndpointContainer {
    pub fn as_endpoint(&self) -> WinFwAllowedEndpoint<'_> {
        WinFwAllowedEndpoint {
            num_clients: self.clients_ptrs.len() as u32,
            clients: self.clients_ptrs.as_ptr(),
            endpoint: WinFwEndpoint {
                ip: self.ip.as_ptr(),
                port: self.port,
                protocol: self.protocol,
            },

            _phantom: std::marker::PhantomData,
        }
    }
}

#[repr(C)]
pub struct WinFwAllowedEndpoint<'a> {
    num_clients: u32,
    clients: *const *const libc::wchar_t,
    endpoint: WinFwEndpoint,

    _phantom: std::marker::PhantomData<&'a WinFwAllowedEndpointContainer>,
}

#[repr(C)]
pub struct WinFwAllowedTunnelTraffic {
    pub type_: WinFwAllowedTunnelTrafficType,
    pub endpoint1: *const WinFwEndpoint,
    pub endpoint2: *const WinFwEndpoint,
}

#[repr(u8)]
#[derive(Clone, Copy)]
pub enum WinFwAllowedTunnelTrafficType {
    None,
    All,
    One,
    Two,
}

impl From<&AllowedTunnelTraffic> for WinFwAllowedTunnelTrafficType {
    fn from(traffic: &AllowedTunnelTraffic) -> Self {
        match traffic {
            AllowedTunnelTraffic::None => WinFwAllowedTunnelTrafficType::None,
            AllowedTunnelTraffic::All => WinFwAllowedTunnelTrafficType::All,
            AllowedTunnelTraffic::One(..) => WinFwAllowedTunnelTrafficType::One,
            AllowedTunnelTraffic::Two(..) => WinFwAllowedTunnelTrafficType::Two,
        }
    }
}

#[repr(C)]
pub struct WinFwEndpoint {
    pub ip: *const libc::wchar_t,
    pub port: u16,
    pub protocol: WinFwProt,
}

#[repr(u8)]
#[derive(Clone, Copy)]
pub enum WinFwProt {
    Tcp = 0u8,
    Udp = 1u8,
}

impl From<TransportProtocol> for WinFwProt {
    fn from(prot: TransportProtocol) -> WinFwProt {
        match prot {
            TransportProtocol::Tcp => WinFwProt::Tcp,
            TransportProtocol::Udp => WinFwProt::Udp,
        }
    }
}

#[repr(C)]
#[allow(non_snake_case)]
pub struct WinFwSettings {
    permitDhcp: bool,
    permitLan: bool,
}

impl WinFwSettings {
    pub fn new(permit_lan: bool) -> WinFwSettings {
        WinFwSettings {
            permitDhcp: true,
            permitLan: permit_lan,
        }
    }
}

#[allow(dead_code)]
#[repr(u32)]
#[derive(Clone, Copy)]
pub enum WinFwCleanupPolicy {
    ContinueBlocking = 0,
    ResetFirewall = 1,
    BlockingUntilReboot = 2,
}

ffi_error!(InitializationResult, Error::Initialization);
ffi_error!(DeinitializationResult, Error::Deinitialization);

#[derive(Debug)]
#[allow(dead_code)]
#[repr(u32)]
pub enum WinFwPolicyStatus {
    Success = 0,
    GeneralFailure = 1,
    LockTimeout = 2,
}

impl WinFwPolicyStatus {
    pub fn into_result(self) -> Result<(), super::FirewallPolicyError> {
        match self {
            WinFwPolicyStatus::Success => Ok(()),
            WinFwPolicyStatus::GeneralFailure => Err(super::FirewallPolicyError::Generic),
            WinFwPolicyStatus::LockTimeout => {
                // TODO: Obtain application name and string from WinFw
                Err(super::FirewallPolicyError::Locked(None))
            }
        }
    }
}

impl From<WinFwPolicyStatus> for Result<(), super::FirewallPolicyError> {
    fn from(val: WinFwPolicyStatus) -> Self {
        val.into_result()
    }
}

unsafe extern "system" {
    #[link_name = "WinFw_Initialize"]
    pub fn WinFw_Initialize(
        timeout: libc::c_uint,
        sink: Option<LogSink>,
        sink_context: *const c_char,
    ) -> InitializationResult;

    #[link_name = "WinFw_InitializeBlocked"]
    pub fn WinFw_InitializeBlocked(
        timeout: libc::c_uint,
        settings: &WinFwSettings,
        allowed_endpoint: *const WinFwAllowedEndpoint<'_>,
        sink: Option<LogSink>,
        sink_context: *const c_char,
    ) -> InitializationResult;

    #[link_name = "WinFw_Deinitialize"]
    pub fn WinFw_Deinitialize(cleanupPolicy: WinFwCleanupPolicy) -> DeinitializationResult;

    #[link_name = "WinFw_ApplyPolicyConnecting"]
    pub fn WinFw_ApplyPolicyConnecting(
        settings: &WinFwSettings,
        relay: &WinFwEndpoint,
        relayClient: *const *const libc::wchar_t,
        relayClientLen: usize,
        tunnelIfaceAlias: *const libc::wchar_t,
        allowedEndpoint: *const WinFwAllowedEndpoint<'_>,
        allowedTunnelTraffic: &WinFwAllowedTunnelTraffic,
    ) -> WinFwPolicyStatus;

    #[link_name = "WinFw_ApplyPolicyConnected"]
    pub fn WinFw_ApplyPolicyConnected(
        settings: &WinFwSettings,
        relay: &WinFwEndpoint,
        relayClient: *const *const libc::wchar_t,
        relayClientLen: usize,
        tunnelIfaceAlias: *const libc::wchar_t,
        tunnelDnsServers: *const *const libc::wchar_t,
        numTunnelDnsServers: usize,
        nonTunnelDnsServers: *const *const libc::wchar_t,
        numNonTunnelDnsServers: usize,
    ) -> WinFwPolicyStatus;

    #[link_name = "WinFw_ApplyPolicyBlocked"]
    pub fn WinFw_ApplyPolicyBlocked(
        settings: &WinFwSettings,
        allowed_endpoint: *const WinFwAllowedEndpoint<'_>,
    ) -> WinFwPolicyStatus;

    #[link_name = "WinFw_Reset"]
    pub fn WinFw_Reset() -> WinFwPolicyStatus;
}

use crate::{dns::ResolvedDnsConfig, tunnel::TunnelMetadata};

use std::{ffi::CStr, io, net::IpAddr, ptr, sync::LazyLock};

use self::winfw::*;
use super::{FirewallArguments, FirewallPolicy, InitialFirewallState};
use talpid_types::{
    ErrorExt,
    net::{AllowedEndpoint, AllowedTunnelTraffic},
    tunnel::FirewallPolicyError,
};
use widestring::WideCString;
use windows_sys::Win32::Globalization::{CP_ACP, MultiByteToWideChar};

mod hyperv;

const HYPERV_LEAK_WARNING_MSG: &str = "Hyper-V (e.g. WSL machines) may leak in blocked states.";

// `COMLibrary` must be initialized for per thread, so use TLS
thread_local! {
    static WMI: Option<wmi::WMIConnection> = {
        let result = hyperv::init_wmi();
        if matches!(&result, Err(hyperv::Error::ObtainHyperVClass(_))) {
            log::warn!("The Hyper-V firewall is not available. {HYPERV_LEAK_WARNING_MSG}");
            return None;
        }
        consume_and_log_hyperv_err(
            "Initialize COM and WMI",
            result,
        )
    };
}

/// Enable or disable blocking Hyper-V rule
static BLOCK_HYPERV: LazyLock<bool> = LazyLock::new(|| {
    let enable = std::env::var("TALPID_FIREWALL_BLOCK_HYPERV")
        .map(|v| v != "0")
        .unwrap_or(true);

    if !enable {
        log::debug!("Hyper-V block rule disabled by TALPID_FIREWALL_BLOCK_HYPERV");
    }

    enable
});

/// Errors that can happen when configuring the Windows firewall.
#[derive(thiserror::Error, Debug)]
pub enum Error {
    /// Failure to initialize windows firewall module
    #[error("Failed to initialize windows firewall module")]
    Initialization,

    /// Failure to deinitialize windows firewall module
    #[error("Failed to deinitialize windows firewall module")]
    Deinitialization,

    /// Failure to apply a firewall _connecting_ policy
    #[error("Failed to apply connecting firewall policy")]
    ApplyingConnectingPolicy(#[source] FirewallPolicyError),

    /// Failure to apply a firewall _connected_ policy
    #[error("Failed to apply connected firewall policy")]
    ApplyingConnectedPolicy(#[source] FirewallPolicyError),

    /// Failure to apply firewall _blocked_ policy
    #[error("Failed to apply blocked firewall policy")]
    ApplyingBlockedPolicy(#[source] FirewallPolicyError),

    /// Failure to reset firewall policies
    #[error("Failed to reset firewall policies")]
    ResettingPolicy(#[source] FirewallPolicyError),
}

/// Timeout for acquiring the WFP transaction lock
const WINFW_TIMEOUT_SECONDS: u32 = 5;

const LOGGING_CONTEXT: &[u8] = b"WinFw\0";

/// The Windows implementation for the firewall.
pub struct Firewall(());

impl Firewall {
    pub fn from_args(args: FirewallArguments) -> Result<Self, Error> {
        if let InitialFirewallState::Blocked(allowed_endpoint) = args.initial_state {
            Self::initialize_blocked(allowed_endpoint, args.allow_lan)
        } else {
            Self::new()
        }
    }

    pub fn new() -> Result<Self, Error> {
        unsafe {
            WinFw_Initialize(
                WINFW_TIMEOUT_SECONDS,
                Some(log_sink),
                LOGGING_CONTEXT.as_ptr(),
            )
            .into_result()?
        };

        log::trace!("Successfully initialized windows firewall module");
        Ok(Firewall(()))
    }

    fn initialize_blocked(
        allowed_endpoint: AllowedEndpoint,
        allow_lan: bool,
    ) -> Result<Self, Error> {
        let cfg = &WinFwSettings::new(allow_lan);
        let allowed_endpoint = WinFwAllowedEndpointContainer::from(allowed_endpoint);
        unsafe {
            WinFw_InitializeBlocked(
                WINFW_TIMEOUT_SECONDS,
                cfg,
                &allowed_endpoint.as_endpoint(),
                Some(log_sink),
                LOGGING_CONTEXT.as_ptr(),
            )
            .into_result()?
        };
        log::trace!("Successfully initialized windows firewall module to a blocking state");

        with_wmi_if_enabled(|wmi| {
            let result = hyperv::add_blocking_hyperv_firewall_rules(wmi);
            consume_and_log_hyperv_err("Add block-all Hyper-V filter", result);
        });

        Ok(Firewall(()))
    }

    pub fn apply_policy(&mut self, policy: FirewallPolicy) -> Result<(), Error> {
        let should_block_hyperv = matches!(
            policy,
            FirewallPolicy::Connecting { .. } | FirewallPolicy::Blocked { .. }
        );

        let apply_result = match policy {
            FirewallPolicy::Connecting {
                peer_endpoint,
                tunnel,
                allow_lan,
                allowed_endpoint,
                allowed_tunnel_traffic,
            } => {
                let cfg = &WinFwSettings::new(allow_lan);

                self.set_connecting_state(
                    &peer_endpoint,
                    cfg,
                    &tunnel,
                    &WinFwAllowedEndpointContainer::from(allowed_endpoint).as_endpoint(),
                    &allowed_tunnel_traffic,
                )
            }
            FirewallPolicy::Connected {
                peer_endpoint,
                tunnel,
                allow_lan,
                dns_config,
            } => {
                let cfg = &WinFwSettings::new(allow_lan);
                self.set_connected_state(&peer_endpoint, cfg, &tunnel, &dns_config)
            }
            FirewallPolicy::Blocked {
                allow_lan,
                allowed_endpoint,
            } => {
                let cfg = &WinFwSettings::new(allow_lan);
                self.set_blocked_state(
                    cfg,
                    allowed_endpoint.map(WinFwAllowedEndpointContainer::from),
                )
            }
        };

        with_wmi_if_enabled(|wmi| {
            if should_block_hyperv {
                let result = hyperv::add_blocking_hyperv_firewall_rules(wmi);
                consume_and_log_hyperv_err("Add block-all Hyper-V filter", result);
            } else {
                let result = hyperv::remove_blocking_hyperv_firewall_rules(wmi);
                consume_and_log_hyperv_err("Remove block-all Hyper-V filter", result);
            }
        });

        apply_result
    }

    pub fn reset_policy(&mut self) -> Result<(), Error> {
        unsafe { WinFw_Reset().into_result().map_err(Error::ResettingPolicy) }?;

        with_wmi_if_enabled(|wmi| {
            let result = hyperv::remove_blocking_hyperv_firewall_rules(wmi);
            consume_and_log_hyperv_err("Remove block-all Hyper-V filter", result);
        });

        Ok(())
    }

    fn set_connecting_state(
        &mut self,
        endpoint: &AllowedEndpoint,
        winfw_settings: &WinFwSettings,
        tunnel_metadata: &Option<TunnelMetadata>,
        allowed_endpoint: &WinFwAllowedEndpoint<'_>,
        allowed_tunnel_traffic: &AllowedTunnelTraffic,
    ) -> Result<(), Error> {
        log::trace!("Applying 'connecting' firewall policy");
        let ip_str = widestring_ip(endpoint.endpoint.address.ip());
        let winfw_relay = WinFwEndpoint {
            ip: ip_str.as_ptr(),
            port: endpoint.endpoint.address.port(),
            protocol: WinFwProt::from(endpoint.endpoint.protocol),
        };

        // SAFETY: `endpoint1_ip`, `endpoint2_ip`, `endpoint1`, `endpoint2`, `relay_client_wstrs`
        // must not be dropped until `WinFw_ApplyPolicyConnecting` has returned.

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

        let interface_wstr = tunnel_metadata
            .as_ref()
            .map(|metadata| WideCString::from_str_truncate(&metadata.interface));
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

        let res = unsafe {
            WinFw_ApplyPolicyConnecting(
                winfw_settings,
                &winfw_relay,
                relay_client_wstr_ptrs.as_ptr(),
                relay_client_wstr_ptrs_len,
                interface_wstr_ptr,
                allowed_endpoint,
                &allowed_tunnel_traffic,
            )
            .into_result()
            .map_err(Error::ApplyingConnectingPolicy)
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
        res
    }

    fn set_connected_state(
        &mut self,
        endpoint: &AllowedEndpoint,
        winfw_settings: &WinFwSettings,
        tunnel_metadata: &TunnelMetadata,
        dns_config: &ResolvedDnsConfig,
    ) -> Result<(), Error> {
        log::trace!("Applying 'connected' firewall policy");
        let ip_str = widestring_ip(endpoint.endpoint.address.ip());

        let tunnel_alias = WideCString::from_str_truncate(&tunnel_metadata.interface);

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
            .into_result()
            .map_err(Error::ApplyingConnectedPolicy)
        };

        // SAFETY: `relay_client_wstrs` holds memory pointed to by pointers used in C++ and must
        // not be dropped until after `WinFw_ApplyPolicyConnected` has returned.
        drop(relay_client_wstrs);
        result
    }

    fn set_blocked_state(
        &mut self,
        winfw_settings: &WinFwSettings,
        allowed_endpoint: Option<WinFwAllowedEndpointContainer>,
    ) -> Result<(), Error> {
        log::trace!("Applying 'blocked' firewall policy");
        let endpoint = allowed_endpoint
            .as_ref()
            .map(WinFwAllowedEndpointContainer::as_endpoint);

        unsafe {
            WinFw_ApplyPolicyBlocked(
                winfw_settings,
                endpoint
                    .as_ref()
                    .map(|container| container as *const _)
                    .unwrap_or(ptr::null()),
            )
            .into_result()
            .map_err(Error::ApplyingBlockedPolicy)
        }
    }
}

impl Drop for Firewall {
    fn drop(&mut self) {
        if unsafe {
            WinFw_Deinitialize(WinFwCleanupPolicy::ContinueBlocking)
                .into_result()
                .is_ok()
        } {
            log::trace!("Successfully deinitialized windows firewall module");
        } else {
            log::error!("Failed to deinitialize windows firewall module");
        };
    }
}

fn widestring_ip(ip: IpAddr) -> WideCString {
    WideCString::from_str_truncate(ip.to_string())
}

/// Logging callback implementation.
pub extern "system" fn log_sink(
    level: log::Level,
    msg: *const std::ffi::c_char,
    context: *mut std::ffi::c_void,
) {
    if msg.is_null() {
        log::error!("Log message from FFI boundary is NULL");
    } else {
        let target = if context.is_null() {
            "UNKNOWN".into()
        } else {
            unsafe { CStr::from_ptr(context as *const _).to_string_lossy() }
        };

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
}

/// Convert `mb_string`, with the given character encoding `codepage`, to a UTF-16 string.
fn multibyte_to_wide(mb_string: &CStr, codepage: u32) -> Result<Vec<u16>, io::Error> {
    if mb_string.is_empty() {
        return Ok(vec![]);
    }

    // SAFETY: `mb_string` is null-terminated and valid.
    let wc_size = unsafe {
        MultiByteToWideChar(
            codepage,
            0,
            mb_string.as_ptr() as *const u8,
            -1,
            ptr::null_mut(),
            0,
        )
    };

    if wc_size == 0 {
        return Err(io::Error::last_os_error());
    }

    let mut wc_buffer = vec![0u16; usize::try_from(wc_size).unwrap()];

    // SAFETY: `wc_buffer` can contain up to `wc_size` characters, including a null
    // terminator.
    let chars_written = unsafe {
        MultiByteToWideChar(
            codepage,
            0,
            mb_string.as_ptr() as *const u8,
            -1,
            wc_buffer.as_mut_ptr(),
            wc_size,
        )
    };

    if chars_written == 0 {
        return Err(io::Error::last_os_error());
    }

    wc_buffer.truncate(usize::try_from(chars_written - 1).unwrap());

    Ok(wc_buffer)
}

#[cfg(test)]
mod test {
    use super::multibyte_to_wide;
    use windows_sys::Win32::Globalization::CP_UTF8;

    #[test]
    fn test_multibyte_to_wide() {
        // € = 0x20AC in UTF-16
        let converted = multibyte_to_wide(c"€€", CP_UTF8);
        const EXPECTED: &[u16] = &[0x20AC, 0x20AC];
        assert!(
            matches!(converted.as_deref(), Ok(EXPECTED)),
            "expected Ok({EXPECTED:?}), got {converted:?}",
        );

        // boundary case
        let converted = multibyte_to_wide(c"", CP_UTF8);
        assert!(
            matches!(converted.as_deref(), Ok([])),
            "unexpected result {converted:?}"
        );
    }
}

// Convert `result` into an option and log the error, if any.
fn consume_and_log_hyperv_err<T>(
    action: &'static str,
    result: Result<T, hyperv::Error>,
) -> Option<T> {
    result
        .inspect_err(|error| {
            log::error!(
                "{}",
                error.display_chain_with_msg(&format!("{action}. {HYPERV_LEAK_WARNING_MSG}"))
            );
        })
        .ok()
}

// Run a closure with the current thread's WMI connection, if available
fn with_wmi_if_enabled(f: impl FnOnce(&wmi::WMIConnection)) {
    if !*BLOCK_HYPERV {
        return;
    }
    WMI.with(|wmi| {
        if let Some(con) = wmi {
            f(con)
        }
    })
}

#[allow(non_snake_case)]
mod winfw {
    use super::{AllowedEndpoint, AllowedTunnelTraffic, Error, WideCString, widestring_ip};
    use std::ffi::{c_char, c_void};
    use talpid_types::net::TransportProtocol;

    type LogSink = extern "system" fn(level: log::Level, msg: *const c_char, context: *mut c_void);

    pub struct WinFwAllowedEndpointContainer {
        _clients: Box<[WideCString]>,
        clients_ptrs: Box<[*const u16]>,
        ip: WideCString,
        port: u16,
        protocol: WinFwProt,
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
            sink_context: *const u8,
        ) -> InitializationResult;

        #[link_name = "WinFw_InitializeBlocked"]
        pub fn WinFw_InitializeBlocked(
            timeout: libc::c_uint,
            settings: &WinFwSettings,
            allowed_endpoint: *const WinFwAllowedEndpoint<'_>,
            sink: Option<LogSink>,
            sink_context: *const u8,
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
}

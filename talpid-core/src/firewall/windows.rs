use crate::tunnel::TunnelMetadata;

use std::{ffi::CStr, io, net::IpAddr, path::Path, ptr};

use self::winfw::*;
use super::{FirewallArguments, FirewallPolicy, InitialFirewallState};
use talpid_types::{
    net::{AllowedEndpoint, AllowedTunnelTraffic, Endpoint},
    tunnel::FirewallPolicyError,
};
use widestring::WideCString;
use windows_sys::Win32::Globalization::{MultiByteToWideChar, CP_ACP};

/// Errors that can happen when configuring the Windows firewall.
#[derive(err_derive::Error, Debug)]
#[error(no_from)]
pub enum Error {
    /// Failure to initialize windows firewall module
    #[error(display = "Failed to initialize windows firewall module")]
    Initialization,

    /// Failure to deinitialize windows firewall module
    #[error(display = "Failed to deinitialize windows firewall module")]
    Deinitialization,

    /// Failure to apply a firewall _connecting_ policy
    #[error(display = "Failed to apply connecting firewall policy")]
    ApplyingConnectingPolicy(#[error(source)] FirewallPolicyError),

    /// Failure to apply a firewall _connected_ policy
    #[error(display = "Failed to apply connected firewall policy")]
    ApplyingConnectedPolicy(#[error(source)] FirewallPolicyError),

    /// Failure to apply firewall _blocked_ policy
    #[error(display = "Failed to apply blocked firewall policy")]
    ApplyingBlockedPolicy(#[error(source)] FirewallPolicyError),

    /// Failure to reset firewall policies
    #[error(display = "Failed to reset firewall policies")]
    ResettingPolicy(#[error(source)] FirewallPolicyError),
}

/// Timeout for acquiring the WFP transaction lock
const WINFW_TIMEOUT_SECONDS: u32 = 5;

const LOGGING_CONTEXT: &[u8] = b"WinFw\0";

/// The Windows implementation for the firewall and DNS.
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
                &cfg,
                &allowed_endpoint.as_endpoint(),
                Some(log_sink),
                LOGGING_CONTEXT.as_ptr(),
            )
            .into_result()?
        };
        log::trace!("Successfully initialized windows firewall module to a blocking state");
        Ok(Firewall(()))
    }

    pub fn apply_policy(&mut self, policy: FirewallPolicy) -> Result<(), Error> {
        match policy {
            FirewallPolicy::Connecting {
                peer_endpoint,
                tunnel,
                allow_lan,
                allowed_endpoint,
                allowed_tunnel_traffic,
                relay_client,
            } => {
                let cfg = &WinFwSettings::new(allow_lan);

                self.set_connecting_state(
                    &peer_endpoint,
                    &cfg,
                    &tunnel,
                    &WinFwAllowedEndpointContainer::from(allowed_endpoint).as_endpoint(),
                    &allowed_tunnel_traffic,
                    &relay_client,
                )
            }
            FirewallPolicy::Connected {
                peer_endpoint,
                tunnel,
                allow_lan,
                dns_servers,
                relay_client,
            } => {
                let cfg = &WinFwSettings::new(allow_lan);
                self.set_connected_state(&peer_endpoint, &cfg, &tunnel, &dns_servers, &relay_client)
            }
            FirewallPolicy::Blocked {
                allow_lan,
                allowed_endpoint,
            } => {
                let cfg = &WinFwSettings::new(allow_lan);
                self.set_blocked_state(
                    &cfg,
                    allowed_endpoint.map(|endpoint| WinFwAllowedEndpointContainer::from(endpoint)),
                )
            }
        }
    }

    pub fn reset_policy(&mut self) -> Result<(), Error> {
        unsafe { WinFw_Reset().into_result().map_err(Error::ResettingPolicy) }?;
        Ok(())
    }

    fn set_connecting_state(
        &mut self,
        endpoint: &Endpoint,
        winfw_settings: &WinFwSettings,
        tunnel_metadata: &Option<TunnelMetadata>,
        allowed_endpoint: &WinFwAllowedEndpoint<'_>,
        allowed_tunnel_traffic: &AllowedTunnelTraffic,
        relay_client: &Path,
    ) -> Result<(), Error> {
        log::trace!("Applying 'connecting' firewall policy");
        let ip_str = widestring_ip(endpoint.address.ip());
        let winfw_relay = WinFwEndpoint {
            ip: ip_str.as_ptr(),
            port: endpoint.address.port(),
            protocol: WinFwProt::from(endpoint.protocol),
        };

        let relay_client = WideCString::from_os_str_truncate(relay_client);

        let interface_wstr = tunnel_metadata
            .as_ref()
            .map(|metadata| WideCString::from_str_truncate(&metadata.interface));
        let interface_wstr_ptr = if let Some(ref wstr) = interface_wstr {
            wstr.as_ptr()
        } else {
            ptr::null()
        };

        let allowed_tun_ip;
        let allowed_tunnel_endpoint =
            if let AllowedTunnelTraffic::Only(endpoint) = allowed_tunnel_traffic {
                allowed_tun_ip = widestring_ip(endpoint.address.ip());
                Some(WinFwEndpoint {
                    ip: allowed_tun_ip.as_ptr(),
                    port: endpoint.address.port(),
                    protocol: WinFwProt::from(endpoint.protocol),
                })
            } else {
                None
            };
        let allowed_tunnel_traffic = WinFwAllowedTunnelTraffic {
            type_: WinFwAllowedTunnelTrafficType::from(allowed_tunnel_traffic),
            endpoint: allowed_tunnel_endpoint
                .as_ref()
                .map(|ep| ep as *const _)
                .unwrap_or(ptr::null()),
        };

        unsafe {
            WinFw_ApplyPolicyConnecting(
                winfw_settings,
                &winfw_relay,
                relay_client.as_ptr(),
                interface_wstr_ptr,
                allowed_endpoint,
                &allowed_tunnel_traffic,
            )
            .into_result()
            .map_err(Error::ApplyingConnectingPolicy)
        }
    }

    fn set_connected_state(
        &mut self,
        endpoint: &Endpoint,
        winfw_settings: &WinFwSettings,
        tunnel_metadata: &TunnelMetadata,
        dns_servers: &[IpAddr],
        relay_client: &Path,
    ) -> Result<(), Error> {
        log::trace!("Applying 'connected' firewall policy");
        let ip_str = widestring_ip(endpoint.address.ip());
        let v4_gateway = widestring_ip(tunnel_metadata.ipv4_gateway.into());
        let v6_gateway = tunnel_metadata
            .ipv6_gateway
            .map(|v6_ip| widestring_ip(v6_ip.into()));

        let tunnel_alias = WideCString::from_str_truncate(&tunnel_metadata.interface);

        // ip_str, gateway_str and tunnel_alias have to outlive winfw_relay
        let winfw_relay = WinFwEndpoint {
            ip: ip_str.as_ptr(),
            port: endpoint.address.port(),
            protocol: WinFwProt::from(endpoint.protocol),
        };

        let v6_gateway_ptr = match &v6_gateway {
            Some(v6_ip) => v6_ip.as_ptr(),
            None => ptr::null(),
        };

        let relay_client = WideCString::from_os_str_truncate(relay_client);

        let dns_servers: Vec<WideCString> =
            dns_servers.iter().cloned().map(widestring_ip).collect();
        let dns_servers: Vec<*const u16> = dns_servers.iter().map(|ip| ip.as_ptr()).collect();

        unsafe {
            WinFw_ApplyPolicyConnected(
                winfw_settings,
                &winfw_relay,
                relay_client.as_ptr(),
                tunnel_alias.as_ptr(),
                v4_gateway.as_ptr(),
                v6_gateway_ptr,
                dns_servers.as_ptr(),
                dns_servers.len(),
            )
            .into_result()
            .map_err(Error::ApplyingConnectedPolicy)
        }
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
    msg: *const libc::c_char,
    context: *mut libc::c_void,
) {
    if msg.is_null() {
        log::error!("Log message from FFI boundary is NULL");
    } else {
        let rust_log_level = log::Level::from(level);
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
                .level(rust_log_level)
                .target(&target)
                .args(format_args!("{}", managed_msg))
                .build(),
        );
    }
}

fn multibyte_to_wide(mb_string: &CStr, codepage: u32) -> Result<Vec<u16>, io::Error> {
    if unsafe { *mb_string.as_ptr() } == 0 {
        return Ok(vec![]);
    }

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

    let mut wc_buffer = Vec::with_capacity(wc_size as usize);

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

    unsafe { wc_buffer.set_len((chars_written - 1) as usize) };

    Ok(wc_buffer)
}

#[allow(non_snake_case)]
mod winfw {
    use super::{widestring_ip, AllowedEndpoint, AllowedTunnelTraffic, Error, WideCString};
    use libc;
    use talpid_types::net::TransportProtocol;

    type LogSink =
        extern "system" fn(level: log::Level, msg: *const libc::c_char, context: *mut libc::c_void);

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
                .map(|client| WideCString::from_os_str_truncate(client))
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
        pub endpoint: *const WinFwEndpoint,
    }

    #[repr(u8)]
    #[derive(Clone, Copy)]
    pub enum WinFwAllowedTunnelTrafficType {
        None,
        All,
        Only,
    }

    impl From<&AllowedTunnelTraffic> for WinFwAllowedTunnelTrafficType {
        fn from(traffic: &AllowedTunnelTraffic) -> Self {
            match traffic {
                AllowedTunnelTraffic::None => WinFwAllowedTunnelTrafficType::None,
                AllowedTunnelTraffic::All => WinFwAllowedTunnelTrafficType::All,
                AllowedTunnelTraffic::Only(..) => WinFwAllowedTunnelTrafficType::Only,
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

    impl Into<Result<(), super::FirewallPolicyError>> for WinFwPolicyStatus {
        fn into(self) -> Result<(), super::FirewallPolicyError> {
            self.into_result()
        }
    }

    extern "system" {
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
            relayClient: *const libc::wchar_t,
            tunnelIfaceAlias: *const libc::wchar_t,
            allowedEndpoint: *const WinFwAllowedEndpoint<'_>,
            allowedTunnelTraffic: &WinFwAllowedTunnelTraffic,
        ) -> WinFwPolicyStatus;

        #[link_name = "WinFw_ApplyPolicyConnected"]
        pub fn WinFw_ApplyPolicyConnected(
            settings: &WinFwSettings,
            relay: &WinFwEndpoint,
            relayClient: *const libc::wchar_t,
            tunnelIfaceAlias: *const libc::wchar_t,
            v4Gateway: *const libc::wchar_t,
            v6Gateway: *const libc::wchar_t,
            dnsServers: *const *const libc::wchar_t,
            numDnsServers: usize,
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

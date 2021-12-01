use crate::{logging::windows::log_sink, tunnel::TunnelMetadata};

use std::{net::IpAddr, path::Path, ptr};

use self::winfw::*;
use super::{FirewallArguments, FirewallPolicy, FirewallT, InitialFirewallState};
use crate::winnet;
use log::{debug, error, trace};
use talpid_types::{
    net::{AllowedEndpoint, Endpoint},
    tunnel::FirewallPolicyError,
};
use widestring::WideCString;

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

    /// Failure to set virtual adapter metric
    #[error(display = "Unable to set virtual adapter metric")]
    SetTunMetric(#[error(source)] crate::winnet::Error),
}

const WINFW_TIMEOUT_SECONDS: u32 = 2;

/// The Windows implementation for the firewall and DNS.
pub struct Firewall(());

impl FirewallT for Firewall {
    type Error = Error;

    fn new(args: FirewallArguments) -> Result<Self, Self::Error> {
        let logging_context = b"WinFw\0".as_ptr();

        if let InitialFirewallState::Blocked(allowed_endpoint) = args.initial_state {
            let cfg = &WinFwSettings::new(args.allow_lan);
            let allowed_endpoint = WinFwAllowedEndpoint::from(allowed_endpoint);
            unsafe {
                WinFw_InitializeBlocked(
                    WINFW_TIMEOUT_SECONDS,
                    &cfg,
                    &allowed_endpoint,
                    Some(log_sink),
                    logging_context,
                )
                .into_result()?
            };
        } else {
            unsafe {
                WinFw_Initialize(WINFW_TIMEOUT_SECONDS, Some(log_sink), logging_context)
                    .into_result()?
            };
        }

        trace!("Successfully initialized windows firewall module");
        Ok(Firewall(()))
    }

    fn apply_policy(&mut self, policy: FirewallPolicy) -> Result<(), Self::Error> {
        match policy {
            FirewallPolicy::Connecting {
                peer_endpoint,
                tunnel,
                allow_lan,
                allowed_endpoint,
                relay_client,
            } => {
                let cfg = &WinFwSettings::new(allow_lan);

                self.set_connecting_state(
                    &peer_endpoint,
                    &cfg,
                    &tunnel,
                    WinFwAllowedEndpoint::from(allowed_endpoint),
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
                self.set_blocked_state(&cfg, WinFwAllowedEndpoint::from(allowed_endpoint))
            }
        }
    }

    fn reset_policy(&mut self) -> Result<(), Self::Error> {
        unsafe { WinFw_Reset().into_result().map_err(Error::ResettingPolicy) }?;
        Ok(())
    }
}

impl Drop for Firewall {
    fn drop(&mut self) {
        if unsafe {
            WinFw_Deinitialize(WinFwCleanupPolicy::ContinueBlocking)
                .into_result()
                .is_ok()
        } {
            trace!("Successfully deinitialized windows firewall module");
        } else {
            error!("Failed to deinitialize windows firewall module");
        };
    }
}

impl Firewall {
    fn set_connecting_state(
        &mut self,
        endpoint: &Endpoint,
        winfw_settings: &WinFwSettings,
        tunnel_metadata: &Option<TunnelMetadata>,
        allowed_endpoint: WinFwAllowedEndpoint,
        relay_client: &Path,
    ) -> Result<(), Error> {
        trace!("Applying 'connecting' firewall policy");
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

        unsafe {
            WinFw_ApplyPolicyConnecting(
                winfw_settings,
                &winfw_relay,
                relay_client.as_ptr(),
                interface_wstr_ptr,
                &allowed_endpoint,
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
        trace!("Applying 'connected' firewall policy");
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

        let metrics_set = winnet::ensure_best_metric_for_interface(&tunnel_metadata.interface)
            .map_err(Error::SetTunMetric)?;

        if metrics_set {
            debug!("Network interface metrics were changed");
        } else {
            debug!("Network interface metrics were not changed");
        }

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
        allowed_endpoint: WinFwAllowedEndpoint,
    ) -> Result<(), Error> {
        trace!("Applying 'blocked' firewall policy");
        unsafe {
            WinFw_ApplyPolicyBlocked(winfw_settings, &allowed_endpoint)
                .into_result()
                .map_err(Error::ApplyingBlockedPolicy)
        }
    }
}

fn widestring_ip(ip: IpAddr) -> WideCString {
    WideCString::from_str_truncate(ip.to_string())
}

#[allow(non_snake_case)]
mod winfw {
    use super::{widestring_ip, AllowedEndpoint, Error, WideCString};
    use crate::logging::windows::LogSink;
    use libc;
    use talpid_types::net::TransportProtocol;

    #[repr(C)]
    pub struct WinFwAllowedEndpoint {
        num_clients: u32,
        clients: *const *const libc::wchar_t,
        endpoint: WinFwEndpoint,
    }

    impl From<AllowedEndpoint> for WinFwAllowedEndpoint {
        fn from(endpoint: AllowedEndpoint) -> Self {
            let allowed_endpoint_ip = widestring_ip(endpoint.endpoint.address.ip());

            let clients = endpoint
                .clients
                .iter()
                .map(|client| WideCString::from_os_str_truncate(client).into_raw() as *const _)
                .collect::<Vec<_>>();

            let (clients, num_clients) = vec_into_raw_parts(clients);

            WinFwAllowedEndpoint {
                num_clients: num_clients as u32,
                clients,
                endpoint: WinFwEndpoint {
                    ip: allowed_endpoint_ip.into_raw(),
                    port: endpoint.endpoint.address.port(),
                    protocol: WinFwProt::from(endpoint.endpoint.protocol),
                },
            }
        }
    }

    impl Drop for WinFwAllowedEndpoint {
        fn drop(&mut self) {
            // Drop paths
            let clients: Vec<*mut u16> =
                unsafe { vec_from_raw_parts(self.clients as *mut _, self.num_clients as usize) };
            for client in clients {
                unsafe { WideCString::from_raw(client) };
            }

            // Drop address
            unsafe { WideCString::from_raw(self.endpoint.ip as *mut _) };
        }
    }

    /// Deconstructs a vec into raw parts without capacity.
    fn vec_into_raw_parts<T>(v: Vec<T>) -> (*mut T, usize) {
        let mut raw = v.into_boxed_slice();
        let ptr = raw.as_mut_ptr();
        let len = raw.len();
        std::mem::forget(raw);
        (ptr, len)
    }

    /// Constructs a vec from raw parts without capacity.
    unsafe fn vec_from_raw_parts<T>(v: *mut T, len: usize) -> Vec<T> {
        Box::from_raw(std::slice::from_raw_parts_mut(v, len)).into_vec()
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
            allowed_endpoint: *const WinFwAllowedEndpoint,
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
            allowed_endpoint: *const WinFwAllowedEndpoint,
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
            allowed_endpoint: *const WinFwAllowedEndpoint,
        ) -> WinFwPolicyStatus;

        #[link_name = "WinFw_Reset"]
        pub fn WinFw_Reset() -> WinFwPolicyStatus;
    }
}

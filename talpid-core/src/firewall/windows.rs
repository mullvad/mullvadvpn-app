use crate::logging::windows::log_sink;

use std::{net::IpAddr, path::Path, ptr};

use self::winfw::*;
use super::{FirewallArguments, FirewallPolicy, FirewallT};
use crate::winnet;
use log::{debug, error, trace};
use std::os::windows::ffi::OsStrExt;
use talpid_types::{net::Endpoint, tunnel::FirewallPolicyError};
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

    /// Failure to set TAP adapter metric
    #[error(display = "Unable to set TAP adapter metric")]
    SetTapMetric(#[error(source)] crate::winnet::Error),
}

const WINFW_TIMEOUT_SECONDS: u32 = 2;

/// The Windows implementation for the firewall and DNS.
pub struct Firewall(());

impl FirewallT for Firewall {
    type Error = Error;

    fn new(args: FirewallArguments) -> Result<Self, Self::Error> {
        let logging_context = b"WinFw\0".as_ptr();

        if args.initialize_blocked {
            let cfg = &WinFwSettings::new(args.allow_lan);
            unsafe {
                WinFw_InitializeBlocked(
                    WINFW_TIMEOUT_SECONDS,
                    &cfg,
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
                pingable_hosts,
                allow_lan,
                relay_client,
            } => {
                let cfg = &WinFwSettings::new(allow_lan);
                // TODO: Determine interface alias at runtime
                self.set_connecting_state(
                    &peer_endpoint,
                    &cfg,
                    "wg-mullvad".to_string(),
                    &pingable_hosts,
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
            FirewallPolicy::Blocked { allow_lan } => {
                let cfg = &WinFwSettings::new(allow_lan);
                self.set_blocked_state(&cfg)
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
        _tunnel_iface_alias: String,
        pingable_hosts: &Vec<IpAddr>,
        relay_client: &Path,
    ) -> Result<(), Error> {
        trace!("Applying 'connecting' firewall policy");
        let ip_str = Self::widestring_ip(endpoint.address.ip());
        let winfw_relay = WinFwRelay {
            ip: ip_str.as_ptr(),
            port: endpoint.address.port(),
            protocol: WinFwProt::from(endpoint.protocol),
        };

        let mut relay_client: Vec<u16> = relay_client.as_os_str().encode_wide().collect();
        relay_client.push(0u16);

        let pingable_addresses = pingable_hosts
            .iter()
            .map(|ip| Self::widestring_ip(*ip))
            .collect::<Vec<_>>();
        let pingable_address_ptrs = pingable_addresses
            .iter()
            .map(|ip| ip.as_ptr())
            .collect::<Vec<_>>();

        let pingable_hosts = if !pingable_address_ptrs.is_empty() {
            Some(WinFwPingableHosts {
                interfaceAlias: ptr::null(),
                addresses: pingable_address_ptrs.as_ptr(),
                num_addresses: pingable_addresses.len(),
            })
        } else {
            None
        };

        unsafe {
            WinFw_ApplyPolicyConnecting(
                winfw_settings,
                &winfw_relay,
                relay_client.as_ptr(),
                pingable_hosts.as_ptr(),
            )
            .into_result()
            .map_err(Error::ApplyingConnectingPolicy)
        }
    }

    fn widestring_ip(ip: IpAddr) -> WideCString {
        let buf = ip.to_string().encode_utf16().collect::<Vec<_>>();
        WideCString::new(buf).unwrap()
    }

    fn set_connected_state(
        &mut self,
        endpoint: &Endpoint,
        winfw_settings: &WinFwSettings,
        tunnel_metadata: &crate::tunnel::TunnelMetadata,
        dns_servers: &[IpAddr],
        relay_client: &Path,
    ) -> Result<(), Error> {
        trace!("Applying 'connected' firewall policy");
        let ip_str = Self::widestring_ip(endpoint.address.ip());
        let v4_gateway = Self::widestring_ip(tunnel_metadata.ipv4_gateway.into());
        let v6_gateway = tunnel_metadata
            .ipv6_gateway
            .map(|v6_ip| Self::widestring_ip(v6_ip.into()));

        let tunnel_alias =
            WideCString::new(tunnel_metadata.interface.encode_utf16().collect::<Vec<_>>()).unwrap();

        // ip_str, gateway_str and tunnel_alias have to outlive winfw_relay
        let winfw_relay = WinFwRelay {
            ip: ip_str.as_ptr(),
            port: endpoint.address.port(),
            protocol: WinFwProt::from(endpoint.protocol),
        };

        let metrics_set = winnet::ensure_best_metric_for_interface(&tunnel_metadata.interface)
            .map_err(Error::SetTapMetric)?;

        if metrics_set {
            debug!("Network interface metrics were changed");
        } else {
            debug!("Network interface metrics were not changed");
        }

        let v6_gateway_ptr = match &v6_gateway {
            Some(v6_ip) => v6_ip.as_ptr(),
            None => ptr::null(),
        };

        let mut relay_client: Vec<u16> = relay_client.as_os_str().encode_wide().collect();
        relay_client.push(0u16);

        unsafe {
            WinFw_ApplyPolicyConnected(
                winfw_settings,
                &winfw_relay,
                relay_client.as_ptr(),
                tunnel_alias.as_ptr(),
                v4_gateway.as_ptr(),
                v6_gateway_ptr,
            )
            .into_result()
            .map_err(Error::ApplyingConnectedPolicy)
        }
    }

    fn set_blocked_state(&mut self, winfw_settings: &WinFwSettings) -> Result<(), Error> {
        trace!("Applying 'blocked' firewall policy");
        unsafe {
            WinFw_ApplyPolicyBlocked(winfw_settings)
                .into_result()
                .map_err(Error::ApplyingBlockedPolicy)
        }
    }
}

trait NullablePointer<T> {
    fn as_ptr(&self) -> *const T;
}

impl<T> NullablePointer<T> for Option<T> {
    fn as_ptr(&self) -> *const T {
        match self {
            Some(ref value) => value,
            None => ptr::null(),
        }
    }
}

#[allow(non_snake_case)]
mod winfw {
    use super::Error;
    use crate::logging::windows::LogSink;
    use libc;
    use talpid_types::net::TransportProtocol;

    #[repr(C)]
    pub struct WinFwRelay {
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

    #[repr(C)]
    pub struct WinFwPingableHosts {
        // a null pointer implies that all interfaces will be able to ping the supplied addresses
        pub interfaceAlias: *const libc::wchar_t,
        pub addresses: *const *const libc::wchar_t,
        pub num_addresses: usize,
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
            sink: Option<LogSink>,
            sink_context: *const u8,
        ) -> InitializationResult;

        #[link_name = "WinFw_Deinitialize"]
        pub fn WinFw_Deinitialize(cleanupPolicy: WinFwCleanupPolicy) -> DeinitializationResult;

        #[link_name = "WinFw_ApplyPolicyConnecting"]
        pub fn WinFw_ApplyPolicyConnecting(
            settings: &WinFwSettings,
            relay: &WinFwRelay,
            relayClient: *const libc::wchar_t,
            pingable_hosts: *const WinFwPingableHosts,
        ) -> WinFwPolicyStatus;

        #[link_name = "WinFw_ApplyPolicyConnected"]
        pub fn WinFw_ApplyPolicyConnected(
            settings: &WinFwSettings,
            relay: &WinFwRelay,
            relayClient: *const libc::wchar_t,
            tunnelIfaceAlias: *const libc::wchar_t,
            v4Gateway: *const libc::wchar_t,
            v6Gateway: *const libc::wchar_t,
        ) -> WinFwPolicyStatus;

        #[link_name = "WinFw_ApplyPolicyBlocked"]
        pub fn WinFw_ApplyPolicyBlocked(settings: &WinFwSettings) -> WinFwPolicyStatus;

        #[link_name = "WinFw_Reset"]
        pub fn WinFw_Reset() -> WinFwPolicyStatus;
    }
}

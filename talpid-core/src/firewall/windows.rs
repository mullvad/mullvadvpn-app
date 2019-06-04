use libc::{c_char, c_void};
use std::{net::IpAddr, ptr};

use self::winfw::*;
use super::{FirewallArguments, FirewallPolicy, FirewallT};
use crate::winnet;
use log::{debug, error, trace};
use talpid_types::net::Endpoint;
use widestring::WideCString;


/// Errors that can happen when configuring the Windows firewall.
#[derive(err_derive::Error, Debug)]
pub enum Error {
    /// Failure to initialize windows firewall module
    #[error(display = "Failed to initialize windows firewall module")]
    Initialization,

    /// Failure to deinitialize windows firewall module
    #[error(display = "Failed to deinitialize windows firewall module")]
    Deinitialization,

    /// Failure to apply a firewall _connecting_ policy
    #[error(display = "Failed to apply connecting firewall policy")]
    ApplyingConnectingPolicy,

    /// Failure to apply a firewall _connected_ policy
    #[error(display = "Failed to apply connected firewall policy")]
    ApplyingConnectedPolicy,

    /// Failure to apply firewall _blocked_ policy
    #[error(display = "Failed to apply blocked firewall policy")]
    ApplyingBlockedPolicy,

    /// Failure to reset firewall policies
    #[error(display = "Failed to reset firewall policies")]
    ResettingPolicy,

    /// Failure to set TAP adapter metric
    #[error(display = "Unable to set TAP adapter metric")]
    SetTapMetric(#[error(source)] crate::winnet::Error),
}

const WINFW_TIMEOUT_SECONDS: u32 = 2;

/// The Windows implementation for the firewall and DNS.
pub struct Firewall(());

extern "system" fn error_sink(msg: *const c_char, _ctx: *mut c_void) {
    use std::ffi::CStr;
    if msg.is_null() {
        log::error!("Log message from FFI boundary is NULL");
    } else {
        log::error!("{}", unsafe { CStr::from_ptr(msg).to_string_lossy() });
    }
}

impl FirewallT for Firewall {
    type Error = Error;

    fn new(args: FirewallArguments) -> Result<Self, Self::Error> {
        if args.initialize_blocked {
            let cfg = &WinFwSettings::new(args.allow_lan.unwrap());
            unsafe {
                WinFw_InitializeBlocked(
                    WINFW_TIMEOUT_SECONDS,
                    &cfg,
                    Some(error_sink),
                    ptr::null_mut(),
                )
                .into_result()?
            };
        } else {
            unsafe {
                WinFw_Initialize(WINFW_TIMEOUT_SECONDS, Some(error_sink), ptr::null_mut())
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
            } => {
                let cfg = &WinFwSettings::new(allow_lan);
                // TODO: Determine interface alias at runtime
                self.set_connecting_state(
                    &peer_endpoint,
                    &cfg,
                    "wg-mullvad".to_string(),
                    &pingable_hosts,
                )
            }
            FirewallPolicy::Connected {
                peer_endpoint,
                tunnel,
                allow_lan,
            } => {
                let cfg = &WinFwSettings::new(allow_lan);
                self.set_connected_state(&peer_endpoint, &cfg, &tunnel)
            }
            FirewallPolicy::Blocked { allow_lan } => {
                let cfg = &WinFwSettings::new(allow_lan);
                self.set_blocked_state(&cfg)
            }
        }
    }

    fn reset_policy(&mut self) -> Result<(), Self::Error> {
        unsafe { WinFw_Reset().into_result() }?;
        Ok(())
    }
}

impl Drop for Firewall {
    fn drop(&mut self) {
        if unsafe { WinFw_Deinitialize().into_result().is_ok() } {
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
    ) -> Result<(), Error> {
        trace!("Applying 'connecting' firewall policy");
        let ip_str = Self::widestring_ip(endpoint.address.ip());

        // ip_str has to outlive winfw_relay
        let winfw_relay = WinFwRelay {
            ip: ip_str.as_ptr(),
            port: endpoint.address.port(),
            protocol: WinFwProt::from(endpoint.protocol),
        };

        if pingable_hosts.is_empty() {
            unsafe {
                return WinFw_ApplyPolicyConnecting(winfw_settings, &winfw_relay, ptr::null())
                    .into_result();
            }
        }

        let pingable_addresses = pingable_hosts
            .iter()
            .map(|ip| Self::widestring_ip(*ip))
            .collect::<Vec<_>>();
        let pingable_address_ptrs = pingable_addresses
            .iter()
            .map(|ip| ip.as_ptr())
            .collect::<Vec<_>>();

        let pingable_hosts = WinFwPingableHosts {
            interfaceAlias: ptr::null(),
            addresses: pingable_address_ptrs.as_ptr(),
            num_addresses: pingable_addresses.len(),
        };

        unsafe {
            WinFw_ApplyPolicyConnecting(winfw_settings, &winfw_relay, &pingable_hosts).into_result()
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

        let metrics_set = winnet::ensure_top_metric_for_interface(&tunnel_metadata.interface)
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

        unsafe {
            WinFw_ApplyPolicyConnected(
                winfw_settings,
                &winfw_relay,
                tunnel_alias.as_ptr(),
                v4_gateway.as_ptr(),
                v6_gateway_ptr,
            )
            .into_result()
        }
    }

    fn set_blocked_state(&mut self, winfw_settings: &WinFwSettings) -> Result<(), Error> {
        trace!("Applying 'blocked' firewall policy");
        unsafe { WinFw_ApplyPolicyBlocked(winfw_settings).into_result() }
    }
}


#[allow(non_snake_case)]
mod winfw {
    use super::Error;
    use libc;
    use talpid_types::net::TransportProtocol;

    /// logging callback type for use with `winfw.dll`.
    pub type ErrorSink = extern "system" fn(msg: *const libc::c_char, ctx: *mut libc::c_void);

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

    ffi_error!(InitializationResult, Error::Initialization);
    ffi_error!(DeinitializationResult, Error::Deinitialization);
    ffi_error!(ApplyConnectingResult, Error::ApplyingConnectingPolicy);
    ffi_error!(ApplyConnectedResult, Error::ApplyingConnectedPolicy);
    ffi_error!(ApplyBlockedResult, Error::ApplyingBlockedPolicy);
    ffi_error!(ResettingPolicyResult, Error::ResettingPolicy);

    extern "system" {
        #[link_name = "WinFw_Initialize"]
        pub fn WinFw_Initialize(
            timeout: libc::c_uint,
            sink: Option<ErrorSink>,
            sink_context: *mut libc::c_void,
        ) -> InitializationResult;

        #[link_name = "WinFw_InitializeBlocked"]
        pub fn WinFw_InitializeBlocked(
            timeout: libc::c_uint,
            settings: &WinFwSettings,
            sink: Option<ErrorSink>,
            sink_context: *mut libc::c_void,
        ) -> InitializationResult;

        #[link_name = "WinFw_Deinitialize"]
        pub fn WinFw_Deinitialize() -> DeinitializationResult;

        #[link_name = "WinFw_ApplyPolicyConnecting"]
        pub fn WinFw_ApplyPolicyConnecting(
            settings: &WinFwSettings,
            relay: &WinFwRelay,
            pingable_hosts: *const WinFwPingableHosts,
        ) -> ApplyConnectingResult;

        #[link_name = "WinFw_ApplyPolicyConnected"]
        pub fn WinFw_ApplyPolicyConnected(
            settings: &WinFwSettings,
            relay: &WinFwRelay,
            tunnelIfaceAlias: *const libc::wchar_t,
            v4Gateway: *const libc::wchar_t,
            v6Gateway: *const libc::wchar_t,
        ) -> ApplyConnectedResult;

        #[link_name = "WinFw_ApplyPolicyBlocked"]
        pub fn WinFw_ApplyPolicyBlocked(settings: &WinFwSettings) -> ApplyBlockedResult;

        #[link_name = "WinFw_Reset"]
        pub fn WinFw_Reset() -> ResettingPolicyResult;
    }
}

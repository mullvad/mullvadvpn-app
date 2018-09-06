extern crate widestring;

use super::{NetworkSecurityT, SecurityPolicy};
use std::net::IpAddr;
use std::path::Path;
use std::ptr;

use self::winfw::*;
use talpid_types::net::Endpoint;

use self::widestring::WideCString;
use ffi;


mod dns;
mod route;
mod system_state;

use self::dns::WinDns;

error_chain! {
    errors {
        /// Failure to initialize windows firewall module
        Initialization {
            description("Failed to initialise windows firewall module")
        }

        /// Failure to deinitialize windows firewall module
        Deinitialization {
            description("Failed to deinitialize windows firewall module")
        }

        /// Failure to apply a firewall _connecting_ policy
        ApplyingConnectingPolicy {
            description("Failed to apply firewall policy for when the daemon is connecting to a tunnel")
        }

        /// Failure to apply a firewall _connected_ policy
        ApplyingConnectedPolicy {
            description("Failed to apply firewall policy for when the daemon is connected to a tunnel")
        }

        /// Failure to apply firewall _blocked_ policy
        ApplyingBlockedPolicy {
            description("Failed to apply blocked security policy")
        }

        /// Failure to reset firewall policies
        ResettingPolicy {
            description("Failed to reset firewall policies")
        }
    }

    links {
        WinDns(dns::Error, dns::ErrorKind) #[doc = "WinDNS failure"];
        WinRoute(route::Error, route::ErrorKind) #[doc = "Failure to modify system routing metrics"];
    }
}

const WINFW_TIMEOUT_SECONDS: u32 = 2;

/// The Windows implementation for the firewall and DNS.
pub struct NetworkSecurity {
    dns: WinDns,
}

impl NetworkSecurityT for NetworkSecurity {
    type Error = Error;

    fn new(cache_dir: impl AsRef<Path>) -> Result<Self> {
        let windns = WinDns::new(cache_dir)?;
        unsafe {
            WinFw_Initialize(
                WINFW_TIMEOUT_SECONDS,
                Some(ffi::error_sink),
                ptr::null_mut(),
            ).into_result()?
        };
        trace!("Successfully initialized windows firewall module");
        Ok(NetworkSecurity { dns: windns })
    }

    fn apply_policy(&mut self, policy: SecurityPolicy) -> Result<()> {
        match policy {
            SecurityPolicy::Connecting {
                relay_endpoint,
                allow_lan,
            } => {
                let cfg = &WinFwSettings::new(allow_lan);
                self.set_connecting_state(&relay_endpoint, &cfg)
            }
            SecurityPolicy::Connected {
                relay_endpoint,
                tunnel,
                allow_lan,
            } => {
                let cfg = &WinFwSettings::new(allow_lan);
                self.set_connected_state(&relay_endpoint, &cfg, &tunnel)
            }
            SecurityPolicy::Blocked { allow_lan } => {
                let cfg = &WinFwSettings::new(allow_lan);
                self.set_blocked_state(&cfg)
            }
        }
    }

    fn reset_policy(&mut self) -> Result<()> {
        self.dns.reset_dns()?;
        unsafe { WinFw_Reset().into_result() }?;
        Ok(())
    }
}

impl Drop for NetworkSecurity {
    fn drop(&mut self) {
        if unsafe { WinFw_Deinitialize().into_result().is_ok() } {
            trace!("Successfully deinitialized windows firewall module");
        } else {
            error!("Failed to deinitialize windows firewall module");
        };
    }
}

impl NetworkSecurity {
    fn set_connecting_state(
        &mut self,
        endpoint: &Endpoint,
        winfw_settings: &WinFwSettings,
    ) -> Result<()> {
        trace!("Applying 'connecting' firewall policy");
        let ip_str = Self::widestring_ip(&endpoint.address.ip());

        // ip_str has to outlive winfw_relay
        let winfw_relay = WinFwRelay {
            ip: ip_str.as_wide_c_str().as_ptr(),
            port: endpoint.address.port(),
            protocol: WinFwProt::from(endpoint.protocol),
        };

        unsafe { WinFw_ApplyPolicyConnecting(winfw_settings, &winfw_relay).into_result() }
    }

    fn widestring_ip(ip: &IpAddr) -> WideCString {
        let buf = ip.to_string().encode_utf16().collect::<Vec<_>>();
        WideCString::new(buf).unwrap()
    }

    fn set_connected_state(
        &mut self,
        endpoint: &Endpoint,
        winfw_settings: &WinFwSettings,
        tunnel_metadata: &::tunnel::TunnelMetadata,
    ) -> Result<()> {
        trace!("Applying 'connected' firewall policy");
        let ip_str = Self::widestring_ip(&endpoint.address.ip());
        let gateway_str = Self::widestring_ip(&tunnel_metadata.gateway.into());

        let tunnel_alias =
            WideCString::new(tunnel_metadata.interface.encode_utf16().collect::<Vec<_>>()).unwrap();

        // ip_str, gateway_str and tunnel_alias have to outlive winfw_relay
        let winfw_relay = WinFwRelay {
            ip: ip_str.as_wide_c_str().as_ptr(),
            port: endpoint.address.port(),
            protocol: WinFwProt::from(endpoint.protocol),
        };

        self.dns.set_dns(&vec![tunnel_metadata.gateway.into()])?;

        let metrics_set = route::ensure_top_metric_for_interface(&tunnel_metadata.interface)?;
        if metrics_set {
            debug!("Network interface metrics were changed");
        } else {
            debug!("Network interface metrics were not changed");
        }


        unsafe {
            WinFw_ApplyPolicyConnected(
                winfw_settings,
                &winfw_relay,
                tunnel_alias.as_wide_c_str().as_ptr(),
                gateway_str.as_wide_c_str().as_ptr(),
            ).into_result()
        }
    }

    fn set_blocked_state(&mut self, winfw_settings: &WinFwSettings) -> Result<()> {
        trace!("Applying 'blocked' firewall policy");
        unsafe { WinFw_ApplyPolicyBlocked(winfw_settings).into_result() }
    }
}


#[allow(non_snake_case)]
mod winfw {
    use super::{ErrorKind, Result};
    use ffi;
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

    ffi_error!(InitializationResult, ErrorKind::Initialization.into());
    ffi_error!(DeinitializationResult, ErrorKind::Deinitialization.into());
    ffi_error!(
        ApplyConnectingResult,
        ErrorKind::ApplyingConnectingPolicy.into()
    );
    ffi_error!(
        ApplyConnectedResult,
        ErrorKind::ApplyingConnectedPolicy.into()
    );
    ffi_error!(ApplyBlockedResult, ErrorKind::ApplyingBlockedPolicy.into());
    ffi_error!(ResettingPolicyResult, ErrorKind::ResettingPolicy.into());

    extern "system" {
        #[link_name(WinFw_Initialize)]
        pub fn WinFw_Initialize(
            timeout: libc::c_uint,
            sink: Option<ffi::ErrorSink>,
            sink_context: *mut libc::c_void,
        ) -> InitializationResult;

        #[link_name(WinFw_Deinitialize)]
        pub fn WinFw_Deinitialize() -> DeinitializationResult;

        #[link_name(WinFw_ApplyPolicyConnecting)]
        pub fn WinFw_ApplyPolicyConnecting(
            settings: &WinFwSettings,
            relay: &WinFwRelay,
        ) -> ApplyConnectingResult;

        #[link_name(WinFw_ApplyPolicyConnected)]
        pub fn WinFw_ApplyPolicyConnected(
            settings: &WinFwSettings,
            relay: &WinFwRelay,
            tunnelIfaceAlias: *const libc::wchar_t,
            primaryDns: *const libc::wchar_t,
        ) -> ApplyConnectedResult;

        #[link_name(WinFw_ApplyPolicyBlocked)]
        pub fn WinFw_ApplyPolicyBlocked(settings: &WinFwSettings) -> ApplyBlockedResult;

        #[link_name(WinFw_Reset)]
        pub fn WinFw_Reset() -> ResettingPolicyResult;
    }
}

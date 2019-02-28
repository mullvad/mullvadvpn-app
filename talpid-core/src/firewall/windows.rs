use std::{net::IpAddr, ptr};

use self::winfw::*;
use super::{FirewallPolicy, FirewallT};
use crate::winnet;
use log::{debug, error, trace};
use talpid_types::net::Endpoint;
use widestring::WideCString;

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
            description("Failed to apply blocked firewall policy")
        }

        /// Failure to reset firewall policies
        ResettingPolicy {
            description("Failed to reset firewall policies")
        }

        /// Failure to set TAP adapter metric
        SetTapMetric {
            description("Unable to set TAP adapter metric")
        }
    }
}

const WINFW_TIMEOUT_SECONDS: u32 = 2;

/// The Windows implementation for the firewall and DNS.
pub struct Firewall(());

impl FirewallT for Firewall {
    type Error = Error;

    fn new() -> Result<Self> {
        unsafe {
            WinFw_Initialize(
                WINFW_TIMEOUT_SECONDS,
                Some(winnet::error_sink),
                ptr::null_mut(),
            )
            .into_result()?
        };
        trace!("Successfully initialized windows firewall module");
        Ok(Firewall(()))
    }

    fn apply_policy(&mut self, policy: FirewallPolicy) -> Result<()> {
        match policy {
            FirewallPolicy::Connecting {
                peer_endpoint,
                // TODO: Allow ICMP traffic to a list of hosts for wireguard
                pingable_hosts: _,
                allow_lan,
            } => {
                let cfg = &WinFwSettings::new(allow_lan);
                self.set_connecting_state(&peer_endpoint, &cfg)
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

    fn reset_policy(&mut self) -> Result<()> {
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
    ) -> Result<()> {
        trace!("Applying 'connecting' firewall policy");
        let ip_str = Self::widestring_ip(endpoint.address.ip());

        // ip_str has to outlive winfw_relay
        let winfw_relay = WinFwRelay {
            ip: ip_str.as_wide_c_str().as_ptr(),
            port: endpoint.address.port(),
            protocol: WinFwProt::from(endpoint.protocol),
        };

        unsafe { WinFw_ApplyPolicyConnecting(winfw_settings, &winfw_relay).into_result() }
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
    ) -> Result<()> {
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
            ip: ip_str.as_wide_c_str().as_ptr(),
            port: endpoint.address.port(),
            protocol: WinFwProt::from(endpoint.protocol),
        };

        let metrics_set = winnet::ensure_top_metric_for_interface(&tunnel_metadata.interface)
            .chain_err(|| ErrorKind::SetTapMetric)?;

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
                tunnel_alias.as_wide_c_str().as_ptr(),
                v4_gateway.as_ptr(),
                v6_gateway_ptr,
            )
            .into_result()
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
    use crate::winnet;
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
        #[link_name = "WinFw_Initialize"]
        pub fn WinFw_Initialize(
            timeout: libc::c_uint,
            sink: Option<winnet::ErrorSink>,
            sink_context: *mut libc::c_void,
        ) -> InitializationResult;

        #[link_name = "WinFw_Deinitialize"]
        pub fn WinFw_Deinitialize() -> DeinitializationResult;

        #[link_name = "WinFw_ApplyPolicyConnecting"]
        pub fn WinFw_ApplyPolicyConnecting(
            settings: &WinFwSettings,
            relay: &WinFwRelay,
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

extern crate libc;
extern crate widestring;

use super::{Firewall, SecurityPolicy};
use std::net::IpAddr;
use std::ptr;

use self::ffi::*;
use talpid_types::net::Endpoint;

use self::widestring::WideCString;

error_chain!{
    errors{
        #[doc = "Windows firewall module error"]
        WfpctlFailure(desc: &'static str){
            description("Opaque Wfpctl failure")
            display("Wfpctl failed when {}", desc)
        }
    }
}

const WFPCTL_TIMEOUT_SECONDS: u32 = 2;

/// The Windows implementation for the `Firewall` trait.
pub struct WindowsFirewall {
    _unused: [u8; 0],
}

impl Firewall for WindowsFirewall {
    type Error = Error;

    fn new() -> Result<Self> {
        let ok =
            unsafe { Wfpctl_Initialize(WFPCTL_TIMEOUT_SECONDS, Some(error_sink), ptr::null_mut()) };
        ok.into_result("initialise wfpctl").map(|_| {
            trace!("Successfully initialized wfpctl");
            WindowsFirewall{_unused: []}
        })
    }

    fn apply_policy(&mut self, policy: SecurityPolicy) -> Result<()> {
        match policy {
            SecurityPolicy::Connecting {
                relay_endpoint,
                allow_lan,
            } => {
                let cfg = &WfpCtlSettings::new(allow_lan);
                self.set_connecting_state(&relay_endpoint, &cfg)
            }
            SecurityPolicy::Connected {
                relay_endpoint,
                tunnel,
                allow_lan,
            } => {
                let cfg = &WfpCtlSettings::new(allow_lan);
                self.set_connected_state(&relay_endpoint, &cfg, &tunnel)
            }
        }
    }

    fn reset_policy(&mut self) -> Result<()> {
        trace!("Resetting firewall policy");
        let ok = unsafe { Wfpctl_Reset() };
        ok.into_result("resetting firewall")
    }
}

impl Drop for WindowsFirewall {
    fn drop(&mut self) {
        if unsafe { Wfpctl_Deinitialize().is_ok() } {
            trace!("Successfully deinitialized wfpctl");
        } else {
            error!("Failed to deinitialize wfpctl");
        };
    }
}

impl WindowsFirewall {
    fn set_connecting_state(
        &mut self,
        endpoint: &Endpoint,
        wfp_settings: &WfpCtlSettings,
    ) -> Result<()> {
        trace!("Applying 'connecting' firewall policy");
        let ip_str = Self::widestring_ip(&endpoint.address.ip());

        // ip_str has to outlive wfp_relay
        let wfp_relay = WfpCtlRelay {
            ip: ip_str.as_wide_c_str().as_ptr(),
            port: endpoint.address.port(),
            protocol: WfpCtlProt::from(endpoint.protocol),
        };

        let ok = unsafe { Wfpctl_ApplyPolicyConnecting(wfp_settings, &wfp_relay) };
        ok.into_result("applying 'connecting' policy")
    }

    fn widestring_ip(ip: &IpAddr) -> WideCString {
        let buf = ip.to_string().encode_utf16().collect::<Vec<_>>();
        WideCString::new(buf).unwrap()
    }

    fn set_connected_state(
        &mut self,
        endpoint: &Endpoint,
        wfp_settings: &WfpCtlSettings,
        tunnel_metadata: &::tunnel::TunnelMetadata,
    ) -> Result<()> {
        trace!("Applying 'connected' firewall policy");
        let ip_str = Self::widestring_ip(&endpoint.address.ip());
        let gateway_str = Self::widestring_ip(&tunnel_metadata.gateway.into());

        let tunnel_alias =
            WideCString::new(tunnel_metadata.interface.encode_utf16().collect::<Vec<_>>()).unwrap();

        // ip_str, gateway_str and tunnel_alias have to outlive wfp_relay
        let wfp_relay = WfpCtlRelay {
            ip: ip_str.as_wide_c_str().as_ptr(),
            port: endpoint.address.port(),
            protocol: WfpCtlProt::from(endpoint.protocol),
        };

        let ok = unsafe {
            Wfpctl_ApplyPolicyConnected(
                wfp_settings,
                &wfp_relay,
                tunnel_alias.as_wide_c_str().as_ptr(),
                gateway_str.as_wide_c_str().as_ptr(),
            )
        };
        ok.into_result("applying 'connected' policy")
    }
}


#[allow(non_snake_case)]
mod ffi {

    use super::{ErrorKind, Result};
    use super::libc;
    use std::ffi::CStr;
    use std::os::raw::c_char;
    use talpid_types::net::TransportProtocol;
    use std::ptr;

    #[repr(C)]
    pub struct WfpCtlResult {
        ok: bool,
    }

    impl WfpCtlResult {
        pub fn into_result(self, description: &'static str) -> Result<()> {
            match self.ok {
                true => Ok(()),
                false => Err(ErrorKind::WfpctlFailure(description).into()),
            }
        }

        pub fn is_ok(&self) -> bool {
            self.ok
        }
    }

    pub type ErrorSink = extern "system" fn(msg: *const c_char, ctx: *mut libc::c_void);

    pub extern "system" fn error_sink(msg: *const c_char, _ctx: *mut libc::c_void) {
        if msg == ptr::null() {
            error!("log message from wfpctl is NULL");
        } else {
            error!("{}", unsafe { CStr::from_ptr(msg).to_string_lossy() });
        }
    }

    #[repr(C)]
    pub struct WfpCtlRelay {
        pub ip: *const libc::wchar_t,
        pub port: u16,
        pub protocol: WfpCtlProt,
    }

    #[repr(u8)]
    #[derive(Clone, Copy)]
    pub enum WfpCtlProt {
        Tcp = 0u8,
        Udp = 1u8,
    }

    impl From<TransportProtocol> for WfpCtlProt {
        fn from(prot: TransportProtocol) -> WfpCtlProt {
            match prot {
                TransportProtocol::Tcp => WfpCtlProt::Tcp,
                TransportProtocol::Udp => WfpCtlProt::Udp,
            }
        }
    }

    #[repr(C)]
    pub struct WfpCtlSettings {
        permitDhcp: bool,
        permitLan: bool,
    }

    impl WfpCtlSettings {
        pub fn new(permit_lan: bool) -> WfpCtlSettings {
            WfpCtlSettings {
                permitDhcp: true,
                permitLan: permit_lan,
            }
        }
    }

    extern "system" {
        #[link_name(Wfpctl_Initialize)]
        pub fn Wfpctl_Initialize(
            timeout: libc::c_uint,
            sink: Option<ErrorSink>,
            sink_context: *mut libc::c_void,
        ) -> WfpCtlResult;

        #[link_name(Wfpctl_Deinitialize)]
        pub fn Wfpctl_Deinitialize() -> WfpCtlResult;

        #[link_name(Wfpctl_ApplyPolicyConnecting)]
        pub fn Wfpctl_ApplyPolicyConnecting(
            settings: &WfpCtlSettings,
            relay: &WfpCtlRelay,
        ) -> WfpCtlResult;

        #[link_name(Wfpctl_ApplyPolicyConnected)]
        pub fn Wfpctl_ApplyPolicyConnected(
            settings: &WfpCtlSettings,
            relay: &WfpCtlRelay,
            tunnelIfaceAlias: *const libc::wchar_t,
            primaryDns: *const libc::wchar_t,
        ) -> WfpCtlResult;

        #[link_name(Wfpctl_Reset)]
        pub fn Wfpctl_Reset() -> WfpCtlResult;
    }
}

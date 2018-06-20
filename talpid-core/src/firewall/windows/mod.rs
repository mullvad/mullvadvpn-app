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
        WinFwFailure(desc: &'static str){
            description("Opaque WinFw failure")
            display("WinFw failed when {}", desc)
        }
    }
}

const WINFW_TIMEOUT_SECONDS: u32 = 2;

/// The Windows implementation for the `Firewall` trait.
pub struct WindowsFirewall {
    _unused: [u8; 0],
}

impl Firewall for WindowsFirewall {
    type Error = Error;

    fn new() -> Result<Self> {
        let ok =
            unsafe { WinFw_Initialize(WINFW_TIMEOUT_SECONDS, Some(error_sink), ptr::null_mut()) };
        ok.into_result("initialize WinFw").map(|_| {
            trace!("Successfully initialized WinFw");
            WindowsFirewall { _unused: [] }
        })
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
        }
    }

    fn reset_policy(&mut self) -> Result<()> {
        trace!("Resetting firewall policy");
        let ok = unsafe { WinFw_Reset() };
        ok.into_result("resetting firewall")
    }
}

impl Drop for WindowsFirewall {
    fn drop(&mut self) {
        if unsafe { WinFw_Deinitialize().is_ok() } {
            trace!("Successfully deinitialized WinFw");
        } else {
            error!("Failed to deinitialize WinFw");
        };
    }
}

impl WindowsFirewall {
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

        let ok = unsafe { WinFw_ApplyPolicyConnecting(winfw_settings, &winfw_relay) };
        ok.into_result("applying 'connecting' policy")
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

        let ok = unsafe {
            WinFw_ApplyPolicyConnected(
                winfw_settings,
                &winfw_relay,
                tunnel_alias.as_wide_c_str().as_ptr(),
                gateway_str.as_wide_c_str().as_ptr(),
            )
        };
        ok.into_result("applying 'connected' policy")
    }
}


#[allow(non_snake_case)]
mod ffi {

    extern crate libc;
    use super::{ErrorKind, Result};
    use std::ffi::CStr;
    use std::os::raw::c_char;
    use std::ptr;
    use talpid_types::net::TransportProtocol;

    #[repr(C)]
    pub struct WinFwResult {
        ok: bool,
    }

    impl WinFwResult {
        pub fn into_result(self, description: &'static str) -> Result<()> {
            match self.ok {
                true => Ok(()),
                false => Err(ErrorKind::WinFwFailure(description).into()),
            }
        }

        pub fn is_ok(&self) -> bool {
            self.ok
        }
    }

    pub type ErrorSink = extern "system" fn(msg: *const c_char, ctx: *mut libc::c_void);

    pub extern "system" fn error_sink(msg: *const c_char, _ctx: *mut libc::c_void) {
        if msg == ptr::null() {
            error!("log message from WinFw is NULL");
        } else {
            error!("{}", unsafe { CStr::from_ptr(msg).to_string_lossy() });
        }
    }

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

    extern "system" {
        #[link_name(WinFw_Initialize)]
        pub fn WinFw_Initialize(
            timeout: libc::c_uint,
            sink: Option<ErrorSink>,
            sink_context: *mut libc::c_void,
        ) -> WinFwResult;

        #[link_name(WinFw_Deinitialize)]
        pub fn WinFw_Deinitialize() -> WinFwResult;

        #[link_name(WinFw_ApplyPolicyConnecting)]
        pub fn WinFw_ApplyPolicyConnecting(
            settings: &WinFwSettings,
            relay: &WinFwRelay,
        ) -> WinFwResult;

        #[link_name(WinFw_ApplyPolicyConnected)]
        pub fn WinFw_ApplyPolicyConnected(
            settings: &WinFwSettings,
            relay: &WinFwRelay,
            tunnelIfaceAlias: *const libc::wchar_t,
            primaryDns: *const libc::wchar_t,
        ) -> WinFwResult;

        #[link_name(WinFw_Reset)]
        pub fn WinFw_Reset() -> WinFwResult;
    }
}

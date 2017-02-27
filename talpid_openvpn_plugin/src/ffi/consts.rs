/// Constants for OpenVPN. Taken from include/openvpn-plugin.h in the OpenVPN repository:
/// https://github.com/OpenVPN/openvpn/blob/master/include/openvpn-plugin.h.in

use std::collections::HashMap;
use std::os::raw::c_int;


// All types of events that a plugin can receive from OpenVPN.
pub const OPENVPN_PLUGIN_UP: c_int = 0;
pub const OPENVPN_PLUGIN_DOWN: c_int = 1;
pub const OPENVPN_PLUGIN_ROUTE_UP: c_int = 2;
pub const OPENVPN_PLUGIN_IPCHANGE: c_int = 3;
pub const OPENVPN_PLUGIN_TLS_VERIFY: c_int = 4;
pub const OPENVPN_PLUGIN_AUTH_USER_PASS_VERIFY: c_int = 5;
pub const OPENVPN_PLUGIN_CLIENT_CONNECT: c_int = 6;
pub const OPENVPN_PLUGIN_CLIENT_DISCONNECT: c_int = 7;
pub const OPENVPN_PLUGIN_LEARN_ADDRESS: c_int = 8;
pub const OPENVPN_PLUGIN_CLIENT_CONNECT_V2: c_int = 9;
pub const OPENVPN_PLUGIN_TLS_FINAL: c_int = 10;
pub const OPENVPN_PLUGIN_ENABLE_PF: c_int = 11;
pub const OPENVPN_PLUGIN_ROUTE_PREDOWN: c_int = 12;
pub const OPENVPN_PLUGIN_N: c_int = 13;
error_chain!{
    errors {
        InvalidEnumVariant {
            description("Integer does not match any enum variant")
        }
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub enum OpenVpnPluginEvent {
    Up = 0,
    Down = 1,
    RouteUp = 2,
    IpChange = 3,
    TlsVerify = 4,
    AuthUserPassVerify = 5,
    ClientConnect = 6,
    ClientDisconnect = 7,
    LearnAddress = 8,
    ClientConnectV2 = 9,
    TlsFinal = 10,
    EnablePf = 11,
    RoutePredown = 12,
    N = 13,
}

impl OpenVpnPluginEvent {
    pub fn from_int(i: c_int) -> Result<OpenVpnPluginEvent> {
        if i >= OpenVpnPluginEvent::Up as c_int && i <= OpenVpnPluginEvent::N as c_int {
            Ok(unsafe { ::std::mem::transmute_copy::<c_int, OpenVpnPluginEvent>(&i) })
        } else {
            Err(ErrorKind::InvalidEnumVariant.into())
        }
    }
}


lazy_static! {
    pub static ref PLUGIN_EVENT_NAMES: HashMap<c_int, &'static str> = {
        let mut map = HashMap::new();
        map.insert(OPENVPN_PLUGIN_UP, "PLUGIN_UP");
        map.insert(OPENVPN_PLUGIN_DOWN, "PLUGIN_DOWN");
        map.insert(OPENVPN_PLUGIN_ROUTE_UP, "PLUGIN_ROUTE_UP");
        map.insert(OPENVPN_PLUGIN_IPCHANGE, "PLUGIN_IPCHANGE");
        map.insert(OPENVPN_PLUGIN_TLS_VERIFY, "PLUGIN_TLS_VERIFY");
        map.insert(OPENVPN_PLUGIN_AUTH_USER_PASS_VERIFY, "PLUGIN_AUTH_USER_PASS_VERIFY");
        map.insert(OPENVPN_PLUGIN_CLIENT_CONNECT, "PLUGIN_CLIENT_CONNECT");
        map.insert(OPENVPN_PLUGIN_CLIENT_DISCONNECT, "PLUGIN_CLIENT_DISCONNECT");
        map.insert(OPENVPN_PLUGIN_LEARN_ADDRESS, "PLUGIN_LEARN_ADDRESS");
        map.insert(OPENVPN_PLUGIN_CLIENT_CONNECT_V2, "PLUGIN_CLIENT_CONNECT_V2");
        map.insert(OPENVPN_PLUGIN_TLS_FINAL, "PLUGIN_TLS_FINAL");
        map.insert(OPENVPN_PLUGIN_ENABLE_PF, "PLUGIN_ENABLE_PF");
        map.insert(OPENVPN_PLUGIN_ROUTE_PREDOWN, "PLUGIN_ROUTE_PREDOWN");
        map.insert(OPENVPN_PLUGIN_N, "PLUGIN_N");
        map
    };
}

/// Returns the name of an OPENVPN_PLUGIN_* constant.
pub fn plugin_event_name(num: c_int) -> &'static str {
    PLUGIN_EVENT_NAMES.get(&num).map(|s| *s).unwrap_or("UNKNOWN")
}


// Return values. Returned from the plugin to OpenVPN to indicate success or failure. Can also
// Accept (success) or decline (error) an operation, such as incoming client connection attempt.
pub const OPENVPN_PLUGIN_FUNC_SUCCESS: c_int = 0;
pub const OPENVPN_PLUGIN_FUNC_ERROR: c_int = 1;
pub const OPENVPN_PLUGIN_FUNC_DEFERRED: c_int = 2;


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_int_first() {
        let result = OpenVpnPluginEvent::from_int(0);
        assert_matches!(result, Ok(OpenVpnPluginEvent::Up));
    }

    #[test]
    fn from_int_last() {
        let result = OpenVpnPluginEvent::from_int(13);
        assert_matches!(result, Ok(OpenVpnPluginEvent::N));
    }

    #[test]
    fn from_int_all_valid() {
        for i in 0..13 {
            if OpenVpnPluginEvent::from_int(0).is_err() {
                panic!("{} not covered", i);
            }
        }
    }

    #[test]
    fn from_int_negative() {
        let result = OpenVpnPluginEvent::from_int(-5);
        assert_matches!(result, Err(Error(ErrorKind::InvalidEnumVariant, _)));
    }

    #[test]
    fn from_int_invalid() {
        let result = OpenVpnPluginEvent::from_int(14);
        assert_matches!(result, Err(Error(ErrorKind::InvalidEnumVariant, _)));
    }

    #[test]
    fn event_enum_to_str() {
        let result = format!("{:?}", OpenVpnPluginEvent::Up);
        assert_eq!("Up", result);
    }
}

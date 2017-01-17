/// Constants for OpenVPN. Taken from include/openvpn-plugin.h in the OpenVPN repository:
/// https://github.com/OpenVPN/openvpn/blob/master/include/openvpn-plugin.h.in

use std::collections::HashMap;
use std::os::raw::{c_uint, c_int};


// All types of events that a plugin can receive from OpenVPN.
pub const OPENVPN_PLUGIN_UP: c_uint = 0;
pub const OPENVPN_PLUGIN_DOWN: c_uint = 1;
pub const OPENVPN_PLUGIN_ROUTE_UP: c_uint = 2;
pub const OPENVPN_PLUGIN_IPCHANGE: c_uint = 3;
pub const OPENVPN_PLUGIN_TLS_VERIFY: c_uint = 4;
pub const OPENVPN_PLUGIN_AUTH_USER_PASS_VERIFY: c_uint = 5;
pub const OPENVPN_PLUGIN_CLIENT_CONNECT: c_uint = 6;
pub const OPENVPN_PLUGIN_CLIENT_DISCONNECT: c_uint = 7;
pub const OPENVPN_PLUGIN_LEARN_ADDRESS: c_uint = 8;
pub const OPENVPN_PLUGIN_CLIENT_CONNECT_V2: c_uint = 9;
pub const OPENVPN_PLUGIN_TLS_FINAL: c_uint = 10;
pub const OPENVPN_PLUGIN_ENABLE_PF: c_uint = 11;
pub const OPENVPN_PLUGIN_ROUTE_PREDOWN: c_uint = 12;
pub const OPENVPN_PLUGIN_N: c_uint = 13;

lazy_static! {
    pub static ref PLUGIN_EVENT_NAMES: HashMap<c_uint, &'static str> = {
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
pub fn plugin_event_name(num: c_uint) -> Option<&'static str> {
    PLUGIN_EVENT_NAMES.get(&num).map(|s| *s)
}


// Return values. Returned from the plugin to OpenVPN to indicate success or failure. Can also
// Accept (success) or decline (error) an operation, such as incoming client connection attempt.
pub const OPENVPN_PLUGIN_FUNC_SUCCESS: c_int = 0;
pub const OPENVPN_PLUGIN_FUNC_ERROR: c_int = 1;
pub const OPENVPN_PLUGIN_FUNC_DEFERRED: c_int = 2;

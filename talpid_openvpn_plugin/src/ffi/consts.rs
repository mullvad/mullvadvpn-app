/// Constants for OpenVPN. Taken from include/openvpn-plugin.h in the OpenVPN repository:
/// https://github.com/OpenVPN/openvpn/blob/master/include/openvpn-plugin.h.in

use std::os::raw::c_int;

error_chain!{
    errors {
        InvalidEnumVariant(i: c_int) {
            description("Integer does not match any enum variant")
            display("{} is not a valid OPENVPN_PLUGIN_* constant", i)
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
            Err(ErrorKind::InvalidEnumVariant(i).into())
        }
    }
}


// Return values. Returned from the plugin to OpenVPN to indicate success or failure. Can also
// Accept (success) or decline (error) an operation, such as incoming client connection attempt.
pub const OPENVPN_PLUGIN_FUNC_SUCCESS: c_int = 0;
pub const OPENVPN_PLUGIN_FUNC_ERROR: c_int = 1;
#[allow(dead_code)]
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
            if OpenVpnPluginEvent::from_int(i).is_err() {
                panic!("{} not covered", i);
            }
        }
    }

    #[test]
    fn from_int_negative() {
        let result = OpenVpnPluginEvent::from_int(-5);
        assert_matches!(result, Err(Error(ErrorKind::InvalidEnumVariant(-5), _)));
    }

    #[test]
    fn from_int_invalid() {
        let result = OpenVpnPluginEvent::from_int(14);
        assert_matches!(result, Err(Error(ErrorKind::InvalidEnumVariant(14), _)));
    }

    #[test]
    fn event_enum_to_str() {
        let result = format!("{:?}", OpenVpnPluginEvent::Up);
        assert_eq!("Up", result);
    }
}

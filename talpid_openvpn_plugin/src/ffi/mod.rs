// FFI definitions for OpenVPN. See include/openvpn-plugin.h in the OpenVPN repository for
// the original declarations of these structs and functions along with documentation for them:
// https://github.com/OpenVPN/openvpn/blob/master/include/openvpn-plugin.h.in

use std::os::raw::c_int;

mod consts;
pub use self::consts::*;

mod structs;
pub use self::structs::*;

pub mod parse;



/// Translates a collection of `OpenVpnPluginEvent` instances into a bitmask in the format OpenVPN
/// expects it.
pub fn events_to_bitmask(events: &[OpenVpnPluginEvent]) -> c_int {
    let mut bitmask: c_int = 0;
    for event in events {
        bitmask |= 1 << (*event as i32);
    }
    bitmask
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn events_to_bitmask_no_events() {
        let result = events_to_bitmask(&[]);
        assert_eq!(0, result);
    }

    #[test]
    fn events_to_bitmask_one_event() {
        let result = events_to_bitmask(&[OpenVpnPluginEvent::Up]);
        assert_eq!(0b1, result);
    }

    #[test]
    fn events_to_bitmask_another_event() {
        let result = events_to_bitmask(&[OpenVpnPluginEvent::RouteUp]);
        assert_eq!(0b100, result);
    }

    #[test]
    fn events_to_bitmask_many_events() {
        let result = events_to_bitmask(&[OpenVpnPluginEvent::RouteUp, OpenVpnPluginEvent::N]);
        assert_eq!((1 << 13) + (1 << 2), result);
    }
}

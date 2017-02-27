// FFI definitions for OpenVPN. See include/openvpn-plugin.h in the OpenVPN repository for
// the original declarations of these structs and functions along with documentation for them:
// https://github.com/OpenVPN/openvpn/blob/master/include/openvpn-plugin.h.in

use std::os::raw::{c_char, c_int, c_uint, c_void};


#[allow(dead_code)]
pub mod consts;
use self::consts::*;

mod parse;

error_chain!{
    errors {
        InvalidEventType {
            description("Invalid event type constant")
        }
        ParseEnv {
            description("Unable to parse environment variables from OpenVPN")
        }
    }
}


/// Struct sent to `openvpn_plugin_open_v3` containing input values.
#[repr(C)]
pub struct openvpn_plugin_args_open_in {
    type_mask: c_int,
    argv: *const *const c_char,
    envp: *const *const c_char,
    callbacks: *const c_void,
    ssl_api: ovpnSSLAPI,
    ovpn_version: *const c_char,
    ovpn_version_major: c_uint,
    ovpn_version_minor: c_uint,
    ovpn_version_patch: *const c_char,
}

#[allow(dead_code)]
#[repr(C)]
enum ovpnSSLAPI {
    SSLAPI_NONE,
    SSLAPI_OPENSSL,
    SSLAPI_MBEDTLS,
}

/// Struct used for returning values from `openvpn_plugin_open_v3` to OpenVPN.
#[repr(C)]
pub struct openvpn_plugin_args_open_return {
    type_mask: c_int,
    handle: *const c_void,
    return_list: *const c_void,
}

/// Struct sent to `openvpn_plugin_func_v3` containing input values.
#[repr(C)]
pub struct openvpn_plugin_args_func_in {
    event_type: c_int,
    argv: *const *const c_char,
    envp: *const *const c_char,
    handle: *const c_void,
    per_client_context: *const c_void,
    current_cert_depth: c_int,
    current_cert: *const c_void,
}

/// Struct used for returning values from `openvpn_plugin_func_v3` to OpenVPN.
#[repr(C)]
pub struct openvpn_plugin_args_func_return {
    return_list: *const c_void,
}


/// Called by OpenVPN when the plugin is first loaded.
/// Used to register which events the plugin wants to listen to (`type_mask`). Can also return an
/// arbitrary object inside `handle` that will then be passed to all subsequent calls to the
/// plugin.
#[no_mangle]
pub extern "C" fn openvpn_plugin_open_v3(_version: c_int,
                                         _args: *const openvpn_plugin_args_open_in,
                                         retptr: *mut openvpn_plugin_args_open_return)
                                         -> c_int {
    println!("openvpn_plugin_open_v3()");
    unsafe {
        (*retptr).type_mask = events_to_bitmask(::INTERESTING_EVENTS);
    }
    OPENVPN_PLUGIN_FUNC_SUCCESS
}

/// Called by OpenVPN just before the plugin is unloaded. Should correctly close the plugin and
/// deallocate any `handle` initialized by the plugin in `openvpn_plugin_open_v3`
#[no_mangle]
pub extern "C" fn openvpn_plugin_close_v1(_handle: *const c_void) {
    println!("openvpn_plugin_close_v1()");
}

/// Called by OpenVPN for each OPENVPN_PLUGIN_* event that it registered for in
/// `openvpn_plugin_open_v3`
#[no_mangle]
pub extern "C" fn openvpn_plugin_func_v3(_version: c_int,
                                         args: *const openvpn_plugin_args_func_in,
                                         _retptr: *const openvpn_plugin_args_func_return)
                                         -> c_int {
    // TODO(linus): Add logging of errors
    match openvpn_plugin_func_v3_internal(args) {
        Ok(_) => OPENVPN_PLUGIN_FUNC_SUCCESS,
        Err(_) => OPENVPN_PLUGIN_FUNC_ERROR,
    }
}

fn openvpn_plugin_func_v3_internal(args: *const openvpn_plugin_args_func_in) -> Result<()> {
    let event_type = unsafe { (*args).event_type };
    let event = OpenVpnPluginEvent::from_int(event_type).chain_err(|| ErrorKind::InvalidEventType)?;
    println!("openvpn_plugin_func_v3({:?})", event);

    Ok(())
}


/// Translates a collection of `OpenVpnPluginEvent` instances into a bitmask in the format OpenVPN
/// expects it.
fn events_to_bitmask(events: &[OpenVpnPluginEvent]) -> c_int {
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

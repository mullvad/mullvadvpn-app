// FFI definitions for OpenVPN. See include/openvpn-plugin.h in the OpenVPN repository for
// the original declarations of these structs and functions along with documentation for them:
// https://github.com/OpenVPN/openvpn/blob/master/include/openvpn-plugin.h.in

use std::os::raw::{c_char, c_int, c_uint, c_void};


#[allow(dead_code)]
mod consts;
use self::consts::*;

mod parse;


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
                                         _retptr: *mut openvpn_plugin_args_open_return)
                                         -> c_int {
    println!("openvpn_plugin_open_v3()");
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
    let event_name = unsafe { consts::plugin_event_name((*args).event_type) };
    println!("openvpn_plugin_func_v3({})", event_name);
    OPENVPN_PLUGIN_FUNC_SUCCESS
}

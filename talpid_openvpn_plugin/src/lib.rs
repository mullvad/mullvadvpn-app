#[macro_use]
extern crate lazy_static;

#[macro_use]
extern crate error_chain;

#[macro_use]
extern crate assert_matches;

mod ffi;

/// Publicly export the functions making up the public interface of the plugin. These are the C FFI
/// functions called by OpenVPN.
pub use ffi::{openvpn_plugin_open_v3, openvpn_plugin_close_v1, openvpn_plugin_func_v3};

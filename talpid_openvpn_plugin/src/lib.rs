#[macro_use]
extern crate lazy_static;

#[macro_use]
extern crate error_chain;

#[cfg(test)]
#[macro_use]
extern crate assert_matches;

mod ffi;

/// Publicly export the functions making up the public interface of the plugin. These are the C FFI
/// functions called by OpenVPN.
pub use ffi::{openvpn_plugin_open_v3, openvpn_plugin_close_v1, openvpn_plugin_func_v3};

use ffi::consts::OpenVpnPluginEvent;


/// All the OpenVPN events this plugin will register for listening to. Edit this variable to change
/// events.
pub static INTERESTING_EVENTS: &'static [OpenVpnPluginEvent] = &[OpenVpnPluginEvent::Up,
                                                                 OpenVpnPluginEvent::RoutePredown];

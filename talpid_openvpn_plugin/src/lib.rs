#[macro_use]
extern crate lazy_static;

#[macro_use]
extern crate error_chain;

#[cfg(test)]
#[macro_use]
extern crate assert_matches;

use std::collections::HashMap;


mod ffi;

/// Publicly export the functions making up the public interface of the plugin. These are the C FFI
/// functions called by OpenVPN.
pub use ffi::{openvpn_plugin_open_v3, openvpn_plugin_close_v1, openvpn_plugin_func_v3};

use ffi::consts::OpenVpnPluginEvent;


error_chain!{}


/// Struct processing OpenVPN events and notifies listeners over IPC
struct EventProcessor;

impl EventProcessor {
    pub fn new() -> Result<EventProcessor> {
        Ok(EventProcessor)
    }

    pub fn process_event(&mut self, event: OpenVpnPluginEvent, env: HashMap<String, String>) {
        // TODO(linus): This is where we should send events to core.
        println!("Hello from EventProcessor: {:?}", event);
    }
}

impl Drop for EventProcessor {
    fn drop(&mut self) {
        // TODO(linus): If we need, this is where we send some shutdown event or similar to core.
        println!("Dropping EventProcessor!");
    }
}

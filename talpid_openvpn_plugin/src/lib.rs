#[macro_use]
extern crate lazy_static;

#[macro_use]
extern crate error_chain;

#[macro_use]
extern crate log;
extern crate env_logger;

#[cfg(test)]
#[macro_use]
extern crate assert_matches;

use std::os::raw::{c_int, c_void};

mod ffi;
mod processing;

use ffi::OpenVpnPluginEvent;
use processing::EventProcessor;


error_chain!{
    errors {
        InitHandleFailed {
            description("Unable to initialize event processor")
        }
        InvalidEventType {
            description("Invalid event type constant")
        }
        ParseEnvFailed {
            description("Unable to parse environment variables from OpenVPN")
        }
    }
}


/// All the OpenVPN events this plugin will register for listening to. Edit this variable to change
/// events.
pub static INTERESTING_EVENTS: &'static [OpenVpnPluginEvent] = &[OpenVpnPluginEvent::Up,
                                                                 OpenVpnPluginEvent::RoutePredown];


/// Called by OpenVPN when the plugin is first loaded.
/// Used to register which events the plugin wants to listen to (`args.type_mask`). Can also set an
/// arbitrary pointer inside `args.handle` that will then be passed to all subsequent calls to the
/// plugin.
#[no_mangle]
pub extern "C" fn openvpn_plugin_open_v3(_version: c_int,
                                         _args: *const ffi::openvpn_plugin_args_open_in,
                                         retptr: *mut ffi::openvpn_plugin_args_open_return)
                                         -> c_int {
    if init_logger().is_err() {
        return ffi::OPENVPN_PLUGIN_FUNC_ERROR;
    }
    match openvpn_plugin_open_v3_internal(retptr) {
        Ok(_) => ffi::OPENVPN_PLUGIN_FUNC_SUCCESS,
        Err(e) => {
            log_error("Unable to initialize plugin", &e);
            ffi::OPENVPN_PLUGIN_FUNC_ERROR
        }
    }
}

fn openvpn_plugin_open_v3_internal(retptr: *mut ffi::openvpn_plugin_args_open_return)
                                   -> Result<()> {
    debug!("Initializing plugin");
    let handle = Box::new(EventProcessor::new().chain_err(|| ErrorKind::InitHandleFailed)?);
    unsafe {
        (*retptr).type_mask = ffi::events_to_bitmask(INTERESTING_EVENTS);
        // Converting the handle into a raw pointer will make it escape Rust deallocation. See
        // `openvpn_plugin_close_v1` for deallocation.
        (*retptr).handle = Box::into_raw(handle) as *const c_void;
    }
    Ok(())
}


/// Called by OpenVPN just before the plugin is unloaded. Should correctly close the plugin and
/// deallocate any `handle` initialized by the plugin in `openvpn_plugin_open_v3`
#[no_mangle]
pub extern "C" fn openvpn_plugin_close_v1(handle: *const c_void) {
    debug!("Unloading plugin");
    // IMPORTANT: Bring the handle object back from a raw pointer. This will cause the handle
    // object to be properly deallocated right here.
    let _ = unsafe { Box::from_raw(handle as *mut EventProcessor) };
}


/// Called by OpenVPN for each OPENVPN_PLUGIN_* event that it registered for in
/// `openvpn_plugin_open_v3`
#[no_mangle]
pub extern "C" fn openvpn_plugin_func_v3(_version: c_int,
                                         args: *const ffi::openvpn_plugin_args_func_in,
                                         _retptr: *const ffi::openvpn_plugin_args_func_return)
                                         -> c_int {
    match openvpn_plugin_func_v3_internal(args) {
        Ok(_) => ffi::OPENVPN_PLUGIN_FUNC_SUCCESS,
        Err(e) => {
            log_error("Error while processing event", &e);
            ffi::OPENVPN_PLUGIN_FUNC_ERROR
        }
    }
}

fn openvpn_plugin_func_v3_internal(args: *const ffi::openvpn_plugin_args_func_in) -> Result<()> {
    let event_type = unsafe { (*args).event_type };
    let event = OpenVpnPluginEvent::from_int(event_type).chain_err(|| ErrorKind::InvalidEventType)?;
    debug!("Received event: {:?}", event);
    let env = unsafe { ffi::parse::env((*args).envp) }.chain_err(|| ErrorKind::ParseEnvFailed)?;

    let mut handle = unsafe { Box::from_raw((*args).handle as *mut EventProcessor) };
    handle.process_event(event, env);
    // Convert the handle back to a raw pointer to not deallocate it when we return.
    Box::into_raw(handle);

    Ok(())
}



pub fn init_logger() -> ::std::result::Result<(), ()> {
    env_logger::init().or_else(|e| {
        use std::io::Write;
        let mut stderr = ::std::io::stderr();
        writeln!(&mut stderr, "Unable to initialize logging: {}", e)
            .expect("Unable to write to stderr");
        Err(())
    })
}

pub fn log_error(msg: &str, error: &Error) {
    error!("{}", msg);
    for e in error.iter() {
        error!("caused by: {}", e);
    }
    // When running with RUST_BACKTRACE=1, print backtrace.
    if let Some(backtrace) = error.backtrace() {
        error!("backtrace: {:?}", backtrace);
    }
}

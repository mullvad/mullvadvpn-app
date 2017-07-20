#[macro_use]
extern crate error_chain;
#[macro_use]
extern crate log;
extern crate env_logger;

#[macro_use]
extern crate openvpn_plugin;
extern crate talpid_ipc;

use openvpn_plugin::types::{EventResult, OpenVpnPluginEvent};
use std::collections::HashMap;
use std::ffi::CString;

mod processing;
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
        ParseArgsFailed {
            description("Unable to parse arguments from OpenVPN")
        }
        EventProcessingFailed {
            description("Failed to process the event")
        }
    }
}


/// All the OpenVPN events this plugin will register for listening to. Edit this variable to change
/// events.
pub static INTERESTING_EVENTS: &'static [OpenVpnPluginEvent] =
    &[OpenVpnPluginEvent::Up, OpenVpnPluginEvent::RoutePredown];

openvpn_plugin!(
    ::openvpn_open,
    ::openvpn_close,
    ::openvpn_event,
    ::EventProcessor
);

fn openvpn_open(args: &[CString],
                _env: &HashMap<CString, CString>)
                -> Result<(Vec<OpenVpnPluginEvent>, EventProcessor)> {
    env_logger::init().chain_err(|| "Failed to bootstrap logging system")?;
    debug!("Initializing plugin");

    let core_server_id = parse_args(args)?;
    info!("Connecting back to talpid core at {}", core_server_id);
    let processor = EventProcessor::new(core_server_id).chain_err(|| ErrorKind::InitHandleFailed)?;

    Ok((INTERESTING_EVENTS.to_vec(), processor))
}

fn parse_args(args: &[CString]) -> Result<talpid_ipc::IpcServerId> {
    let mut args_iter = openvpn_plugin::ffi::parse::string_array_utf8(args)
        .chain_err(|| ErrorKind::ParseArgsFailed)?
        .into_iter();
    let _plugin_path = args_iter.next();
    let core_server_id: talpid_ipc::IpcServerId = args_iter.next()
        .ok_or_else(|| ErrorKind::Msg("No core server id given as first argument".to_owned()))?;
    Ok(core_server_id)
}


fn openvpn_close(_handle: EventProcessor) {
    debug!("Unloading plugin");
}

fn openvpn_event(event: OpenVpnPluginEvent,
                 _args: &[CString],
                 env: &HashMap<CString, CString>,
                 handle: &mut EventProcessor)
                 -> Result<EventResult> {
    debug!("Received event: {:?}", event);

    let parsed_env = openvpn_plugin::ffi::parse::env_utf8(env)
        .chain_err(|| ErrorKind::ParseEnvFailed)?;

    handle.process_event(event, parsed_env).chain_err(|| ErrorKind::EventProcessingFailed)?;
    Ok(EventResult::Success)
}

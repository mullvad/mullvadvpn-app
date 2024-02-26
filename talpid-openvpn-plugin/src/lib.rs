use openvpn_plugin::{openvpn_plugin, EventResult, EventType};
use std::{collections::HashMap, ffi::CString, io, sync::Mutex};
use talpid_types::ErrorExt;

mod processing;
use crate::processing::EventProcessor;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("No core server id given as first argument")]
    MissingCoreServerId,

    #[error("Failed to send an event to daemon over the IPC channel")]
    SendEvent(#[source] tonic::Status),

    #[error("Unable to start Tokio runtime")]
    CreateRuntime(#[source] io::Error),

    #[error("Unable to create IPC transport")]
    CreateTransport(#[source] tonic::transport::Error),

    #[error("Unable to parse environment variables from OpenVPN")]
    ParseEnvFailed(#[source] std::str::Utf8Error),

    #[error("Unable to parse arguments from OpenVPN")]
    ParseArgsFailed(#[source] std::str::Utf8Error),

    #[error("Unhandled event type: {:?}", _0)]
    UnhandledEvent(openvpn_plugin::EventType),
}

/// All the OpenVPN events this plugin will register for listening to. Edit this variable to change
/// events.
pub static INTERESTING_EVENTS: &[EventType] = &[
    EventType::AuthFailed,
    EventType::Up,
    EventType::RouteUp,
    EventType::RoutePredown,
];

openvpn_plugin!(
    crate::openvpn_open,
    crate::openvpn_close,
    crate::openvpn_event,
    crate::Mutex<Option<EventProcessor>>
);

pub struct Arguments {
    ipc_socket_path: String,
}

fn openvpn_open(
    args: Vec<CString>,
    _env: HashMap<CString, CString>,
) -> Result<(Vec<EventType>, Mutex<Option<EventProcessor>>), Error> {
    env_logger::init();
    log::debug!("Initializing plugin");

    let arguments = parse_args(&args)?;
    log::info!(
        "Connecting back to talpid core at {}",
        arguments.ipc_socket_path
    );
    let processor = EventProcessor::new(arguments)?;

    Ok((INTERESTING_EVENTS.to_vec(), Mutex::new(Some(processor))))
}

fn parse_args(args: &[CString]) -> Result<Arguments, Error> {
    let mut args_iter = openvpn_plugin::ffi::parse::string_array_utf8(args)
        .map_err(Error::ParseArgsFailed)?
        .into_iter();

    let _plugin_path = args_iter.next();
    let ipc_socket_path: String = args_iter.next().ok_or(Error::MissingCoreServerId)?;

    Ok(Arguments { ipc_socket_path })
}

fn openvpn_close(_handle: Mutex<Option<EventProcessor>>) {
    log::info!("Unloading plugin");
}

fn openvpn_event(
    event: EventType,
    _args: Vec<CString>,
    env: HashMap<CString, CString>,
    handle: &mut Mutex<Option<EventProcessor>>,
) -> Result<EventResult, Error> {
    log::debug!("Received event: {:?}", event);

    let parsed_env = openvpn_plugin::ffi::parse::env_utf8(&env).map_err(Error::ParseEnvFailed)?;

    let ctx = handle
        .get_mut()
        .expect("failed to obtain mutex for EventProcessor");
    if let Some(processor) = ctx.as_mut() {
        match processor.process_event(event, parsed_env) {
            Ok(()) => Ok(EventResult::Success),
            Err(e) => {
                log::error!("{}", e.display_chain());
                *ctx = None;
                Ok(EventResult::Failure)
            }
        }
    } else {
        log::error!("Client has been closed");
        Ok(EventResult::Failure)
    }
}

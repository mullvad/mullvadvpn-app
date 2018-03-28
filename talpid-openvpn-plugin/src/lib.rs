//! # License
//!
//! Copyright (C) 2017  Amagicom AB
//!
//! This program is free software: you can redistribute it and/or modify it under the terms of the
//! GNU General Public License as published by the Free Software Foundation, either version 3 of
//! the License, or (at your option) any later version.

extern crate env_logger;
#[macro_use]
extern crate error_chain;
#[macro_use]
extern crate log;

#[macro_use]
extern crate jsonrpc_client_core;
extern crate futures;
extern crate jsonrpc_client_ipc;
#[macro_use]
extern crate openvpn_plugin;
extern crate tokio_core;

use error_chain::ChainedError;
use openvpn_plugin::types::{EventResult, OpenVpnPluginEvent};
use std::collections::HashMap;
use std::ffi::CString;
use std::sync::Mutex;


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
pub static INTERESTING_EVENTS: &'static [OpenVpnPluginEvent] = &[
    OpenVpnPluginEvent::AuthFailed,
    OpenVpnPluginEvent::Up,
    OpenVpnPluginEvent::RoutePredown,
];

openvpn_plugin!(
    ::openvpn_open,
    ::openvpn_close,
    ::openvpn_event,
    ::Mutex<EventProcessor>
);

pub struct Arguments {
    ipc_socket_path: String,
}

fn openvpn_open(
    args: Vec<CString>,
    _env: HashMap<CString, CString>,
) -> Result<(Vec<OpenVpnPluginEvent>, Mutex<EventProcessor>)> {
    env_logger::init();
    debug!("Initializing plugin");

    let arguments = parse_args(&args)?;
    info!(
        "Connecting back to talpid core at {}",
        arguments.ipc_socket_path
    );
    let processor = EventProcessor::new(arguments).chain_err(|| ErrorKind::InitHandleFailed)?;

    Ok((INTERESTING_EVENTS.to_vec(), Mutex::new(processor)))
}

fn parse_args(args: &[CString]) -> Result<Arguments> {
    let mut args_iter = openvpn_plugin::ffi::parse::string_array_utf8(args)
        .chain_err(|| ErrorKind::ParseArgsFailed)?
        .into_iter();

    let _plugin_path = args_iter.next();
    let ipc_socket_path: String = args_iter
        .next()
        .ok_or_else(|| ErrorKind::Msg("No core server id given as first argument".to_owned()))?;

    Ok(Arguments { ipc_socket_path })
}


fn openvpn_close(_handle: Mutex<EventProcessor>) {
    info!("Unloading plugin");
}

fn openvpn_event(
    event: OpenVpnPluginEvent,
    _args: Vec<CString>,
    env: HashMap<CString, CString>,
    handle: &mut Mutex<EventProcessor>,
) -> Result<EventResult> {
    debug!("Received event: {:?}", event);

    let parsed_env =
        openvpn_plugin::ffi::parse::env_utf8(&env).chain_err(|| ErrorKind::ParseEnvFailed)?;

    let result = handle
        .lock()
        .expect("failed to obtain mutex for EventProcessor")
        .process_event(event, parsed_env)
        .chain_err(|| ErrorKind::EventProcessingFailed);
    match result {
        Ok(()) => Ok(EventResult::Success),
        Err(e) => {
            error!("{}", e.display_chain());
            Ok(EventResult::Failure)
        }
    }
}

#![deny(missing_docs)]

//! The core components of the talpidaemon VPN client.
//!
//! # License
//!
//! Copyright (C) 2017  Amagicom AB
//!
//! This program is free software: you can redistribute it and/or modify it under the terms of the
//! GNU General Public License as published by the Free Software Foundation, either version 3 of
//! the License, or (at your option) any later version.

extern crate atty;
extern crate duct;
#[macro_use]
extern crate log;

#[macro_use]
extern crate error_chain;
#[cfg(target_os = "linux")]
extern crate failure;
extern crate futures;
#[cfg(unix)]
extern crate ipnetwork;
extern crate jsonrpc_core;
#[macro_use]
extern crate jsonrpc_macros;
#[cfg(unix)]
#[macro_use]
extern crate lazy_static;
extern crate libc;
extern crate shell_escape;
extern crate tokio_core;
extern crate uuid;
#[cfg(target_os = "linux")]
extern crate which;
#[cfg(windows)]
extern crate winreg;

extern crate openvpn_plugin;
extern crate talpid_ipc;
extern crate talpid_types;

#[cfg(target_os = "linux")]
#[macro_use]
extern crate nftnl;

#[cfg(windows)]
#[macro_use]
mod ffi;

/// Working with processes.
pub mod process;

/// Abstracts over different VPN tunnel technologies
pub mod tunnel;

/// Helper function to preserve previous log files.
pub mod logging;

/// Abstractions and extra features on `std::mpsc`
pub mod mpsc;

/// Abstractions over operating system network security settings.
pub mod security;

/// State machine to handle tunnel configuration.
pub mod tunnel_state_machine;

mod mktemp;

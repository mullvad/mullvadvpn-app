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
extern crate log;

#[macro_use]
extern crate error_chain;
#[cfg(target_os = "linux")]
extern crate failure;
extern crate futures;
#[cfg(unix)]
extern crate ipnetwork;
extern crate jsonrpc_core;
extern crate jsonrpc_macros;
#[cfg(unix)]
extern crate lazy_static;
extern crate libc;
#[cfg(unix)]
extern crate nix;
extern crate shell_escape;
extern crate tokio_core;
#[cfg(unix)]
extern crate tun;
extern crate uuid;
#[cfg(target_os = "linux")]
extern crate which;
#[cfg(windows)]
extern crate widestring;
#[cfg(windows)]
extern crate winreg;

extern crate openvpn_plugin;
extern crate talpid_ipc;
extern crate talpid_types;

#[cfg(windows)]
mod winnet;

#[cfg(unix)]
/// Working with IP interface devices
pub mod network_interface;
#[cfg(unix)]
/// Abstraction over operating system routing table.
pub mod routing;

mod offline;

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

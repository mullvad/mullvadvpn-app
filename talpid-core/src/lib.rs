#![deny(missing_docs)]
#![deny(rust_2018_idioms)]

//! The core components of the talpidaemon VPN client.
//!
//! # License
//!
//! Copyright (C) 2017  Amagicom AB
//!
//! This program is free software: you can redistribute it and/or modify it under the terms of the
//! GNU General Public License as published by the Free Software Foundation, either version 3 of
//! the License, or (at your option) any later version.

/// Misc FFI utilities.
#[cfg(windows)]
#[macro_use]
mod ffi;

/// Misc networking functions for Windows.
#[cfg(windows)]
mod winnet;

#[cfg(any(target_os = "linux", target_os = "macos"))]
/// Working with IP interface devices
pub mod network_interface;
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

/// Abstractions over operating system firewalls.
pub mod firewall;

/// Abstractions over operating system DNS settings.
pub mod dns;

/// State machine to handle tunnel configuration.
pub mod tunnel_state_machine;

#[cfg(not(target_os = "android"))]
/// Internal code for managing bundled proxy software.
mod proxy;

#[cfg(not(target_os = "android"))]
mod mktemp;

/// Misc utilities for the Linux platform.
#[cfg(target_os = "linux")]
mod linux;

/// A pair of functions to monitor and establish connectivity with ICMP
mod ping_monitor;

#[cfg(target_os = "windows")]
#[macro_use]
extern crate lazy_static;

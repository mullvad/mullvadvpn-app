//! The core components of the talpidaemon VPN client.

#![deny(missing_docs)]
#![deny(rust_2018_idioms)]
#![recursion_limit = "1024"]

/// Misc FFI utilities.
#[cfg(windows)]
#[macro_use]
mod ffi;

/// Windows API wrappers and utilities
#[cfg(target_os = "windows")]
pub mod windows;

#[cfg(any(target_os = "linux", target_os = "macos"))]
/// Working with IP interface devices
pub mod network_interface;
/// Abstraction over operating system routing table.
pub mod routing;

mod offline;

/// Split tunneling
pub mod split_tunnel;

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

/// Future utilities
pub mod future_retry;

#[cfg(not(target_os = "android"))]
/// Internal code for managing bundled proxy software.
mod proxy;

#[cfg(not(target_os = "android"))]
mod mktemp;

/// Misc utilities for the Linux platform.
#[cfg(target_os = "linux")]
mod linux;

/// A pair of functions to monitor and establish connectivity with ICMP
pub mod ping_monitor;

/// A resolver that's controlled by the tunnel state machine
#[cfg(target_os = "macos")]
pub mod resolver;

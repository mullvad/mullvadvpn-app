//! The core components of the talpidaemon VPN client.

#![deny(missing_docs)]
#![recursion_limit = "1024"]

/// Window API wrappers and utilities
#[cfg(target_os = "windows")]
pub mod window;

mod offline;

/// Abstracts over different VPN tunnel technologies
pub mod tunnel;

/// Helper function to preserve previous log files.
pub mod logging;

/// Abstractions and extra features on `std::mpsc`
pub mod mpsc;

/// State machine to handle tunnel configuration.
pub mod tunnel_state_machine;

/// A resolver that's controlled by the tunnel state machine
#[cfg(target_os = "macos")]
pub(crate) mod resolver;

/// Connectivity monitor for Android
#[cfg(target_os = "android")]
pub mod connectivity_listener;

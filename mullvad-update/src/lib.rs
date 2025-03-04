//! Support functions for securely installing or updating Mullvad VPN

#[cfg(all(feature = "client", any(target_os = "windows", target_os = "macos")))]
mod client;

#[cfg(all(feature = "client", any(target_os = "windows", target_os = "macos")))]
pub use client::*;

pub mod version;

/// Parser and serializer for version metadata
pub mod format;

//! Support functions for securely installing or updating Mullvad VPN

#[cfg(feature = "client")]
mod client;

#[cfg(feature = "client")]
pub use client::*;

pub mod version;

/// Parser and serializer for version metadata
pub mod format;

#[cfg(feature = "client")]
pub mod hash;

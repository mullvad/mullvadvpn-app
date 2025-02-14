//! Support functions for securely installing or updating Mullvad VPN

pub mod api;
pub mod app;
pub mod dir;
pub mod fetch;
pub mod verify;
pub mod version;

/// Parser and serializer for version metadata
pub mod format;

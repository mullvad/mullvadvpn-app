#[cfg(target_os = "android")]
pub mod android;
pub mod net;
pub mod tunnel;

#[cfg(target_os = "windows")]
pub mod split_tunnel;

pub mod drop_guard;

mod error;
use std::time::SystemTime;

pub use error::*;

/// Contains bytes sent and received through a tunnel
// FIXME: dedup
#[derive(Default, PartialEq, Eq, Clone, Debug)]
pub struct Stats {
    pub tx_bytes: u64,
    pub rx_bytes: u64,
    pub last_handshake_time: Option<SystemTime>,
}

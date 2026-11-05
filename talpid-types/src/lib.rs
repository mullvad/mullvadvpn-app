pub mod aligned_bytes;
#[cfg(target_os = "android")]
pub mod android;
pub mod net;
#[cfg(target_os = "windows")]
pub mod split_tunnel;
pub mod tunnel;
#[cfg(target_os = "windows")]
pub mod windows;

pub mod drop_guard;

use std::time::SystemTime;

/// Contains bytes sent and received through a tunnel
#[derive(Default, PartialEq, Eq, Clone, Debug)]
pub struct Stats {
    pub tx_bytes: u64,
    pub rx_bytes: u64,
    pub last_handshake_time: Option<SystemTime>,
}

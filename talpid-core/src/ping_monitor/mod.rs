#[cfg(any(target_os = "android", target_os = "macos", target_os = "linux"))]
#[path = "unix.rs"]
mod imp;

#[path = "win.rs"]
mod imp;

pub use imp::{monitor_ping, ping, Error};

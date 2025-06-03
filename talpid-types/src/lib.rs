#[cfg(target_os = "android")]
pub mod android;
pub mod net;
pub mod tunnel;

#[cfg(target_os = "linux")]
pub mod cgroup;

#[cfg(target_os = "windows")]
pub mod split_tunnel;

mod error;
pub use error::*;

#[cfg(any(target_os = "android", target_os = "macos", target_os = "linux"))]
#[path = "unix.rs"]
mod imp;


#[cfg(target_os = "windows")]
#[path = "win.rs"]
mod imp;

pub use imp::{Error, Pinger};

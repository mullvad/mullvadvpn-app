#[cfg(target_os = "linux")]
#[path = "linux.rs"]
mod imp;

#[cfg(windows)]
#[path = "windows.rs"]
mod imp;

#[cfg(target_os = "macos")]
#[path = "macos.rs"]
mod imp;

pub use imp::*;

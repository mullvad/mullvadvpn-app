#[cfg(all(target_os = "linux"))]
#[path = "linux_v1.rs"]
mod imp;

mod imp;

#[cfg(windows)]
#[path = "windows/mod.rs"]
mod imp;

#[cfg(target_os = "macos")]
#[path = "macos/mod.rs"]
mod imp;

#[cfg(target_os = "android")]
#[path = "android.rs"]
mod imp;

pub use imp::*;

#[cfg(target_os = "linux")]
#[path = "linux.rs"]
mod imp;

#[cfg(target_os = "linux")]
pub use imp::*;

#[cfg(windows)]
#[path = "windows/mod.rs"]
mod imp;

#[cfg(windows)]
pub use imp::*;

#[cfg(target_os = "android")]
#[path = "android.rs"]
mod imp;

#[cfg(target_os = "android")]
pub use imp::*;

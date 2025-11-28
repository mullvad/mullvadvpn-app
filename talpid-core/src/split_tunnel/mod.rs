#[cfg(all(target_os = "linux", not(feature = "cgroups_v2")))]
#[path = "linux_v1.rs"]
mod imp;

#[cfg(all(target_os = "linux", feature = "cgroups_v2"))]
#[path = "linux_v2.rs"]
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

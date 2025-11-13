#[cfg(all(target_os = "linux", not(feature = "linux-netns")))]
#[path = "linux_croups_v1.rs"]
mod imp;

#[cfg(all(target_os = "linux", feature = "linux-netns"))]
#[path = "linux_netns.rs"]
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

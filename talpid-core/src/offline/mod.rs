#[cfg(target_os = "macos")]
#[path = "macos.rs"]
mod imp;

#[cfg(target_os = "windows")]
#[path = "windows.rs"]
mod imp;

#[cfg(not(any(windows, target_os = "macos")))]
#[path = "dummy.rs"]
mod imp;

pub use self::imp::{is_offline, spawn_monitor};

#[cfg(target_os = "macos")]
#[path = "macos.rs"]
mod imp;

#[cfg(not(target_os = "macos"))]
#[path = "dummy.rs"]
mod imp;

pub use self::imp::{is_offline, spawn_monitor};

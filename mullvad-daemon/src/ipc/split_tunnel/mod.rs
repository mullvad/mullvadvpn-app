#[cfg(target_os = "linux")]
#[path = "linux.rs"]
pub(super) mod imp;

#[cfg(windows)]
#[path = "windows.rs"]
pub(super) mod imp;

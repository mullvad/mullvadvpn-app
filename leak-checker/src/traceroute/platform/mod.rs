/// Traceroute implementation for windows.
#[cfg(target_os = "windows")]
pub mod windows;

/// Traceroute implementation for unix.
#[cfg(unix)]
pub mod unix;

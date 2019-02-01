#[cfg(target_os = "macos")]
#[path = "macos.rs"]
mod imp;

#[cfg(target_os = "linux")]
#[path = "linux/mod.rs"]
mod imp;

#[cfg(windows)]
#[path = "windows.rs"]
mod imp;

pub use self::imp::{DnsError, Error};

pub use self::imp::{DnsMonitor, Error as DnsError};
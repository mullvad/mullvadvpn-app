mod arch;
#[cfg(target_os = "linux")]
#[path = "linux.rs"]
mod imp;

#[cfg(target_os = "macos")]
#[path = "macos.rs"]
mod imp;

#[cfg(windows)]
#[path = "windows.rs"]
mod imp;

#[cfg(target_os = "android")]
#[path = "android.rs"]
mod imp;

#[cfg(target_os = "macos")]
pub use self::imp::MacosVersion;
#[cfg(windows)]
pub use self::imp::WindowsVersion;
pub use self::imp::{extra_metadata, short_version, version};

pub use arch::get_native_arch;
pub use arch::Architecture;

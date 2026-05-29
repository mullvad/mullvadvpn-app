#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "macos")]
mod macos;

mod rule;
pub use rule::{BlockRule, Endpoints};

#[cfg(target_os = "linux")]
pub use linux::BlockList;
#[cfg(target_os = "macos")]
pub use macos::BlockList;

#[cfg(target_os = "macos")]
pub use macos::{apply_dnat, cleanup_dnat};

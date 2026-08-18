#[cfg(target_os = "linux")]
mod linux;

mod rule;
pub use rule::{BlockRule, Endpoints};

#[cfg(target_os = "linux")]
pub use linux::BlockList;

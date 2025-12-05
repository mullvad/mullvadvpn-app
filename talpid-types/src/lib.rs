#[cfg(target_os = "android")]
pub mod android;
pub mod net;
pub mod tunnel;

#[cfg(target_os = "windows")]
pub mod split_tunnel;

pub mod drop_guard;

mod error;
pub use error::*;

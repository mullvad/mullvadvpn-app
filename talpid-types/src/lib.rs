#[cfg(target_os = "android")]
pub mod android;
pub mod net;
pub mod tunnel;

#[cfg(any(target_os = "windows", target_os = "android"))]
pub mod split_tunnel;

pub mod drop_guard;

mod error;
pub use error::*;

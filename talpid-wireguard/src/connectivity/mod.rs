mod check;
mod constants;
mod error;
#[cfg(test)]
mod mock;
mod monitor;
mod pinger;

#[cfg(all(target_os = "android", feature = "wireguard-go"))]
pub use check::CancelReceiver;
pub use check::{CancelToken, Check};
pub use error::Error;
pub use monitor::Monitor;

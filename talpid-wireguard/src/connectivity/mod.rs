mod check;
mod constants;
mod error;
#[cfg(test)]
mod mock;
mod monitor;
mod pinger;

#[cfg(any(target_os = "android", force_wireguard_handshake))]
pub use check::Cancellable;
pub use check::Check;
pub use error::Error;
pub use monitor::Monitor;

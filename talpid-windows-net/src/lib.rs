//! Interface with low-level windows specific bits.

#![deny(missing_docs)]
#![deny(rust_2018_idioms)]

/// Nicer interfaces with Windows networking code.
#[cfg(windows)]
pub mod net;
#[cfg(windows)]
pub use net::*;

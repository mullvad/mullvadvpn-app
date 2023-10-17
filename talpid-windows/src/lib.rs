//! Interface with low-level Windows-specific bits.

#![deny(missing_docs)]
#![deny(rust_2018_idioms)]
#![cfg(windows)]

/// I/O
pub mod io;

/// Networking
pub mod net;

/// Synchronization
pub mod sync;

/// Processes
pub mod process;

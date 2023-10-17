//! Interface with low-level windows specific bits.

#![deny(missing_docs)]
#![deny(rust_2018_idioms)]

/// Windows I/O
#[cfg(windows)]
pub mod io;

/// Synchronization (event objects, etc.)
#[cfg(windows)]
pub mod sync;

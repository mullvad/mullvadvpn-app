//! Interface with low-level Windows-specific bits.

#![deny(missing_docs)]
#![cfg(windows)]

/// File system
pub mod fs;

/// I/O
pub mod io;

/// Networking
pub mod net;

/// Synchronization
pub mod sync;

/// Processes
pub mod process;

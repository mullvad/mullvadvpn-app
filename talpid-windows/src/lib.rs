//! Interface with low-level Windows-specific bits.

#![deny(missing_docs)]
#![cfg(windows)]

/// Environment
pub mod env;

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

/// String functions
pub mod string;

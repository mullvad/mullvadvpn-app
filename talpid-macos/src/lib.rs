//! Interface with macOS-specific bits.

#![deny(missing_docs)]
#![cfg(target_os = "macos")]

/// Processes
pub mod process;

/// TCC approval checks
mod fda;

/// Check whether the current process has full-disk access enabled.
pub use fda::has_full_disk_access;

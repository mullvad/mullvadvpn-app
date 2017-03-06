#![deny(missing_docs)]

//! The core components of the talpidaemon VPN client.

#[cfg(test)]
#[macro_use]
extern crate assert_matches;

extern crate clonablechild;

#[macro_use]
extern crate error_chain;

extern crate talpid_ipc;

/// Working with processes.
pub mod process;

/// Network primitives.
pub mod net;

#![deny(missing_docs)]

//! The core components of the talpidaemon VPN client.

#[macro_use]
extern crate assert_matches;

extern crate clonablechild;

/// Working with processes.
pub mod process;

/// Network primitives.
pub mod net;

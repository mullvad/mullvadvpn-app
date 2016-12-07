#![deny(missing_docs)]

//! The core components of the talpidaemon VPN client.

extern crate regex;

/// Working with processes.
pub mod process;

/// Network primitives.
pub mod net;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {}
}

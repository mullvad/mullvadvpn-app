#[cfg(test)]
#[macro_use]
extern crate assert_matches;

#[macro_use]
extern crate error_chain;

mod ipc;
pub use ipc::*;

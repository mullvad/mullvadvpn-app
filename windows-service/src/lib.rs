#![cfg(windows)]

#[macro_use]
extern crate bitflags;
#[macro_use]
extern crate error_chain;
extern crate widestring;
extern crate winapi;

pub use error_chain::ChainedError;

pub mod service;
pub mod service_control_handler;
pub mod service_manager;
#[macro_use]
pub mod service_dispatcher;

mod shell_escape;

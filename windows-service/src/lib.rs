#![cfg(windows)]

#[macro_use]
extern crate bitflags;
#[macro_use]
pub extern crate error_chain;
extern crate widestring;
extern crate winapi;

pub mod service;
pub mod service_control_handler;
pub mod service_manager;
#[macro_use]
pub mod service_dispatcher;

mod shell_escape;

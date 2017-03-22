#[macro_use]
extern crate error_chain;
#[macro_use]
extern crate log;

extern crate serde;

#[cfg(windows)]
#[path = "nop_ipc.rs"]
mod ipc_impl;

#[cfg(not(windows))]
#[path = "zmq_ipc.rs"]
mod ipc_impl;

pub use self::ipc_impl::*;

pub mod http_ipc;


/// An Id created by the Ipc server that the client can use to connect to it
pub type IpcServerId = String;

error_chain!{
    errors {
        ReadFailure {
            description("Could not read IPC message")
        }
        ParseFailure {
            description("Unable to serialize/deserialize message")
        }
        CouldNotStartServer {
            description("Failed to start the IPC server")
        }
        SendError {
            description("Unable to send message")
        }
    }
}

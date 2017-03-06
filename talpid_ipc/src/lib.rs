#[macro_use]
extern crate error_chain;

extern crate serde;

#[cfg(windows)]
#[path = "nop_ipc.rs"]
mod ipc_impl;

#[cfg(not(windows))]
#[path = "zmq_ipc.rs"]
mod ipc_impl;

pub use self::ipc_impl::*;


/// An Id created by the Ipc server that the client can use to connect to it
type IpcServerId = String;

error_chain!{
    errors {
        ReadFailure {
            description("Could not read IPC message")
        }
        CouldNotStartServer {
            description("Failed to start the IPC server")
        }
        SendError {
            description("Unable to send message")
        }
    }
}

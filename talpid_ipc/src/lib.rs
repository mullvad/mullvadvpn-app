#[macro_use]
extern crate error_chain;

#[cfg(windows)]
#[path = "nop_ipc.rs"]
mod ipc_impl;

#[cfg(not(windows))]
#[path = "zmq_ipc.rs"]
mod ipc_impl;

pub use self::ipc_impl::*;

/// The type signature for functions accepting messages from the server.
/// If the server fails in delivering the message for any reason it will
/// put the cause in the Err part of the `Result`.
pub type OnMessage<MessageType> = FnMut(Result<MessageType>) + Send + 'static;

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
    }
}

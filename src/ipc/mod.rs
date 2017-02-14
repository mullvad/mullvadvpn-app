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

/// The server end of our Inter-Process Communcation implementation.
pub trait IpcServer {
    type MessageType;

    /// Starts listening to incoming IPC connections on the specified port.
    /// Messages are sent to the `on_message` callback. If anything went wrong
    /// when reading or parsing the message, the message will be an `Err`.
    /// NOTE that this does not apply to errors regarding whether the server
    /// could start or not, those are returned directly by this function.
    ///
    /// This function is non-blocking and thus spawns a thread where it
    /// listens to messages.
    fn start(self, port: u16, on_message: Box<OnMessage<Self::MessageType>>) -> Result<()>;
}

error_chain!{
    errors {
        ReadFailure {
            description("Could not read IPC message")
        }
        CouldNotStartServer {
            description("Failed to start the IPC server")
        }
        InvalidMessage(message: Vec<u8>) {
            description("The IPC server got a message it did not know how to handle")
        }
    }
}

/// This file only exists because we cannot get ZeroMQ to work on
/// Windows. This is not a valid IPC implementation and us using
/// it on Windows will result in a non-functioning client.
///
/// We plan on trying with ZMQ again in the future.
/// Erik, 2017-02-09

use ipc::{IpcServer, OnMessage, ErrorKind, Result};

pub struct NopIpcServer;
impl IpcServer for NopIpcServer {
    type MessageType = String;

    fn start(self, _on_message: Box<OnMessage<Self::MessageType>>) -> Result<()> {
        Err(ErrorKind::CouldNotStartServer.into())
    }
}

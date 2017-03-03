use super::{OnMessage, ErrorKind, Result, IpcServerId};

/// This implementation only exists because we cannot get ZeroMQ to work on
/// Windows. This is not a valid IPC implementation and us using
/// it on Windows will result in a non-functioning client.
///
/// We plan on trying with ZMQ again in the future.
/// Erik, 2017-02-09
fn start_new_server(_on_message: Box<OnMessage<Vec<u8>>>) -> Result<IpcServerId> {
    Err(ErrorKind::CouldNotStartServer.into())
}

pub struct IpcClient;
impl IpcClient {
    pub fn new(server_id: IpcServerId) -> Self {
        IpcClient
    }

    pub fn send(mut self, message: &[u8]) -> Result<()> {
        Err(ErrorKind::SendError.into())
    }
}

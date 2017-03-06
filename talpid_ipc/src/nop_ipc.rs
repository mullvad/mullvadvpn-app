use super::{ErrorKind, Result, IpcServerId};

use serde;

/// This implementation only exists because we cannot get ZeroMQ to work on
/// Windows. This is not a valid IPC implementation and us using
/// it on Windows will result in a non-functioning client.
///
/// We plan on trying with ZMQ again in the future.
/// Erik, 2017-02-09
pub fn start_new_server<T, F>(on_message: F) -> Result<IpcServerId>
    where T: serde::Deserialize + 'static,
          F: FnMut(Result<T>) + Send + 'static
{
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

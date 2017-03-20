use super::{ErrorKind, Result, IpcServerId};

use serde;

/// This implementation only exists because we cannot get ZeroMQ to work on
/// Windows. This is not a valid IPC implementation and us using
/// it on Windows will result in a non-functioning client.
///
/// We plan on trying with ZMQ again in the future.
/// Erik, 2017-02-09
pub fn start_new_server<T, F>(_on_message: F) -> Result<IpcServerId>
    where T: serde::Deserialize + 'static,
          F: FnMut(Result<T>) + Send + 'static
{
    bail!(ErrorKind::CouldNotStartServer);
}

pub struct IpcClient<T>
    where T: serde::Serialize
{
    _phantom: ::std::marker::PhantomData<T>,
}

impl<T> IpcClient<T>
    where T: serde::Serialize
{
    pub fn new(_server_id: IpcServerId) -> Self {
        IpcClient { _phantom: ::std::marker::PhantomData }
    }

    pub fn send(&mut self, _message: &T) -> Result<()> {
        bail!(ErrorKind::SendError);
    }
}

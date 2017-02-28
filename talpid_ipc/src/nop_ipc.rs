use super::{OnMessage, ErrorKind, Result};

/// This implementation only exists because we cannot get ZeroMQ to work on
/// Windows. This is not a valid IPC implementation and us using
/// it on Windows will result in a non-functioning client.
///
/// We plan on trying with ZMQ again in the future.
/// Erik, 2017-02-09
pub struct Server;
impl<T> Server<T>
    where T: 'static
{
    fn start(self, _on_message: Box<OnMessage<T>>) -> Result<()> {
        Err(ErrorKind::CouldNotStartServer.into())
    }
}

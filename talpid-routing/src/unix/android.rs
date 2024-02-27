use crate::imp::RouteManagerCommand;
use futures::{channel::mpsc, stream::StreamExt};

/// Stub error type for routing errors on Android.
#[derive(Debug, thiserror::Error)]
#[error("Failed to send shutdown result")]
pub struct Error;

/// Stub route manager for Android
pub struct RouteManagerImpl {}

impl RouteManagerImpl {
    #[allow(clippy::unused_async)]
    pub async fn new() -> Result<Self, Error> {
        Ok(RouteManagerImpl {})
    }

    pub(crate) async fn run(
        self,
        manage_rx: mpsc::UnboundedReceiver<RouteManagerCommand>,
    ) -> Result<(), Error> {
        let mut manage_rx = manage_rx.fuse();
        while let Some(command) = manage_rx.next().await {
            match command {
                RouteManagerCommand::Shutdown(tx) => {
                    tx.send(()).map_err(|()| Error)?;
                    break;
                }
                RouteManagerCommand::AddRoutes(_routes, tx) => {
                    let _ = tx.send(Ok(()));
                }
                RouteManagerCommand::ClearRoutes => (),
            }
        }
        Ok(())
    }
}

use crate::{imp::RouteManagerCommand, RequiredRoute};
use futures::{channel::mpsc, stream::StreamExt};
use std::collections::HashSet;

/// Stub error type for routing errors on Android.
#[derive(Debug, err_derive::Error)]
#[error(display = "Failed to send shutdown result")]
pub struct Error;

/// Stub route manager for Android
pub struct RouteManagerImpl {}

impl RouteManagerImpl {
    pub async fn new(_required_routes: HashSet<RequiredRoute>) -> Result<Self, Error> {
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

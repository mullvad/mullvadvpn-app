use crate::routing::{imp::RouteManagerCommand, RequiredRoute};
use futures01::{stream::Stream, sync::mpsc};
use std::collections::HashSet;

/// Stub error type for routing errors on Android.
#[derive(Debug, err_derive::Error)]
#[error(display = "Failed to send shutdown result")]
pub struct Error;

/// Stub route manager for Android
pub struct RouteManagerImpl {
    manage_rx: mpsc::UnboundedReceiver<RouteManagerCommand>,
}

impl RouteManagerImpl {
    pub fn new(
        _required_routes: HashSet<RequiredRoute>,
        manage_rx: mpsc::UnboundedReceiver<RouteManagerCommand>,
    ) -> Result<Self, Error> {
        Ok(RouteManagerImpl { manage_rx })
    }

    pub fn wait(self) -> Result<(), Error> {
        for msg in self.manage_rx.wait() {
            if let Ok(command) = msg {
                if let RouteManagerCommand::Shutdown(tx) = command {
                    tx.send(()).map_err(|()| Error)?;
                    break;
                }
            }
        }
        Ok(())
    }
}

use futures01::{sync::oneshot, Async, Future};
use ipnetwork::IpNetwork;
use std::collections::HashMap;

/// Stub error type for routing errors on Android.
#[derive(Debug, err_derive::Error)]
#[error(display = "Failed to send shutdown result")]
pub struct Error;

/// Stub route manager for Android
pub struct RouteManagerImpl {
    shutdown_rx: oneshot::Receiver<oneshot::Sender<()>>,
}

impl RouteManagerImpl {
    pub fn new(
        _required_routes: HashMap<IpNetwork, super::NetNode>,
        shutdown_rx: oneshot::Receiver<oneshot::Sender<()>>,
    ) -> Result<Self, Error> {
        Ok(RouteManagerImpl { shutdown_rx })
    }
}

impl Future for RouteManagerImpl {
    type Item = ();
    type Error = Error;

    fn poll(&mut self) -> Result<Async<()>, Error> {
        match self.shutdown_rx.poll() {
            Ok(Async::Ready(result_tx)) => {
                result_tx.send(()).map_err(|()| Error)?;
                Ok(Async::Ready(()))
            }
            Ok(Async::NotReady) => Ok(Async::NotReady),
            Err(_) => Ok(Async::Ready(())),
        }
    }
}

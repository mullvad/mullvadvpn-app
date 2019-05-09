use futures::{sync::oneshot, Async, Future};
use ipnetwork::IpNetwork;
use std::collections::HashMap;

/// Stub error type for routing errors on Android.
#[derive(Debug, err_derive::Error)]
#[error(display = "Unknown Android routing error")]
pub struct Error;

/// Stub route manager for Android
pub struct RouteManagerImpl;

impl RouteManagerImpl {
    pub fn new(
        _required_routes: HashMap<IpNetwork, super::NetNode>,
        _shutdown_rx: oneshot::Receiver<oneshot::Sender<()>>,
    ) -> Result<Self, Error> {
        Ok(Self {})
    }
}

impl Future for RouteManagerImpl {
    type Item = ();
    type Error = Error;
    fn poll(&mut self) -> Result<Async<()>, Error> {
        Ok(Async::Ready(()))
    }
}

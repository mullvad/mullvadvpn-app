#![cfg_attr(target_os = "android", allow(dead_code))]
#![cfg_attr(target_os = "windows", allow(dead_code))]
// TODO: remove the allow(dead_code) for android once it's up to scratch.
use super::NetNode;
use futures01::{sync::oneshot, Future};
use ipnetwork::IpNetwork;
use std::{collections::HashMap, sync::mpsc::sync_channel};

#[cfg(target_os = "macos")]
#[path = "macos.rs"]
mod imp;

#[cfg(target_os = "linux")]
#[path = "linux/mod.rs"]
mod imp;

#[cfg(target_os = "android")]
#[path = "android.rs"]
mod imp;

pub use imp::Error as PlatformError;

/// Errors that can be encountered whilst initializing RouteManager
#[derive(err_derive::Error, Debug)]
pub enum Error {
    /// Routing manager thread panicked before starting routing manager
    #[error(display = "Routing manager thread panicked before starting routing manager")]
    RoutingManagerThreadPanic,
    /// Platform sepcific error occured
    #[error(display = "Failed to create route manager")]
    FailedToInitializeManager(#[error(source)] imp::Error),
    /// Failed to spawn route manager future
    #[error(display = "Failed to spawn route manager on the provided executor")]
    FailedToSpawnManager,
}

/// RouteManager applies a set of routes to the route table.
/// If a destination has to be routed through the default node,
/// the route will be adjusted dynamically when the default route changes.
pub struct RouteManager {
    tx: Option<oneshot::Sender<oneshot::Sender<()>>>,
}

impl RouteManager {
    /// Constructs a RouteManager and applies the required routes.
    /// Takes a map of network destinations and network nodes as an argument, and applies said
    /// routes.
    pub fn new(required_routes: HashMap<IpNetwork, NetNode>) -> Result<Self, Error> {
        let (tx, rx) = oneshot::channel();
        let (start_tx, start_rx) = sync_channel(1);

        std::thread::spawn(
            move || match imp::RouteManagerImpl::new(required_routes, rx) {
                Ok(route_manager) => {
                    let _ = start_tx.send(Ok(()));
                    if let Err(e) = route_manager.wait() {
                        log::error!("Route manager failed - {}", e);
                    }
                }
                Err(e) => {
                    let _ = start_tx.send(Err(Error::FailedToInitializeManager(e)));
                }
            },
        );
        match start_rx.recv() {
            Ok(Ok(())) => Ok(Self { tx: Some(tx) }),
            Ok(Err(e)) => Err(e),
            Err(_) => Err(Error::RoutingManagerThreadPanic),
        }
    }

    /// Stops RouteManager and removes all of the applied routes.
    pub fn stop(&mut self) {
        if let Some(tx) = self.tx.take() {
            let (wait_tx, wait_rx) = oneshot::channel();
            if tx.send(wait_tx).is_err() {
                log::error!("RouteManager already down!");
                return;
            }

            if wait_rx.wait().is_err() {
                log::error!("RouteManager paniced while shutting down");
            }
        }
    }
}

impl Drop for RouteManager {
    fn drop(&mut self) {
        self.stop();
    }
}

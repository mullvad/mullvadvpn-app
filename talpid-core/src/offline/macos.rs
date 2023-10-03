//! This module has been reimplemented multiple times, often to no avail, with main issues being
//! that the app gets stuck in an offline state, blocking all internet access and preventing the
//! user from connecting to a relay.
//!
//! Currently, this functionality is implemented by watching for changes to the default route
//! in [`RouteManager`] using a `PF_ROUTE` socket. If there is no default route for neither IPv4 nor
//! IPv6, the host is considered to be offline.
use futures::{channel::mpsc::UnboundedSender, StreamExt};
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc, Mutex,
};
use talpid_routing::{DefaultRouteEvent, RouteManagerHandle};

#[derive(err_derive::Error, Debug)]
pub enum Error {
    #[error(display = "Failed to initialize route monitor")]
    StartMonitorError(#[error(source)] talpid_routing::Error),
}

pub struct MonitorHandle {
    state: Arc<Mutex<ConnectivityState>>,
    _notify_tx: Arc<UnboundedSender<bool>>,
}

struct ConnectivityState {
    v4_connectivity: bool,
    v6_connectivity: bool,
}

impl ConnectivityState {
    fn get_connectivity(&self) -> bool {
        self.v4_connectivity || self.v6_connectivity
    }
}

impl MonitorHandle {
    /// Host is considered to be offline if macOS doesn't assign a non-tunnel default route
    #[allow(clippy::unused_async)]
    pub async fn host_is_offline(&self) -> bool {
        let state = self.state.lock().unwrap();
        !state.get_connectivity()
    }
}

pub async fn spawn_monitor(
    notify_tx: UnboundedSender<bool>,
    route_manager_handle: RouteManagerHandle,
) -> Result<MonitorHandle, Error> {
    let notify_tx = Arc::new(notify_tx);

    let (v4_connectivity, v6_connectivity) = match route_manager_handle.get_default_routes().await {
        Ok((v4_route, v6_route)) => (v4_route.is_some(), v6_route.is_some()),
        Err(error) => {
            log::warn!("Failed to initialize offline monitor: {error}");
            // Fail open: Assume that we have connectivity if we cannot determine the existence of
            // a default route, since we don't want to block the user from connecting
            (true, true)
        }
    };

    let state = ConnectivityState {
        v4_connectivity,
        v6_connectivity,
    };
    let initial_connectivity = state.get_connectivity();
    let state = Arc::new(Mutex::new(state));

    let mut route_listener = route_manager_handle.default_route_listener().await?;
    let weak_state = Arc::downgrade(&state);
    let weak_notify_tx = Arc::downgrade(&notify_tx);

    // Detect changes to the default route
    tokio::spawn(async move {
        let mut state_update_handle: Option<tokio::task::JoinHandle<()>> = None;
        let prev_notified_state = Arc::new(AtomicBool::new(initial_connectivity));

        while let Some(event) = route_listener.next().await {
            let state = match weak_state.upgrade() {
                Some(state) => state,
                None => break,
            };

            let mut state = state.lock().unwrap();

            log::trace!("Default route event: {event:?}");

            let previous_connectivity = state.get_connectivity();

            match event {
                DefaultRouteEvent::AddedOrChangedV4 => {
                    state.v4_connectivity = true;
                }
                DefaultRouteEvent::AddedOrChangedV6 => {
                    state.v6_connectivity = true;
                }
                DefaultRouteEvent::RemovedV4 => {
                    state.v4_connectivity = false;
                }
                DefaultRouteEvent::RemovedV6 => {
                    state.v6_connectivity = false;
                }
            }

            let new_connectivity = state.get_connectivity();
            if previous_connectivity != new_connectivity {
                if let Some(update_state) = state_update_handle.take() {
                    update_state.abort();
                }

                let prev_notified = prev_notified_state.clone();

                let notify_copy = weak_notify_tx.clone();
                let update_task = tokio::spawn(async move {
                    let notify_tx = match notify_copy.upgrade() {
                        Some(tx) => tx,
                        None => return,
                    };

                    if prev_notified.swap(new_connectivity, Ordering::AcqRel) == new_connectivity {
                        // We don't care about network changes here
                        return;
                    }

                    log::info!(
                        "Connectivity changed: {}",
                        if new_connectivity {
                            "Connected"
                        } else {
                            "Offline"
                        }
                    );

                    let _ = notify_tx.unbounded_send(!new_connectivity);
                });

                state_update_handle = Some(update_task);
            }
        }

        log::trace!("Offline monitor exiting");
    });

    Ok(MonitorHandle {
        state,
        _notify_tx: notify_tx,
    })
}

//! This module has been reimplemented multiple times, often to no avail, with main issues being
//! that the app gets stuck in an offline state, blocking all internet access and preventing the
//! user from connecting to a relay.
//!
//! See [RouteManagerHandle::default_route_listener].
//!
//! This offline monitor synthesizes an offline state between network switches and before coming
//! online from an offline state. This is done to work around issues with DNS being blocked due
//! to macOS's connectivity check. In the offline state, a DNS server on localhost prevents the
//! connectivity check from being blocked.
use futures::{
    channel::mpsc::UnboundedSender,
    future::{Fuse, FutureExt},
    select, StreamExt,
};
use std::{
    sync::{Arc, Mutex},
    time::Duration,
};
use talpid_routing::{DefaultRouteEvent, RouteManagerHandle};

const SYNTHETIC_OFFLINE_DURATION: Duration = Duration::from_secs(1);

#[derive(err_derive::Error, Debug)]
pub enum Error {
    #[error(display = "Failed to initialize route monitor")]
    StartMonitorError(#[error(source)] talpid_routing::Error),
}

pub struct MonitorHandle {
    state: Arc<Mutex<ConnectivityState>>,
    _notify_tx: Arc<UnboundedSender<bool>>,
}

#[derive(Clone)]
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
    /// Return whether the host is offline
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

    // note: begin observing before initializing the state
    let route_listener = route_manager_handle.default_route_listener().await?;

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
    let mut real_state = state.clone();

    let state = Arc::new(Mutex::new(state));

    let weak_state = Arc::downgrade(&state);
    let weak_notify_tx = Arc::downgrade(&notify_tx);

    // Detect changes to the default route
    tokio::spawn(async move {
        let mut timeout = Fuse::terminated();
        let mut route_listener = route_listener.fuse();

        loop {
            select! {
                _ = timeout => {
                    // Update shared state
                    let Some(state) = weak_state.upgrade() else {
                        break;
                    };
                    let mut state = state.lock().unwrap();
                    *state = real_state.clone();

                    if state.get_connectivity() {
                        log::info!("Connectivity changed: Connected");
                        let Some(tx) = weak_notify_tx.upgrade() else {
                            break;
                        };
                        let _ = tx.unbounded_send(false);
                    }
                }

                route_event = route_listener.next() => {
                    let Some(event) = route_event else {
                        break;
                    };

                    // Update real state
                    match event {
                        DefaultRouteEvent::AddedOrChangedV4 => {
                            real_state.v4_connectivity = true;
                        }
                        DefaultRouteEvent::AddedOrChangedV6 => {
                            real_state.v6_connectivity = true;
                        }
                        DefaultRouteEvent::RemovedV4 => {
                            real_state.v4_connectivity = false;
                        }
                        DefaultRouteEvent::RemovedV6 => {
                            real_state.v6_connectivity = false;
                        }
                    }

                    // Synthesize offline state
                    // Update shared state
                    let Some(state) = weak_state.upgrade() else {
                        break;
                    };
                    let mut state = state.lock().unwrap();
                    let previous_connectivity = state.get_connectivity();
                    state.v4_connectivity = false;
                    state.v6_connectivity = false;

                    if previous_connectivity {
                        let Some(tx) = weak_notify_tx.upgrade() else {
                            break;
                        };
                        let _ = tx.unbounded_send(true);
                        log::info!("Connectivity changed: Offline");
                    }
                    if real_state.get_connectivity() {
                        timeout = Box::pin(tokio::time::sleep(SYNTHETIC_OFFLINE_DURATION)).fuse();
                    }
                }
            }
        }

        log::trace!("Offline monitor exiting");
    });

    Ok(MonitorHandle {
        state,
        _notify_tx: notify_tx,
    })
}

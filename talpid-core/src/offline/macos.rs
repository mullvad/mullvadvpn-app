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
use talpid_types::net::Connectivity;

const SYNTHETIC_OFFLINE_DURATION: Duration = Duration::from_secs(1);

#[derive(err_derive::Error, Debug)]
pub enum Error {
    #[error(display = "Failed to initialize route monitor")]
    StartMonitorError(#[error(source)] talpid_routing::Error),
}

pub struct MonitorHandle {
    state: Arc<Mutex<Connectivity>>,
    _notify_tx: Arc<UnboundedSender<Connectivity>>,
}

impl MonitorHandle {
    /// Return whether the host is offline
    #[allow(clippy::unused_async)]
    pub async fn host_is_offline(&self) -> Connectivity {
        let state = self.state.lock().unwrap();
        *state
    }
}

pub async fn spawn_monitor(
    notify_tx: UnboundedSender<Connectivity>,
    route_manager_handle: RouteManagerHandle,
) -> Result<MonitorHandle, Error> {
    let notify_tx = Arc::new(notify_tx);

    // note: begin observing before initializing the state
    let route_listener = route_manager_handle.default_route_listener().await?;

    let (ipv4, ipv6) = match route_manager_handle.get_default_routes().await {
        Ok((v4_route, v6_route)) => (v4_route.is_some(), v6_route.is_some()),
        Err(error) => {
            log::warn!("Failed to initialize offline monitor: {error}");
            // Fail open: Assume that we have connectivity if we cannot determine the existence of
            // a default route, since we don't want to block the user from connecting
            (true, true)
        }
    };

    let state = Connectivity::Status { ipv4, ipv6 };
    let mut real_state = state;

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
                    if real_state.is_online() {
                        assert_ne!(*state, real_state);
                        log::info!("Connectivity changed: Connected");
                        let Some(tx) = weak_notify_tx.upgrade() else {
                            break;
                        };
                        let _ = tx.unbounded_send(real_state);
                    }

                    *state = real_state;
                }

                route_event = route_listener.next() => {
                    let Some(event) = route_event else {
                        break;
                    };

                    // Update real state
                    match event {
                        DefaultRouteEvent::AddedOrChangedV4 => {
                            real_state.set_ipv4(true);
                        }
                        DefaultRouteEvent::AddedOrChangedV6 => {
                            real_state.set_ipv6(true);
                        }
                        DefaultRouteEvent::RemovedV4 => {
                            real_state.set_ipv4(false);
                        }
                        DefaultRouteEvent::RemovedV6 => {
                            real_state.set_ipv6(false);
                        }
                    }

                    // Synthesize offline state
                    // Update shared state
                    let Some(state) = weak_state.upgrade() else {
                        break;
                    };
                    let mut state = state.lock().unwrap();
                    let previous_connectivity = *state;
                    state.set_ipv4(false);
                    state.set_ipv6(false);

                    if previous_connectivity.is_online() {
                        let Some(tx) = weak_notify_tx.upgrade() else {
                            break;
                        };
                        let _ = tx.unbounded_send(*state);
                        log::info!("Connectivity changed: Offline");
                    }

                    if real_state.is_online() {
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

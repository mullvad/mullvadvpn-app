//! This module has been reimplemented multiple times, often to no avail, with main issues being
//! that the app gets stuck in an offline state, blocking all internet access and preventing the
//! user from connecting to a relay.
//!
//! See [RouteManagerHandle::default_route_listener].
use futures::{channel::mpsc::UnboundedSender, StreamExt};
use std::sync::{Arc, Mutex};
use talpid_routing::{DefaultRouteEvent, RouteManagerHandle};
use talpid_types::net::Connectivity;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Failed to initialize route monitor")]
    StartMonitorError(#[from] talpid_routing::Error),
}

pub struct MonitorHandle {
    state: Arc<Mutex<ConnectivityInner>>,
    _notify_tx: Arc<UnboundedSender<Connectivity>>,
}

impl MonitorHandle {
    /// Return whether the host is offline
    #[allow(clippy::unused_async)]
    pub async fn connectivity(&self) -> Connectivity {
        let state = self.state.lock().unwrap();
        state.into_connectivity()
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct ConnectivityInner {
    /// Whether IPv4 connectivity seems to be available on the host.
    ipv4: bool,
    /// Whether IPv6 connectivity seems to be available on the host.
    ipv6: bool,
}

impl ConnectivityInner {
    fn into_connectivity(self) -> Connectivity {
        Connectivity::new(self.ipv4, self.ipv6)
    }

    fn is_online(&self) -> bool {
        self.into_connectivity().is_online()
    }
}

pub async fn spawn_monitor(
    notify_tx: UnboundedSender<Connectivity>,
    route_manager: RouteManagerHandle,
) -> Result<MonitorHandle, Error> {
    let notify_tx = Arc::new(notify_tx);

    // note: begin observing before initializing the state
    let route_listener = route_manager.default_route_listener().await?;

    let (ipv4, ipv6) = match route_manager.get_default_routes().await {
        Ok((v4_route, v6_route)) => (v4_route.is_some(), v6_route.is_some()),
        Err(error) => {
            log::warn!("Failed to initialize offline monitor: {error}");
            // Fail open: Assume that we have connectivity if we cannot determine the existence of
            // a default route, since we don't want to block the user from connecting
            (true, true)
        }
    };

    let state = Arc::new(Mutex::new(ConnectivityInner { ipv4, ipv6 }));

    let weak_state = Arc::downgrade(&state);
    let weak_notify_tx = Arc::downgrade(&notify_tx);

    // Detect changes to the default route
    tokio::spawn(async move {
        let mut route_listener = route_listener.fuse();

        while let Some(event) = route_listener.next().await {
            talpid_types::detect_flood!();

            // Update real state
            let Some(state) = weak_state.upgrade() else {
                break;
            };
            let mut state = state.lock().unwrap();
            let previous_state = *state;

            match event {
                DefaultRouteEvent::AddedOrChangedV4 => {
                    state.ipv4 = true;
                }
                DefaultRouteEvent::AddedOrChangedV6 => {
                    state.ipv6 = true;
                }
                DefaultRouteEvent::RemovedV4 => {
                    state.ipv4 = false;
                }
                DefaultRouteEvent::RemovedV6 => {
                    state.ipv6 = false;
                }
            }

            if previous_state != *state {
                if state.is_online() {
                    log::info!("Connectivity changed: Connected");
                } else {
                    log::info!("Connectivity changed: Offline");
                }
                let Some(tx) = weak_notify_tx.upgrade() else {
                    break;
                };
                let _ = tx.unbounded_send(state.into_connectivity());
            }
        }

        log::trace!("Offline monitor exiting");
    });

    Ok(MonitorHandle {
        state,
        _notify_tx: notify_tx,
    })
}

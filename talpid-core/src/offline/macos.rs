//! This module has been reimplemented multiple times, often to no avail, with main issues being
//! that the app gets stuck in an offline state, blocking all internet access and preventing the
//! user from connecting to a relay.
//!
//! Currently, this functionality is implemented by using `route monitor -n` to observe routing
//! table changes and then use the CLI once more to query if there exists a default route.
//! Generally, it is assumed that a machine is online if there exists a route to a public IP
//! address that isn't using a tunnel adapter. On macOS, there were various ways of deducing this:
//! - watching the `State:/Network/Global/IPv4`  key in SystemConfiguration via
//!  `system-configuration-rs`, relying on a CoreFoundation runloop to drive callbacks.
//!   The issue with this is that sometimes during early boot or after a re-install, the callbacks
//!   won't be called, often leaving the daemon stuck in an offline state.
//! - setting a callback via [`SCNetworkReachability`]. The callback should be called whenever the
//!   reachability of a remote host changes, but sometimes the callbacks just don't get called.
//! - [`NWPathMonitor`] is a macOS native interface to watch changes in the routing table. It works
//!   great, but it seems to deliver updates before they actually get added to the routing table,
//!   effectively calling our callbacks with routes that aren't yet usable, so starting tunnels
//!   would fail anyway. This would be the API to use if we were able to bind the sockets our tunnel
//!   implementations would use, but that is far too much complexity.
//!
//! [`SCNetworkReachability`]: https://developer.apple.com/documentation/systemconfiguration/scnetworkreachability-g7d
//! [`NWPathMonitor`]: https://developer.apple.com/documentation/network/nwpathmonitor
use futures::{channel::mpsc::UnboundedSender, StreamExt};
use talpid_routing::{RouteManagerHandle, DefaultRouteEvent};
use std::{sync::{Arc, Mutex, atomic::{AtomicBool, Ordering}}, time::Duration};

/// How long to wait before announcing changes to the offline state
const DEBOUNCE_INTERVAL: Duration = Duration::from_secs(2);

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
        Ok((v4_route, v6_route)) => {
            (v4_route.is_some(), v6_route.is_some())
        }
        Err(error) => {
            log::warn!("Failed to initialize offline monitor: {error}");
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

                    // Debounce event updates
                    tokio::time::sleep(DEBOUNCE_INTERVAL).await;

                    if prev_notified.swap(new_connectivity, Ordering::AcqRel) == new_connectivity {
                        // We don't care about network changes here
                        return;
                    }

                    log::info!("Connectivity changed: {}", if new_connectivity {
                        "Connected"
                    } else {
                        "Offline"
                    });

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

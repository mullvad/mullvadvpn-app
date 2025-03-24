use crate::window::{PowerManagementEvent, PowerManagementListener};
use futures::channel::mpsc::UnboundedSender;
use parking_lot::Mutex;
use std::{
    io,
    sync::{Arc, Weak},
    time::Duration,
};
use talpid_routing::{get_best_default_route, CallbackHandle, EventType, RouteManagerHandle};
use talpid_types::{net::Connectivity, ErrorExt};
use talpid_windows::net::AddressFamily;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Unable to create listener thread")]
    ThreadCreationError(#[from] io::Error),
    #[error("Failed to start connectivity monitor")]
    ConnectivityMonitorError(#[from] talpid_routing::Error),
}

pub struct BroadcastListener {
    system_state: Arc<Mutex<SystemState>>,
    _callback_handle: CallbackHandle,
    _notify_tx: Arc<UnboundedSender<Connectivity>>,
}

impl BroadcastListener {
    pub async fn start(
        notify_tx: UnboundedSender<Connectivity>,
        route_manager: RouteManagerHandle,
        mut power_mgmt_rx: PowerManagementListener,
    ) -> Result<Self, Error> {
        let notify_tx = Arc::new(notify_tx);
        let (ipv4, ipv6) = Self::check_initial_connectivity();
        let system_state = Arc::new(Mutex::new(SystemState {
            connectivity: ConnectivityInner {
                ipv4,
                ipv6,
                suspended: false,
            },
            notify_tx: Arc::downgrade(&notify_tx),
        }));

        let state = system_state.clone();
        tokio::spawn(async move {
            while let Some(event) = power_mgmt_rx.next().await {
                match event {
                    PowerManagementEvent::Suspend => {
                        log::debug!("Machine is preparing to enter sleep mode");
                        apply_system_state_change(state.clone(), StateChange::Suspended(true));
                    }
                    PowerManagementEvent::ResumeAutomatic => {
                        let state_copy = state.clone();
                        tokio::spawn(async move {
                            // Tunnel will be unavailable for approximately 2 seconds on a healthy
                            // machine.
                            tokio::time::sleep(Duration::from_secs(5)).await;
                            log::debug!("Tunnel device is presumed to have been re-initialized");
                            apply_system_state_change(state_copy, StateChange::Suspended(false));
                        });
                    }
                    _ => (),
                }
            }
        });

        let callback_handle =
            Self::setup_network_connectivity_listener(system_state.clone(), route_manager).await?;

        Ok(BroadcastListener {
            system_state,
            _callback_handle: callback_handle,
            _notify_tx: notify_tx,
        })
    }

    fn check_initial_connectivity() -> (bool, bool) {
        let v4_connectivity = get_best_default_route(AddressFamily::Ipv4)
            .map(|route| route.is_some())
            .unwrap_or_else(|error| {
                log::error!(
                    "{}",
                    error.display_chain_with_msg("Failed to check initial IPv4 connectivity")
                );
                true
            });
        let v6_connectivity = get_best_default_route(AddressFamily::Ipv6)
            .map(|route| route.is_some())
            .unwrap_or_else(|error| {
                log::error!(
                    "{}",
                    error.display_chain_with_msg("Failed to check initial IPv6 connectivity")
                );
                true
            });

        let is_online = v4_connectivity || v6_connectivity;
        log::info!("Initial connectivity: {}", is_offline_str(!is_online));

        (v4_connectivity, v6_connectivity)
    }

    /// The caller must make sure the `system_state` reference is valid
    /// until after `WinNet_DeactivateConnectivityMonitor` has been called.
    async fn setup_network_connectivity_listener(
        system_state: Arc<Mutex<SystemState>>,
        route_manager: RouteManagerHandle,
    ) -> Result<CallbackHandle, Error> {
        let change_handle = route_manager
            .add_default_route_change_callback(Box::new(move |event, addr_family| {
                Self::connectivity_callback(event, addr_family, &system_state)
            }))
            .await
            .map_err(Error::ConnectivityMonitorError)?;
        Ok(change_handle)
    }

    fn connectivity_callback(
        event_type: EventType<'_>,
        family: AddressFamily,
        state_lock: &Arc<Mutex<SystemState>>,
    ) {
        use talpid_routing::EventType::*;

        if matches!(event_type, UpdatedDetails(_)) {
            // ignore changes that don't affect the route
            return;
        }

        let connectivity = event_type != Removed;
        let change = match family {
            AddressFamily::Ipv4 => StateChange::NetworkV4Connectivity(connectivity),
            AddressFamily::Ipv6 => StateChange::NetworkV6Connectivity(connectivity),
        };
        let mut state = state_lock.lock();
        state.apply_change(change);
    }

    #[allow(clippy::unused_async)]
    pub async fn connectivity(&self) -> Connectivity {
        let state = self.system_state.lock();
        state.connectivity.into_connectivity()
    }
}

#[derive(Debug)]
enum StateChange {
    NetworkV4Connectivity(bool),
    NetworkV6Connectivity(bool),
    Suspended(bool),
}

struct SystemState {
    connectivity: ConnectivityInner,
    notify_tx: Weak<UnboundedSender<Connectivity>>,
}

impl SystemState {
    fn apply_change(&mut self, change: StateChange) {
        let old_state = self.is_offline_currently();
        match change {
            StateChange::NetworkV4Connectivity(connectivity) => {
                self.connectivity.ipv4 = connectivity;
            }
            StateChange::NetworkV6Connectivity(connectivity) => {
                self.connectivity.ipv6 = connectivity;
            }
            StateChange::Suspended(suspended) => {
                self.connectivity.suspended = suspended;
            }
        };

        let new_state = self.connectivity.is_offline();
        if old_state != new_state {
            log::info!("Connectivity changed: {}", is_offline_str(new_state));
            if let Some(notify_tx) = self.notify_tx.upgrade() {
                if let Err(e) = notify_tx.unbounded_send(self.connectivity.into_connectivity()) {
                    log::error!("Failed to send new offline state to daemon: {}", e);
                }
            }
        }
    }

    fn is_offline_currently(&self) -> bool {
        self.connectivity.is_offline()
    }
}

// If `offline` is true, return "Offline". Otherwise, return "Connected".
fn is_offline_str(offline: bool) -> &'static str {
    if offline {
        "Offline"
    } else {
        "Connected"
    }
}

pub type MonitorHandle = BroadcastListener;

pub async fn spawn_monitor(
    sender: UnboundedSender<Connectivity>,
    route_manager: RouteManagerHandle,
) -> Result<MonitorHandle, Error> {
    let power_mgmt_rx = crate::window::PowerManagementListener::new();
    BroadcastListener::start(sender, route_manager, power_mgmt_rx).await
}

fn apply_system_state_change(state: Arc<Mutex<SystemState>>, change: StateChange) {
    let mut state = state.lock();
    state.apply_change(change);
}

#[derive(Clone, Copy, Debug)]
struct ConnectivityInner {
    /// Whether IPv4 connectivity seems to be available on the host.
    ipv4: bool,
    /// Whether IPv6 connectivity seems to be available on the host.
    ipv6: bool,
    /// The host is suspended.
    suspended: bool,
}

impl ConnectivityInner {
    /// Map [`ConnectivityInner`] to the public [`Connectivity`].
    ///
    /// # Note
    ///
    /// If the host is suspended, there is a great likelihood that we should
    /// consider the host to be offline. We synthesize this by setting both
    /// `ipv4` and `ipv6` availability to `false`.
    fn into_connectivity(self) -> Connectivity {
        if self.suspended {
            Connectivity::Offline
        } else {
            Connectivity::new(self.ipv4, self.ipv6)
        }
    }

    /// See [`Connectivity::is_offline`] for details.
    fn is_offline(&self) -> bool {
        self.into_connectivity().is_offline()
    }
}

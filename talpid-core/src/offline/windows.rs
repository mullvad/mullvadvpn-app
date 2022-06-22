use crate::{
    windows::window::{PowerManagementEvent, PowerManagementListener},
    winnet,
};
use futures::channel::mpsc::UnboundedSender;
use parking_lot::Mutex;
use std::{
    ffi::c_void,
    io,
    sync::{Arc, Weak},
    time::Duration,
};
use talpid_types::ErrorExt;

#[derive(err_derive::Error, Debug)]
pub enum Error {
    #[error(display = "Unable to create listener thread")]
    ThreadCreationError(#[error(source)] io::Error),
    #[error(display = "Failed to start connectivity monitor")]
    ConnectivityMonitorError(#[error(source)] winnet::DefaultRouteCallbackError),
}

pub struct BroadcastListener {
    system_state: Arc<Mutex<SystemState>>,
    _callback_handle: winnet::WinNetCallbackHandle,
    _notify_tx: Arc<UnboundedSender<bool>>,
}

unsafe impl Send for BroadcastListener {}

impl BroadcastListener {
    pub fn start(
        notify_tx: UnboundedSender<bool>,
        mut power_mgmt_rx: PowerManagementListener,
    ) -> Result<Self, Error> {
        let notify_tx = Arc::new(notify_tx);
        let (v4_connectivity, v6_connectivity) = Self::check_initial_connectivity();
        let system_state = Arc::new(Mutex::new(SystemState {
            v4_connectivity,
            v6_connectivity,
            suspended: false,
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
            unsafe { Self::setup_network_connectivity_listener(system_state.clone())? };

        Ok(BroadcastListener {
            system_state,
            _callback_handle: callback_handle,
            _notify_tx: notify_tx,
        })
    }

    fn check_initial_connectivity() -> (bool, bool) {
        let v4_connectivity = winnet::get_best_default_route(winnet::WinNetAddrFamily::IPV4)
            .map(|route| route.is_some())
            .unwrap_or_else(|error| {
                log::error!(
                    "{}",
                    error.display_chain_with_msg("Failed to check initial IPv4 connectivity")
                );
                true
            });
        let v6_connectivity = winnet::get_best_default_route(winnet::WinNetAddrFamily::IPV6)
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
    unsafe fn setup_network_connectivity_listener(
        system_state: Arc<Mutex<SystemState>>,
    ) -> Result<winnet::WinNetCallbackHandle, Error> {
        let change_handle = winnet::add_default_route_change_callback(
            Some(Self::connectivity_callback),
            system_state,
        )?;
        Ok(change_handle)
    }

    unsafe extern "system" fn connectivity_callback(
        event_type: winnet::WinNetDefaultRouteChangeEventType,
        family: winnet::WinNetAddrFamily,
        _default_route: winnet::WinNetDefaultRoute,
        ctx: *mut c_void,
    ) {
        use winnet::WinNetDefaultRouteChangeEventType::*;

        if event_type == DefaultRouteUpdatedDetails {
            // ignore changes that don't affect the route
            return;
        }

        let state_lock: &mut Arc<Mutex<SystemState>> = &mut *(ctx as *mut _);
        let connectivity = event_type != DefaultRouteRemoved;
        let change = match family {
            winnet::WinNetAddrFamily::IPV4 => StateChange::NetworkV4Connectivity(connectivity),
            winnet::WinNetAddrFamily::IPV6 => StateChange::NetworkV6Connectivity(connectivity),
        };
        let mut state = state_lock.lock();
        state.apply_change(change);
    }

    pub async fn host_is_offline(&self) -> bool {
        let state = self.system_state.lock();
        state.is_offline_currently()
    }
}

#[derive(Debug)]
enum StateChange {
    NetworkV4Connectivity(bool),
    NetworkV6Connectivity(bool),
    Suspended(bool),
}

struct SystemState {
    v4_connectivity: bool,
    v6_connectivity: bool,
    suspended: bool,
    notify_tx: Weak<UnboundedSender<bool>>,
}

impl SystemState {
    fn apply_change(&mut self, change: StateChange) {
        let old_state = self.is_offline_currently();
        match change {
            StateChange::NetworkV4Connectivity(connectivity) => {
                self.v4_connectivity = connectivity;
            }

            StateChange::NetworkV6Connectivity(connectivity) => {
                self.v6_connectivity = connectivity;
            }

            StateChange::Suspended(suspended) => {
                self.suspended = suspended;
            }
        };

        let new_state = self.is_offline_currently();
        if old_state != new_state {
            log::info!("Connectivity changed: {}", is_offline_str(new_state));
            if let Some(notify_tx) = self.notify_tx.upgrade() {
                if let Err(e) = notify_tx.unbounded_send(new_state) {
                    log::error!("Failed to send new offline state to daemon: {}", e);
                }
            }
        }
    }

    fn is_offline_currently(&self) -> bool {
        (!self.v4_connectivity && !self.v6_connectivity) || self.suspended
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
    sender: UnboundedSender<bool>,
    power_mgmt_rx: PowerManagementListener,
) -> Result<MonitorHandle, Error> {
    BroadcastListener::start(sender, power_mgmt_rx)
}

fn apply_system_state_change(state: Arc<Mutex<SystemState>>, change: StateChange) {
    let mut state = state.lock();
    state.apply_change(change);
}

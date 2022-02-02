use crate::{
    windows::window::{create_hidden_window, WindowCloseHandle},
    winnet,
};
use futures::channel::mpsc::UnboundedSender;
use parking_lot::Mutex;
use std::{
    ffi::c_void,
    io,
    sync::{Arc, Weak},
    thread,
    time::Duration,
};
use talpid_types::ErrorExt;
use winapi::um::winuser::{
    DefWindowProcW, PBT_APMRESUMEAUTOMATIC, PBT_APMSUSPEND, WM_POWERBROADCAST,
};

#[derive(err_derive::Error, Debug)]
pub enum Error {
    #[error(display = "Unable to create listener thread")]
    ThreadCreationError(#[error(source)] io::Error),
    #[error(display = "Failed to start connectivity monitor")]
    ConnectivityMonitorError(#[error(source)] winnet::DefaultRouteCallbackError),
}

pub struct BroadcastListener {
    window: WindowCloseHandle,
    system_state: Arc<Mutex<SystemState>>,
    _callback_handle: winnet::WinNetCallbackHandle,
    _notify_tx: Arc<UnboundedSender<bool>>,
}

unsafe impl Send for BroadcastListener {}

impl BroadcastListener {
    pub fn start(notify_tx: UnboundedSender<bool>) -> Result<Self, Error> {
        let notify_tx = Arc::new(notify_tx);
        let (v4_connectivity, v6_connectivity) = Self::check_initial_connectivity();
        let system_state = Arc::new(Mutex::new(SystemState {
            v4_connectivity,
            v6_connectivity,
            suspended: false,
            notify_tx: Arc::downgrade(&notify_tx),
        }));

        let power_broadcast_state_ref = system_state.clone();

        let power_broadcast_callback = move |window, message, wparam, lparam| {
            let state = power_broadcast_state_ref.clone();
            if message == WM_POWERBROADCAST {
                if wparam == PBT_APMSUSPEND {
                    log::debug!("Machine is preparing to enter sleep mode");
                    apply_system_state_change(state, StateChange::Suspended(true));
                } else if wparam == PBT_APMRESUMEAUTOMATIC {
                    log::debug!("Machine is returning from sleep mode");
                    thread::spawn(move || {
                        // TAP will be unavailable for approximately 2 seconds on a healthy machine.
                        thread::sleep(Duration::from_secs(5));
                        log::debug!("TAP is presumed to have been re-initialized");
                        apply_system_state_change(state, StateChange::Suspended(false));
                    });
                }
            }
            unsafe { DefWindowProcW(window, message, wparam, lparam) }
        };

        let window = create_hidden_window(power_broadcast_callback);

        let callback_handle =
            unsafe { Self::setup_network_connectivity_listener(system_state.clone())? };

        Ok(BroadcastListener {
            window,
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
        let state_lock: &mut Arc<Mutex<SystemState>> = &mut *(ctx as *mut _);
        let connectivity = match event_type {
            winnet::WinNetDefaultRouteChangeEventType::DefaultRouteChanged => true,
            winnet::WinNetDefaultRouteChangeEventType::DefaultRouteRemoved => false,
        };
        let change = match family {
            winnet::WinNetAddrFamily::IPV4 => StateChange::NetworkV4Connectivity(connectivity),
            winnet::WinNetAddrFamily::IPV6 => StateChange::NetworkV6Connectivity(connectivity),
        };
        let mut state = state_lock.lock();
        state.apply_change(change);
    }

    pub async fn is_offline(&self) -> bool {
        let state = self.system_state.lock();
        state.is_offline_currently()
    }
}

impl Drop for BroadcastListener {
    fn drop(&mut self) {
        self.window.close();
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

pub async fn spawn_monitor(sender: UnboundedSender<bool>) -> Result<MonitorHandle, Error> {
    BroadcastListener::start(sender)
}

fn apply_system_state_change(state: Arc<Mutex<SystemState>>, change: StateChange) {
    let mut state = state.lock();
    state.apply_change(change);
}

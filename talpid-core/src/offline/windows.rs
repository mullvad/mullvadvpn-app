use crate::{logging::windows::log_sink, tunnel_state_machine::TunnelCommand, winnet};
use futures::channel::mpsc::UnboundedSender;
use parking_lot::Mutex;
use std::{
    ffi::c_void,
    io,
    sync::{Arc, Weak},
};


#[derive(err_derive::Error, Debug)]
pub enum Error {
    #[error(display = "Unable to create listener thread")]
    ThreadCreationError(#[error(source)] io::Error),
    #[error(display = "Failed to start connectivity monitor")]
    ConnectivityMonitorError,
}


pub struct BroadcastListener {
    _system_state: Arc<Mutex<SystemState>>,
}

unsafe impl Send for BroadcastListener {}

impl BroadcastListener {
    pub fn start(sender: Weak<UnboundedSender<TunnelCommand>>) -> Result<Self, Error> {
        let mut system_state = Arc::new(Mutex::new(SystemState {
            network_connectivity: None,
            daemon_channel: sender,
        }));

        unsafe { Self::setup_network_connectivity_listener(&mut system_state)? };

        Ok(BroadcastListener {
            _system_state: system_state,
        })
    }

    /// The caller must make sure the `system_state` reference is valid
    /// until after `WinNet_DeactivateConnectivityMonitor` has been called.
    unsafe fn setup_network_connectivity_listener(
        system_state: &Mutex<SystemState>,
    ) -> Result<(), Error> {
        let callback_context = system_state as *const _ as *mut libc::c_void;
        if !winnet::WinNet_ActivateConnectivityMonitor(
            Some(Self::connectivity_callback),
            callback_context,
            Some(log_sink),
            b"Connectivity monitor\0".as_ptr(),
        ) {
            return Err(Error::ConnectivityMonitorError);
        }
        Ok(())
    }

    unsafe extern "system" fn connectivity_callback(connectivity: bool, context: *mut c_void) {
        let state_lock: &mut Mutex<SystemState> = &mut *(context as *mut _);
        let mut state = state_lock.lock();
        state.apply_change(StateChange::NetworkConnectivity(connectivity));
    }

    pub async fn is_offline(&self) -> bool {
        let state = self._system_state.lock();
        state.is_offline_currently().unwrap_or(false)
    }
}

impl Drop for BroadcastListener {
    fn drop(&mut self) {
        unsafe {
            winnet::WinNet_DeactivateConnectivityMonitor();
        }
    }
}

#[derive(Debug)]
enum StateChange {
    NetworkConnectivity(bool),
}

struct SystemState {
    network_connectivity: Option<bool>,
    daemon_channel: Weak<UnboundedSender<TunnelCommand>>,
}

impl SystemState {
    fn apply_change(&mut self, change: StateChange) {
        let old_state = self.is_offline_currently();
        match change {
            StateChange::NetworkConnectivity(connectivity) => {
                self.network_connectivity = Some(connectivity);
            }
        };

        let new_state = self.is_offline_currently();
        if old_state != new_state {
            if let Some(daemon_channel) = self.daemon_channel.upgrade() {
                if let Err(e) = daemon_channel
                    .unbounded_send(TunnelCommand::IsOffline(new_state.unwrap_or(false)))
                {
                    log::error!("Failed to send new offline state to daemon: {}", e);
                }
            }
        }
    }

    fn is_offline_currently(&self) -> Option<bool> {
        Some(!self.network_connectivity?)
    }
}

pub type MonitorHandle = BroadcastListener;

pub async fn spawn_monitor(
    sender: Weak<UnboundedSender<TunnelCommand>>,
) -> Result<MonitorHandle, Error> {
    BroadcastListener::start(sender)
}

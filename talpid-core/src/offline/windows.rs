use crate::winnet;
use futures::channel::mpsc::UnboundedSender;
use parking_lot::Mutex;
use std::{
    ffi::c_void,
    io,
    mem::zeroed,
    os::windows::io::{IntoRawHandle, RawHandle},
    ptr,
    sync::{Arc, Weak},
    thread,
    time::Duration,
};
use talpid_types::ErrorExt;
use winapi::{
    shared::{
        basetsd::LONG_PTR,
        minwindef::{DWORD, LPARAM, LRESULT, UINT, WPARAM},
        windef::HWND,
    },
    um::{
        handleapi::CloseHandle,
        libloaderapi::GetModuleHandleW,
        processthreadsapi::GetThreadId,
        synchapi::WaitForSingleObject,
        winbase::INFINITE,
        winuser::{
            CreateWindowExW, DefWindowProcW, DestroyWindow, DispatchMessageW, GetMessageW,
            GetWindowLongPtrW, PostQuitMessage, PostThreadMessageW, SetWindowLongPtrW,
            GWLP_USERDATA, GWLP_WNDPROC, PBT_APMRESUMEAUTOMATIC, PBT_APMSUSPEND, WM_DESTROY,
            WM_POWERBROADCAST, WM_USER,
        },
    },
};

const CLASS_NAME: &[u8] = b"S\0T\0A\0T\0I\0C\0\0\0";
const REQUEST_THREAD_SHUTDOWN: UINT = WM_USER + 1;


#[derive(err_derive::Error, Debug)]
pub enum Error {
    #[error(display = "Unable to create listener thread")]
    ThreadCreationError(#[error(source)] io::Error),
    #[error(display = "Failed to start connectivity monitor")]
    ConnectivityMonitorError(#[error(source)] winnet::DefaultRouteCallbackError),
}


pub struct BroadcastListener {
    thread_handle: RawHandle,
    thread_id: DWORD,
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

        let power_broadcast_callback = move |message: UINT, wparam: WPARAM, _lparam: LPARAM| {
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
        };

        let join_handle = thread::Builder::new()
            .spawn(move || unsafe {
                Self::message_pump(power_broadcast_callback);
            })
            .map_err(Error::ThreadCreationError)?;

        let real_handle = join_handle.into_raw_handle();

        let callback_handle =
            unsafe { Self::setup_network_connectivity_listener(system_state.clone())? };

        Ok(BroadcastListener {
            thread_handle: real_handle,
            thread_id: unsafe { GetThreadId(real_handle) },
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

    unsafe fn message_pump<F>(client_callback: F)
    where
        F: Fn(UINT, WPARAM, LPARAM),
    {
        let dummy_window = CreateWindowExW(
            0,
            CLASS_NAME.as_ptr() as *const u16,
            ptr::null_mut(),
            0,
            0,
            0,
            0,
            0,
            ptr::null_mut(),
            ptr::null_mut(),
            GetModuleHandleW(ptr::null_mut()),
            ptr::null_mut(),
        );

        // Move callback information to the heap.
        // This enables us to reach the callback through a "thin pointer".
        let boxed_callback = Box::new(client_callback);

        // Detach callback from Box.
        let raw_callback = Box::into_raw(boxed_callback) as *mut c_void;

        SetWindowLongPtrW(dummy_window, GWLP_USERDATA, raw_callback as LONG_PTR);
        SetWindowLongPtrW(
            dummy_window,
            GWLP_WNDPROC,
            Self::window_procedure::<F> as LONG_PTR,
        );

        let mut msg = zeroed();

        loop {
            let status = GetMessageW(&mut msg, 0 as HWND, 0, 0);

            if status < 0 {
                continue;
            }
            if status == 0 {
                break;
            }

            if msg.hwnd.is_null() {
                if msg.message == REQUEST_THREAD_SHUTDOWN {
                    DestroyWindow(dummy_window);
                }
            } else {
                DispatchMessageW(&mut msg);
            }
        }

        // Reattach callback to Box for proper clean-up.
        let _ = Box::from_raw(raw_callback as *mut F);
    }

    unsafe extern "system" fn window_procedure<F>(
        window: HWND,
        message: UINT,
        wparam: WPARAM,
        lparam: LPARAM,
    ) -> LRESULT
    where
        F: Fn(UINT, WPARAM, LPARAM),
    {
        let raw_callback = GetWindowLongPtrW(window, GWLP_USERDATA);

        if raw_callback != 0 {
            let typed_callback = &mut *(raw_callback as *mut F);
            typed_callback(message, wparam, lparam);
        }

        if message == WM_DESTROY {
            PostQuitMessage(0);
            return 0;
        }

        DefWindowProcW(window, message, wparam, lparam)
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
        unsafe {
            PostThreadMessageW(self.thread_id, REQUEST_THREAD_SHUTDOWN, 0, 0);
            WaitForSingleObject(self.thread_handle, INFINITE);
            CloseHandle(self.thread_handle);
        }
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

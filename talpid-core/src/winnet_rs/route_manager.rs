use super::*;
use std::ffi::c_void;
use std::sync::mpsc::{channel, RecvTimeoutError, Sender};
use std::sync::{Arc, Mutex, Weak};
use std::time::{Duration, Instant};
use windows_sys::Win32::Foundation::{BOOLEAN, HANDLE, NO_ERROR};
use windows_sys::Win32::NetworkManagement::IpHelper::{
    CancelMibChangeNotify2, ConvertInterfaceLuidToIndex, NotifyIpInterfaceChange,
    NotifyRouteChange2, NotifyUnicastIpAddressChange, MIB_IPINTERFACE_ROW, MIB_NOTIFICATION_TYPE,
    MIB_UNICASTIPADDRESS_ROW,
};

pub struct RouteManager {
    route_monitor_v4: DefaultRouteMonitor,
    route_monitor_v6: DefaultRouteMonitor,
}

impl RouteManager {
    fn new() -> Result<Self> {
        Ok(Self {
            route_monitor_v4: DefaultRouteMonitor::new(
                AddressFamily::Ipv4,
                |event_type, route| {},
            )?,
            route_monitor_v6: DefaultRouteMonitor::new(
                AddressFamily::Ipv6,
                |event_type, route| {},
            )?,
        })
    }

    fn default_route_changed(
        family: AddressFamily,
        event_type: EventType,
        route: &Option<WinNetDefaultRoute>,
    ) {
    }
}

struct DefaultRouteMonitorContext {
    callback: Box<dyn Fn(EventType, &Option<WinNetDefaultRoute>) + Send + 'static>,
    refresh_current_route: bool,
    family: AddressFamily,
    best_route: Option<WinNetDefaultRoute>,
}

impl DefaultRouteMonitorContext {
    fn new(
        callback: Box<dyn Fn(EventType, &Option<WinNetDefaultRoute>) + Send + 'static>,
        family: AddressFamily,
    ) -> Self {
        Self {
            callback,
            best_route: None,
            refresh_current_route: false,
            family,
        }
    }

    fn update_refresh_flag(&mut self, luid: &NET_LUID_LH, index: u32) {
        if let Some(best_route) = &self.best_route {
            // SAFETY: luid is a union but both fields are finally represented by u64, as such any access is valid
            if unsafe { luid.Value } == unsafe { best_route.interface_luid.Value } {
                self.refresh_current_route = true;
                return;
            }
            // SAFETY: luid is a union but both fields are finally represented by u64, as such any access is valid
            if unsafe { luid.Value } != 0 {
                return;
            }

            let mut default_interface_index = 0;
            let route_luid = best_route.interface_luid;
            // SAFETY: No clear safety specifications
            if NO_ERROR as i32
                == unsafe { ConvertInterfaceLuidToIndex(&route_luid, &mut default_interface_index) }
            {
                self.refresh_current_route = index == default_interface_index;
            } else {
                self.refresh_current_route = true;
            }
        }
    }

    fn evaluate_routes(&mut self) {
        let refresh_current = self.refresh_current_route;
        self.refresh_current_route = false;

        let current_best_route = get_best_default_route(self.family).ok().flatten();

        match (&self.best_route, current_best_route) {
            (None, None) => (),
            (None, Some(current_best_route)) => {
                self.best_route = Some(current_best_route);
                (self.callback)(EventType::Updated, &self.best_route);
            }
            (Some(_), None) => {
                self.best_route = None;
                (self.callback)(EventType::Removed, &None);
            }
            (Some(best_route), Some(current_best_route)) => {
                if best_route != &current_best_route {
                    self.best_route = Some(current_best_route);
                    (self.callback)(EventType::Updated, &self.best_route);
                } else if refresh_current {
                    (self.callback)(EventType::UpdatedDetails, &self.best_route);
                }
            }
        }
    }
}

struct DefaultRouteMonitor {
    // SAFETY: These handles must be dropped before the context. This will happen automatically if it is handled by DefaultRouteMonitors drop implementation
    notify_change_handles: Option<(Handle, Handle, Handle)>,
    // SAFETY: Context must be dropped after all of the notifier handles have been dropped in order to guarantee none of them use its pointer.
    // This will be dropped by DefaultRouteMonitors drop implementation.
    // SAFETY: The content of this pointer is not allowed to be mutated at any point except for in the drop implementation
    context: *mut ContextAndBurstGuard,
}

impl std::ops::Drop for DefaultRouteMonitor {
    fn drop(&mut self) {
        drop(self.notify_change_handles.take());
        // SAFETY: This pointer was created by Box::into_raw and is not modified since then.
        // This function is also only called once
        drop(unsafe { Box::from_raw(self.context) });
    }
}

struct Handle(*mut HANDLE);

impl std::ops::Drop for Handle {
    fn drop(&mut self) {
        // SAFETY: There is no clear safety specification on this function. However self.0 should point to a handle that has
        // been allocated by windows and should be non-null. Even if it would be null that would cause a panic rather than UB.
        unsafe {
            if NO_ERROR as i32 != CancelMibChangeNotify2(*self.0) {
                panic!("Could not cancel change notification callback")
            }
        }
    }
}

const WIN_FALSE: BOOLEAN = 0;

enum EventType {
    Updated,
    UpdatedDetails,
    Removed,
}

// SAFETY: This struct must be `Sync` otherwise it is not allowed to be sent between threads.
// Having only `Mutex<T>` or `Arc<Mutex<T>>` fields guarantees that it is `Sync`
struct ContextAndBurstGuard {
    context: Arc<Mutex<DefaultRouteMonitorContext>>,
    burst_guard: Mutex<BurstGuard>,
}

impl DefaultRouteMonitor {
    fn new<F: Fn(EventType, &Option<WinNetDefaultRoute>) + Send + 'static>(
        family: AddressFamily,
        callback: F,
    ) -> Result<Self> {
        let context = Arc::new(Mutex::new(DefaultRouteMonitorContext::new(
            Box::new(callback),
            family,
        )));

        let moved_context = context.clone();
        let burst_guard = Mutex::new(BurstGuard::new(move || {
            moved_context.lock().unwrap().evaluate_routes();
        }));

        // SAFETY: We need to send the ContextAndBurstGuard to the windows notification functions as a raw pointer.
        // This imposes the requirement it is not mutated or dropped until after those notifications are guaranteed to not run.
        // This happens when the DefaultRouteMonitor is dropped and not before then. It also imposes the requirement that ContextAndBurstGuard is `Sync` since we will send
        // references to it to other threads. This requirement is fullfilled since all fields of `ContextAndBurstGuard` are wrapped in either a Arc<Mutex> or Mutex.
        let context_and_burst = Box::into_raw(Box::new(ContextAndBurstGuard {
            context,
            burst_guard,
        }));

        let handles = match Self::register_callbacks(family, context_and_burst) {
            Ok(handles) => handles,
            Err(e) => {
                // Clean up the memory leak in case of error
                // SAFETY: We created context_and_burst from `Box::into_raw()` and it has not been modified since.
                // All of the handles have been freed at this point so there will be no risk of UAF.
                drop(unsafe { Box::from_raw(context_and_burst) });
                return Err(e);
            }
        };

        let monitor = Self {
            context: context_and_burst,
            notify_change_handles: Some(handles),
        };

        // We must set the best default route after we have registered listeners in order to avoid race conditions.
        {
            // SAFETY: `monitor.context` will be valid since monitor will handle dropping it. No mutation happens here
            // since we are using a Mutex.
            let context = &unsafe { &*(monitor.context) }.context;
            let mut context = context.lock().unwrap();
            context.best_route = get_best_default_route(context.family)?;
        }

        Ok(monitor)
    }

    fn register_callbacks(
        family: AddressFamily,
        context_and_burst: *mut ContextAndBurstGuard,
    ) -> Result<(Handle, Handle, Handle)> {
        let family = family.to_af_family();

        // NotifyRouteChange2
        // We must provide a raw pointer that points to the context that will be used in the callbacks.
        // We provide a Mutex for the state turned into a Weak pointer turned into a raw pointer in order to not have to manually deallocate
        // the memory after we cancel the callbacks. This will leak the weak pointer but the context state itself will be correctly dropped
        // when DefaultRouteManager is dropped.
        let context_ptr = context_and_burst as *const ContextAndBurstGuard;
        let handle_ptr = std::ptr::null_mut();
        // SAFETY: No clear safety specifications, context_ptr must be valid for as long as handle has not been dropped.
        if NO_ERROR as i32
            != unsafe {
                NotifyRouteChange2(
                    family,
                    Some(route_change_callback),
                    context_ptr as *const _,
                    WIN_FALSE,
                    handle_ptr,
                )
            }
        {
            return Err(Error::WindowsApi);
        }
        let notify_route_change_handle = Handle(handle_ptr);

        // NotifyIpInterfaceChange
        let handle_ptr = std::ptr::null_mut();
        // SAFETY: No clear safety specifications, context_ptr must be valid for as long as handle has not been dropped.
        if NO_ERROR as i32
            != unsafe {
                NotifyIpInterfaceChange(
                    family,
                    Some(interface_change_callback),
                    context_ptr as *const _,
                    WIN_FALSE,
                    handle_ptr,
                )
            }
        {
            return Err(Error::WindowsApi);
        }
        let notify_interface_change_handle = Handle(handle_ptr);

        // NotifyUnicastIpAddressChange
        let handle_ptr = std::ptr::null_mut();
        // SAFETY: No clear safety specifications, context_ptr must be valid for as long as handle has not been dropped.
        if NO_ERROR as i32
            != unsafe {
                NotifyUnicastIpAddressChange(
                    family,
                    Some(ip_address_change_callback),
                    context_ptr as *const _,
                    WIN_FALSE,
                    handle_ptr,
                )
            }
        {
            return Err(Error::WindowsApi);
        }
        let notify_address_change_handle = Handle(handle_ptr);

        Ok((
            notify_route_change_handle,
            notify_interface_change_handle,
            notify_address_change_handle,
        ))
    }
}

// SAFETY: `context` is a Arc::<ContextAndBurstGuard>::as_ptr() pointer which is fine to dereference as long as there exists a strong count to the Arc.
// This is guaranteed to be the case as we store an `Arc` in the DefaultRouteMonitor struct. When this struct is dropped it will
// cancel this callback and guarantee that it is not called after that point, only after then is the final `Arc` dropped.
unsafe extern "system" fn route_change_callback(
    context: *const c_void,
    row: *const MIB_IPFORWARD_ROW2,
    notification_type: MIB_NOTIFICATION_TYPE,
) {
    // SAFETY: We assume Windows provides this pointer correctly
    let row = unsafe { &*row };

    if row.DestinationPrefix.PrefixLength != 0 || !route_has_gateway(row) {
        return;
    }

    // SAFETY: context must not be dropped or modified until this callback has been cancelled.
    let context_and_burst: &ContextAndBurstGuard =
        unsafe { &*(context as *const ContextAndBurstGuard) };
    let mut context = context_and_burst.context.lock().unwrap();

    context.update_refresh_flag(&row.InterfaceLuid, row.InterfaceIndex);
    context_and_burst.burst_guard.lock().unwrap().trigger();
}

// SAFETY: `context` is a Arc::<ContextAndBurstGuard>::as_ptr() pointer which is fine to dereference as long as there exists a strong count to the Arc.
// This is guaranteed to be the case as we store an `Arc` in the DefaultRouteMonitor struct. When this struct is dropped it will
// cancel this callback and guarantee that it is not called after that point, only after then is the final `Arc` dropped.
unsafe extern "system" fn interface_change_callback(
    context: *const c_void,
    row: *const MIB_IPINTERFACE_ROW,
    notification_type: MIB_NOTIFICATION_TYPE,
) {
    // SAFETY: We assume Windows provides this pointer correctly
    let row = unsafe { &*row };

    // SAFETY: context must not be dropped or modified until this callback has been cancelled.
    let context_and_burst: &ContextAndBurstGuard =
        unsafe { &*(context as *const ContextAndBurstGuard) };
    let mut context = context_and_burst.context.lock().unwrap();

    context.update_refresh_flag(&row.InterfaceLuid, row.InterfaceIndex);
    context_and_burst.burst_guard.lock().unwrap().trigger();
}

// `context` is a Weak<Mutex<DefaultRouteManagerContext>>::into_raw() pointer and should be used with Weak::from_raw()
// SAFETY: After converting `context` into a `Weak` it must also be stopped from being dropped by calling Weak::into_raw().
// If this is not done the reference will be dropped and calling `Weak::from_raw()` the next time is undefined behavior.
unsafe extern "system" fn ip_address_change_callback(
    context: *const c_void,
    row: *const MIB_UNICASTIPADDRESS_ROW,
    notification_type: MIB_NOTIFICATION_TYPE,
) {
    // SAFETY: We assume Windows provides this pointer correctly
    let row = unsafe { &*row };

    // SAFETY: context must not be dropped or modified until this callback has been cancelled.
    let context_and_burst: &ContextAndBurstGuard =
        unsafe { &*(context as *const ContextAndBurstGuard) };
    let mut context = context_and_burst.context.lock().unwrap();

    context.update_refresh_flag(&row.InterfaceLuid, row.InterfaceIndex);
    context_and_burst.burst_guard.lock().unwrap().trigger();
}

/// BurstGuard is a wrapper for a function that protects that function from being called too many times in a short amount of time.
/// To call the function use `burst_guard.trigger()`, at that point `BurstGuard` will wait for `buffer_period` and if no more calls to
/// `trigger` are made then it will call the wrapped function. If another call to `trigger` is made during this wait then it will wait
/// another `buffer_period`, this happens over and over until either `longest_buffer_period` time has elapsed or until no call to `trigger`
/// has been made in `buffer_period`. At which point the wrapped function will be called.
struct BurstGuard {
    sender: Sender<()>,
}

impl BurstGuard {
    fn new<F: Fn() + Send + 'static>(callback: F) -> Self {
        /// This is the period of time the `BurstGuard` will wait for a new trigger to be sent before it calls the callback.
        const BURST_BUFFER_PERIOD: Duration = Duration::from_millis(200);
        /// This is the longest period that the `BurstGuard` will wait from the first trigger till it calls the callback.
        const BURST_LONGEST_BUFFER_PERIOD: Duration = Duration::from_secs(2);

        let (sender, listener) = channel();
        std::thread::spawn(move || {
            while let Ok(()) = listener.recv() {
                let start = Instant::now();
                loop {
                    match listener.recv_timeout(BURST_BUFFER_PERIOD) {
                        Ok(()) => {
                            if start.elapsed() >= BURST_LONGEST_BUFFER_PERIOD {
                                callback();
                                break;
                            }
                        }
                        Err(RecvTimeoutError::Timeout) => {
                            callback();
                            break;
                        }
                        Err(RecvTimeoutError::Disconnected) => {
                            break;
                        }
                    }
                }
            }
        });
        Self { sender }
    }

    /// Non-blocking
    fn trigger(&self) {
        self.sender.send(()).unwrap();
    }
}

use super::*;
use windows_sys::Win32::NetworkManagement::IpHelper::{NotifyRouteChange2, NotifyIpInterfaceChange, NotifyUnicastIpAddressChange, CancelMibChangeNotify2, ConvertInterfaceLuidToIndex, MIB_NOTIFICATION_TYPE, MIB_IPINTERFACE_ROW, MIB_UNICASTIPADDRESS_ROW};
use windows_sys::Win32::Foundation::{BOOLEAN, HANDLE, NO_ERROR};
use std::ffi::c_void;
use std::sync::{Arc, Weak, Mutex};
use std::sync::mpsc::{Sender, channel, RecvTimeoutError};
use std::time::{Instant, Duration};

pub struct RouteManager {
    route_monitor_v4: DefaultRouteMonitor,
    route_monitor_v6: DefaultRouteMonitor,
}

impl RouteManager {
    fn new() -> Result<Self> {
        Ok(Self {
            route_monitor_v4: DefaultRouteMonitor::new(AddressFamily::Ipv4, |event_type, route| {

            })?,
            route_monitor_v6: DefaultRouteMonitor::new(AddressFamily::Ipv6, |event_type, route| {

            })?,
        })
    }

    fn default_route_changed(family: AddressFamily, event_type: EventType, route: &Option<WinNetDefaultRoute>) {

    }
}

// TODO: Improve name
struct DefaultRouteMonitorContext {
    callback: Box<dyn Fn(EventType, &Option<WinNetDefaultRoute>) + Send + 'static>,
    refresh_current_route: bool,
    family: AddressFamily,
    best_route: Option<WinNetDefaultRoute>,
}

impl DefaultRouteMonitorContext {
    fn new(callback: Box<dyn Fn(EventType, &Option<WinNetDefaultRoute>) + Send + 'static>, family: AddressFamily) -> Result<Self> {
        let ctx = Self {
            callback,
            best_route: get_best_default_route(family)?,
            refresh_current_route: false,
            family,
        };
        Ok(ctx)
    }

    fn update_refresh_flag(&mut self, luid: &NET_LUID_LH, index: u32) {
        if let Some(best_route) = &self.best_route {
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
            // TODO: Different from c++ impl, check to make sure it makes sense
            if NO_ERROR as i32 == unsafe { ConvertInterfaceLuidToIndex(&route_luid, &mut default_interface_index) } {
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

// SAFETY: The value wrapped in ReadOnly is not allowed to be mutated until dropped.
struct ReadOnly(Box<OuterContext>);

impl std::ops::Deref for ReadOnly {
    type Target = Box<OuterContext>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

struct DefaultRouteMonitor {
    notify_route_change_handle: Handle,
    notify_interface_change_handle: Handle,
    notify_address_change_handle: Handle,
    // SAFETY: Context must be dropped after all of the notifier handles have been dropped in order to guarantee none of them use its pointer.
    // context is also wrapped in a ReadOnly struct in order to guarantee no mutation happens to it after pointers have been given to the notifier functions.
    context: ReadOnly,
}

// TODO: We could potentially save the raw pointer to the weak pointer that we provided to the notification function,
// that way we would be able to use Weak::from_raw(ptr) in drop and free the memory.
// However it is not clear exactly how the notification function handles that memory and using it after us dropping it
// would be very bad.
// Use already existing struct
struct Handle(HANDLE);

impl std::ops::Drop for Handle {
    fn drop(&mut self) {
        // SAFETY: There is no clear safety specification on this function. However self.0 should point to a handle that has
        // been allocated by windows and should be non-null. Even if it is non-null this function tolerates that but would error.
        unsafe {
            if NO_ERROR as i32 != CancelMibChangeNotify2(self.0) {
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

// TODO: Improve name
struct OuterContext {
    context: Arc<Mutex<DefaultRouteMonitorContext>>,
    burst_guard: Mutex<BurstGuard>,
}

impl DefaultRouteMonitor {
    fn new<F: Fn(EventType, &Option<WinNetDefaultRoute>) + Send + 'static>(family: AddressFamily, callback: F) -> Result<Self> {
        let callback = Box::new(callback);
        // FIXME: We reordered this from the c++ code so it calls get_best_default_route in the beginning before the
        // notification calls, this might cause some weird behavior even if it at first glance looks fine.
        let context = Arc::new(Mutex::new(DefaultRouteMonitorContext::new(callback, family)?));

        let moved_context = context.clone();
        let burst_guard = Mutex::new(BurstGuard::new(move || {
            moved_context.lock().unwrap().evaluate_routes();
        }));

        // SAFETY: We need to send the OuterContext to the windows notification functions. In order to do that we will cast the Box as a *const OuterContext
        // and then cast that as a c_void pointer. This imposes the requirement that the Box is not mutated or dropped until after those notifications are no guaranteed to not run.
        // This happens when the DefaultRouteMonitor is dropped and not before then. It also imposes the requirement that OuterContext is `Sync` since we will send
        // references to it to other threads. This requirement is fullfilled since all fields of `OuterContext` are wrapped in either a Arc<Mutex> or Mutex.
        let outer_context = ReadOnly(Box::new(OuterContext {
            context,
            burst_guard,
        }));

        let family = family.to_af_family();

        // NotifyRouteChange2
        // We must provide a raw pointer that points to the context that will be used in the callbacks.
        // We provide a Mutex for the state turned into a Weak pointer turned into a raw pointer in order to not have to manually deallocate
        // the memory after we cancel the callbacks. This will leak the weak pointer but the context state itself will be correctly dropped
        // when DefaultRouteManager is dropped.
        let context_ptr = &**outer_context as *const OuterContext;
        let handle_ptr = std::ptr::null_mut();
        if NO_ERROR as i32 != unsafe {
            NotifyRouteChange2(u16::try_from(family).map_err(|_| Error::Conversion)?, Some(route_change_callback), context_ptr as *const _, WIN_FALSE, handle_ptr)
        } {
            return Err(Error::WindowsApi);
        }
        // SAFETY: NotifyRouteChange2 is guaranteed not to be an error here so handle_ptr is guaranteed to be non-null so dereferencing is safe
        let notify_route_change_handle = Handle(unsafe { *handle_ptr });

        // NotifyIpInterfaceChange
        let handle_ptr = std::ptr::null_mut();
        if NO_ERROR as i32 != unsafe {
            NotifyIpInterfaceChange(u16::try_from(family).map_err(|_| Error::Conversion)?, Some(interface_change_callback), context_ptr as *const _, WIN_FALSE, handle_ptr)
        } {
            return Err(Error::WindowsApi);
        }
        // SAFETY: NotifyIpInterfaceChange is guaranteed not to be an error here so handle_ptr is guaranteed to be non-null so dereferencing is safe
        let notify_interface_change_handle = Handle(unsafe { *handle_ptr });

        // NotifyUnicastIpAddressChange
        let handle_ptr = std::ptr::null_mut();
        if NO_ERROR as i32 != unsafe {
            NotifyUnicastIpAddressChange(u16::try_from(family).map_err(|_| Error::Conversion)?, Some(ip_address_change_callback), context_ptr as *const _, WIN_FALSE, handle_ptr)
        } {
            return Err(Error::WindowsApi);
        }
        // SAFETY: NotifyUnicastIpAddressChange is guaranteed not to be an error here so handle_ptr is guaranteed to be non-null so dereferencing is safe
        let notify_address_change_handle = Handle(unsafe { *handle_ptr });

        Ok(Self {
            context: outer_context,
            notify_route_change_handle,
            notify_interface_change_handle,
            notify_address_change_handle,
        })
    }

}

// SAFETY: `context` is a Arc::<OuterContext>::as_ptr() pointer which is fine to dereference as long as there exists a strong count to the Arc.
// This is guaranteed to be the case as we store an `Arc` in the DefaultRouteMonitor struct. When this struct is dropped it will
// cancel this callback and guarantee that it is not called after that point, only after then is the final `Arc` dropped.
unsafe extern "system" fn route_change_callback(context: *const c_void, row: *const MIB_IPFORWARD_ROW2, notification_type: MIB_NOTIFICATION_TYPE) {
    let row = unsafe { &*row };

    if row.DestinationPrefix.PrefixLength != 0 || !route_has_gateway(row) {
        return;
    }

    let outer_context: &OuterContext = unsafe { &*(context as *const OuterContext) };
    let mut context = outer_context.context.lock().unwrap();

    context.update_refresh_flag(&row.InterfaceLuid, row.InterfaceIndex);
    outer_context.burst_guard.lock().unwrap().trigger();
}

// SAFETY: `context` is a Arc::<OuterContext>::as_ptr() pointer which is fine to dereference as long as there exists a strong count to the Arc.
// This is guaranteed to be the case as we store an `Arc` in the DefaultRouteMonitor struct. When this struct is dropped it will
// cancel this callback and guarantee that it is not called after that point, only after then is the final `Arc` dropped.
unsafe extern "system" fn interface_change_callback(context: *const c_void, row: *const MIB_IPINTERFACE_ROW, notification_type: MIB_NOTIFICATION_TYPE) {
    let row = unsafe { &*row };

    let outer_context: &OuterContext = unsafe { &*(context as *const OuterContext) };
    let mut context = outer_context.context.lock().unwrap();

    context.update_refresh_flag(&row.InterfaceLuid, row.InterfaceIndex);
    outer_context.burst_guard.lock().unwrap().trigger();
}

// `context` is a Weak<Mutex<DefaultRouteManagerContext>>::into_raw() pointer and should be used with Weak::from_raw()
// SAFETY: After converting `context` into a `Weak` it must also be stopped from being dropped by calling Weak::into_raw().
// If this is not done the reference will be dropped and calling `Weak::from_raw()` the next time is undefined behavior.
unsafe extern "system" fn ip_address_change_callback(context: *const c_void, row: *const MIB_UNICASTIPADDRESS_ROW, notification_type: MIB_NOTIFICATION_TYPE) {
    let row = unsafe { &*row };

    let outer_context: &OuterContext = unsafe { &*(context as *const OuterContext) };
    let mut context = outer_context.context.lock().unwrap();

    context.update_refresh_flag(&row.InterfaceLuid, row.InterfaceIndex);
    outer_context.burst_guard.lock().unwrap().trigger();
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
        Self {
            sender
        }
    }

    /// Non-blocking
    fn trigger(&self) {
        self.sender.send(()).unwrap();
    }
}
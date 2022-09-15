use super::*;
use windows::Win32::NetworkManagement::IpHelper::{NotifyRouteChange2, NotifyIpInterfaceChange, NotifyUnicastIpAddressChange, CancelMibChangeNotify2, ConvertInterfaceLuidToIndex, MIB_NOTIFICATION_TYPE, MIB_IPINTERFACE_ROW, MIB_UNICASTIPADDRESS_ROW};
use windows::Win32::Foundation::{BOOLEAN, HANDLE};
use std::ffi::c_void;
use std::sync::{Arc, Weak, Mutex};
use std::sync::mpsc::{Sender, channel, RecvTimeoutError};
use std::time::{Instant, Duration};

pub struct RouteManager {

}

impl RouteManager {
    fn new() -> Self {
        Self {

        }
    }
}

const BURST_DURATION: Duration = Duration::from_millis(200);
const BURST_INTERVAL: Duration = Duration::from_secs(2);

struct DefaultRouteMonitorContext {
    callback: Box<dyn Fn(EventType, &Option<WinNetDefaultRoute>) + Send + 'static>,
    refresh_current_route: bool,
    family: WinNetAddrFamily,
    best_route: Option<WinNetDefaultRoute>,
}

impl DefaultRouteMonitorContext {
    fn new(callback: Box<dyn Fn(EventType, &Option<WinNetDefaultRoute>) + Send + 'static>, family: WinNetAddrFamily) -> Result<Self> {
        Ok(Self {
            callback,
            best_route: get_best_default_route(family)?,
            refresh_current_route: false,
            family,
        })
    }

    fn update_refresh_flag(&mut self, luid: &NET_LUID_LH, index: u32) {
        match &self.best_route {
            None => return,
            Some(best_route) => {
                // SAFETY: luid is a union but both fields are finally represented by u64, as such any access is valid
                if unsafe { luid.Value } == best_route.interface_luid {
                    self.refresh_current_route = true;
                }
                // SAFETY: luid is a union but both fields are finally represented by u64, as such any access is valid
                if unsafe { luid.Value } != 0 {
                    return;
                }

                let mut default_interface_index = 0;
                let route_luid = NET_LUID_LH { Value: best_route.interface_luid };
                // TODO: Different from c++ impl, check to make sure it makes sense
                match unsafe { ConvertInterfaceLuidToIndex(&route_luid, &mut default_interface_index) } {
                    Ok(()) => self.refresh_current_route = index == default_interface_index,
                    Err(_) => self.refresh_current_route = true,
                }
            }
        }
    }

    fn evaluate_routes(&mut self) {
        let refresh_current = self.refresh_current_route;
        self.refresh_current_route = false;

        let current_best_route = get_best_default_route(self.family).ok().flatten();

        match (self.best_route, current_best_route) {
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
                if best_route != current_best_route {
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
    context: Arc<Mutex<DefaultRouteMonitorContext>>,
    burst_guard: BurstGuard,
    notify_route_change_handle: Handle,
    notify_interface_change_handle: Handle,
    notify_address_change_handle: Handle,
}

// TODO: We could potentially save the raw pointer to the weak pointer that we provided to the notification function,
// that way we would be able to use Weak::from_raw(ptr) in drop and free the memory.
// However it is not clear exactly how the notification function handles that memory and using it after us dropping it
// would be very bad.
struct Handle(HANDLE);

impl std::ops::Drop for Handle {
    fn drop(&mut self) {
        // SAFETY: There is no clear safety specification on this function. However self.0 should point to a handle that has
        // been allocated by windows and should be non-null. Even if it is non-null this function tolerates that but would error.
        unsafe {
            CancelMibChangeNotify2(self.0).unwrap();
        }
    }
}

const WIN_FALSE: BOOLEAN = BOOLEAN(0);

enum EventType {
    Updated,
    UpdatedDetails,
    Removed,
}

struct OuterContext {
    context: Arc<Mutex<DefaultRouteMonitorContext>>,
    burst_guard: BurstGuard,
}

impl DefaultRouteMonitor {
    fn new<F: Fn(EventType, &Option<WinNetDefaultRoute>) + Send + 'static>(family: WinNetAddrFamily, callback: F) -> Result<Self> {
        let callback = Box::new(callback);
        // FIXME: We reordered this from the c++ code so it calls get_best_default_route in the beginning before the
        // notification calls, this might cause some weird behavior even if it at first glance looks fine.
        let context = Arc::new(Mutex::new(DefaultRouteMonitorContext::new(callback, family)?));
        let moved_context = context.clone();
        let burst_guard = BurstGuard::new(move || {
            moved_context.lock().unwrap().evaluate_routes();
        }, BURST_DURATION, BURST_INTERVAL);
        let outer_context = OuterContext {
            context,
            burst_guard,
        };
        //let context = Arc::new(Mutex::new(DefaultRouteMonitorContext {
        //    callback,
        //    best_route: get_best_default_route(family)?,
        //    refresh_current_route: false,
        //    family,
        //}));

        let family = family.to_windows_family();

        // NotifyRouteChange2
        // We must provide a raw pointer that points to the context that will be used in the callbacks.
        // We provide a Mutex for the state turned into a Weak pointer turned into a raw pointer in order to not have to manually deallocate
        // the memory after we cancel the callbacks. This will leak the weak pointer but the context state itself will be correctly dropped
        // when DefaultRouteManager is dropped.
        let context_ptr = Weak::into_raw(Arc::downgrade(&context));
        let handle_ptr = std::ptr::null_mut();
        unsafe {
            NotifyRouteChange2(u16::try_from(family.0).map_err(|_| Error::Conversion)?, Some(route_change_callback), context_ptr as *const _, WIN_FALSE, handle_ptr)
        }.map_err(|_| Error::WindowsApi)?;
        // SAFETY: NotifyRouteChange2 is guaranteed not to be an error here so handle_ptr is guaranteed to be non-null so dereferencing is safe
        let notify_route_change_handle = Handle(unsafe { *handle_ptr });

        // NotifyIpInterfaceChange
        let context_ptr = Weak::into_raw(Arc::downgrade(&context));
        let handle_ptr = std::ptr::null_mut();
        unsafe {
            NotifyIpInterfaceChange(u16::try_from(family.0).map_err(|_| Error::Conversion)?, Some(interface_change_callback), context_ptr as *const _, WIN_FALSE, handle_ptr)
        }.map_err(|_| Error::WindowsApi)?;
        // SAFETY: NotifyIpInterfaceChange is guaranteed not to be an error here so handle_ptr is guaranteed to be non-null so dereferencing is safe
        let notify_interface_change_handle = Handle(unsafe { *handle_ptr });

        // NotifyUnicastIpAddressChange
        let context_ptr = Weak::into_raw(Arc::downgrade(&context));
        let handle_ptr = std::ptr::null_mut();
        unsafe {
            NotifyUnicastIpAddressChange(u16::try_from(family.0).map_err(|_| Error::Conversion)?, Some(ip_address_change_callback), context_ptr as *const _, WIN_FALSE, handle_ptr)
        }.map_err(|_| Error::WindowsApi)?;
        // SAFETY: NotifyUnicastIpAddressChange is guaranteed not to be an error here so handle_ptr is guaranteed to be non-null so dereferencing is safe
        let notify_address_change_handle = Handle(unsafe { *handle_ptr });

        Ok(Self {
            outer_context,
            notify_route_change_handle,
            notify_interface_change_handle,
            notify_address_change_handle,
        })
    }

}

// `context` is a Weak<Mutex<DefaultRouteManagerContext>>::into_raw() pointer and should be used with Weak::from_raw()
// SAFETY: After converting `context` into a `Weak` it must also be stopped from being dropped by calling Weak::into_raw().
// If this is not done the reference will be dropped and calling `Weak::from_raw()` the next time is undefined behavior.
unsafe extern "system" fn route_change_callback(context: *const c_void, row: *const MIB_IPFORWARD_ROW2, notification_type: MIB_NOTIFICATION_TYPE) {
    let row = unsafe { &*row };
    if row.DestinationPrefix.PrefixLength != 0 || !route_has_gateway(row) {
        return;
    }

    let context_weak: Weak<Mutex<DefaultRouteMonitorContext>> = unsafe { Weak::from_raw(context as *const _) };
    // Unwrap should always succeed since this callback is only called when context has not been dropped.
    let context = context_weak.upgrade().unwrap();
    let mut context = context.lock().unwrap();
    context.update_refresh_flag(&row.InterfaceLuid, row.InterfaceIndex);
    // TODO: evaluates routes guard trigger
    context.burst_guard.trigger();
    // SAFETY: In order for successive calls we are not allowed to drop the weak pointer here as that would decrement the weak counter.
    std::mem::forget(context_weak);
}

unsafe extern "system" fn interface_change_callback(context: *const c_void, row: *const MIB_IPINTERFACE_ROW, notification_type: MIB_NOTIFICATION_TYPE) {
}

unsafe extern "system" fn ip_address_change_callback(context: *const c_void, row: *const MIB_UNICASTIPADDRESS_ROW, notification_type: MIB_NOTIFICATION_TYPE) {
}

/// BurstGuard is a wrapper for a function that protects that function from being called too many times in a short amount of time.
/// To call the function use `burst_guard.trigger()`, at that point `BurstGuard` will wait for `burst_duration` and if no more calls to
/// `trigger` are made then it will call the wrapped function. If another call to `trigger` is made during this wait then it will wait
/// another `burst_duration`, this happens over and over until either `burst_interval` time has elapsed or until no call to `trigger`
/// has been made in `burst_duration`. At which point the wrapped function will be called.
struct BurstGuard {
    sender: Sender<()>,
}

impl BurstGuard {
    fn new<F: Fn() + Send + 'static>(callback: F, burst_duration: Duration, burst_interval: Duration) -> Self {
        let (sender, listener) = channel();
        std::thread::spawn(move || {
            while let Ok(()) = listener.recv() {
                let start = Instant::now();
                let mut timeout = burst_duration;
                loop {
                    match listener.recv_timeout(timeout) {
                        Ok(()) => {
                            match burst_interval.checked_sub(start.elapsed()) {
                                Some(diff) => {
                                    if diff < burst_duration {
                                        timeout = diff;
                                    } else {
                                        timeout = burst_duration;
                                    }
                                }
                                None => {
                                    callback();
                                    break;
                                }
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
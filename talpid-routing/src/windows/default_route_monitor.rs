use super::{
    get_best_default_route, get_best_default_route::route_has_gateway, Error, InterfaceAndGateway,
    Result,
};
use crate::debounce::BurstGuard;

use std::{
    ffi::c_void,
    sync::{Arc, Mutex},
    time::Duration,
};
use talpid_types::win32_err;
use windows_sys::Win32::{
    Foundation::{BOOLEAN, HANDLE},
    NetworkManagement::{
        IpHelper::{
            CancelMibChangeNotify2, ConvertInterfaceLuidToIndex, NotifyIpInterfaceChange,
            NotifyRouteChange2, NotifyUnicastIpAddressChange, MIB_IPFORWARD_ROW2,
            MIB_IPINTERFACE_ROW, MIB_NOTIFICATION_TYPE, MIB_UNICASTIPADDRESS_ROW,
        },
        Ndis::NET_LUID_LH,
    },
};

use talpid_windows::net::AddressFamily;

const WIN_FALSE: BOOLEAN = 0;

struct DefaultRouteMonitorContext {
    callback: Box<dyn for<'a> Fn(EventType<'a>) + Send + 'static>,
    refresh_current_route: bool,
    family: AddressFamily,
    best_route: Option<InterfaceAndGateway>,
}

impl DefaultRouteMonitorContext {
    fn new(
        callback: Box<dyn for<'a> Fn(EventType<'a>) + Send + 'static>,
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
            // SAFETY: luid is a union but both fields are finally represented by u64, as such any
            // access is valid
            if unsafe { luid.Value } == unsafe { best_route.iface.Value } {
                self.refresh_current_route = true;
                return;
            }
            // SAFETY: luid is a union but both fields are finally represented by u64, as such any
            // access is valid
            if unsafe { luid.Value } != 0 {
                return;
            }

            let mut default_interface_index = 0;
            let route_luid = best_route.iface;
            // SAFETY: No clear safety specifications
            match win32_err!(unsafe {
                ConvertInterfaceLuidToIndex(&route_luid, &mut default_interface_index)
            }) {
                Ok(()) => self.refresh_current_route = index == default_interface_index,
                Err(_) => self.refresh_current_route = true,
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
                (self.callback)(EventType::Updated(self.best_route.as_ref().unwrap()));
            }
            (Some(_), None) => {
                self.best_route = None;
                (self.callback)(EventType::Removed);
            }
            (Some(best_route), Some(current_best_route)) => {
                if best_route != &current_best_route {
                    self.best_route = Some(current_best_route);
                    (self.callback)(EventType::Updated(self.best_route.as_ref().unwrap()));
                } else if refresh_current {
                    (self.callback)(EventType::UpdatedDetails(self.best_route.as_ref().unwrap()));
                }
            }
        }
    }
}

pub struct DefaultRouteMonitor {
    // SAFETY: These handles must be dropped before the context. This will happen automatically if
    // it is handled by DefaultRouteMonitors drop implementation
    notify_change_handles: Option<(NotifyChangeHandle, NotifyChangeHandle, NotifyChangeHandle)>,
    // SAFETY: Context must be dropped after all of the notifier handles have been dropped in order
    // to guarantee none of them use its pointer. This will be dropped by DefaultRouteMonitors
    // drop implementation. SAFETY: The content of this pointer is not allowed to be mutated at
    // any point except for in the drop implementation
    context: *const ContextAndBurstGuard,
}

/// SAFETY: DefaultRouteMonitor is `Send` since `NotifyChangeHandle` is `Send` and
/// `ContextAndBurstGuard` is `Sync` as it holds Mutex<T> and Arc<Mutex<T>> fields.
unsafe impl Send for DefaultRouteMonitor {}

impl Drop for DefaultRouteMonitor {
    fn drop(&mut self) {
        drop(self.notify_change_handles.take());
        // SAFETY: This pointer was created by Box::into_raw and is not modified since then.
        // This drop function is also only called once
        let context = unsafe { Box::from_raw(self.context as *mut ContextAndBurstGuard) };

        // Stop the burst guard
        let context = context.burst_guard.into_inner().unwrap();
        context.stop();
    }
}

struct NotifyChangeHandle(HANDLE);

/// SAFETY: NotifyChangeHandle is `Send` since it holds sole ownership of a pointer provided by C
unsafe impl Send for NotifyChangeHandle {}

impl Drop for NotifyChangeHandle {
    fn drop(&mut self) {
        // SAFETY: There is no clear safety specification on this function. However self.0 should
        // point to a handle that has been allocated by windows and should be non-null. Even
        // if it would be null that would cause a panic rather than UB.
        if let Err(e) = win32_err!(unsafe { CancelMibChangeNotify2(self.0) }) {
            // If this callback is called after we free the context that could result in UB, in
            // order to avoid that we panic.
            panic!("Could not cancel change notification callback: {}", e)
        }
    }
}

#[derive(PartialEq, Clone, Copy)]
/// The type of route update passed to the callback
pub enum EventType<'a> {
    /// New route
    Updated(&'a InterfaceAndGateway),
    /// Updated details of the same old route
    UpdatedDetails(&'a InterfaceAndGateway),
    /// Route removed
    Removed,
}

// SAFETY: This struct must be `Sync` otherwise it is not allowed to be sent between threads.
// Having only `Mutex<T>` or `Arc<Mutex<T>>` fields guarantees that it is `Sync`
struct ContextAndBurstGuard {
    context: Arc<Mutex<DefaultRouteMonitorContext>>,
    burst_guard: Mutex<BurstGuard>,
}

impl DefaultRouteMonitor {
    pub fn new<F: for<'a> Fn(EventType<'a>) + Send + 'static>(
        family: AddressFamily,
        callback: F,
    ) -> Result<Self> {
        let context = Arc::new(Mutex::new(DefaultRouteMonitorContext::new(
            Box::new(callback),
            family,
        )));

        const BURST_BUFFER_PERIOD: Duration = Duration::from_millis(200);
        const BURST_LONGEST_BUFFER_PERIOD: Duration = Duration::from_secs(2);

        let moved_context = context.clone();
        let burst_guard = Mutex::new(BurstGuard::new(
            BURST_BUFFER_PERIOD,
            BURST_LONGEST_BUFFER_PERIOD,
            move || {
                moved_context.lock().unwrap().evaluate_routes();
            },
        ));

        // SAFETY: We need to send the ContextAndBurstGuard to the windows notification functions as
        // a raw pointer. This imposes the requirement it is not mutated or dropped until
        // after those notifications are guaranteed to not run. This happens when the
        // DefaultRouteMonitor is dropped and not before then. It also imposes the requirement that
        // ContextAndBurstGuard is `Sync` since we will send references to it to other
        // threads. This requirement is fulfilled since all fields of `ContextAndBurstGuard` are
        // wrapped in either a Arc<Mutex> or Mutex.
        let context_and_burst = Box::into_raw(Box::new(ContextAndBurstGuard {
            context,
            burst_guard,
        })) as *const _;

        let handles = match Self::register_callbacks(family, context_and_burst) {
            Ok(handles) => handles,
            Err(e) => {
                // Clean up the memory leak in case of error
                // SAFETY: We created context_and_burst from `Box::into_raw()` and it has not been
                // modified since. All of the handles have been freed at this point
                // so there will be no risk of UAF.
                drop(unsafe { Box::from_raw(context_and_burst as *mut ContextAndBurstGuard) });
                return Err(e);
            }
        };

        let monitor = Self {
            context: context_and_burst,
            notify_change_handles: Some(handles),
        };

        // We must set the best default route after we have registered listeners in order to avoid
        // race conditions.
        {
            // SAFETY: `monitor.context` will be valid since monitor will handle dropping it. No
            // mutation happens here since we are using a Mutex.
            let context = &unsafe { &*(monitor.context) }.context;
            let mut context = context.lock().unwrap();
            context.best_route = get_best_default_route(context.family)?;
        }

        Ok(monitor)
    }

    fn register_callbacks(
        family: AddressFamily,
        context_and_burst: *const ContextAndBurstGuard,
    ) -> Result<(NotifyChangeHandle, NotifyChangeHandle, NotifyChangeHandle)> {
        let family = family.to_af_family();

        // We must provide a raw pointer that points to the context that will be used in the
        // callbacks. We provide a Mutex for the state turned into a Weak pointer turned
        // into a raw pointer in order to not have to manually deallocate the memory after
        // we cancel the callbacks. This will leak the weak pointer but the context state itself
        // will be correctly dropped when DefaultRouteManager is dropped.
        let context_ptr = context_and_burst;
        let mut handle_ptr = 0;
        // SAFETY: No clear safety specifications, context_ptr must be valid for as long as handle
        // has not been dropped.
        win32_err!(unsafe {
            NotifyRouteChange2(
                family,
                Some(route_change_callback),
                context_ptr as *const _,
                WIN_FALSE,
                &mut handle_ptr,
            )
        })
        .map_err(Error::RegisterNotifyRouteCallback)?;
        let notify_route_change_handle = NotifyChangeHandle(handle_ptr);

        let mut handle_ptr = 0;
        // SAFETY: No clear safety specifications, context_ptr must be valid for as long as handle
        // has not been dropped.
        win32_err!(unsafe {
            NotifyIpInterfaceChange(
                family,
                Some(interface_change_callback),
                context_ptr as *const _,
                WIN_FALSE,
                &mut handle_ptr,
            )
        })
        .map_err(Error::RegisterNotifyIpInterfaceCallback)?;
        let notify_interface_change_handle = NotifyChangeHandle(handle_ptr);

        let mut handle_ptr = 0;
        // SAFETY: No clear safety specifications, context_ptr must be valid for as long as handle
        // has not been dropped.
        win32_err!(unsafe {
            NotifyUnicastIpAddressChange(
                family,
                Some(ip_address_change_callback),
                context_ptr as *const _,
                WIN_FALSE,
                &mut handle_ptr,
            )
        })
        .map_err(Error::RegisterNotifyUnicastIpAddressCallback)?;
        let notify_address_change_handle = NotifyChangeHandle(handle_ptr);

        Ok((
            notify_route_change_handle,
            notify_interface_change_handle,
            notify_address_change_handle,
        ))
    }
}

// SAFETY: `context` is a Box::into_raw() pointer which may only be used as a non-mutable reference.
// It is guaranteed by the DefaultRouteMonitor to not be dropped before this function is guaranteed
// to not be called again.
unsafe extern "system" fn route_change_callback(
    context: *const c_void,
    row: *const MIB_IPFORWARD_ROW2,
    _notification_type: MIB_NOTIFICATION_TYPE,
) {
    // SAFETY: We assume Windows provides this pointer correctly
    let row = &*row;

    if row.DestinationPrefix.PrefixLength != 0 || !route_has_gateway(row) {
        return;
    }

    // SAFETY: context must not be dropped or modified until this callback has been cancelled.
    let context_and_burst: &ContextAndBurstGuard = &*(context as *const ContextAndBurstGuard);
    let mut context = context_and_burst.context.lock().unwrap();

    context.update_refresh_flag(&row.InterfaceLuid, row.InterfaceIndex);
    context_and_burst.burst_guard.lock().unwrap().trigger();
}

// SAFETY: `context` is a Box::into_raw() pointer which may only be used as a non-mutable reference.
// It is guaranteed by the DefaultRouteMonitor to not be dropped before this function is guaranteed
// to not be called again.
unsafe extern "system" fn interface_change_callback(
    context: *const c_void,
    row: *const MIB_IPINTERFACE_ROW,
    _notification_type: MIB_NOTIFICATION_TYPE,
) {
    // SAFETY: We assume Windows provides this pointer correctly
    let row = &*row;

    // SAFETY: context must not be dropped or modified until this callback has been cancelled.
    let context_and_burst: &ContextAndBurstGuard = &*(context as *const ContextAndBurstGuard);
    let mut context = context_and_burst.context.lock().unwrap();

    context.update_refresh_flag(&row.InterfaceLuid, row.InterfaceIndex);
    context_and_burst.burst_guard.lock().unwrap().trigger();
}

// SAFETY: `context` is a Box::into_raw() pointer which may only be used as a non-mutable reference.
// It is guaranteed by the DefaultRouteMonitor to not be dropped before this function is guaranteed
// to not be called again.
unsafe extern "system" fn ip_address_change_callback(
    context: *const c_void,
    row: *const MIB_UNICASTIPADDRESS_ROW,
    _notification_type: MIB_NOTIFICATION_TYPE,
) {
    // SAFETY: We assume Windows provides this pointer correctly
    let row = &*row;

    // SAFETY: context must not be dropped or modified until this callback has been cancelled.
    let context_and_burst: &ContextAndBurstGuard = &*(context as *const ContextAndBurstGuard);
    let mut context = context_and_burst.context.lock().unwrap();

    context.update_refresh_flag(&row.InterfaceLuid, row.InterfaceIndex);
    context_and_burst.burst_guard.lock().unwrap().trigger();
}

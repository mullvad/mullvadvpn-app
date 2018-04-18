use std::ffi::OsStr;
use std::io;
use widestring::WideCString;
use winapi::shared::winerror::{ERROR_CALL_NOT_IMPLEMENTED, NO_ERROR};
use winapi::um::winsvc;

use service::{ServiceControl, ServiceStatus};

mod errors {
    error_chain! {
        errors {
            InvalidServiceName {
                description("Invalid service name")
            }
        }
        foreign_links {
            System(::std::io::Error);
        }
    }
}
pub use self::errors::*;

/// Struct that holds unique token for updating the status of the corresponding service.
#[derive(Debug, Clone, Copy)]
pub struct ServiceStatusHandle(winsvc::SERVICE_STATUS_HANDLE);

impl ServiceStatusHandle {
    fn from_handle(handle: winsvc::SERVICE_STATUS_HANDLE) -> Self {
        ServiceStatusHandle(handle)
    }

    /// Report the new service status to the system
    pub fn set_service_status(&self, service_status: ServiceStatus) -> io::Result<()> {
        let mut raw_service_status = service_status.to_raw();
        let result = unsafe { winsvc::SetServiceStatus(self.0, &mut raw_service_status) };
        if result == 0 {
            Err(io::Error::last_os_error())
        } else {
            Ok(())
        }
    }
}

// Underlying SERVICE_STATUS_HANDLE is thread safe.
// See remarks section for more info:
// https://msdn.microsoft.com/en-us/library/windows/desktop/ms686241(v=vs.85).aspx
unsafe impl Send for ServiceStatusHandle {}

/// Abstraction over the return value of service control handler.
/// The meaning of each of variants in this enum depends on the type of received event.
/// See the "Return value" section of corresponding MSDN article for more info:
/// https://msdn.microsoft.com/en-us/library/windows/desktop/ms683241(v=vs.85).aspx
#[derive(Debug)]
pub enum ServiceControlHandlerResult {
    /// Either used to aknowledge the call or grant the permission in advanced events.
    NoError,
    /// The received event is not implemented.
    NotImplemented,
    /// This variant is used to deny permission and return the reason error code in advanced
    /// events.
    Other(u32),
}

impl ServiceControlHandlerResult {
    pub fn to_raw(&self) -> u32 {
        match *self {
            ServiceControlHandlerResult::NoError => NO_ERROR,
            ServiceControlHandlerResult::NotImplemented => ERROR_CALL_NOT_IMPLEMENTED,
            ServiceControlHandlerResult::Other(code) => code,
        }
    }
}

/// Register a closure for receiving service events.
/// Returns `ServiceStatusHandle` that can be used to report the service status back to the system.
pub fn register_control_handler<
    S: AsRef<OsStr>,
    F: Fn(ServiceControl) -> ServiceControlHandlerResult + 'static,
>(
    service_name: S,
    event_handler: F,
) -> Result<ServiceStatusHandle> {
    // Move closure data on heap.
    // The Box<HandlerFn> is a trait object and is stored on stack at this point.
    let heap_event_handler = Box::new(event_handler) as Box<HandlerFn>;

    // Box again to move trait object to heap.
    let boxed_event_handler: Box<Box<HandlerFn>> = Box::new(heap_event_handler);

    // Important: leak the Box<Box<HandlerFn>> which will be released in `service_control_handler`.
    let context = Box::into_raw(boxed_event_handler) as *mut ::std::os::raw::c_void;

    let service_name =
        WideCString::from_str(service_name).chain_err(|| ErrorKind::InvalidServiceName)?;
    let status_handle = unsafe {
        winsvc::RegisterServiceCtrlHandlerExW(
            service_name.as_ptr(),
            Some(service_control_handler),
            context,
        )
    };

    if status_handle.is_null() {
        Err(io::Error::last_os_error().into())
    } else {
        Ok(ServiceStatusHandle::from_handle(status_handle))
    }
}

/// Alias for control event handler closure.
type HandlerFn = Fn(ServiceControl) -> ServiceControlHandlerResult;

/// Static service control handler
#[allow(dead_code)]
extern "system" fn service_control_handler(
    control: u32,
    _event_type: u32,
    _event_data: *mut ::std::os::raw::c_void,
    context: *mut ::std::os::raw::c_void,
) -> u32 {
    // Important: cast context to &mut Box<HandlerFn> without taking ownership.
    let handler_fn = unsafe { &mut *(context as *mut Box<HandlerFn>) };

    match ServiceControl::from_raw(control) {
        Ok(service_control) => {
            let return_code = ((handler_fn)(service_control)).to_raw();

            // Important: release context upon Stop, Shutdown or Preshutdown at the end of the
            // service lifecycle.
            match service_control {
                ServiceControl::Stop | ServiceControl::Shutdown | ServiceControl::Preshutdown => {
                    let _owned_boxed_handler: Box<Box<HandlerFn>> =
                        unsafe { Box::from_raw(context as *mut Box<HandlerFn>) };
                }
                _ => (),
            };

            return_code
        }

        // Report all unknown control commands as unimplemented
        Err(_) => ServiceControlHandlerResult::NotImplemented.to_raw(),
    }
}

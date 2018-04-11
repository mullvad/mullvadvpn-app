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

/// The only useful codes that can be returned from this function are `NO_ERROR`,
/// `ERROR_CALL_NOT_IMPLEMENTED`
type HandlerFn<'a> = Fn(&'a ServiceStatusHandle, ServiceControl) -> ServiceControlHandlerResult;

/// Struct that describes a service event handler.
/// Since this struct connects to the service control dispatcher
/// it should be only instantiated from `service_main`.
pub struct ServiceControlHandler<'a> {
    status_handle: Option<ServiceStatusHandle>,
    handler_closure: &'a HandlerFn<'a>,
}

impl<'a> ServiceControlHandler<'a> {
    pub fn new<T: AsRef<OsStr>>(
        service_name: T,
        handler_closure: &'a HandlerFn<'a>,
    ) -> Result<Self> {
        let mut handler = ServiceControlHandler {
            status_handle: None,
            handler_closure,
        };

        // Danger: pass the pointer to this instance via context
        let context = &mut handler as *mut _ as *mut ::std::os::raw::c_void;

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
            handler.status_handle = Some(ServiceStatusHandle::from_handle(status_handle));
            Ok(handler)
        }
    }

    fn handle_event(&'a self, control: ServiceControl) -> ServiceControlHandlerResult {
        let status_handle = self.status_handle.as_ref().unwrap();
        (self.handler_closure)(status_handle, control)
    }
}

/// Static service control handler
#[allow(dead_code)]
extern "system" fn service_control_handler(
    control: u32,
    _event_type: u32,
    _event_data: *mut ::std::os::raw::c_void,
    context: *mut ::std::os::raw::c_void,
) -> u32 {
    // Danger: cast the context to ServiceControlHandler
    let event_handler = unsafe { &*(context as *mut ServiceControlHandler) };
    let service_control = ServiceControl::from_raw(control);

    match service_control {
        Ok(service_control) => event_handler.handle_event(service_control).to_raw(),

        // Report all unknown control commands as unimplemented
        Err(_) => ServiceControlHandlerResult::NotImplemented.to_raw(),
    }
}

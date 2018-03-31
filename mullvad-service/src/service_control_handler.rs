use std::ffi::OsStr;
use std::io;

use winapi::shared::winerror::ERROR_CALL_NOT_IMPLEMENTED;
use winapi::um::winsvc;

use service::{ServiceControl, ServiceStatus};
use widestring::to_wide_with_nul;

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

type HandlerFn<'a> = Fn(&'a ServiceStatusHandle, ServiceControl) -> u32;

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
    ) -> io::Result<Self> {
        let mut handler = ServiceControlHandler {
            status_handle: None,
            handler_closure,
        };

        // Danger: pass the pointer to this instance via context
        let context = &mut handler as *mut _ as *mut ::std::os::raw::c_void;

        let service_name = to_wide_with_nul(service_name);
        let status_handle = unsafe {
            winsvc::RegisterServiceCtrlHandlerExW(
                service_name.as_ptr(),
                Some(service_control_handler),
                context,
            )
        };

        if status_handle.is_null() {
            Err(io::Error::last_os_error())
        } else {
            handler.status_handle = Some(ServiceStatusHandle::from_handle(status_handle));
            Ok(handler)
        }
    }

    fn handle_event(&'a self, control: ServiceControl) -> u32 {
        let status_handle = self.status_handle.as_ref().unwrap();
        (self.handler_closure)(status_handle, control)
    }
}

/// Static service control handler
#[allow(dead_code)]
extern "system" fn service_control_handler(
    control: u32,
    event_type: u32,
    event_data: *mut ::std::os::raw::c_void,
    context: *mut ::std::os::raw::c_void,
) -> u32 {
    // Danger: cast the context to ServiceControlHandler
    let event_handler = unsafe { &*(context as *mut ServiceControlHandler) };
    let service_control = ServiceControl::from_raw(control);

    match service_control {
        Ok(service_control) => {
            debug!("Received service control event: {:?}", service_control);
            event_handler.handle_event(service_control)
        }

        // Report all unknown control commands as unimplemented
        Err(ref e) => {
            warn!("Received unrecognized service control request: {}", e);
            ERROR_CALL_NOT_IMPLEMENTED
        }
    }
}

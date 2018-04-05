use std::borrow::Cow;
use std::ffi::{OsStr, OsString};
use std::{io, ptr};

use widestring::WideCString;
use winapi::um::winsvc;

use service::{Service, ServiceAccess, ServiceInfo};
use shell_escape;

/// Flags describing access permissions for ServiceManager
bitflags! {
    pub struct ServiceManagerAccess: u32 {
        /// Can connect to service control manager
        const CONNECT = winsvc::SC_MANAGER_CONNECT;

        /// Can create services
        const CREATE_SERVICE = winsvc::SC_MANAGER_CREATE_SERVICE;

        /// Can enumerate services
        const ENUMERATE_SERVICE = winsvc::SC_MANAGER_ENUMERATE_SERVICE;
    }
}

/// Service control manager
pub struct ServiceManager(winsvc::SC_HANDLE);

impl ServiceManager {
    /// Private initializer
    /// Passing None for machine connects to local machine
    /// Passing None for database connects to active database
    fn new<M: AsRef<OsStr>, D: AsRef<OsStr>>(
        machine: Option<M>,
        database: Option<D>,
        request_access: ServiceManagerAccess,
    ) -> io::Result<Self> {
        let machine_name = machine.map(|ref s| unsafe { WideCString::from_str_unchecked(s) });
        let database_name = database.map(|ref s| unsafe { WideCString::from_str_unchecked(s) });

        let handle = unsafe {
            winsvc::OpenSCManagerW(
                machine_name.map_or(ptr::null(), |s| s.as_ptr()),
                database_name.map_or(ptr::null(), |s| s.as_ptr()),
                request_access.bits(),
            )
        };

        if handle.is_null() {
            Err(io::Error::last_os_error())
        } else {
            Ok(ServiceManager(handle))
        }
    }

    /// Passing None for database connects to active database
    pub fn local_computer<T: AsRef<OsStr>>(
        database: Option<T>,
        request_access: ServiceManagerAccess,
    ) -> io::Result<Self> {
        ServiceManager::new(None::<&OsStr>, database, request_access)
    }

    /// Passing None for database connects to active database
    pub fn remote_computer<M: AsRef<OsStr>, D: AsRef<OsStr>>(
        machine: M,
        database: Option<D>,
        request_access: ServiceManagerAccess,
    ) -> io::Result<Self> {
        ServiceManager::new(Some(machine), database, request_access)
    }

    pub fn create_service(
        &self,
        service_info: ServiceInfo,
        service_access: ServiceAccess,
    ) -> io::Result<Service> {
        // escape executable path
        let launch_executable = shell_escape::escape(Cow::Borrowed(
            service_info.executable_path.as_os_str(),
        )).into_owned();

        // escape launch arguments
        let launch_arguments = service_info
            .launch_arguments
            .into_iter()
            .map(|s| shell_escape::escape(Cow::Owned(s)).into_owned())
            .collect::<Vec<OsString>>();

        // combine escaped executable path and arguments into command
        let mut launch_command = OsString::new();
        launch_command.push(launch_executable);
        for launch_argument in launch_arguments.iter() {
            launch_command.push(" ");
            launch_command.push(launch_argument);
        }

        let service_name = unsafe { WideCString::from_str_unchecked(service_info.name) };
        let display_name = unsafe { WideCString::from_str_unchecked(service_info.display_name) };
        let wide_launch_command = unsafe { WideCString::from_str_unchecked(launch_command) };
        let account_name = service_info
            .account_name
            .map(|ref s| unsafe { WideCString::from_str_unchecked(s) });
        let account_password = service_info
            .account_password
            .map(|ref s| unsafe { WideCString::from_str_unchecked(s) });

        let service_handle = unsafe {
            winsvc::CreateServiceW(
                self.0,
                service_name.as_ptr(),
                display_name.as_ptr(),
                service_access.bits(),
                service_info.service_type.to_raw(),
                service_info.start_type.to_raw(),
                service_info.error_control.to_raw(),
                wide_launch_command.as_ptr(),
                ptr::null(),     // load ordering group
                ptr::null_mut(), // tag id within the load ordering group
                ptr::null(),     // service dependencies
                account_name.map_or(ptr::null(), |s| s.as_ptr()),
                account_password.map_or(ptr::null(), |s| s.as_ptr()),
            )
        };

        if service_handle.is_null() {
            Err(io::Error::last_os_error())
        } else {
            Ok(unsafe { Service::from_handle(service_handle) })
        }
    }

    pub fn open_service<T: AsRef<OsStr>>(
        &self,
        name: T,
        request_access: ServiceAccess,
    ) -> io::Result<Service> {
        let service_name = unsafe { WideCString::from_str_unchecked(name) };
        let service_handle =
            unsafe { winsvc::OpenServiceW(self.0, service_name.as_ptr(), request_access.bits()) };

        if service_handle.is_null() {
            Err(io::Error::last_os_error())
        } else {
            Ok(unsafe { Service::from_handle(service_handle) })
        }
    }
}

impl Drop for ServiceManager {
    fn drop(&mut self) {
        unsafe { winsvc::CloseServiceHandle(self.0) };
    }
}

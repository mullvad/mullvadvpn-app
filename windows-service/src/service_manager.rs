use std::borrow::Cow;
use std::ffi::OsStr;
use std::{io, ptr};

use widestring::{NulError, WideCString, WideString};
use winapi::um::winsvc;

use service::{Service, ServiceAccess, ServiceInfo};
use shell_escape;

mod errors {
    error_chain! {
        errors {
            InvalidAccountName {
                description("Invalid account name")
            }
            InvalidAccountPassword {
                description("Invalid account password")
            }
            InvalidDisplayName {
                description("Invalid display name")
            }
            InvalidDatabaseName {
                description("Invalid database name")
            }
            InvalidExecutablePath {
                description("Invalid executable path")
            }
            InvalidLaunchArgument {
                description("Invalid launch argument")
            }
            InvalidMachineName {
                description("Invalid machine name")
            }
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
    ) -> Result<Self> {
        let machine_name = to_wide(machine).chain_err(|| ErrorKind::InvalidMachineName)?;
        let database_name = to_wide(database).chain_err(|| ErrorKind::InvalidDatabaseName)?;
        let handle = unsafe {
            winsvc::OpenSCManagerW(
                machine_name.map_or(ptr::null(), |s| s.as_ptr()),
                database_name.map_or(ptr::null(), |s| s.as_ptr()),
                request_access.bits(),
            )
        };

        if handle.is_null() {
            Err(io::Error::last_os_error().into())
        } else {
            Ok(ServiceManager(handle))
        }
    }

    /// Passing None for database connects to active database
    pub fn local_computer<D: AsRef<OsStr>>(
        database: Option<D>,
        request_access: ServiceManagerAccess,
    ) -> Result<Self> {
        ServiceManager::new(None::<&OsStr>, database, request_access)
    }

    /// Passing None for database connects to active database
    pub fn remote_computer<M: AsRef<OsStr>, D: AsRef<OsStr>>(
        machine: M,
        database: Option<D>,
        request_access: ServiceManagerAccess,
    ) -> Result<Self> {
        ServiceManager::new(Some(machine), database, request_access)
    }

    pub fn create_service(
        &self,
        service_info: ServiceInfo,
        service_access: ServiceAccess,
    ) -> Result<Service> {
        let service_name =
            WideCString::from_str(service_info.name).chain_err(|| ErrorKind::InvalidServiceName)?;
        let display_name = WideCString::from_str(service_info.display_name)
            .chain_err(|| ErrorKind::InvalidDisplayName)?;
        let account_name =
            to_wide(service_info.account_name).chain_err(|| ErrorKind::InvalidAccountName)?;
        let account_password =
            to_wide(service_info.account_password).chain_err(|| ErrorKind::InvalidAccountPassword)?;

        // escape executable path and arguments and combine them into single command
        let executable_path = escape_wide(service_info.executable_path)
            .chain_err(|| ErrorKind::InvalidExecutablePath)?;

        let mut launch_command_buffer = WideString::new();
        launch_command_buffer.push(executable_path);

        for launch_argument in service_info.launch_arguments.iter() {
            let wide = escape_wide(launch_argument).chain_err(|| ErrorKind::InvalidLaunchArgument)?;

            launch_command_buffer.push_str(" ");
            launch_command_buffer.push(wide);
        }

        let launch_command = WideCString::from_wide_str(launch_command_buffer).unwrap();

        let service_handle = unsafe {
            winsvc::CreateServiceW(
                self.0,
                service_name.as_ptr(),
                display_name.as_ptr(),
                service_access.bits(),
                service_info.service_type.to_raw(),
                service_info.start_type.to_raw(),
                service_info.error_control.to_raw(),
                launch_command.as_ptr(),
                ptr::null(),     // load ordering group
                ptr::null_mut(), // tag id within the load ordering group
                ptr::null(),     // service dependencies
                account_name.map_or(ptr::null(), |s| s.as_ptr()),
                account_password.map_or(ptr::null(), |s| s.as_ptr()),
            )
        };

        if service_handle.is_null() {
            Err(io::Error::last_os_error().into())
        } else {
            Ok(unsafe { Service::from_handle(service_handle) })
        }
    }

    pub fn open_service<T: AsRef<OsStr>>(
        &self,
        name: T,
        request_access: ServiceAccess,
    ) -> Result<Service> {
        let service_name = WideCString::from_str(name).chain_err(|| ErrorKind::InvalidServiceName)?;
        let service_handle =
            unsafe { winsvc::OpenServiceW(self.0, service_name.as_ptr(), request_access.bits()) };

        if service_handle.is_null() {
            Err(io::Error::last_os_error().into())
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

fn to_wide<T: AsRef<OsStr>>(s: Option<T>) -> ::std::result::Result<Option<WideCString>, NulError> {
    if let Some(s) = s {
        Ok(Some(WideCString::from_str(s)?))
    } else {
        Ok(None)
    }
}

fn escape_wide<T: AsRef<OsStr>>(s: T) -> ::std::result::Result<WideString, NulError> {
    let escaped = shell_escape::escape(Cow::Borrowed(s.as_ref()));
    let wide = WideCString::from_str(escaped)?;
    Ok(wide.to_wide_string())
}

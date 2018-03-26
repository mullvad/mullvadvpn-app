use std::ffi::OsStr;
use std::io;
use std::ptr;

use service::{Service, ServiceAccess, ServiceInfo};
use widestring::to_wide_with_nul;
use winapi::um::winsvc;

/// Enum describing access permissions for ServiceManager
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u32)]
pub enum ServiceManagerAccess {
    All = winsvc::SC_MANAGER_ALL_ACCESS,
    Connect = winsvc::SC_MANAGER_CONNECT,
    CreateService = winsvc::SC_MANAGER_CREATE_SERVICE,
    EnumerateService = winsvc::SC_MANAGER_ENUMERATE_SERVICE,
}

impl ServiceManagerAccess {
    pub fn to_raw(&self) -> u32 {
        *self as u32
    }

    pub fn raw_mask(values: &[ServiceManagerAccess]) -> u32 {
        values.iter().fold(0, |acc, &x| (acc | x.to_raw()))
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
        access_mask: &[ServiceManagerAccess],
    ) -> io::Result<Self> {
        let machine_name = machine.map(to_wide_with_nul);
        let machine_ptr = machine_name.map_or(ptr::null(), |vec| vec.as_ptr());

        let database_name = database.map(to_wide_with_nul);
        let database_ptr = database_name.map_or(ptr::null(), |vec| vec.as_ptr());

        let handle = unsafe {
            winsvc::OpenSCManagerW(
                machine_ptr,
                database_ptr,
                ServiceManagerAccess::raw_mask(access_mask),
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
        access_mask: &[ServiceManagerAccess],
    ) -> io::Result<Self> {
        ServiceManager::new(None::<&OsStr>, database, access_mask)
    }

    /// Passing None for database connects to active database
    pub fn remote_computer<T: AsRef<OsStr>, Y: AsRef<OsStr>>(
        machine: T,
        database: Option<Y>,
        access_mask: &[ServiceManagerAccess],
    ) -> io::Result<Self> {
        ServiceManager::new(Some(machine), database, access_mask)
    }

    pub fn create_service(
        &self,
        service_info: ServiceInfo,
        access_mask: &[ServiceAccess],
    ) -> io::Result<Service> {
        let service_name = to_wide_with_nul(service_info.name);
        let display_name = to_wide_with_nul(service_info.display_name);
        let executable_path = to_wide_with_nul(service_info.executable_path);
        let account_name = service_info.account_name.map(to_wide_with_nul);
        let account_name_ptr = account_name.map_or(ptr::null(), |vec| vec.as_ptr());
        let account_password = service_info.account_password.map(to_wide_with_nul);
        let account_password_ptr = account_password.map_or(ptr::null(), |vec| vec.as_ptr());

        let service_handle = unsafe {
            winsvc::CreateServiceW(
                self.0,
                service_name.as_ptr(),
                display_name.as_ptr(),
                ServiceAccess::raw_mask(access_mask),
                service_info.service_type.to_raw(),
                service_info.start_type.to_raw(),
                service_info.error_control.to_raw(),
                executable_path.as_ptr(),
                ptr::null(),     // load ordering group
                ptr::null_mut(), // tag id within the load ordering group
                ptr::null(),     // service dependencies
                account_name_ptr,
                account_password_ptr,
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
        access_mask: &[ServiceAccess],
    ) -> io::Result<Service> {
        let service_name = to_wide_with_nul(name);
        let service_handle = unsafe {
            winsvc::OpenServiceW(
                self.0,
                service_name.as_ptr(),
                ServiceAccess::raw_mask(access_mask),
            )
        };

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

use std::ffi::OsStr;
use std::io;
use std::ptr;

use service::{Service, ServiceAccess, ServiceInfo};
use widestring::to_wide_with_nul;
use winapi::um::winsvc;

/// Enum describing access permissions for SCManager
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SCManagerAccess {
    All,
    Connect,
    CreateService,
    EnumerateService,
}

impl SCManagerAccess {
    pub fn to_raw(&self) -> u32 {
        match self {
            &SCManagerAccess::All => winsvc::SC_MANAGER_ALL_ACCESS,
            &SCManagerAccess::Connect => winsvc::SC_MANAGER_CONNECT,
            &SCManagerAccess::CreateService => winsvc::SC_MANAGER_CREATE_SERVICE,
            &SCManagerAccess::EnumerateService => winsvc::SC_MANAGER_ENUMERATE_SERVICE,
        }
    }

    pub fn raw_mask(values: &[SCManagerAccess]) -> u32 {
        values.iter().fold(0, |acc, &x| (acc | x.to_raw()))
    }
}

/// Service control manager
pub struct SCManager(winsvc::SC_HANDLE);

impl SCManager {
    /// Designated initializer
    /// Passing None for machine connects to local machine
    /// Passing None for database connects to active database
    pub fn new<MACHINE: AsRef<OsStr>, DATABASE: AsRef<OsStr>>(
        machine: Option<MACHINE>,
        database: Option<DATABASE>,
        access_mask: &[SCManagerAccess],
    ) -> io::Result<Self> {
        let machine_name = machine.map(to_wide_with_nul);
        let machine_ptr = machine_name.map_or(ptr::null(), |vec| vec.as_ptr());

        let database_name = database.map(to_wide_with_nul);
        let database_ptr = database_name.map_or(ptr::null(), |vec| vec.as_ptr());

        let handle = unsafe {
            winsvc::OpenSCManagerW(
                machine_ptr,
                database_ptr,
                SCManagerAccess::raw_mask(access_mask),
            )
        };

        if handle.is_null() {
            Err(io::Error::last_os_error())
        } else {
            Ok(SCManager(handle))
        }
    }

    pub fn local_computer<DATABASE: AsRef<OsStr>>(
        database: DATABASE,
        access_mask: &[SCManagerAccess],
    ) -> io::Result<Self> {
        SCManager::new(None::<&OsStr>, Some(database), access_mask)
    }

    pub fn active_database(access_mask: &[SCManagerAccess]) -> io::Result<Self> {
        SCManager::new(None::<&OsStr>, None::<&OsStr>, access_mask)
    }

    pub fn create_service(&self, service_info: ServiceInfo) -> io::Result<Service> {
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
                ServiceAccess::raw_mask(&service_info.service_access),
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
            Ok(Service(service_handle))
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
            Ok(Service(service_handle))
        }
    }
}

impl Drop for SCManager {
    fn drop(&mut self) {
        unsafe { winsvc::CloseServiceHandle(self.0) };
    }
}

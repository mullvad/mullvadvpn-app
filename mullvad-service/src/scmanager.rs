use std;
use std::io;
use std::ffi::OsStr;

use winapi::um::winsvc;
use service::{Service, ServiceAccessMask, ServiceInfo};
use widestring::to_wide_with_nul;

/// Enum describing access permissions for SCManager
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SCManagerAccess {
    All,
    Connect,
    CreateService,
    EnumerateService,
}

impl From<SCManagerAccess> for u32 {
    fn from(access: SCManagerAccess) -> Self {
        match access {
            SCManagerAccess::All => winsvc::SC_MANAGER_ALL_ACCESS,
            SCManagerAccess::Connect => winsvc::SC_MANAGER_CONNECT,
            SCManagerAccess::CreateService => winsvc::SC_MANAGER_CREATE_SERVICE,
            SCManagerAccess::EnumerateService => winsvc::SC_MANAGER_ENUMERATE_SERVICE,
        }
    }
}

/// Bitwise mask helper for SCManagerAccess
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SCManagerAccessMask(Vec<SCManagerAccess>);
impl SCManagerAccessMask {
    pub fn new(set: &[SCManagerAccess]) -> Self {
        SCManagerAccessMask(set.to_vec())
    }
}

impl<'a> From<&'a SCManagerAccessMask> for u32 {
    fn from(mask: &SCManagerAccessMask) -> Self {
        mask.0.iter().fold(0, |acc, &x| (acc | u32::from(x)))
    }
}

/// Service control manager
pub struct SCManager(winsvc::SC_HANDLE);
impl SCManager {

    /// Designated initializer
    /// Passing None for machine connects to local machine
    /// Passing None for database connects to active database
    pub fn new<MACHINE: AsRef<OsStr>, DATABASE: AsRef<OsStr>>(machine: Option<MACHINE>, database: Option<DATABASE>, access_mask: SCManagerAccessMask) -> io::Result<Self> {        
        let machine_name = machine.map(|s| to_wide_with_nul(s));
        let machine_ptr = machine_name.map_or(std::ptr::null(), |vec| vec.as_ptr());

        let database_name = database.map(|s| to_wide_with_nul(s));
        let database_ptr = database_name.map_or(std::ptr::null(), |vec| vec.as_ptr());
        
        let raw_access_mask: u32 = (&access_mask).into();
        let handle = unsafe { winsvc::OpenSCManagerW(machine_ptr, database_ptr, raw_access_mask) };
        
        if handle.is_null() {
            Err(io::Error::last_os_error())
        } else {
            Ok(SCManager(handle))
        }
    }

    pub fn local_computer<DATABASE: AsRef<OsStr>>(database: DATABASE, access_mask: SCManagerAccessMask) -> io::Result<Self> {
        SCManager::new(None::<&OsStr>, Some(database), access_mask)
    }

    pub fn active_database(access_mask: SCManagerAccessMask) -> io::Result<Self> {
        SCManager::new(None::<&OsStr>, None::<&OsStr>, access_mask)
    }

    pub fn create_service(&self, service_info: ServiceInfo) -> io::Result<Service> {
        let service_name = to_wide_with_nul(service_info.name);
        let display_name = to_wide_with_nul(service_info.display_name);
        let executable_path = to_wide_with_nul(service_info.executable_path);

        let account_name = service_info.account_name.map(|s| to_wide_with_nul(s));
        let account_name_ptr = account_name.map_or(std::ptr::null(), |vec| vec.as_ptr());

        let account_password = service_info.account_password.map(|s| to_wide_with_nul(s));
        let account_password_ptr = account_password.map_or(std::ptr::null(), |vec| vec.as_ptr());

        let raw_service_access_mask: u32 = (&service_info.service_access).into();
        let raw_service_type: u32 = service_info.service_type.into();
        let raw_start_type: u32 = service_info.start_type.into();
        let raw_error_control: u32 = service_info.error_control.into();

        let service_handle = unsafe { winsvc::CreateServiceW(
            self.0,
            service_name.as_ptr(),
            display_name.as_ptr(),
            raw_service_access_mask,
            raw_service_type,
            raw_start_type,
            raw_error_control,
            executable_path.as_ptr(),
            std::ptr::null(), // load ordering group
            std::ptr::null_mut(), // tag id within the load ordering group
            std::ptr::null(), // service dependencies
            account_name_ptr,
            account_password_ptr,
        ) };

        if service_handle.is_null() {
            Err(io::Error::last_os_error())
        } else {
            Ok(Service(service_handle))
        }
    }

    pub fn open_service<T: AsRef<OsStr>>(&self, name: T, access_mask: ServiceAccessMask) -> io::Result<Service> {
        let service_name = to_wide_with_nul(name);
        let raw_access_mask: u32 = (&access_mask).into();
        let service_handle = unsafe { winsvc::OpenServiceW(self.0, service_name.as_ptr(), raw_access_mask) };
        
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
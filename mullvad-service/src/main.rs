extern crate winapi;

use std::ffi::{OsStr, OsString};
use std::os::windows::prelude::*;
use std::error;
use std::io;

use winapi::um::winsvc;
use winapi::um::winnt;

static SERVICE_NAME: &'static str = "Mullvad";
static SERVICE_DISPLAY_NAME: &'static str = "Mullvad VPN Service";

fn main() {
    if let Some(command) = std::env::args().nth(1) {
        match command.as_ref() {
            "-install" | "/install" => {
                if let Err(e) = install_service() {
                    println!("Failed to install the service: {}", e);
                } else {
                    println!("Installed the service.");
                }
            },
            "-remove" | "/remove" => {
                if let Err(e) = remove_service() {
                    println!("Failed to remove the service: {}", e);
                } else {
                    println!("Removed the service.");
                }
            },
            _ => println!("Unsupported command: {}", command),
        }
    } else {
        println!("Usage:");
        println!("-install to install the service");
        println!("-remove to uninstall the service")
    }
}

fn install_service() -> Result<(), io::Error> {
    let access_mask = SCManagerAccessMask::new(&[SCManagerAccess::Connect, SCManagerAccess::CreateService]);
    let service_manager = SCManager::active_database(access_mask)?;
    let service_info = get_service_info();
    service_manager.create_service(service_info).map(|_| ())
}

fn remove_service() -> Result<(), io::Error> {
    let access_mask = SCManagerAccessMask::new(&[SCManagerAccess::Connect, SCManagerAccess::CreateService]);
    let service_manager = SCManager::active_database(access_mask)?;

    let request_access_mask = ServiceAccessMask::new(&[ServiceAccess::QueryStatus, ServiceAccess::Stop]);
    let _service = service_manager.open_service(SERVICE_NAME, request_access_mask)?;
    
    // TBD: stop and delete
    
    Ok(())
}

fn get_service_info() -> ServiceInfo {
    let executable_path = std::env::current_exe().unwrap();
    ServiceInfo {
        name: OsString::from(SERVICE_NAME),
        display_name: OsString::from(SERVICE_DISPLAY_NAME),
        service_access: ServiceAccessMask::new(&[ServiceAccess::QueryStatus]),
        service_type: ServiceType::OwnProcess,
        start_type: ServiceStartType::OnDemand, // TBD: change to AutoStart
        error_control: ServiceErrorControl::Normal,
        executable_path: OsString::from(executable_path),
        account_name: None, // run as System
        account_password: None,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum SCManagerAccess {
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

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct SCManagerAccessMask(Vec<SCManagerAccess>);
impl SCManagerAccessMask {
    fn new(set: &[SCManagerAccess]) -> Self {
        SCManagerAccessMask(set.to_vec())
    }
}

impl<'a> From<&'a SCManagerAccessMask> for u32 {
    fn from(mask: &SCManagerAccessMask) -> Self {
        mask.0.iter().fold(0, |acc, &x| (acc | u32::from(x)))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum ServiceType {
    OwnProcess,
}

impl From<ServiceType> for u32 {
    fn from(service_type: ServiceType) -> Self {
        match service_type {
            ServiceType::OwnProcess => winnt::SERVICE_WIN32_OWN_PROCESS,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum ServiceAccess {
    QueryStatus,
    Start,
    Stop,
}

impl From<ServiceAccess> for u32 {
    fn from(access: ServiceAccess) -> Self {
        match access {
            ServiceAccess::QueryStatus => winsvc::SERVICE_QUERY_STATUS,
            ServiceAccess::Start => winsvc::SERVICE_START,
            ServiceAccess::Stop => winsvc::SERVICE_STOP,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct ServiceAccessMask(Vec<ServiceAccess>);
impl ServiceAccessMask {
    fn new(set: &[ServiceAccess]) -> Self {
        ServiceAccessMask(set.to_vec())
    }
}

impl<'a> From<&'a ServiceAccessMask> for u32 {
    fn from(mask: &ServiceAccessMask) -> Self {
        mask.0.iter().fold(0, |acc, &x| (acc | u32::from(x)))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum ServiceStartType {
    AutoStart,
    OnDemand,
    Disabled,
}

impl From<ServiceStartType> for u32 {
    fn from(start_type: ServiceStartType) -> Self {
        match start_type {
            ServiceStartType::AutoStart => winnt::SERVICE_AUTO_START,
            ServiceStartType::OnDemand => winnt::SERVICE_DEMAND_START,
            ServiceStartType::Disabled => winnt::SERVICE_DISABLED,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum ServiceErrorControl {
    Critical,
    Ignore,
    Normal,
    Severe
}

impl From<ServiceErrorControl> for u32 {
    fn from(error_control: ServiceErrorControl) -> Self {
        match error_control {
            ServiceErrorControl::Critical => winnt::SERVICE_ERROR_NORMAL,
            ServiceErrorControl::Ignore => winnt::SERVICE_ERROR_IGNORE,
            ServiceErrorControl::Normal => winnt::SERVICE_ERROR_NORMAL,
            ServiceErrorControl::Severe => winnt::SERVICE_ERROR_SEVERE,
        }
    }
}

struct ServiceInfo {
    name: OsString, 
    display_name: OsString,
    service_access: ServiceAccessMask,
    service_type: ServiceType,
    start_type: ServiceStartType,
    error_control: ServiceErrorControl,
    executable_path: OsString,
    account_name: Option<OsString>, // use None to run as LocalSystem
    account_password: Option<OsString>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum ServiceControl {
    Stop,
}

impl From<ServiceControl> for u32 {
    fn from(control_command: ServiceControl) -> Self {
        match control_command {
            ServiceControl::Stop => winsvc::SERVICE_CONTROL_STOP,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum ServiceState {
    Stopped,
    StartPending,
    StopPending,
    Running,
    ContinuePending,
    PausePending,
    Paused,
}

#[derive(Debug, Clone)]
struct ConversionError;

impl std::error::Error for ConversionError {
    fn description(&self) -> &str {
        "Conversion error"
    }
    fn cause(&self) -> Option<&std::error::Error> {
        None
    }
}
impl std::fmt::Display for ConversionError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Conversion error")
    }
}

trait TryConvertFrom<T: ?Sized> where Self: Sized {
    type Error;
    fn try_convert_from(value: T) -> Result<Self, Self::Error>;
}

impl TryConvertFrom<u32> for ServiceState {
    type Error = ConversionError;

    fn try_convert_from(raw_state: u32) -> Result<Self, Self::Error> {
        match raw_state {
            winsvc::SERVICE_STOPPED => Ok(ServiceState::Stopped),
            winsvc::SERVICE_START_PENDING => Ok(ServiceState::StartPending),
            winsvc::SERVICE_STOP_PENDING => Ok(ServiceState::StopPending),
            winsvc::SERVICE_RUNNING => Ok(ServiceState::Running),
            winsvc::SERVICE_CONTINUE_PENDING => Ok(ServiceState::ContinuePending),
            winsvc::SERVICE_PAUSE_PENDING => Ok(ServiceState::PausePending),
            winsvc::SERVICE_PAUSED => Ok(ServiceState::Paused),
            _ => Err(ConversionError),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct ServiceStatus {
    current_state: ServiceState,
}

impl TryConvertFrom<winsvc::SERVICE_STATUS> for ServiceStatus {
    type Error = ConversionError;

    fn try_convert_from(raw_status: winsvc::SERVICE_STATUS) -> Result<Self, Self::Error> {
        let current_state = ServiceState::try_convert_from(raw_status.dwCurrentState as u32)?;
        Ok(ServiceStatus {
            current_state: current_state
        })
    }
}

struct Service(winsvc::SC_HANDLE);
impl Service {
    fn send_control_command(&self, command: ServiceControl) -> Result<ServiceStatus, io::Error> {
        let mut raw_status = unsafe { std::mem::zeroed::<winsvc::SERVICE_STATUS>() };
        let raw_command: u32 = command.into();
        let success = unsafe { winsvc::ControlService(self.0, raw_command, &mut raw_status) };

        if success == 1 {
            // TBD: expected io::Error but got Conversion error
            Ok(ServiceStatus::try_convert_from(raw_status).unwrap())
        } else {
            Err(io::Error::last_os_error())
        }
    }

    //fn stop(&self) -> Result<ServiceStatus, Error> {
    //    self.send_control_command(command: ServiceControl::Stop)
    //}
}

impl Drop for Service {
    fn drop(&mut self) {
        unsafe { winsvc::CloseServiceHandle(self.0) };
    }
}

struct SCManager(winsvc::SC_HANDLE);
impl SCManager {
    fn new<MACHINE: AsRef<OsStr>, DATABASE: AsRef<OsStr>>(machine: Option<MACHINE>, database: Option<DATABASE>, access_mask: SCManagerAccessMask) -> Result<Self, io::Error> {        
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

    fn local_computer<DATABASE: AsRef<OsStr>>(database: DATABASE, access_mask: SCManagerAccessMask) -> Result<Self, io::Error> {
        SCManager::new(None::<&OsStr>, Some(database), access_mask)
    }

    fn active_database(access_mask: SCManagerAccessMask) -> Result<Self, io::Error> {
        SCManager::new(None::<&OsStr>, None::<&OsStr>, access_mask)
    }

    fn create_service(&self, service_info: ServiceInfo) -> Result<Service, io::Error> {
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

    fn open_service<T: AsRef<OsStr>>(&self, name: T, access_mask: ServiceAccessMask) -> Result<Service, io::Error> {
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

fn to_wide_with_nul<T: AsRef<OsStr>>(os_string: T) -> Vec<u16> {
    os_string.as_ref().encode_wide().chain(Some(0).into_iter()).collect::<Vec<_>>()
}
use std::error;
use std::ffi::OsString;
use std::fmt;
use std::io;
use std::mem;

use winapi::um::{winnt, winsvc};

use errors::RawConversionError;

#[derive(Debug)]
pub enum ServiceError {
    RawConversion(RawConversionError),
    System(io::Error),
}

impl error::Error for ServiceError {
    fn description(&self) -> &str {
        "Service error"
    }

    fn cause(&self) -> Option<&error::Error> {
        match self {
            &ServiceError::RawConversion(ref err) => Some(err),
            &ServiceError::System(ref io_err) => Some(io_err),
        }
    }
}

impl fmt::Display for ServiceError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            &ServiceError::RawConversion(ref err) => err.fmt(f),
            &ServiceError::System(ref io_err) => io_err.fmt(f),
        }
    }
}

impl From<io::Error> for ServiceError {
    fn from(io_error: io::Error) -> Self {
        ServiceError::System(io_error)
    }
}

/// Enum describing types of windows services
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ServiceType {
    /// Service that runs in its own process.
    OwnProcess,
}

impl ServiceType {
    pub fn to_raw(&self) -> u32 {
        match self {
            &ServiceType::OwnProcess => winnt::SERVICE_WIN32_OWN_PROCESS,
        }
    }
}

/// Enum describing the access permissions when working with Services
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ServiceAccess {
    QueryStatus,
    Start,
    Stop,
    Delete,
}

impl ServiceAccess {
    pub fn to_raw(&self) -> u32 {
        match self {
            &ServiceAccess::QueryStatus => winsvc::SERVICE_QUERY_STATUS,
            &ServiceAccess::Start => winsvc::SERVICE_START,
            &ServiceAccess::Stop => winsvc::SERVICE_STOP,
            &ServiceAccess::Delete => winnt::DELETE,
        }
    }

    pub fn raw_mask(values: &[ServiceAccess]) -> u32 {
        values.iter().fold(0, |acc, &x| (acc | x.to_raw()))
    }
}

/// Enum describing the start options for windows services
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ServiceStartType {
    /// Autostart on system startup
    AutoStart,
    /// Service is enabled, can be started manually
    OnDemand,
    /// Disabled service
    Disabled,
}

impl ServiceStartType {
    pub fn to_raw(&self) -> u32 {
        match self {
            &ServiceStartType::AutoStart => winnt::SERVICE_AUTO_START,
            &ServiceStartType::OnDemand => winnt::SERVICE_DEMAND_START,
            &ServiceStartType::Disabled => winnt::SERVICE_DISABLED,
        }
    }
}

/// Error handling strategy for service failures
/// See https://msdn.microsoft.com/en-us/library/windows/desktop/ms682450(v=vs.85).aspx
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ServiceErrorControl {
    Critical,
    Ignore,
    Normal,
    Severe,
}

impl ServiceErrorControl {
    pub fn to_raw(&self) -> u32 {
        match self {
            &ServiceErrorControl::Critical => winnt::SERVICE_ERROR_NORMAL,
            &ServiceErrorControl::Ignore => winnt::SERVICE_ERROR_IGNORE,
            &ServiceErrorControl::Normal => winnt::SERVICE_ERROR_NORMAL,
            &ServiceErrorControl::Severe => winnt::SERVICE_ERROR_SEVERE,
        }
    }
}

/// A struct that describes the service
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ServiceInfo {
    pub name: OsString,
    pub display_name: OsString,
    pub service_access: Vec<ServiceAccess>,
    pub service_type: ServiceType,
    pub start_type: ServiceStartType,
    pub error_control: ServiceErrorControl,
    pub executable_path: OsString,
    pub account_name: Option<OsString>, // use None to run as LocalSystem
    pub account_password: Option<OsString>,
}

// Private enum describing the service control operations
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum ServiceControl {
    Stop,
}

impl ServiceControl {
    fn to_raw(&self) -> u32 {
        match self {
            &ServiceControl::Stop => winsvc::SERVICE_CONTROL_STOP,
        }
    }
}

/// Service state returned as a part of ServiceStatus
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ServiceState {
    Stopped,
    StartPending,
    StopPending,
    Running,
    ContinuePending,
    PausePending,
    Paused,
}

impl ServiceState {
    pub fn from_raw(raw_state: u32) -> Result<Self, RawConversionError> {
        match raw_state {
            winsvc::SERVICE_STOPPED => Ok(ServiceState::Stopped),
            winsvc::SERVICE_START_PENDING => Ok(ServiceState::StartPending),
            winsvc::SERVICE_STOP_PENDING => Ok(ServiceState::StopPending),
            winsvc::SERVICE_RUNNING => Ok(ServiceState::Running),
            winsvc::SERVICE_CONTINUE_PENDING => Ok(ServiceState::ContinuePending),
            winsvc::SERVICE_PAUSE_PENDING => Ok(ServiceState::PausePending),
            winsvc::SERVICE_PAUSED => Ok(ServiceState::Paused),
            _ => Err(RawConversionError),
        }
    }
}

/// Service status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ServiceStatus {
    /// Current state of the service
    pub current_state: ServiceState,
}

impl ServiceStatus {
    pub fn from_raw(raw_status: winsvc::SERVICE_STATUS) -> Result<Self, RawConversionError> {
        let current_state = ServiceState::from_raw(raw_status.dwCurrentState as u32)?;
        Ok(ServiceStatus { current_state })
    }
}

pub struct Service(pub winsvc::SC_HANDLE);

impl Service {
    pub fn stop(&self) -> Result<ServiceStatus, ServiceError> {
        self.send_control_command(ServiceControl::Stop)
    }

    pub fn query_status(&self) -> Result<ServiceStatus, ServiceError> {
        let mut raw_status = unsafe { mem::zeroed::<winsvc::SERVICE_STATUS>() };
        let success = unsafe { winsvc::QueryServiceStatus(self.0, &mut raw_status) };
        if success == 1 {
            ServiceStatus::from_raw(raw_status).map_err(|err| ServiceError::RawConversion(err))
        } else {
            Err(io::Error::last_os_error().into())
        }
    }

    pub fn delete(self) -> io::Result<()> {
        let success = unsafe { winsvc::DeleteService(self.0) };
        if success == 1 {
            Ok(())
        } else {
            Err(io::Error::last_os_error())
        }
    }

    fn send_control_command(&self, command: ServiceControl) -> Result<ServiceStatus, ServiceError> {
        let mut raw_status = unsafe { mem::zeroed::<winsvc::SERVICE_STATUS>() };
        let success = unsafe { winsvc::ControlService(self.0, command.to_raw(), &mut raw_status) };

        if success == 1 {
            ServiceStatus::from_raw(raw_status).map_err(|err| ServiceError::RawConversion(err))
        } else {
            Err(io::Error::last_os_error().into())
        }
    }
}

impl Drop for Service {
    fn drop(&mut self) {
        unsafe { winsvc::CloseServiceHandle(self.0) };
    }
}

use std::error;
use std::ffi::OsString;
use std::fmt;
use std::io;
use std::mem;

use winapi::um::{winnt, winsvc};

#[derive(Debug)]
pub enum ServiceError {
    InvalidServiceState(u32),
    System(io::Error),
}

impl error::Error for ServiceError {
    fn description(&self) -> &str {
        "Service error"
    }

    fn cause(&self) -> Option<&error::Error> {
        match *self {
            ServiceError::InvalidServiceState(_) => None,
            ServiceError::System(ref io_err) => Some(io_err),
        }
    }
}

impl fmt::Display for ServiceError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            ServiceError::InvalidServiceState(raw_value) => {
                write!(f, "Invalid service state value: {}", raw_value)
            }
            ServiceError::System(_) => write!(f, "System call error"),
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
#[repr(u32)]
pub enum ServiceType {
    /// Service that runs in its own process.
    OwnProcess = winnt::SERVICE_WIN32_OWN_PROCESS,
}

impl ServiceType {
    pub fn to_raw(&self) -> u32 {
        *self as u32
    }
}

/// Enum describing the access permissions when working with Services
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u32)]
pub enum ServiceAccess {
    QueryStatus = winsvc::SERVICE_QUERY_STATUS,
    Start = winsvc::SERVICE_START,
    Stop = winsvc::SERVICE_STOP,
    Delete = winnt::DELETE,
}

impl ServiceAccess {
    pub fn to_raw(&self) -> u32 {
        *self as u32
    }

    pub fn raw_mask(values: &[ServiceAccess]) -> u32 {
        values.iter().fold(0, |acc, &x| (acc | x.to_raw()))
    }
}

/// Enum describing the start options for windows services
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u32)]
pub enum ServiceStartType {
    /// Autostart on system startup
    AutoStart = winnt::SERVICE_AUTO_START,
    /// Service is enabled, can be started manually
    OnDemand = winnt::SERVICE_DEMAND_START,
    /// Disabled service
    Disabled = winnt::SERVICE_DISABLED,
}

impl ServiceStartType {
    pub fn to_raw(&self) -> u32 {
        *self as u32
    }
}

/// Error handling strategy for service failures.
/// See https://msdn.microsoft.com/en-us/library/windows/desktop/ms682450(v=vs.85).aspx
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u32)]
pub enum ServiceErrorControl {
    Critical = winnt::SERVICE_ERROR_CRITICAL,
    Ignore = winnt::SERVICE_ERROR_IGNORE,
    Normal = winnt::SERVICE_ERROR_NORMAL,
    Severe = winnt::SERVICE_ERROR_SEVERE,
}

impl ServiceErrorControl {
    pub fn to_raw(&self) -> u32 {
        *self as u32
    }
}

/// A struct that describes the service
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ServiceInfo {
    pub name: OsString,
    pub display_name: OsString,
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
    fn from_raw(raw_state: u32) -> Result<Self, ServiceError> {
        match raw_state {
            winsvc::SERVICE_STOPPED => Ok(ServiceState::Stopped),
            winsvc::SERVICE_START_PENDING => Ok(ServiceState::StartPending),
            winsvc::SERVICE_STOP_PENDING => Ok(ServiceState::StopPending),
            winsvc::SERVICE_RUNNING => Ok(ServiceState::Running),
            winsvc::SERVICE_CONTINUE_PENDING => Ok(ServiceState::ContinuePending),
            winsvc::SERVICE_PAUSE_PENDING => Ok(ServiceState::PausePending),
            winsvc::SERVICE_PAUSED => Ok(ServiceState::Paused),
            other => Err(ServiceError::InvalidServiceState(other)),
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
    fn from_raw(raw_status: winsvc::SERVICE_STATUS) -> Result<Self, ServiceError> {
        let current_state = ServiceState::from_raw(raw_status.dwCurrentState as u32)?;
        Ok(ServiceStatus { current_state })
    }
}

pub struct Service(pub(super) winsvc::SC_HANDLE);

impl Service {
    pub fn stop(&self) -> Result<ServiceStatus, ServiceError> {
        self.send_control_command(ServiceControl::Stop)
    }

    pub fn query_status(&self) -> Result<ServiceStatus, ServiceError> {
        let mut raw_status = unsafe { mem::zeroed::<winsvc::SERVICE_STATUS>() };
        let success = unsafe { winsvc::QueryServiceStatus(self.0, &mut raw_status) };
        if success == 1 {
            ServiceStatus::from_raw(raw_status)
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
            ServiceStatus::from_raw(raw_status).map_err(|err| err.into())
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

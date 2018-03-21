use std;
use std::io;
use std::ffi::OsString;

use winapi::um::winsvc;
use winapi::um::winnt;

use conversion::TryConvertFrom;
use errors::ConversionError;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ServiceType {
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
pub enum ServiceAccess {
    QueryStatus,
    Start,
    Stop,
    Delete,
}

impl From<ServiceAccess> for u32 {
    fn from(access: ServiceAccess) -> Self {
        match access {
            ServiceAccess::QueryStatus => winsvc::SERVICE_QUERY_STATUS,
            ServiceAccess::Start => winsvc::SERVICE_START,
            ServiceAccess::Stop => winsvc::SERVICE_STOP,
            ServiceAccess::Delete => winnt::DELETE,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ServiceAccessMask(Vec<ServiceAccess>);
impl ServiceAccessMask {
    pub fn new(set: &[ServiceAccess]) -> Self {
        ServiceAccessMask(set.to_vec())
    }
}

impl<'a> From<&'a ServiceAccessMask> for u32 {
    fn from(mask: &ServiceAccessMask) -> Self {
        mask.0.iter().fold(0, |acc, &x| (acc | u32::from(x)))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ServiceStartType {
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
pub enum ServiceErrorControl {
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

pub struct ServiceInfo {
    pub name: OsString, 
    pub display_name: OsString,
    pub service_access: ServiceAccessMask,
    pub service_type: ServiceType,
    pub start_type: ServiceStartType,
    pub error_control: ServiceErrorControl,
    pub executable_path: OsString,
    pub account_name: Option<OsString>, // use None to run as LocalSystem
    pub account_password: Option<OsString>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ServiceControl {
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
pub enum ServiceState {
    Stopped,
    StartPending,
    StopPending,
    Running,
    ContinuePending,
    PausePending,
    Paused,
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
pub struct ServiceStatus {
    pub current_state: ServiceState,
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

pub struct Service(pub winsvc::SC_HANDLE);
impl Service {
    pub fn stop(&self) -> Result<ServiceStatus, io::Error> {
        self.send_control_command(ServiceControl::Stop)
    }

    pub fn query_status(&self) -> Result<ServiceStatus, io::Error> {
        let mut raw_status = unsafe { std::mem::zeroed::<winsvc::SERVICE_STATUS>() };
        let success = unsafe { winsvc::QueryServiceStatus(self.0, &mut raw_status) };

        if success == 1 {
            // TBD: expected io::Error but got Conversion error
            Ok(ServiceStatus::try_convert_from(raw_status).unwrap())
        } else {
            Err(io::Error::last_os_error())
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
}

impl Drop for Service {
    fn drop(&mut self) {
        unsafe { winsvc::CloseServiceHandle(self.0) };
    }
}
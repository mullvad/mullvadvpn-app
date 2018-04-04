use std::ffi::OsString;
use std::path::PathBuf;
use std::time::Duration;
use std::{error, fmt, io, mem};

use winapi::shared::winerror::ERROR_SERVICE_SPECIFIC_ERROR;
use winapi::um::{winnt, winsvc};

#[derive(Debug)]
pub enum ServiceError {
    InvalidServiceType(u32),
    InvalidServiceState(u32),
    InvalidServiceControl(u32),
    System(io::Error),
}

impl error::Error for ServiceError {
    fn description(&self) -> &str {
        "Service error"
    }

    fn cause(&self) -> Option<&error::Error> {
        match *self {
            ServiceError::System(ref io_err) => Some(io_err),
            _ => None,
        }
    }
}

impl fmt::Display for ServiceError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            ServiceError::InvalidServiceType(raw_value) => {
                write!(f, "Invalid service type value: {}", raw_value)
            }
            ServiceError::InvalidServiceState(raw_value) => {
                write!(f, "Invalid service state value: {}", raw_value)
            }
            ServiceError::InvalidServiceControl(raw_value) => {
                write!(f, "Invalid service control value: {}", raw_value)
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
    pub fn from_raw(raw_value: u32) -> Result<Self, ServiceError> {
        let service_type = match raw_value {
            x if x == ServiceType::OwnProcess.to_raw() => ServiceType::OwnProcess,
            _ => Err(ServiceError::InvalidServiceType(raw_value))?,
        };
        Ok(service_type)
    }

    pub fn to_raw(&self) -> u32 {
        *self as u32
    }
}

/// Struct describing the access permissions when working with Services
#[derive(Builder, Debug)]
pub struct ServiceAccess {
    /// Can query the service status
    #[builder(default)]
    pub query_status: bool,

    /// Can start the service
    #[builder(default)]
    pub start: bool,

    // Can stop the service
    #[builder(default)]
    pub stop: bool,

    /// Can pause or continue the service execution
    #[builder(default)]
    pub pause_continue: bool,

    /// Can ask the service to report its status
    #[builder(default)]
    pub interrogate: bool,

    /// Can delete the service
    #[builder(default)]
    pub delete: bool,
}

impl ServiceAccess {
    pub fn to_raw(&self) -> u32 {
        let mut mask: u32 = 0;

        if self.query_status {
            mask |= winsvc::SERVICE_QUERY_STATUS;
        }

        if self.start {
            mask |= winsvc::SERVICE_START;
        }

        if self.stop {
            mask |= winsvc::SERVICE_STOP;
        }

        if self.pause_continue {
            mask |= winsvc::SERVICE_PAUSE_CONTINUE;
        }

        if self.interrogate {
            mask |= winsvc::SERVICE_INTERROGATE;
        }

        if self.delete {
            mask |= winnt::DELETE;
        }

        mask
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
    /// Service name
    pub name: OsString,

    /// Friendly service name
    pub display_name: OsString,

    pub service_type: ServiceType,
    pub start_type: ServiceStartType,
    pub error_control: ServiceErrorControl,

    /// Path to the service binary.
    pub executable_path: PathBuf,

    /// Launch arguments passed to `main` when system starts the service.
    /// This is not the same as arguments passed to `service_main`.
    pub launch_arguments: Vec<String>,

    /// Account to use for running the service.
    /// for example: NT Authority\System.
    /// use `None` to run as LocalSystem.
    pub account_name: Option<OsString>,

    /// Account password.
    /// For system accounts this should normally be `None`.
    pub account_password: Option<OsString>,
}

// Enum describing the service control operations
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u32)]
pub enum ServiceControl {
    Continue = winsvc::SERVICE_CONTROL_CONTINUE,
    Interrogate = winsvc::SERVICE_CONTROL_INTERROGATE,
    NetBindAdd = winsvc::SERVICE_CONTROL_NETBINDADD,
    NetBindDisable = winsvc::SERVICE_CONTROL_NETBINDDISABLE,
    NetBindEnable = winsvc::SERVICE_CONTROL_NETBINDENABLE,
    NetBindRemove = winsvc::SERVICE_CONTROL_NETBINDREMOVE,
    ParamChange = winsvc::SERVICE_CONTROL_PARAMCHANGE,
    Pause = winsvc::SERVICE_CONTROL_PAUSE,
    Preshutdown = winsvc::SERVICE_CONTROL_PRESHUTDOWN,
    Shutdown = winsvc::SERVICE_CONTROL_SHUTDOWN,
    Stop = winsvc::SERVICE_CONTROL_STOP,
}

impl ServiceControl {
    pub fn from_raw(raw_value: u32) -> Result<Self, ServiceError> {
        let service_control = match raw_value {
            x if x == ServiceControl::Continue.to_raw() => ServiceControl::Continue,
            x if x == ServiceControl::Interrogate.to_raw() => ServiceControl::Interrogate,
            x if x == ServiceControl::NetBindAdd.to_raw() => ServiceControl::NetBindAdd,
            x if x == ServiceControl::NetBindDisable.to_raw() => ServiceControl::NetBindDisable,
            x if x == ServiceControl::NetBindEnable.to_raw() => ServiceControl::NetBindEnable,
            x if x == ServiceControl::NetBindRemove.to_raw() => ServiceControl::NetBindRemove,
            x if x == ServiceControl::ParamChange.to_raw() => ServiceControl::ParamChange,
            x if x == ServiceControl::Pause.to_raw() => ServiceControl::Pause,
            x if x == ServiceControl::Preshutdown.to_raw() => ServiceControl::Preshutdown,
            x if x == ServiceControl::Shutdown.to_raw() => ServiceControl::Shutdown,
            x if x == ServiceControl::Stop.to_raw() => ServiceControl::Stop,
            other => Err(ServiceError::InvalidServiceControl(other))?,
        };
        Ok(service_control)
    }

    pub fn to_raw(&self) -> u32 {
        *self as u32
    }
}

/// Service state returned as a part of ServiceStatus
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u32)]
pub enum ServiceState {
    Stopped = winsvc::SERVICE_STOPPED,
    StartPending = winsvc::SERVICE_START_PENDING,
    StopPending = winsvc::SERVICE_STOP_PENDING,
    Running = winsvc::SERVICE_RUNNING,
    ContinuePending = winsvc::SERVICE_CONTINUE_PENDING,
    PausePending = winsvc::SERVICE_PAUSE_PENDING,
    Paused = winsvc::SERVICE_PAUSED,
}

impl ServiceState {
    fn from_raw(raw_state: u32) -> Result<Self, ServiceError> {
        let service_state = match raw_state {
            x if x == ServiceState::Stopped.to_raw() => ServiceState::Stopped,
            x if x == ServiceState::StartPending.to_raw() => ServiceState::StartPending,
            x if x == ServiceState::StopPending.to_raw() => ServiceState::StopPending,
            x if x == ServiceState::Running.to_raw() => ServiceState::Running,
            x if x == ServiceState::ContinuePending.to_raw() => ServiceState::ContinuePending,
            x if x == ServiceState::PausePending.to_raw() => ServiceState::PausePending,
            x if x == ServiceState::Paused.to_raw() => ServiceState::Paused,
            other => Err(ServiceError::InvalidServiceState(other))?,
        };
        Ok(service_state)
    }

    fn to_raw(&self) -> u32 {
        *self as u32
    }
}

/// Service exit code abstraction.
///
/// This struct provides a logic around the relationship between `win32_exit_code` and
/// `service_specific_exit_code`.
///
/// The service can either return a win32 error code or a custom error
/// code. In that case `win32_exit_code` has to be set to `ERROR_SERVICE_SPECIFIC_ERROR` and
/// the `service_specific_exit_code` assigned with custom error code.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ServiceExitCode {
    Win32(u32),
    ServiceSpecific(u32),
}

impl ServiceExitCode {
    fn copy_to(&self, raw_service_status: &mut winsvc::SERVICE_STATUS) {
        match *self {
            ServiceExitCode::Win32(win32_error_code) => {
                raw_service_status.dwWin32ExitCode = win32_error_code;
                raw_service_status.dwServiceSpecificExitCode = 0;
            }
            ServiceExitCode::ServiceSpecific(service_error_code) => {
                raw_service_status.dwWin32ExitCode = ERROR_SERVICE_SPECIFIC_ERROR;
                raw_service_status.dwServiceSpecificExitCode = service_error_code;
            }
        }
    }

    fn from_raw_service_status(raw_service_status: &winsvc::SERVICE_STATUS) -> Self {
        if raw_service_status.dwWin32ExitCode == ERROR_SERVICE_SPECIFIC_ERROR {
            ServiceExitCode::ServiceSpecific(raw_service_status.dwServiceSpecificExitCode)
        } else {
            ServiceExitCode::Win32(raw_service_status.dwWin32ExitCode)
        }
    }
}

/// Accepted types of service control requests
#[derive(Builder, Debug, Clone)]
#[builder(build_fn(validate = "Self::validate"))]
pub struct ServiceControlAccept {
    /// The service is a network component that can accept changes in its binding without being
    /// stopped and restarted. This allows service to receive `ServiceControl::Netbind*`
    /// family of events.
    #[builder(default)]
    pub netbind_change: bool,

    /// The service can reread its startup parameters without being stopped and restarted.
    #[builder(default)]
    pub param_change: bool,

    /// The service can be paused and continued.
    #[builder(default)]
    pub pause_continue: bool,

    /// The service can perform preshutdown tasks.
    /// Mutually exclusive with shutdown.
    #[builder(default)]
    pub preshutdown: bool,

    /// The service is notified when system shutdown occurs.
    /// Mutually exclusive with preshutdown.
    #[builder(default)]
    pub shutdown: bool,

    /// The service can be stopped.
    #[builder(default)]
    pub stop: bool,
}

impl ServiceControlAcceptBuilder {
    fn validate(&self) -> Result<(), String> {
        // Services that register for preshutdown notifications cannot receive shutdown
        // notification because they have already stopped.
        match (self.preshutdown, self.shutdown) {
            (Some(true), Some(true)) => {
                Err("Preshutdown and shutdown are mutually exclusive.".to_string())
            }
            _ => Ok(()),
        }
    }
}

impl ServiceControlAccept {
    pub(super) fn from_raw(raw_mask: u32) -> Self {
        ServiceControlAccept {
            netbind_change: (raw_mask & winsvc::SERVICE_ACCEPT_NETBINDCHANGE) != 0,
            param_change: (raw_mask & winsvc::SERVICE_ACCEPT_PARAMCHANGE) != 0,
            pause_continue: (raw_mask & winsvc::SERVICE_ACCEPT_PAUSE_CONTINUE) != 0,
            preshutdown: (raw_mask & winsvc::SERVICE_ACCEPT_PRESHUTDOWN) != 0,
            shutdown: (raw_mask & winsvc::SERVICE_ACCEPT_SHUTDOWN) != 0,
            stop: (raw_mask & winsvc::SERVICE_ACCEPT_STOP) != 0,
        }
    }

    pub(super) fn to_raw(&self) -> u32 {
        let mut mask: u32 = 0;

        if self.netbind_change {
            mask |= winsvc::SERVICE_ACCEPT_NETBINDCHANGE;
        }

        if self.param_change {
            mask |= winsvc::SERVICE_ACCEPT_PARAMCHANGE;
        }

        if self.pause_continue {
            mask |= winsvc::SERVICE_ACCEPT_PAUSE_CONTINUE;
        }

        if self.preshutdown {
            mask |= winsvc::SERVICE_ACCEPT_PRESHUTDOWN;
        }

        if self.shutdown {
            mask |= winsvc::SERVICE_ACCEPT_SHUTDOWN;
        }

        if self.stop {
            mask |= winsvc::SERVICE_ACCEPT_STOP;
        }

        mask
    }
}

/// Service status
#[derive(Builder, Debug)]
#[builder(build_fn(validate = "Self::validate"))]
pub struct ServiceStatus {
    /// Type of service
    pub service_type: ServiceType,

    /// Current state of the service
    pub current_state: ServiceState,

    /// Control commands that service accepts.
    pub controls_accepted: ServiceControlAccept,

    /// Service exit code
    pub exit_code: ServiceExitCode,

    /// Service initialization progress value that should be increased during a lengthy start,
    /// stop, pause or continue eration. For example the service should increment the value as
    /// it completes each step of initialization.
    /// This value must be zero if the service does not have any pending start, stop, pause or
    /// continue operations.
    pub checkpoint: u32,

    /// Estimated time for pending operation.
    /// This basically works as a timeout until the service manager assumes that the service hung.
    /// This could be either circumvented by updating the `current_state` or incrementing a
    /// `checkpoint` value.
    pub wait_hint: Duration,
}

impl ServiceStatusBuilder {
    fn validate(&self) -> Result<(), String> {
        match (self.current_state, self.checkpoint) {
            (Some(current_state), Some(checkpoint)) => {
                let is_pending_operation = match current_state {
                    ServiceState::StartPending
                    | ServiceState::StopPending
                    | ServiceState::PausePending
                    | ServiceState::ContinuePending => true,
                    _ => false,
                };

                if !is_pending_operation && checkpoint != 0 {
                    Err("Checkpoint can only be used for pending start, stop, pause or continue operations.".to_string())
                } else {
                    Ok(())
                }
            }

            _ => Ok(()),
        }
    }
}

impl ServiceStatus {
    pub(super) fn to_raw(&self) -> winsvc::SERVICE_STATUS {
        let mut raw_status = unsafe { mem::zeroed::<winsvc::SERVICE_STATUS>() };
        raw_status.dwServiceType = self.service_type.to_raw();
        raw_status.dwCurrentState = self.current_state.to_raw();
        raw_status.dwControlsAccepted = self.controls_accepted.to_raw();

        self.exit_code.copy_to(&mut raw_status);

        raw_status.dwCheckPoint = self.checkpoint;

        // we lose precision here but dwWaitHint should never be too big.
        raw_status.dwWaitHint = (self.wait_hint.as_secs() * 1000) as u32;

        raw_status
    }

    fn from_raw(raw_status: winsvc::SERVICE_STATUS) -> Result<Self, ServiceError> {
        Ok(ServiceStatus {
            service_type: ServiceType::from_raw(raw_status.dwServiceType)?,
            current_state: ServiceState::from_raw(raw_status.dwCurrentState)?,
            controls_accepted: ServiceControlAccept::from_raw(raw_status.dwControlsAccepted),
            exit_code: ServiceExitCode::from_raw_service_status(&raw_status),
            checkpoint: raw_status.dwCheckPoint,
            wait_hint: Duration::from_millis(raw_status.dwWaitHint as u64),
        })
    }
}


pub struct Service(winsvc::SC_HANDLE);

impl Service {
    /// Internal constructor
    pub(super) unsafe fn from_handle(handle: winsvc::SC_HANDLE) -> Self {
        Service(handle)
    }

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

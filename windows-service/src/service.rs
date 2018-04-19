use std::ffi::OsString;
use std::path::PathBuf;
use std::time::Duration;
use std::{io, mem};

use winapi::shared::winerror::{ERROR_SERVICE_SPECIFIC_ERROR, NO_ERROR};
use winapi::um::{winnt, winsvc};

mod errors {
    error_chain! {
        errors {
            InvalidServiceType(raw_value: u32) {
                description("Invalid service type value")
                display("Invalid service type value: {}", raw_value)
            }
            InvalidServiceState(raw_value: u32) {
                description("Invalid service state")
                display("Invalid service state value: {}", raw_value)
            }
            InvalidServiceControl(raw_value: u32) {
                description("Invalid service control")
                display("Invalid service control value: {}", raw_value)
            }
        }
        foreign_links {
            System(::std::io::Error);
        }
    }
}
pub use self::errors::*;

/// Enum describing types of windows services
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u32)]
pub enum ServiceType {
    /// Service that runs in its own process.
    OwnProcess = winnt::SERVICE_WIN32_OWN_PROCESS,
}

impl ServiceType {
    pub fn from_raw(raw_value: u32) -> Result<Self> {
        let service_type = match raw_value {
            x if x == ServiceType::OwnProcess.to_raw() => ServiceType::OwnProcess,
            _ => Err(ErrorKind::InvalidServiceType(raw_value))?,
        };
        Ok(service_type)
    }

    pub fn to_raw(&self) -> u32 {
        *self as u32
    }
}

/// Flags describing the access permissions when working with services
bitflags! {
    pub struct ServiceAccess: u32 {
        /// Can query the service status
        const QUERY_STATUS = winsvc::SERVICE_QUERY_STATUS;

        /// Can start the service
        const START = winsvc::SERVICE_START;

        // Can stop the service
        const STOP = winsvc::SERVICE_STOP;

        /// Can pause or continue the service execution
        const PAUSE_CONTINUE = winsvc::SERVICE_PAUSE_CONTINUE;

        /// Can ask the service to report its status
        const INTERROGATE = winsvc::SERVICE_INTERROGATE;

        /// Can delete the service
        const DELETE = winnt::DELETE;
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
    pub launch_arguments: Vec<OsString>,

    /// Account to use for running the service.
    /// for example: NT Authority\System.
    /// use `None` to run as LocalSystem.
    pub account_name: Option<OsString>,

    /// Account password.
    /// For system accounts this should normally be `None`.
    pub account_password: Option<OsString>,
}

/// Enum describing the service control operations
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
    pub fn from_raw(raw_value: u32) -> Result<Self> {
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
            other => Err(ErrorKind::InvalidServiceControl(other))?,
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
    fn from_raw(raw_state: u32) -> Result<Self> {
        let service_state = match raw_state {
            x if x == ServiceState::Stopped.to_raw() => ServiceState::Stopped,
            x if x == ServiceState::StartPending.to_raw() => ServiceState::StartPending,
            x if x == ServiceState::StopPending.to_raw() => ServiceState::StopPending,
            x if x == ServiceState::Running.to_raw() => ServiceState::Running,
            x if x == ServiceState::ContinuePending.to_raw() => ServiceState::ContinuePending,
            x if x == ServiceState::PausePending.to_raw() => ServiceState::PausePending,
            x if x == ServiceState::Paused.to_raw() => ServiceState::Paused,
            other => Err(ErrorKind::InvalidServiceState(other))?,
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
}

impl Default for ServiceExitCode {
    fn default() -> Self {
        ServiceExitCode::Win32(NO_ERROR)
    }
}

impl<'a> From<&'a winsvc::SERVICE_STATUS> for ServiceExitCode {
    fn from(service_status: &'a winsvc::SERVICE_STATUS) -> Self {
        if service_status.dwWin32ExitCode == ERROR_SERVICE_SPECIFIC_ERROR {
            ServiceExitCode::ServiceSpecific(service_status.dwServiceSpecificExitCode)
        } else {
            ServiceExitCode::Win32(service_status.dwWin32ExitCode)
        }
    }
}

/// Flags describing accepted types of service control requests
bitflags! {
    pub struct ServiceControlAccept: u32 {
        /// The service is a network component that can accept changes in its binding without being
        /// stopped and restarted. This allows service to receive `ServiceControl::Netbind*`
        /// family of events.
        const NETBIND_CHANGE = winsvc::SERVICE_ACCEPT_NETBINDCHANGE;

        /// The service can reread its startup parameters without being stopped and restarted.
        const PARAM_CHANGE = winsvc::SERVICE_ACCEPT_PARAMCHANGE;

        /// The service can be paused and continued.
        const PAUSE_CONTINUE = winsvc::SERVICE_ACCEPT_PAUSE_CONTINUE;

        /// The service can perform preshutdown tasks.
        /// Mutually exclusive with shutdown.
        const PRESHUTDOWN = winsvc::SERVICE_ACCEPT_PRESHUTDOWN;

        /// The service is notified when system shutdown occurs.
        /// Mutually exclusive with preshutdown.
        const SHUTDOWN = winsvc::SERVICE_ACCEPT_SHUTDOWN;

        /// The service can be stopped.
        const STOP = winsvc::SERVICE_ACCEPT_STOP;
    }
}

/// Service status.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
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

impl ServiceStatus {
    pub(super) fn to_raw(&self) -> winsvc::SERVICE_STATUS {
        let mut raw_status = unsafe { mem::zeroed::<winsvc::SERVICE_STATUS>() };
        raw_status.dwServiceType = self.service_type.to_raw();
        raw_status.dwCurrentState = self.current_state.to_raw();
        raw_status.dwControlsAccepted = self.controls_accepted.bits();

        self.exit_code.copy_to(&mut raw_status);

        raw_status.dwCheckPoint = self.checkpoint;

        // we lose precision here but dwWaitHint should never be too big.
        raw_status.dwWaitHint = (self.wait_hint.as_secs() * 1000) as u32;

        raw_status
    }

    fn from_raw(raw_status: winsvc::SERVICE_STATUS) -> Result<Self> {
        Ok(ServiceStatus {
            service_type: ServiceType::from_raw(raw_status.dwServiceType)?,
            current_state: ServiceState::from_raw(raw_status.dwCurrentState)?,
            controls_accepted: ServiceControlAccept::from_bits_truncate(
                raw_status.dwControlsAccepted,
            ),
            exit_code: ServiceExitCode::from(&raw_status),
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

    pub fn stop(&self) -> Result<ServiceStatus> {
        self.send_control_command(ServiceControl::Stop)
    }

    pub fn query_status(&self) -> Result<ServiceStatus> {
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

    fn send_control_command(&self, command: ServiceControl) -> Result<ServiceStatus> {
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

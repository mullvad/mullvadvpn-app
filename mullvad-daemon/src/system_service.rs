use crate::cli;
use libc::c_void;
use mullvad_daemon::{runtime::new_runtime_builder, DaemonShutdownHandle};
use std::{
    env,
    ffi::OsString,
    mem, ptr, slice,
    sync::{
        atomic::{AtomicBool, AtomicUsize, Ordering},
        mpsc, Arc,
    },
    thread,
    time::{Duration, Instant},
};
use talpid_types::ErrorExt;
use windows_service::{
    service::{
        PowerEventParam, Service, ServiceAccess, ServiceAction, ServiceActionType, ServiceControl,
        ServiceControlAccept, ServiceDependency, ServiceErrorControl, ServiceExitCode,
        ServiceFailureActions, ServiceFailureResetPeriod, ServiceInfo, ServiceSidType,
        ServiceStartType, ServiceState, ServiceStatus, ServiceType, SessionChangeReason,
    },
    service_control_handler::{self, ServiceControlHandlerResult, ServiceStatusHandle},
    service_dispatcher,
    service_manager::{ServiceManager, ServiceManagerAccess},
};
use windows_sys::Win32::{
    Foundation::{LUID, STATUS_SUCCESS},
    Security::Authentication::Identity::{
        LsaEnumerateLogonSessions, LsaFreeReturnBuffer, LsaGetLogonSessionData,
        SECURITY_LOGON_SESSION_DATA,
    },
};

static SERVICE_NAME: &'static str = "MullvadVPN";
static SERVICE_DISPLAY_NAME: &'static str = "Mullvad VPN Service";
static SERVICE_TYPE: ServiceType = ServiceType::OWN_PROCESS;

const SERVICE_RECOVERY_LAST_RESTART_DELAY: Duration = Duration::from_secs(60 * 10);
const SERVICE_FAILURE_RESET_PERIOD: Duration = Duration::from_secs(60 * 15);

lazy_static::lazy_static! {
    static ref SERVICE_ACCESS: ServiceAccess = ServiceAccess::QUERY_CONFIG
    | ServiceAccess::CHANGE_CONFIG
    | ServiceAccess::START
    | ServiceAccess::DELETE;
}

pub fn run() -> Result<(), String> {
    // Start the service dispatcher.
    // This will block current thread until the service stopped and spawn `service_main` on a
    // background thread.
    service_dispatcher::start(SERVICE_NAME, service_main)
        .map_err(|e| e.display_chain_with_msg("Failed to start a service dispatcher"))
}

windows_service::define_windows_service!(service_main, handle_service_main);

pub fn handle_service_main(_arguments: Vec<OsString>) {
    log::info!("Service started.");

    let (event_tx, event_rx) = mpsc::channel();

    // Register service event handler
    let event_handler = move |control_event| -> ServiceControlHandlerResult {
        match control_event {
            // Notifies a service to report its current status information to the service
            // control manager. Always return NO_ERROR even if not implemented.
            ServiceControl::Interrogate => ServiceControlHandlerResult::NoError,

            ServiceControl::Stop
            | ServiceControl::Preshutdown
            | ServiceControl::PowerEvent(_)
            | ServiceControl::SessionChange(_) => {
                event_tx.send(control_event).unwrap();
                ServiceControlHandlerResult::NoError
            }

            _ => ServiceControlHandlerResult::NotImplemented,
        }
    };
    let status_handle = match service_control_handler::register(SERVICE_NAME, event_handler) {
        Ok(handle) => handle,
        Err(error) => {
            log::error!(
                "{}",
                error.display_chain_with_msg("Failed to register a service control handler")
            );
            return;
        }
    };
    let mut persistent_service_status = PersistentServiceStatus::new(status_handle);
    persistent_service_status
        .set_pending_start(Duration::from_secs(1))
        .unwrap();

    let should_restart = Arc::new(AtomicBool::new(true));

    let log_dir = crate::get_log_dir(cli::get_config()).expect("Log dir should be available here");

    let runtime = new_runtime_builder().build();
    let runtime = match runtime {
        Err(error) => {
            log::error!("{}", error.display_chain());
            persistent_service_status
                .set_stopped(ServiceExitCode::ServiceSpecific(1))
                .unwrap();
            return;
        }
        Ok(runtime) => runtime,
    };

    let result = runtime.block_on(crate::create_daemon(log_dir));
    let result = if let Ok(daemon) = result {
        let shutdown_handle = daemon.shutdown_handle();

        // Register monitor that translates `ServiceControl` to Daemon events
        start_event_monitor(
            persistent_service_status.clone(),
            shutdown_handle,
            event_rx,
            should_restart.clone(),
        );

        persistent_service_status.set_running().unwrap();

        runtime
            .block_on(daemon.run())
            .map_err(|e| e.display_chain())
    } else {
        result.map(|_| ())
    };

    let exit_code = match result {
        Ok(()) => {
            log::info!("Stopping service");
            // check if shutdown signal was sent from the system
            if !should_restart.load(Ordering::Acquire) {
                ServiceExitCode::default()
            } else {
                // otherwise return a non-zero code so that the daemon gets restarted
                ServiceExitCode::ServiceSpecific(1)
            }
        }
        Err(error) => {
            log::error!("{}", error);
            ServiceExitCode::ServiceSpecific(1)
        }
    };

    persistent_service_status.set_stopped(exit_code).unwrap();
}

/// Start event monitor thread that polls for `ServiceControl` and translates them into calls to
/// Daemon.
fn start_event_monitor(
    persistent_service_status: PersistentServiceStatus,
    shutdown_handle: DaemonShutdownHandle,
    event_rx: mpsc::Receiver<ServiceControl>,
    should_restart: Arc<AtomicBool>,
) -> thread::JoinHandle<()> {
    thread::spawn(move || {
        let mut shutdown_handle = ServiceShutdownHandle {
            persistent_service_status,
            shutdown_handle,
            should_restart,
        };
        let mut hibernation_detector = HibernationDetector::new(shutdown_handle.clone());
        for event in event_rx {
            match event {
                ServiceControl::Stop | ServiceControl::Preshutdown => {
                    // If the daemon is closing due to the system shutting down,
                    // keep blocking traffic after the daemon exits.
                    shutdown_handle.shutdown(false, event == ServiceControl::Preshutdown);
                }
                ServiceControl::PowerEvent(details) => match details {
                    PowerEventParam::Suspend => {
                        hibernation_detector.register_suspend();
                    }
                    PowerEventParam::ResumeAutomatic | PowerEventParam::ResumeSuspend => {
                        hibernation_detector.register_resume();
                    }
                    _ => (),
                },
                ServiceControl::SessionChange(details) => {
                    if details.reason == SessionChangeReason::SessionLogoff {
                        hibernation_detector.register_logoff(details.notification.session_id);
                    }
                }
                _ => (),
            }
        }
    })
}

#[derive(Clone)]
struct ServiceShutdownHandle {
    persistent_service_status: PersistentServiceStatus,
    shutdown_handle: DaemonShutdownHandle,
    /// If true, the service will be restarted by the SCM when
    /// the daemon has exited.
    should_restart: Arc<AtomicBool>,
}

impl ServiceShutdownHandle {
    fn shutdown(&mut self, should_restart: bool, is_system_shutdown: bool) {
        self.persistent_service_status
            .set_pending_stop(Duration::from_secs(10))
            .unwrap();

        self.should_restart.store(should_restart, Ordering::Release);
        self.shutdown_handle.shutdown(!is_system_shutdown);
    }
}

/// Service status helper with persistent checkpoint counter.
#[derive(Debug, Clone)]
struct PersistentServiceStatus {
    status_handle: ServiceStatusHandle,
    checkpoint_counter: Arc<AtomicUsize>,
}

impl PersistentServiceStatus {
    fn new(status_handle: ServiceStatusHandle) -> Self {
        PersistentServiceStatus {
            status_handle,
            checkpoint_counter: Arc::new(AtomicUsize::new(1)),
        }
    }

    /// Tell the system that the service is pending start and provide the time estimate until
    /// initialization is complete.
    fn set_pending_start(&mut self, wait_hint: Duration) -> windows_service::Result<()> {
        self.report_status(
            ServiceState::StartPending,
            wait_hint,
            ServiceExitCode::default(),
        )
    }

    /// Tell the system that the service is running.
    fn set_running(&mut self) -> windows_service::Result<()> {
        self.report_status(
            ServiceState::Running,
            Duration::default(),
            ServiceExitCode::default(),
        )
    }

    /// Tell the system that the service is pending stop and provide the time estimate until the
    /// service is stopped.
    fn set_pending_stop(&mut self, wait_hint: Duration) -> windows_service::Result<()> {
        self.report_status(
            ServiceState::StopPending,
            wait_hint,
            ServiceExitCode::default(),
        )
    }

    /// Tell the system that the service is stopped and provide the exit code.
    fn set_stopped(&mut self, exit_code: ServiceExitCode) -> windows_service::Result<()> {
        self.report_status(ServiceState::Stopped, Duration::default(), exit_code)
    }

    /// Private helper to report the service status update.
    fn report_status(
        &mut self,
        next_state: ServiceState,
        wait_hint: Duration,
        exit_code: ServiceExitCode,
    ) -> windows_service::Result<()> {
        // Automatically bump the checkpoint when updating the pending events to tell the system
        // that the service is making a progress in transition from pending to final state.
        // `wait_hint` should reflect the estimated time for transition to complete.
        let checkpoint = match next_state {
            ServiceState::StartPending
            | ServiceState::StopPending
            | ServiceState::ContinuePending
            | ServiceState::PausePending => self.checkpoint_counter.fetch_add(1, Ordering::SeqCst),
            _ => 0,
        };

        let service_status = ServiceStatus {
            service_type: SERVICE_TYPE,
            current_state: next_state,
            controls_accepted: accepted_controls_by_state(next_state),
            exit_code,
            checkpoint: checkpoint as u32,
            wait_hint,
            process_id: None,
        };

        log::debug!(
            "Update service status: {:?}, checkpoint: {}, wait_hint: {:?}",
            service_status.current_state,
            service_status.checkpoint,
            service_status.wait_hint
        );

        self.status_handle.set_service_status(service_status)
    }
}

/// Returns the list of accepted service events at each stage of the service lifecycle.
fn accepted_controls_by_state(state: ServiceState) -> ServiceControlAccept {
    let always_accepted = ServiceControlAccept::POWER_EVENT | ServiceControlAccept::SESSION_CHANGE;
    match state {
        ServiceState::StartPending | ServiceState::PausePending | ServiceState::ContinuePending => {
            ServiceControlAccept::empty()
        }
        ServiceState::Running => {
            always_accepted | ServiceControlAccept::STOP | ServiceControlAccept::PRESHUTDOWN
        }
        ServiceState::Paused => {
            always_accepted | ServiceControlAccept::STOP | ServiceControlAccept::PRESHUTDOWN
        }
        ServiceState::StopPending | ServiceState::Stopped => ServiceControlAccept::empty(),
    }
}

#[derive(err_derive::Error, Debug)]
#[error(no_from)]
pub enum InstallError {
    #[error(display = "Unable to connect to service manager")]
    ConnectServiceManager(#[error(source)] windows_service::Error),

    #[error(display = "Unable to create a service")]
    CreateService(#[error(source)] windows_service::Error),
}

pub fn install_service() -> Result<(), InstallError> {
    let manager_access = ServiceManagerAccess::CONNECT | ServiceManagerAccess::CREATE_SERVICE;
    let service_manager = ServiceManager::local_computer(None::<&str>, manager_access)
        .map_err(InstallError::ConnectServiceManager)?;

    let service = service_manager
        .create_service(&get_service_info(), *SERVICE_ACCESS)
        .or(open_update_service(&service_manager))
        .map_err(InstallError::CreateService)?;

    let recovery_actions = vec![
        ServiceAction {
            action_type: ServiceActionType::Restart,
            delay: Duration::from_secs(3),
        },
        ServiceAction {
            action_type: ServiceActionType::Restart,
            delay: Duration::from_secs(30),
        },
        ServiceAction {
            action_type: ServiceActionType::Restart,
            delay: SERVICE_RECOVERY_LAST_RESTART_DELAY,
        },
    ];

    let failure_actions = ServiceFailureActions {
        reset_period: ServiceFailureResetPeriod::After(SERVICE_FAILURE_RESET_PERIOD),
        reboot_msg: None,
        command: None,
        actions: Some(recovery_actions),
    };

    service
        .update_failure_actions(failure_actions)
        .map_err(InstallError::CreateService)?;
    service
        .set_failure_actions_on_non_crash_failures(true)
        .map_err(InstallError::CreateService)?;

    // Change how the service SID is added to the service process token.
    // WireGuard needs this.
    service
        .set_config_service_sid_info(ServiceSidType::Unrestricted)
        .map_err(InstallError::CreateService)?;

    Ok(())
}

fn open_update_service(
    service_manager: &ServiceManager,
) -> Result<Service, windows_service::Error> {
    let service = service_manager.open_service(SERVICE_NAME, *SERVICE_ACCESS)?;
    service.change_config(&get_service_info())?;
    Ok(service)
}

fn get_service_info() -> ServiceInfo {
    ServiceInfo {
        name: OsString::from(SERVICE_NAME),
        display_name: OsString::from(SERVICE_DISPLAY_NAME),
        service_type: SERVICE_TYPE,
        start_type: ServiceStartType::AutoStart,
        error_control: ServiceErrorControl::Normal,
        executable_path: env::current_exe().unwrap(),
        launch_arguments: vec![OsString::from("--run-as-service"), OsString::from("-v")],
        dependencies: vec![
            // Base Filter Engine
            ServiceDependency::Service(OsString::from("BFE")),
            // Network Store Interface Service
            // This service delivers network notifications (e.g. interface addition/deleting etc).
            ServiceDependency::Service(OsString::from("NSI")),
        ],
        account_name: None, // run as System
        account_password: None,
    }
}

/// Used to track events that taken together would mean the machine is heading towards being
/// hibernated. Typically, the user's session if first terminated. Moments later we should receive a
/// suspension event corresponding to the hibernation of session 0 (kernel and services).
struct HibernationDetector {
    logoff_time: Option<Instant>,
    should_restart: bool,
    shutdown_handle: ServiceShutdownHandle,
}

const SECURITY_LOGON_TYPE_INTERACTIVE: u32 = 2;

impl HibernationDetector {
    fn new(shutdown_handle: ServiceShutdownHandle) -> Self {
        Self {
            logoff_time: None,
            should_restart: false,
            shutdown_handle,
        }
    }

    /// Register a session logoff.
    /// The logoff event is discarded unless the session was/is interactive.
    fn register_logoff(&mut self, session_id: u32) {
        if Self::is_interactive_session(session_id) {
            self.logoff_time = Some(Instant::now());
        }
    }

    fn is_interactive_session(session_id: u32) -> bool {
        let mut logon_session_count = 0u32;
        let mut logon_session_list: *mut LUID = ptr::null_mut();
        let status =
            unsafe { LsaEnumerateLogonSessions(&mut logon_session_count, &mut logon_session_list) };
        if status != STATUS_SUCCESS {
            log::warn!("LsaEnumerateLogonSessions() failed, error code: {}", status);
            return false;
        }
        // SAFETY: `logon_session_list` is not mutated before being freed.
        let logons =
            unsafe { slice::from_raw_parts(logon_session_list, logon_session_count as usize) };
        let mut interactive = false;
        for logon in logons {
            let mut session_data: *mut SECURITY_LOGON_SESSION_DATA = ptr::null_mut();
            // SAFETY: `LsaGetLogonSessionData` does not mutate `logon`
            let status = unsafe { LsaGetLogonSessionData(logon, &mut session_data) };
            if status != STATUS_SUCCESS {
                log::warn!("LsaGetLogonSessionData() failed, error code: {}", status);
                continue;
            }
            let candidate_correct_session = unsafe { *session_data }.Session == session_id;
            let candidate_interactive =
                unsafe { *session_data }.LogonType == SECURITY_LOGON_TYPE_INTERACTIVE;
            unsafe { LsaFreeReturnBuffer(session_data as *mut c_void) };
            if candidate_correct_session {
                interactive = candidate_interactive;
                break;
            }
        }
        mem::drop(logons);
        unsafe { LsaFreeReturnBuffer(logon_session_list as *mut c_void) };
        interactive
    }

    /// Register a machine suspend event.
    fn register_suspend(&mut self) {
        if let Some(logoff_time) = &self.logoff_time {
            if logoff_time.elapsed() < Duration::from_secs(5) {
                log::info!("Pending hibernation detected");
                self.should_restart = true;
            }
        }
    }

    /// Register a machine resume event.
    /// This will restart the service if we are coming back from hibernation.
    fn register_resume(&mut self) {
        if self.should_restart {
            self.should_restart = false;
            log::info!("System is being restored from hibernation. Restarting daemon service");

            // Perform a non-clean shutdown. This will cause the daemon to restart itself.
            self.shutdown_handle.shutdown(true, true);
        }
    }
}

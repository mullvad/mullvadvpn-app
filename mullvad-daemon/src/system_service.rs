#![cfg(windows)]

use std::ffi::OsString;
use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{mpsc, Arc};
use std::time::Duration;
use std::{env, io, thread};

use cli;
use error_chain::ChainedError;
use windows_service::service::{
    ServiceAccess, ServiceControl, ServiceControlAccept, ServiceErrorControl, ServiceExitCode,
    ServiceInfo, ServiceStartType, ServiceState, ServiceStatus, ServiceType,
};
use windows_service::service_control_handler::{
    self, ServiceControlHandlerResult, ServiceStatusHandle,
};
use windows_service::service_dispatcher;
use windows_service::service_manager::{ServiceManager, ServiceManagerAccess};

use super::{get_resource_dir, Daemon, DaemonShutdownHandle, Result, ResultExt};

static SERVICE_NAME: &'static str = "MullvadVPN";
static SERVICE_DISPLAY_NAME: &'static str = "Mullvad VPN Service";
static SERVICE_TYPE: ServiceType = ServiceType::OwnProcess;

pub fn run() -> Result<()> {
    // Start the service dispatcher.
    // This will block current thread until the service stopped and spawn `service_main` on a
    // background thread.
    service_dispatcher::start_dispatcher(SERVICE_NAME, service_main)
        .chain_err(|| "Failed to start a service dispatcher")
}

define_windows_service!(service_main, handle_service_main);

pub fn handle_service_main(arguments: Vec<OsString>) {
    info!("Service started.");
    match run_service(arguments) {
        Ok(_) => info!("Service stopped."),
        Err(ref e) => error!("Service stopped with error: {}", e.display_chain()),
    };
}

fn run_service(_arguments: Vec<OsString>) -> Result<()> {
    let config = cli::get_config();
    let (event_tx, event_rx) = mpsc::channel();

    // Register service event handler
    let event_handler = move |control_event| -> ServiceControlHandlerResult {
        match control_event {
            // Notifies a service to report its current status information to the service
            // control manager. Always return NO_ERROR even if not implemented.
            ServiceControl::Interrogate => ServiceControlHandlerResult::NoError,

            ServiceControl::Shutdown | ServiceControl::Stop => {
                event_tx.send(control_event).unwrap();
                ServiceControlHandlerResult::NoError
            }

            _ => ServiceControlHandlerResult::NotImplemented,
        }
    };
    let status_handle =
        service_control_handler::register_control_handler(SERVICE_NAME, event_handler)
            .chain_err(|| "Failed to register a service control handler")?;
    let mut persistent_service_status = PersistentServiceStatus::new(status_handle);
    persistent_service_status
        .set_pending_start(Duration::from_secs(1))
        .unwrap();

    let resource_dir = get_resource_dir();
    let daemon = Daemon::new(config.tunnel_log_file, resource_dir, config.require_auth)
        .chain_err(|| "Unable to initialize daemon")?;
    let shutdown_handle = daemon.shutdown_handle();

    // Register monitor that translates `ServiceControl` to Daemon events
    start_event_monitor(persistent_service_status.clone(), shutdown_handle, event_rx);

    persistent_service_status.set_running().unwrap();

    let result = daemon.run();

    // TODO: report correct exit code back after running a daemon.
    persistent_service_status
        .set_stopped(ServiceExitCode::default())
        .unwrap();

    result
}

/// Start event monitor thread that polls for `ServiceControl` and translates them into calls to
/// Daemon.
fn start_event_monitor(
    mut persistent_service_status: PersistentServiceStatus,
    shutdown_handle: DaemonShutdownHandle,
    event_rx: mpsc::Receiver<ServiceControl>,
) -> thread::JoinHandle<()> {
    thread::spawn(move || {
        for event in event_rx {
            match event {
                ServiceControl::Stop | ServiceControl::Shutdown => {
                    persistent_service_status
                        .set_pending_stop(Duration::from_secs(3))
                        .unwrap();

                    shutdown_handle.shutdown();
                }
                _ => (),
            }
        }
    })
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
    fn set_pending_start(&mut self, wait_hint: Duration) -> io::Result<()> {
        self.report_status(
            ServiceState::StartPending,
            wait_hint,
            ServiceExitCode::default(),
        )
    }

    /// Tell the system that the service is running.
    fn set_running(&mut self) -> io::Result<()> {
        self.report_status(
            ServiceState::Running,
            Duration::default(),
            ServiceExitCode::default(),
        )
    }

    /// Tell the system that the service is pending stop and provide the time estimate until the
    /// service is stopped.
    fn set_pending_stop(&mut self, wait_hint: Duration) -> io::Result<()> {
        self.report_status(
            ServiceState::StopPending,
            wait_hint,
            ServiceExitCode::default(),
        )
    }

    /// Tell the system that the service is stopped and provide the exit code.
    fn set_stopped(&mut self, exit_code: ServiceExitCode) -> io::Result<()> {
        self.report_status(ServiceState::Stopped, Duration::default(), exit_code)
    }

    /// Private helper to report the service status update.
    fn report_status(
        &mut self,
        next_state: ServiceState,
        wait_hint: Duration,
        exit_code: ServiceExitCode,
    ) -> io::Result<()> {
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
            exit_code: exit_code,
            checkpoint: checkpoint as u32,
            wait_hint: wait_hint,
        };

        debug!(
            "Update service status: {:?}, checkpoint: {}, wait_hint: {:?}",
            service_status.current_state, service_status.checkpoint, service_status.wait_hint
        );

        self.status_handle.set_service_status(service_status)
    }
}

/// Returns the list of accepted service events at each stage of the service lifecycle.
fn accepted_controls_by_state(state: ServiceState) -> ServiceControlAccept {
    match state {
        ServiceState::StartPending | ServiceState::PausePending | ServiceState::ContinuePending => {
            ServiceControlAccept::empty()
        }
        ServiceState::Running => ServiceControlAccept::STOP | ServiceControlAccept::SHUTDOWN,
        ServiceState::Paused => ServiceControlAccept::STOP | ServiceControlAccept::SHUTDOWN,
        ServiceState::StopPending | ServiceState::Stopped => ServiceControlAccept::empty(),
    }
}

pub fn install_service() -> Result<()> {
    let manager_access = ServiceManagerAccess::CONNECT | ServiceManagerAccess::CREATE_SERVICE;
    let service_manager = ServiceManager::local_computer(None::<&str>, manager_access)
        .chain_err(|| "Unable to connect to service manager")?;
    service_manager
        .create_service(get_service_info(), ServiceAccess::empty())
        .map(|_| ())
        .chain_err(|| "Unable to create a service")
}

fn get_service_info() -> ServiceInfo {
    let windows_directory = ::std::env::var_os("WINDIR").unwrap();
    let service_log_file = PathBuf::from(&windows_directory)
        .join("Temp")
        .join("mullvad-service.log");
    let tunnel_log_file = PathBuf::from(&windows_directory)
        .join("Temp")
        .join("mullvad-openvpn.log");

    ServiceInfo {
        name: OsString::from(SERVICE_NAME),
        display_name: OsString::from(SERVICE_DISPLAY_NAME),
        service_type: SERVICE_TYPE,
        start_type: ServiceStartType::AutoStart,
        error_control: ServiceErrorControl::Normal,
        executable_path: env::current_exe().unwrap(),
        launch_arguments: vec![
            OsString::from("--log"),
            OsString::from(service_log_file),
            OsString::from("--tunnel-log"),
            OsString::from(tunnel_log_file),
            OsString::from("--run-as-service"),
            OsString::from("-v"),
        ],
        account_name: None, // run as System
        account_password: None,
    }
}

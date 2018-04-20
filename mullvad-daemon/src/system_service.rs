#![cfg(windows)]

use std::ffi::OsString;
use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::mpsc;
use std::time::Duration;
use std::{env, io, thread};

use log;
use error_chain::ChainedError;
use windows_service::service::{ServiceAccess, ServiceControl, ServiceControlAccept,
                               ServiceErrorControl, ServiceExitCode, ServiceInfo,
                               ServiceStartType, ServiceState, ServiceStatus, ServiceType};
use windows_service::service_control_handler::{self, ServiceControlHandlerResult,
                                               ServiceStatusHandle};
use windows_service::service_dispatcher;
use windows_service::service_manager::{ServiceManager, ServiceManagerAccess};

use super::{get_resource_dir, log_version, logging, Daemon, DaemonShutdownHandle, Result,
            ResultExt};

static SERVICE_NAME: &'static str = "MullvadVPN";
static SERVICE_DISPLAY_NAME: &'static str = "Mullvad VPN Service";

pub fn run() -> Result<()> {
    let windows_directory = ::std::env::var_os("WINDIR").unwrap();
    let global_temp_directory = PathBuf::from(windows_directory).join("Temp");
    let main_log_file = Some(global_temp_directory.join("mullvad-daemon-service.log"));

    logging::init_logger(log::LevelFilter::Debug, main_log_file.as_ref(), true)
        .chain_err(|| "Unable to initialize logger")?;
    log_version();

    // Start the service dispatcher.
    // This will block current thread until the service stopped and spawn `service_main` on a
    // background thread.
    service_dispatcher::start_dispatcher(SERVICE_NAME, service_main)
        .chain_err(|| "Failed to start a service dispatcher")
}

define_windows_service!(service_main, handle_service_main);

pub fn handle_service_main(arguments: Vec<OsString>) {
    debug!("Started as a system service.");

    if let Err(e) = start_service(arguments) {
        error!("System service exited with error: {}", e.display_chain());
    }
}

fn start_service(_arguments: Vec<OsString>) -> Result<()> {
    let (event_tx, event_rx) = mpsc::channel();

    // Register service event handler
    let control_event_tx = event_tx.clone();
    let event_handler = move |control_event| -> ServiceControlHandlerResult {
        match control_event {
            // Notifies a service to report its current status information to the service
            // control manager. Always return NO_ERROR even if not implemented.
            ServiceControl::Interrogate => ServiceControlHandlerResult::NoError,

            ServiceControl::Shutdown | ServiceControl::Stop => {
                control_event_tx
                    .send(ServiceEvent::Control(control_event))
                    .unwrap();
                ServiceControlHandlerResult::NoError
            }

            _ => ServiceControlHandlerResult::NotImplemented,
        }
    };

    // Register system service events handler
    let status_handle =
        service_control_handler::register_control_handler(SERVICE_NAME, event_handler)
            .chain_err(|| "Failed to register a service control handler")?;

    let start_duration_hint = Duration::from_secs(1);
    update_service_status(
        &status_handle,
        ServiceStateUpdate::StartPending(start_duration_hint),
    ).unwrap();

    // Create daemon
    let windows_directory = ::std::env::var_os("WINDIR").unwrap();
    let tunnel_log_file = PathBuf::from(windows_directory)
        .join("Temp")
        .join("mullvad-daemon-openvpn.log");

    let resource_dir = get_resource_dir();
    let daemon = Daemon::new(Some(tunnel_log_file), resource_dir, true)
        .chain_err(|| "Unable to initialize daemon")?;
    let shutdown_handle = daemon.shutdown_handle();

    // Register monitor that translates `ServiceEvent` to Daemon events
    let event_monitor_thread =
        start_event_monitor(status_handle.clone(), shutdown_handle, event_rx);

    update_service_status(&status_handle, ServiceStateUpdate::Running).unwrap();

    let result = daemon.run();

    // shutdown event monitor
    event_tx.send(ServiceEvent::Shutdown).unwrap();
    event_monitor_thread.join().unwrap();

    // TBD: Catch Daemon shutdown and change service status to `ServiceState::StopPending`

    update_service_status(
        &status_handle,
        ServiceStateUpdate::Stopped(ServiceExitCode::Win32(0)),
    ).unwrap();

    result
}

/// Service event is a protocol between control handler and event monitor.
#[derive(Debug)]
enum ServiceEvent {
    Control(ServiceControl),
    Shutdown,
}

/// Start event monitor thread that polls for `ServiceEvent` and translates them into calls to
/// Daemon.
fn start_event_monitor(
    status_handle: ServiceStatusHandle,
    shutdown_handle: DaemonShutdownHandle,
    event_rx: mpsc::Receiver<ServiceEvent>,
) -> thread::JoinHandle<()> {
    thread::spawn(move || match event_rx.recv().unwrap() {
        ServiceEvent::Control(ServiceControl::Stop)
        | ServiceEvent::Control(ServiceControl::Shutdown) => {
            let shutdown_duration_hint = Duration::from_secs(3);

            update_service_status(
                &status_handle,
                ServiceStateUpdate::StopPending(shutdown_duration_hint),
            ).unwrap();

            shutdown_handle.shutdown();
        }
        ServiceEvent::Shutdown => {
            return;
        }
        _ => (),
    })
}

/// Checkpoint counter for ServiceState.
static CHECKPOINT_COUNTER: AtomicUsize = AtomicUsize::new(0);

/// Struct that logically groups information used at different stages of service lifecycle.
#[derive(Debug)]
enum ServiceStateUpdate {
    Running,
    Paused,
    Stopped(ServiceExitCode),
    StartPending(Duration),
    StopPending(Duration),
    ContinuePending(Duration),
    PausePending(Duration),
}

impl ServiceStateUpdate {
    fn get_service_state(&self) -> ServiceState {
        match *self {
            ServiceStateUpdate::Running => ServiceState::Running,
            ServiceStateUpdate::Paused => ServiceState::Paused,
            ServiceStateUpdate::Stopped(_) => ServiceState::Stopped,
            ServiceStateUpdate::StartPending(_) => ServiceState::StartPending,
            ServiceStateUpdate::StopPending(_) => ServiceState::StopPending,
            ServiceStateUpdate::ContinuePending(_) => ServiceState::ContinuePending,
            ServiceStateUpdate::PausePending(_) => ServiceState::PausePending,
        }
    }

    fn get_exit_code(&self) -> ServiceExitCode {
        match *self {
            ServiceStateUpdate::Stopped(exit_code) => exit_code,
            _ => ServiceExitCode::Win32(0),
        }
    }

    fn get_wait_hint(&self) -> Duration {
        match *self {
            ServiceStateUpdate::StartPending(wait_hint)
            | ServiceStateUpdate::StopPending(wait_hint)
            | ServiceStateUpdate::ContinuePending(wait_hint)
            | ServiceStateUpdate::PausePending(wait_hint) => wait_hint,
            _ => Duration::default(),
        }
    }
}

/// Send service status update to the system
fn update_service_status(
    status_handle: &ServiceStatusHandle,
    state_update: ServiceStateUpdate,
) -> io::Result<()> {
    let next_state = state_update.get_service_state();

    // Automatically bump the checkpoint when updating the pending events to tell the system
    // that the service is making a progress in transition from pending to final state.
    // `wait_hint` should reflect the estimated time for transition to complete.
    let checkpoint = match next_state {
        ServiceState::StartPending
        | ServiceState::StopPending
        | ServiceState::ContinuePending
        | ServiceState::PausePending => CHECKPOINT_COUNTER.fetch_add(1, Ordering::SeqCst) + 1,
        _ => 0,
    };

    let service_status = ServiceStatus {
        service_type: ServiceType::OwnProcess,
        current_state: next_state,
        controls_accepted: accepted_controls_by_state(next_state),
        exit_code: state_update.get_exit_code(),
        checkpoint: checkpoint as u32,
        wait_hint: state_update.get_wait_hint(),
    };

    debug!(
        "Update service status: {:?}, checkpoint: {}, wait_hint: {:?}",
        service_status.current_state, service_status.checkpoint, service_status.wait_hint
    );

    status_handle.set_service_status(service_status)
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
    let service_info = get_service_info();
    service_manager
        .create_service(service_info, ServiceAccess::empty())
        .map(|_| ())
        .chain_err(|| "Unable to create a service")
}

fn get_service_info() -> ServiceInfo {
    ServiceInfo {
        name: OsString::from(SERVICE_NAME),
        display_name: OsString::from(SERVICE_DISPLAY_NAME),
        service_type: ServiceType::OwnProcess,
        start_type: ServiceStartType::AutoStart,
        error_control: ServiceErrorControl::Normal,
        executable_path: env::current_exe().unwrap(),
        launch_arguments: vec![OsString::from("--run-as-service")],
        account_name: None, // run as System
        account_password: None,
    }
}

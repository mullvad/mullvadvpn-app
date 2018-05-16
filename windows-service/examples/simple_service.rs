// Simple service example.
//
// All commands mentioned below shall be executed in Command Prompt with Administrator privileges.
//
// Service self-installation: `simple_service.exe --install-service`
// Service self-removal: `simple_service.exe --remove-service`
//
// Start the service: `net start simpleservice`
// Pause the service: `net pause simpleservice`
// Resume the service: `net continue simpleservice`
// Stop the service: `net stop simpleservice`
//
// Simple service outputs all logs in C:\Windows\Temp\simple-service.log.
// If you have GNU tools installed, you can follow the log using:
// `tail -F C:\Windows\Temp\simple-service.log`
//

#[cfg(windows)]
#[macro_use]
extern crate error_chain;
#[cfg(windows)]
#[macro_use]
extern crate log;
#[cfg(windows)]
#[macro_use]
extern crate windows_service;

#[cfg(not(windows))]
fn main() {
    panic!("This program is only intended to run on Windows.");
}

#[cfg(windows)]
fn main() {
    simple_service::run();
}

#[cfg(windows)]
mod simple_service {
    extern crate simplelog;

    use std::ffi::OsString;
    use std::fs::OpenOptions;
    use std::path::PathBuf;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::mpsc;
    use std::time::Duration;
    use std::{env, io, thread, time};

    use self::simplelog::{CombinedLogger, Config, WriteLogger};
    use log::LevelFilter;

    use windows_service::service::{
        ServiceAccess, ServiceControl, ServiceControlAccept, ServiceErrorControl, ServiceExitCode,
        ServiceInfo, ServiceStartType, ServiceState, ServiceStatus, ServiceType,
    };
    use windows_service::service_control_handler::{
        self, ServiceControlHandlerResult, ServiceStatusHandle,
    };
    use windows_service::service_dispatcher;
    use windows_service::service_manager::{ServiceManager, ServiceManagerAccess};
    use windows_service::ChainedError;

    static SERVICE_NAME: &'static str = "SimpleService";
    static SERVICE_DISPLAY_NAME: &'static str = "Simple Service";

    error_chain! {
        errors {
            InstallService {
                description("Failed to install the service")
            }
            RemoveService {
                description("Failed to remove the service")
            }
            OpenLogFile(path: PathBuf) {
                description("Unable to open log file for writing")
                display("Unable to open log file for writing: {}", path.display())
            }
            InitLogger {
                description("Cannot initialize logger")
            }
        }
        foreign_links {
            SetLoggerError(::log::SetLoggerError);
        }
    }

    static CHECKPOINT_COUNTER: AtomicUsize = AtomicUsize::new(0);

    pub fn update_service_status(
        status_handle: &ServiceStatusHandle,
        next_state: ServiceState,
        exit_code: ServiceExitCode,
        wait_hint: Duration,
    ) -> io::Result<()> {
        // Automatically bump the checkpoint when updating the pending events to tell the system
        // that the service is making a progress in transition from pending to final state.
        // `wait_hint` should reflect the estimated time for transition to complete.
        let checkpoint = match next_state {
            ServiceState::ContinuePending
            | ServiceState::PausePending
            | ServiceState::StartPending
            | ServiceState::StopPending => CHECKPOINT_COUNTER.fetch_add(1, Ordering::SeqCst) + 1,
            _ => 0,
        };
        let service_status = ServiceStatus {
            service_type: ServiceType::OwnProcess,
            current_state: next_state,
            controls_accepted: accepted_controls_by_state(next_state),
            exit_code: exit_code,
            checkpoint: checkpoint as u32,
            wait_hint: wait_hint,
        };
        info!(
            "Update service status: {:?}, checkpoint: {}, wait_hint: {:?}",
            service_status.current_state, service_status.checkpoint, service_status.wait_hint
        );
        status_handle.set_service_status(service_status)
    }

    pub fn run() {
        if let Some(command) = env::args().nth(1) {
            match command.as_ref() {
                "--install-service" => {
                    if let Err(e) = install_service() {
                        println!("{}", e.display_chain());
                    } else {
                        println!("Installed the service.");
                    }
                }
                "--remove-service" => {
                    if let Err(e) = remove_service() {
                        println!("{}", e.display_chain());
                    } else {
                        println!("Removed the service.");
                    }
                }
                "--run-service" => {
                    // Setup file logger since there is no stdout when running as a service.
                    if let Err(err) = init_logger() {
                        panic!("Unable to initialize logger: {}", err.display_chain());
                    }

                    // Start the service dispatcher.
                    // This will block current thread until the service stopped.
                    let result = service_dispatcher::start_dispatcher(SERVICE_NAME, service_main);

                    match result {
                        Err(ref e) => {
                            error!("Failed to start service dispatcher: {}", e.display_chain());
                        }
                        Ok(_) => {
                            info!("Service dispatcher exited.");
                        }
                    };
                }
                _ => println!("Unsupported command: {}", command),
            }
        } else {
            println!("Usage:");
            println!("--install-service to install the service");
            println!("--remove-service to uninstall the service");
            println!("--run-service to run the service");
        }
    }

    define_windows_service!(service_main, handle_service_main);

    pub fn handle_service_main(arguments: Vec<OsString>) {
        // Create an event channel to funnel events to worker.
        let (event_tx, event_rx) = mpsc::channel();

        info!("Received arguments: {:?}", arguments);

        // Register service event handler
        let event_handler = move |control_event| -> ServiceControlHandlerResult {
            match control_event {
                // Notifies a service to report its current status information to the service
                // control manager. Always return NO_ERROR even if not implemented.
                ServiceControl::Interrogate => ServiceControlHandlerResult::NoError,

                // Handle primary control events
                ServiceControl::Pause
                | ServiceControl::Continue
                | ServiceControl::Stop
                | ServiceControl::Shutdown => {
                    event_tx.send(control_event).unwrap();
                    ServiceControlHandlerResult::NoError
                }

                _ => ServiceControlHandlerResult::NotImplemented,
            }
        };

        let result = service_control_handler::register_control_handler(SERVICE_NAME, event_handler);
        match result {
            Ok(status_handle) => {
                run_service(status_handle, event_rx);
            }
            Err(ref e) => {
                error!("Cannot register a service control handler: {}", e);
            }
        };

        info!("Quit service main.");
    }

    #[derive(Debug, Copy, Clone)]
    enum DaemonEvent {
        Continue,
        Pause,
        Stop,
    }

    fn start_event_monitor(
        service_status_handle: ServiceStatusHandle,
        event_rx: mpsc::Receiver<ServiceControl>,
        daemon_tx: mpsc::Sender<DaemonEvent>,
    ) -> thread::JoinHandle<()> {
        thread::spawn(move || {
            loop {
                match event_rx.recv().unwrap() {
                    ServiceControl::Pause => {
                        info!("Pausing the service.");

                        update_service_status(
                            &service_status_handle,
                            ServiceState::PausePending,
                            ServiceExitCode::Win32(0),
                            Duration::from_secs(2),
                        ).unwrap();

                        daemon_tx.send(DaemonEvent::Pause).unwrap();
                    }

                    ServiceControl::Continue => {
                        info!("Continuing the service.");

                        update_service_status(
                            &service_status_handle,
                            ServiceState::ContinuePending,
                            ServiceExitCode::Win32(0),
                            Duration::from_secs(2),
                        ).unwrap();

                        daemon_tx.send(DaemonEvent::Continue).unwrap();
                    }

                    ServiceControl::Stop => {
                        info!("Stopping the service.");

                        update_service_status(
                            &service_status_handle,
                            ServiceState::StopPending,
                            ServiceExitCode::Win32(0),
                            Duration::from_secs(2),
                        ).unwrap();

                        daemon_tx.send(DaemonEvent::Stop).unwrap();
                        break; // break the loop
                    }

                    ServiceControl::Shutdown => {
                        info!("Exiting due to shutdown.");

                        update_service_status(
                            &service_status_handle,
                            ServiceState::StopPending,
                            ServiceExitCode::Win32(0),
                            Duration::from_secs(1),
                        ).unwrap();

                        daemon_tx.send(DaemonEvent::Stop).unwrap();
                        break; // break the loop
                    }

                    _ => (),
                };
            }
        })
    }

    fn start_worker(
        service_status_handle: ServiceStatusHandle,
        daemon_rx: mpsc::Receiver<DaemonEvent>,
    ) -> thread::JoinHandle<()> {
        thread::spawn(move || {
            let mut is_running = true;
            let mut is_paused = false;

            // Tell Windows that the service is running now
            update_service_status(
                &service_status_handle,
                ServiceState::Running,
                ServiceExitCode::Win32(0),
                Duration::default(),
            ).unwrap();

            while is_running {
                // Do some work
                if !is_paused {
                    info!("Working...");
                }

                // Poll events
                match daemon_rx.recv_timeout(Duration::from_secs(1)) {
                    Ok(DaemonEvent::Pause) => {
                        is_paused = true;

                        update_service_status(
                            &service_status_handle,
                            ServiceState::Paused,
                            ServiceExitCode::Win32(0),
                            Duration::default(),
                        ).unwrap();
                    }
                    Ok(DaemonEvent::Continue) => {
                        is_paused = false;

                        update_service_status(
                            &service_status_handle,
                            ServiceState::Running,
                            ServiceExitCode::Win32(0),
                            Duration::default(),
                        ).unwrap();
                    }
                    Ok(DaemonEvent::Stop) | Err(mpsc::RecvTimeoutError::Disconnected) => {
                        is_running = false;

                        update_service_status(
                            &service_status_handle,
                            ServiceState::Stopped,
                            ServiceExitCode::Win32(0),
                            Duration::default(),
                        ).unwrap();
                    }
                    Err(mpsc::RecvTimeoutError::Timeout) => (),
                };
            }
        })
    }

    fn run_service(status_handle: ServiceStatusHandle, event_rx: mpsc::Receiver<ServiceControl>) {
        let (daemon_tx, daemon_rx) = mpsc::channel();

        // Tell Windows that the service is starting up
        update_service_status(
            &status_handle,
            ServiceState::StartPending,
            ServiceExitCode::Win32(0),
            Duration::from_secs(5),
        ).unwrap();

        let event_monitor_handle = start_event_monitor(status_handle, event_rx, daemon_tx);
        let worker_thread_handle = start_worker(status_handle, daemon_rx);

        // Block current thread until other threads complete execution
        event_monitor_handle.join().unwrap();
        worker_thread_handle.join().unwrap();
    }

    /// Returns the list of accepted service events at each stage of the service lifecycle.
    fn accepted_controls_by_state(state: ServiceState) -> ServiceControlAccept {
        match state {
            ServiceState::StartPending
            | ServiceState::PausePending
            | ServiceState::ContinuePending => ServiceControlAccept::empty(),
            ServiceState::Running => {
                ServiceControlAccept::STOP
                    | ServiceControlAccept::PAUSE_CONTINUE
                    | ServiceControlAccept::SHUTDOWN
            }
            ServiceState::Paused => {
                ServiceControlAccept::STOP
                    | ServiceControlAccept::PAUSE_CONTINUE
                    | ServiceControlAccept::SHUTDOWN
            }
            ServiceState::StopPending | ServiceState::Stopped => ServiceControlAccept::empty(),
        }
    }

    fn install_service() -> Result<()> {
        let manager_access = ServiceManagerAccess::CONNECT | ServiceManagerAccess::CREATE_SERVICE;
        let service_manager = ServiceManager::local_computer(None::<&str>, manager_access)
            .chain_err(|| ErrorKind::InstallService)?;
        let service_info = get_service_info();
        service_manager
            .create_service(service_info, ServiceAccess::empty())
            .map(|_| ())
            .chain_err(|| ErrorKind::InstallService)
    }

    fn remove_service() -> Result<()> {
        let manager_access = ServiceManagerAccess::CONNECT;
        let service_manager = ServiceManager::local_computer(None::<&str>, manager_access)
            .chain_err(|| ErrorKind::RemoveService)?;

        let service_access =
            ServiceAccess::QUERY_STATUS | ServiceAccess::STOP | ServiceAccess::DELETE;
        let service = service_manager
            .open_service(SERVICE_NAME, service_access)
            .chain_err(|| ErrorKind::RemoveService)?;

        loop {
            let service_status = service
                .query_status()
                .chain_err(|| ErrorKind::RemoveService)?;

            match service_status.current_state {
                ServiceState::StopPending => (),
                ServiceState::Stopped => {
                    println!("Removing the service...");
                    service.delete().chain_err(|| ErrorKind::RemoveService)?;
                    return Ok(()); // explicit return
                }
                _ => {
                    println!("Stopping the service...");
                    service.stop().chain_err(|| ErrorKind::RemoveService)?;
                }
            }

            thread::sleep(time::Duration::from_secs(1))
        }
    }

    fn get_service_info() -> ServiceInfo {
        ServiceInfo {
            name: OsString::from(SERVICE_NAME),
            display_name: OsString::from(SERVICE_DISPLAY_NAME),
            service_type: ServiceType::OwnProcess,
            start_type: ServiceStartType::OnDemand,
            error_control: ServiceErrorControl::Normal,
            executable_path: env::current_exe().unwrap(),
            launch_arguments: vec![OsString::from("--run-service")],
            account_name: None, // run as System
            account_password: None,
        }
    }

    fn init_logger() -> Result<()> {
        let windows_directory = env::var_os("WINDIR").unwrap();
        let log_file_path = PathBuf::from(windows_directory)
            .join("Temp")
            .join("simple-service.log");

        let log_file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(log_file_path.as_path())
            .chain_err(|| ErrorKind::OpenLogFile(log_file_path))?;

        let file_logger = WriteLogger::new(LevelFilter::Trace, Config::default(), log_file);

        CombinedLogger::init(vec![file_logger]).chain_err(|| ErrorKind::InitLogger)
    }

}

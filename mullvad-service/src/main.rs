#![cfg(windows)]

#[macro_use]
extern crate bitflags;
extern crate chrono;
#[macro_use]
extern crate derive_builder;
#[macro_use]
extern crate error_chain;
#[macro_use]
extern crate log;
extern crate shell_escape;
extern crate widestring;
extern crate winapi;

use std::error::Error;
use std::ffi::OsString;
use std::fs::OpenOptions;
use std::sync::mpsc::channel;
use std::{io, thread, time};

use winapi::shared::winerror::{ERROR_CALL_NOT_IMPLEMENTED, NO_ERROR};

mod service_manager;
use service_manager::{ServiceManager, ServiceManagerAccess};

mod service;
use service::{ServiceAccess, ServiceControl, ServiceError, ServiceErrorControl, ServiceInfo,
              ServiceStartType, ServiceState, ServiceType};

mod service_control_handler;
use service_control_handler::ServiceControlHandler;

#[macro_use]
mod service_dispatcher;

mod logging;
use logging::init_logger;

static SERVICE_NAME: &'static str = "Mullvad";
static SERVICE_DISPLAY_NAME: &'static str = "Mullvad VPN Service";

fn main() {
    let log_file = ::std::path::PathBuf::from("C:\\Windows\\Temp\\mullvad-service.log");
    if let Err(e) = OpenOptions::new()
        .append(true)
        .create_new(true)
        .open(log_file.as_path())
    {
        error!("Cannot create a log file: {}", e);
    }

    let _ = init_logger(log::LevelFilter::Trace, Some(&log_file));

    if let Some(command) = std::env::args().nth(1) {
        match command.as_ref() {
            "--install-service" => {
                if let Err(e) = install_service() {
                    error!("Failed to install the service: {}", e);
                } else {
                    info!("Installed the service.");
                }
            }
            "--remove-service" => {
                if let Err(e) = remove_service() {
                    error!("Failed to remove the service: {}", e);
                    if let Some(cause) = e.cause() {
                        error!("Cause: {}", cause);
                    }
                } else {
                    info!("Removed the service.");
                }
            }
            "--service" => {
                // Start the service dispatcher.
                // This will block current thread until the service stopped.
                let result = WindowsService::start_dispatcher();

                match result {
                    Err(ref e) => {
                        error!("Failed to start service dispatcher: {}", e);
                    }
                    Ok(_) => {
                        info!("Service dispatcher exited.");
                    }
                };
            }
            _ => warn!("Unsupported command: {}", command),
        }
    } else {
        info!("Usage:");
        info!("--install-service to install the service");
        info!("--remove-service to uninstall the service");
        info!("--service to run the service");
    }
}

define_windows_service!(WindowsService, SERVICE_NAME, handle_service_main);

fn handle_service_main(arguments: Vec<OsString>) {
    info!("Starting the service...");
    debug!("Service arguments: {:?}", arguments);

    // Create a shutdown channel to release this thread when stopping the service
    let (shutdown_sender, shutdown_receiver) = channel();

    // Service event handler
    let handler = move |ref _status_handle, control_event| -> u32 {
        match control_event {
            // Notifies a service to report its current status information to the service control
            // manager. Always return NO_ERROR even if not implemented.
            ServiceControl::Interrogate => NO_ERROR,

            // Stop daemon on stop or system shutdown
            ServiceControl::Stop | ServiceControl::Shutdown => {
                shutdown_sender.send(()).unwrap();
                NO_ERROR
            }

            _ => ERROR_CALL_NOT_IMPLEMENTED,
        }
    };

    let result = ServiceControlHandler::new(SERVICE_NAME, &handler);
    match result {
        Ok(_) => {
            shutdown_receiver.recv().unwrap();
        }
        Err(e) => {
            error!("Cannot register a service control handler: {}", e);
        }
    }
}

fn install_service() -> Result<(), io::Error> {
    let manager_access = ServiceManagerAccess::CONNECT | ServiceManagerAccess::CREATE_SERVICE;
    let service_manager = ServiceManager::local_computer(None::<&str>, manager_access)?;
    let service_info = get_service_info();
    service_manager
        .create_service(service_info, ServiceAccess::empty())
        .map(|_| ())
}

fn remove_service() -> Result<(), ServiceError> {
    let manager_access = ServiceManagerAccess::CONNECT;
    let service_manager = ServiceManager::local_computer(None::<&str>, manager_access)?;

    let service_access = ServiceAccess::QUERY_STATUS | ServiceAccess::STOP | ServiceAccess::DELETE;
    let service = service_manager.open_service(SERVICE_NAME, service_access)?;

    loop {
        let service_status = service.query_status()?;

        match service_status.current_state {
            ServiceState::StopPending => (),
            ServiceState::Stopped => {
                info!("Removing the service...");
                service.delete()?;
                return Ok(()); // explicit return
            }
            _ => {
                info!("Stopping the service...");
                service.stop()?;
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
        start_type: ServiceStartType::OnDemand, // TBD: change to AutoStart
        error_control: ServiceErrorControl::Normal,
        executable_path: std::env::current_exe().unwrap(),
        launch_arguments: vec![OsString::from("--service")],
        account_name: None, // run as System
        account_password: None,
    }
}

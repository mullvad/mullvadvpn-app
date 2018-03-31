#![cfg(windows)]

extern crate chrono;
#[macro_use]
extern crate error_chain;
#[macro_use]
extern crate log;
extern crate winapi;

use std::error::Error;
use std::ffi::{OsStr, OsString};
use std::fs::OpenOptions;
use std::sync::mpsc::channel;
use std::{io, ptr, thread, time};

use winapi::shared::winerror::{ERROR_CALL_NOT_IMPLEMENTED, NO_ERROR};
use winapi::um::winsvc;

mod service_manager;
use service_manager::{ServiceManager, ServiceManagerAccess};

mod service;
use service::{ServiceAccess, ServiceControl, ServiceError, ServiceErrorControl, ServiceInfo,
              ServiceStartType, ServiceState, ServiceType};

mod service_control_handler;
use service_control_handler::ServiceControlHandler;

mod widestring;
use widestring::{from_raw_wide_string, to_wide_with_nul};

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
            "-install" | "/install" => {
                if let Err(e) = install_service() {
                    error!("Failed to install the service: {}", e);
                } else {
                    info!("Installed the service.");
                }
            }
            "-remove" | "/remove" => {
                if let Err(e) = remove_service() {
                    error!("Failed to remove the service: {}", e);
                    if let Some(cause) = e.cause() {
                        error!("Cause: {}", cause);
                    }
                } else {
                    info!("Removed the service.");
                }
            }
            _ => warn!("Unsupported command: {}", command),
        }
    } else {
        info!("Usage:");
        info!("-install to install the service");
        info!("-remove to uninstall the service");

        // Start service dispatcher which blocks the main thread
        let result = start_service_dispatcher(SERVICE_NAME);
        match result {
            Err(ref e) => {
                error!("Failed to start service dispatcher: {}", e);
            }
            Ok(_) => {
                info!("Service dispatcher exited.");
            }
        }
    }
}

fn start_service_dispatcher<T: AsRef<OsStr>>(service_name: T) -> io::Result<()> {
    let service_name = to_wide_with_nul(service_name);

    let service_table: &[winsvc::SERVICE_TABLE_ENTRYW] = &[
        winsvc::SERVICE_TABLE_ENTRYW {
            lpServiceName: service_name.as_ptr(),
            lpServiceProc: Some(service_main),
        },
        // the last item has to be { null, null }
        winsvc::SERVICE_TABLE_ENTRYW {
            lpServiceName: ptr::null(),
            lpServiceProc: None,
        },
    ];

    debug!(
        "Starting service control dispatcher from thread: {:?}",
        thread::current().id()
    );

    // Blocks current thread until the service is stopped
    // This call spawns a new thread and calls `service_main`
    let result = unsafe { winsvc::StartServiceCtrlDispatcherW(service_table.as_ptr()) };
    if result == 0 {
        Err(io::Error::last_os_error())
    } else {
        Ok(())
    }
}

unsafe fn parse_service_main_arguments(argc: u32, argv: *mut *mut u16) -> Vec<OsString> {
    (0..argc)
        .into_iter()
        .map(|i| {
            let ptr = argv.offset(i as isize);
            from_raw_wide_string(*ptr, 256)
        })
        .collect()
}

/// Main entry point for windows service
/// `start_service_dispatcher` registers this function from `main`
extern "system" fn service_main(argc: u32, argv: *mut *mut u16) {
    info!("Starting the service...");
    debug!("service_main thread: {:?}", thread::current().id());

    // Parse arguments passed by service control manager
    let arguments = unsafe { parse_service_main_arguments(argc, argv) };
    debug!("Service arguments: {:?}", arguments);

    // Create a shutdown channel to release this thread when stopping the service
    let (shutdown_sender, shutdown_receiver) = channel();

    // Service event handler
    let handler = move |status_handle, control_event| -> u32 {
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
    let access_mask = &[
        ServiceManagerAccess::Connect,
        ServiceManagerAccess::CreateService,
    ];
    let service_manager = ServiceManager::local_computer(None::<&str>, access_mask)?;
    let service_info = get_service_info();

    service_manager
        .create_service(service_info, &[])
        .map(|_| ())
}

fn remove_service() -> Result<(), ServiceError> {
    let manager_access = &[
        ServiceManagerAccess::Connect,
        ServiceManagerAccess::CreateService,
    ];
    let service_manager = ServiceManager::local_computer(None::<&str>, manager_access)?;

    let service_access = &[
        ServiceAccess::QueryStatus,
        ServiceAccess::Stop,
        ServiceAccess::Delete,
    ];
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
    let executable_path = std::env::current_exe().unwrap();
    ServiceInfo {
        name: OsString::from(SERVICE_NAME),
        display_name: OsString::from(SERVICE_DISPLAY_NAME),
        service_type: ServiceType::OwnProcess,
        start_type: ServiceStartType::OnDemand, // TBD: change to AutoStart
        error_control: ServiceErrorControl::Normal,
        executable_path: OsString::from(executable_path),
        account_name: None, // run as System
        account_password: None,
    }
}

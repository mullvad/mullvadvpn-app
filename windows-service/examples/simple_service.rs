#![cfg(windows)]

extern crate chrono;
#[macro_use]
extern crate error_chain;
extern crate fern;
#[macro_use]
extern crate log;
#[macro_use]
extern crate windows_service;

use std::ffi::OsString;
use std::fs::OpenOptions;
use std::path::PathBuf;
use std::sync::mpsc::channel;
use std::{env, fmt, io, thread, time};

use windows_service::error_chain::ChainedError;
use windows_service::service::{ServiceAccess, ServiceControl, ServiceErrorControl, ServiceInfo,
                               ServiceStartType, ServiceState, ServiceType};
use windows_service::service_control_handler::{ServiceControlHandler, ServiceControlHandlerResult};
use windows_service::service_dispatcher;
use windows_service::service_manager::{ServiceManager, ServiceManagerAccess};

error_chain! {
    errors {
        InstallService {
            description("Failed to install the service")
        }
        RemoveService {
            description("Failed to remove the service")
        }
        WriteLogFile(path: PathBuf) {
            description("Unable to open log file for writing")
            display("Unable to open log file for writing: {}", path.to_string_lossy())
        }
    }
    foreign_links {
        SetLoggerError(log::SetLoggerError);
    }
}


static SERVICE_NAME: &'static str = "SimpleService";
static SERVICE_DISPLAY_NAME: &'static str = "Simple Service";

fn main() {
    let windows_directory = ::env::var_os("WINDIR").unwrap();
    let log_file = PathBuf::from(windows_directory)
        .join("Temp")
        .join("simple-service.log");

    if let Err(e) = OpenOptions::new()
        .append(true)
        .create_new(true)
        .open(log_file.as_path())
    {
        error!("Cannot create a log file: {}", e);
    }

    let _ = init_logger(log::LevelFilter::Trace, Some(&log_file));

    if let Some(command) = env::args().nth(1) {
        match command.as_ref() {
            "--install-service" => {
                if let Err(e) = install_service() {
                    error!("{}", e.display_chain());
                } else {
                    info!("Installed the service.");
                }
            }
            "--remove-service" => {
                if let Err(e) = remove_service() {
                    error!("{}", e.display_chain());
                } else {
                    info!("Removed the service.");
                }
            }
            "--service" => {
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
            _ => warn!("Unsupported command: {}", command),
        }
    } else {
        info!("Usage:");
        info!("--install-service to install the service");
        info!("--remove-service to uninstall the service");
        info!("--service to run the service");
    }
}

define_windows_service!(service_main, handle_service_main);

fn handle_service_main(arguments: Vec<OsString>) {
    // Create a shutdown channel to release this thread when stopping the service
    let (shutdown_sender, shutdown_receiver) = channel();

    info!("Received arguments: {:?}", arguments);

    // Service event handler
    let handler = move |ref _status_handle, control_event| -> ServiceControlHandlerResult {
        match control_event {
            // Notifies a service to report its current status information to the service
            // control manager. Always return NO_ERROR even if not implemented.
            ServiceControl::Interrogate => ServiceControlHandlerResult::NoError,

            // Stop daemon on stop or system shutdown
            ServiceControl::Stop | ServiceControl::Shutdown => {
                shutdown_sender.send(()).unwrap();
                ServiceControlHandlerResult::NoError
            }

            _ => ServiceControlHandlerResult::NotImplemented,
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

    let service_access = ServiceAccess::QUERY_STATUS | ServiceAccess::STOP | ServiceAccess::DELETE;
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
                info!("Removing the service...");
                service.delete().chain_err(|| ErrorKind::RemoveService)?;
                return Ok(()); // explicit return
            }
            _ => {
                info!("Stopping the service...");
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
        launch_arguments: vec![OsString::from("--service")],
        account_name: None, // run as System
        account_password: None,
    }
}


fn init_logger(log_level: log::LevelFilter, log_file: Option<&PathBuf>) -> Result<()> {
    let mut top_dispatcher = fern::Dispatch::new().level(log_level);
    let stdout_dispatcher = fern::Dispatch::new()
        .format(move |out, message, record| format_log_message(out, message, record))
        .chain(io::stdout());
    top_dispatcher = top_dispatcher.chain(stdout_dispatcher);

    if let Some(ref log_file) = log_file {
        let f = fern::log_file(log_file)
            .chain_err(|| ErrorKind::WriteLogFile(log_file.to_path_buf()))?;
        let file_dispatcher = fern::Dispatch::new()
            .format(|out, message, record| format_log_message(out, message, record))
            .chain(f);
        top_dispatcher = top_dispatcher.chain(file_dispatcher);
    }
    top_dispatcher.apply()?;
    Ok(())
}

fn format_log_message(out: fern::FormatCallback, message: &fmt::Arguments, record: &log::Record) {
    let timestamp = chrono::Local::now().format("%Y-%m-%d %H:%M:%S%.3f");
    out.finish(format_args!(
        "[{}][{}][{}] {}",
        timestamp,
        record.target(),
        record.level(),
        message
    ))
}

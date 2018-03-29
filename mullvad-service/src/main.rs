#![cfg(windows)]

extern crate winapi;

use std::error::Error;
use std::ffi::OsString;
use std::io;
use std::ptr;
use std::thread;
use std::time;

use winapi::shared::minwindef::LPVOID;
use winapi::shared::winerror::{ERROR_CALL_NOT_IMPLEMENTED, NO_ERROR};
use winapi::um::winnt::LPWSTR;
use winapi::um::winsvc;

mod service_manager;
use service_manager::{ServiceManager, ServiceManagerAccess};

mod service;
use service::{ServiceAccess, ServiceControl, ServiceError, ServiceErrorControl, ServiceInfo,
              ServiceStartType, ServiceState, ServiceType};

mod widestring;
use widestring::to_wide_with_nul;

static SERVICE_NAME: &'static str = "Mullvad";
static SERVICE_DISPLAY_NAME: &'static str = "Mullvad VPN Service";

fn main() {
    if let Some(command) = std::env::args().nth(1) {
        match command.as_ref() {
            "-install" | "/install" => {
                if let Err(e) = install_service() {
                    println!("Failed to install the service: {}", e);
                } else {
                    println!("Installed the service.");
                }
            }
            "-remove" | "/remove" => {
                if let Err(e) = remove_service() {
                    println!("Failed to remove the service: {}", e);
                    if let Some(cause) = e.cause() {
                        println!("Cause: {}", cause);
                    }
                } else {
                    println!("Removed the service.");
                }
            }
            _ => println!("Unsupported command: {}", command),
        }
    } else {
        println!("Usage:");
        println!("-install to install the service");
        println!("-remove to uninstall the service");

        if let Err(e) = start_service_dispatcher() {
            println!("Failed to start service dispatcher: {}", e);
        }
    }
}

fn start_service_dispatcher() -> io::Result<()> {
    let service_info = get_service_info();
    let service_name = to_wide_with_nul(service_info.name);

    let service_table: &[winsvc::SERVICE_TABLE_ENTRYW] = &[
        winsvc::SERVICE_TABLE_ENTRYW {
            lpServiceName: service_name.as_ptr(),
            lpServiceProc: Some(service_main),
        },
        winsvc::SERVICE_TABLE_ENTRYW {
            lpServiceName: ptr::null(),
            lpServiceProc: None,
        },
    ];

    // blocks current thread until the service is stopped
    let result = unsafe { winsvc::StartServiceCtrlDispatcherW(service_table.as_ptr()) };
    if result == 0 {
        Err(io::Error::last_os_error())
    } else {
        Ok(())
    }
}

extern "system" fn service_main(argc: u32, argv: *mut LPWSTR) {
    let service_info = get_service_info();
    let service_name = to_wide_with_nul(service_info.name);

    println!("Started service with {} arguments.", argc);

    let service_status_handle = unsafe {
        winsvc::RegisterServiceCtrlHandlerExW(
            service_name.as_ptr(),
            Some(service_control_handler),
            ptr::null_mut(),
        )
    };
    if service_status_handle.is_null() {
        let os_error = io::Error::last_os_error();
        println!("Error calling RegisterServiceCtrlHandlerExW: {}", os_error);
        return;
    }
}

extern "system" fn service_control_handler(
    control: u32,
    event_type: u32,
    event_data: LPVOID,
    context: LPVOID,
) -> u32 {
    let result = ServiceControl::from_raw(control);

    match result {
        Ok(service_control_event) => {
            println!(
                "Received service control event: {:?}",
                service_control_event
            );
            handle_service_control_event(service_control_event)
        }
        Err(ref e) => {
            println!("Received unrecognized service control event: {}", e);
            ERROR_CALL_NOT_IMPLEMENTED
        }
    }
}

/// Service event handler.
/// Please visit MSDN for more details about service events handling:
/// https://msdn.microsoft.com/en-us/library/windows/desktop/ms683241(v=vs.85).aspx
fn handle_service_control_event(control_event: ServiceControl) -> u32 {
    match control_event {
        // Notifies a service to report its current status information to the service control
        // manager. Always return NO_ERROR even if not implemented.
        ServiceControl::Interrogate => NO_ERROR,

        // Stop daemon on stop or system shutdown
        ServiceControl::Stop | ServiceControl::Shutdown => NO_ERROR,

        _ => ERROR_CALL_NOT_IMPLEMENTED,
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
                println!("Removing the service...");
                service.delete()?;
                return Ok(()); // explicit return
            }
            _ => {
                println!("Stopping the service...");
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

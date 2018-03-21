#![cfg(windows)]

extern crate winapi;

use std::ffi::OsString;
use std::io;
use std::thread;
use std::time;

mod errors {
    #[derive(Debug, Clone)]
    pub struct RawConversionError;

    impl ::std::error::Error for RawConversionError {
        fn description(&self) -> &str {
            "Raw conversion error"
        }
        
        fn cause(&self) -> Option<&::std::error::Error> {
            None
        }
    }

    impl ::std::fmt::Display for RawConversionError {
        fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
            write!(f, "Raw conversion error")
        }
    }
}
pub use errors::*;

mod scmanager;
use scmanager::*;

mod service;
use service::*;

mod widestring;

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
            },
            "-remove" | "/remove" => {
                if let Err(e) = remove_service() {
                    println!("Failed to remove the service: {}", e);
                } else {
                    println!("Removed the service.");
                }
            },
            _ => println!("Unsupported command: {}", command),
        }
    } else {
        println!("Usage:");
        println!("-install to install the service");
        println!("-remove to uninstall the service")
    }
}

fn install_service() -> Result<(), io::Error> {
    let access_mask = SCManagerAccessMask::new(&[SCManagerAccess::Connect, SCManagerAccess::CreateService]);
    let service_manager = SCManager::active_database(access_mask)?;
    let service_info = get_service_info();
    service_manager.create_service(service_info).map(|_| ())
}

fn remove_service() -> Result<(), ServiceError> {
    let manager_access = SCManagerAccessMask::new(&[SCManagerAccess::Connect, SCManagerAccess::CreateService]);
    let service_manager = SCManager::active_database(manager_access)?;

    let service_access = ServiceAccessMask::new(&[ServiceAccess::QueryStatus, ServiceAccess::Stop, ServiceAccess::Delete]);
    let service = service_manager.open_service(SERVICE_NAME, service_access)?;

    loop {
        let service_status = service.query_status()?;
        
        match service_status.current_state {
            ServiceState::StopPending => thread::sleep(time::Duration::from_secs(1)),
            ServiceState::Stopped => {
                println!("Removing the service...");
                service.delete()?;
                return Ok(()); // explicit return
            },
            _ => {
                println!("Stopping the service...");
                service.stop()?;
                thread::sleep(time::Duration::from_secs(1));
            }
        }
    }
}

fn get_service_info() -> ServiceInfo {
    let executable_path = std::env::current_exe().unwrap();
    ServiceInfo {
        name: OsString::from(SERVICE_NAME),
        display_name: OsString::from(SERVICE_DISPLAY_NAME),
        service_access: ServiceAccessMask::new(&[ServiceAccess::QueryStatus]),
        service_type: ServiceType::OwnProcess,
        start_type: ServiceStartType::OnDemand, // TBD: change to AutoStart
        error_control: ServiceErrorControl::Normal,
        executable_path: OsString::from(executable_path),
        account_name: None, // run as System
        account_password: None,
    }
}

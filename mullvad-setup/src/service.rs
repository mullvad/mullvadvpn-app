use std::{
    ffi::{OsStr, OsString},
    time::Duration,
};

use super::Error;

use mullvad_daemon::service::{SERVICE_DISPLAY_NAME, SERVICE_NAME};
use windows_service::{
    service::{Service, ServiceAccess, ServiceState},
    service_manager::{ServiceManager, ServiceManagerAccess},
};

use windows_sys::Win32::Foundation::ERROR_SERVICE_ALREADY_RUNNING;

const WAIT_STATUS_TIMEOUT: Duration = Duration::from_secs(8);

/// Start Mullvad VPN service and wait until it is running
pub async fn start() -> Result<(), Error> {
    tokio::task::spawn_blocking(start_and_wait_for_service)
        .await
        .unwrap()
}

fn start_and_wait_for_service() -> Result<(), Error> {
    println!("Starting {SERVICE_DISPLAY_NAME}...");

    let scm = ServiceManager::local_computer(
        None::<OsString>,
        ServiceManagerAccess::CONNECT | ServiceManagerAccess::CREATE_SERVICE,
    )
    .map_err(Error::OpenServiceControlManager)?;

    let service = scm
        .open_service(SERVICE_NAME, ServiceAccess::all())
        .map_err(Error::OpenService)?;

    if let Err(error) = service.start::<&OsStr>(&[]) {
        if let windows_service::Error::Winapi(error) = &error
            && error.raw_os_error() == Some(ERROR_SERVICE_ALREADY_RUNNING as i32)
        {
            return Ok(());
        }
        return Err(Error::StartService(error));
    }

    wait_for_status(&service, ServiceState::Running)
}

fn wait_for_status(service: &Service, target_state: ServiceState) -> Result<(), Error> {
    let initial_time = std::time::Instant::now();
    loop {
        let status = service.query_status().map_err(Error::QueryServiceStatus)?;

        if status.current_state == target_state {
            break;
        }

        if initial_time.elapsed() >= WAIT_STATUS_TIMEOUT {
            return Err(Error::StartServiceTimeout);
        }

        std::thread::sleep(std::time::Duration::from_secs(1));
    }

    Ok(())
}

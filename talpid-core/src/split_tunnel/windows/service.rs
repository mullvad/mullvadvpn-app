use std::{
    ffi::{OsStr, OsString},
    io,
    path::{Path, PathBuf},
    time::Duration,
};
use windows_service::{
    service::{
        Service, ServiceAccess, ServiceErrorControl, ServiceInfo, ServiceStartType, ServiceState,
        ServiceType,
    },
    service_manager::{ServiceManager, ServiceManagerAccess},
};
use windows_sys::Win32::Foundation::{ERROR_SERVICE_ALREADY_RUNNING, ERROR_SERVICE_DOES_NOT_EXIST};

const SPLIT_TUNNEL_SERVICE: &str = "mullvad-split-tunnel";
const SPLIT_TUNNEL_DISPLAY_NAME: &str = "Mullvad Split Tunnel Service";
const DRIVER_FILENAME: &str = "mullvad-split-tunnel.sys";

const WAIT_STATUS_TIMEOUT: Duration = Duration::from_secs(8);

#[derive(err_derive::Error, Debug)]
#[error(no_from)]
pub enum Error {
    /// Failed to open service control manager
    #[error(display = "Failed to connect to service control manager")]
    OpenServiceControlManager(#[error(source)] windows_service::Error),

    /// Failed to create a service handle
    #[error(display = "Failed to open service")]
    OpenServiceHandle(#[error(source)] windows_service::Error),

    /// Failed to start split tunnel service
    #[error(display = "Failed to start split tunnel device driver service")]
    StartService(#[error(source)] windows_service::Error),

    /// Failed to check service status
    #[error(display = "Failed to query service status")]
    QueryServiceStatus(#[error(source)] windows_service::Error),

    /// Failed to open service config
    #[error(display = "Failed to retrieve service config")]
    QueryServiceConfig(#[error(source)] windows_service::Error),

    /// Failed to install ST service
    #[error(display = "Failed to install split tunnel driver")]
    InstallService(#[error(source)] windows_service::Error),

    /// Failed to start ST service
    #[error(display = "Timed out waiting on service to start")]
    StartTimeout,

    /// Failed to connect to existing driver
    #[error(display = "Failed to open service handle")]
    OpenHandle(#[error(source)] super::driver::DeviceHandleError),

    /// Failed to reset existing driver
    #[error(display = "Failed to reset driver state")]
    ResetDriver(#[error(source)] io::Error),
}

pub fn install_driver_if_required(resource_dir: &Path) -> Result<(), Error> {
    let scm = ServiceManager::local_computer(
        None::<OsString>,
        ServiceManagerAccess::CONNECT | ServiceManagerAccess::CREATE_SERVICE,
    )
    .map_err(Error::OpenServiceControlManager)?;

    let expected_syspath = resource_dir.join(DRIVER_FILENAME);

    let service = match scm.open_service(SPLIT_TUNNEL_SERVICE, ServiceAccess::all()) {
        Ok(service) => service,
        Err(error) => {
            return match error {
                windows_service::Error::Winapi(io_error)
                    if io_error.raw_os_error() == Some(ERROR_SERVICE_DOES_NOT_EXIST as i32) =>
                {
                    // TODO: could be marked for deletion
                    install_driver(&scm, &expected_syspath)
                }
                error => Err(Error::OpenServiceHandle(error)),
            };
        }
    };

    if expected_syspath != get_driver_binpath(&service)? {
        log::debug!("ST driver is already installed");
        return start_and_wait_for_service(&service);
    }

    log::debug!("Replacing ST driver due to unexpected path");

    remove_device(service)?;
    install_driver(&scm, &expected_syspath)
}

pub fn stop_driver_service() -> Result<(), Error> {
    let scm = ServiceManager::local_computer(None::<OsString>, ServiceManagerAccess::CONNECT)
        .map_err(Error::OpenServiceControlManager)?;

    let service = match scm.open_service(SPLIT_TUNNEL_SERVICE, ServiceAccess::all()) {
        Ok(service) => service,
        Err(error) => {
            return match error {
                windows_service::Error::Winapi(io_error)
                    if io_error.raw_os_error() == Some(ERROR_SERVICE_DOES_NOT_EXIST as i32) =>
                {
                    Ok(())
                }
                error => Err(Error::OpenServiceHandle(error)),
            };
        }
    };

    stop_service(&service)
}

fn stop_service(service: &Service) -> Result<(), Error> {
    let _ = service.stop();
    wait_for_status(service, ServiceState::Stopped)
}

fn remove_device(service: Service) -> Result<(), Error> {
    reset_driver(&service)?;
    stop_service(&service)?;
    let _ = service.delete();
    Ok(())
}

fn reset_driver(service: &Service) -> Result<(), Error> {
    let status = service.query_status().map_err(Error::QueryServiceStatus)?;

    if status.current_state == ServiceState::Running {
        let old_handle =
            super::driver::DeviceHandle::new_handle_only().map_err(Error::OpenHandle)?;
        old_handle.reset().map_err(Error::ResetDriver)?;
    }

    Ok(())
}

fn install_driver(scm: &ServiceManager, syspath: &Path) -> Result<(), Error> {
    log::debug!("Installing split tunnel driver");

    let service_info = ServiceInfo {
        name: SPLIT_TUNNEL_SERVICE.into(),
        display_name: SPLIT_TUNNEL_DISPLAY_NAME.into(),
        service_type: ServiceType::KERNEL_DRIVER,
        start_type: ServiceStartType::OnDemand,
        error_control: ServiceErrorControl::Normal,
        executable_path: syspath.to_path_buf(),
        launch_arguments: vec![],
        dependencies: vec![],
        account_name: None,
        account_password: None,
    };

    let service = scm
        .create_service(
            &service_info,
            ServiceAccess::START | ServiceAccess::QUERY_STATUS,
        )
        .map_err(Error::InstallService)?;

    start_and_wait_for_service(&service)
}

fn start_and_wait_for_service(service: &Service) -> Result<(), Error> {
    log::debug!("Starting split tunnel service");

    if let Err(error) = service.start::<&OsStr>(&[]) {
        if let windows_service::Error::Winapi(error) = &error {
            if error.raw_os_error() == Some(ERROR_SERVICE_ALREADY_RUNNING as i32) {
                return Ok(());
            }
        }
        return Err(Error::StartService(error));
    }

    wait_for_status(service, ServiceState::Running)
}

fn wait_for_status(service: &Service, target_state: ServiceState) -> Result<(), Error> {
    let initial_time = std::time::Instant::now();
    loop {
        let status = service.query_status().map_err(Error::QueryServiceStatus)?;

        if status.current_state == target_state {
            break;
        }

        if initial_time.elapsed() >= WAIT_STATUS_TIMEOUT {
            return Err(Error::StartTimeout);
        }

        std::thread::sleep(std::time::Duration::from_secs(1));
    }

    Ok(())
}

fn get_driver_binpath(service: &Service) -> Result<PathBuf, Error> {
    let config = service.query_config().map_err(Error::QueryServiceConfig)?;
    Ok(config.executable_path)
}

use std::{io, os::windows::prelude::OsStrExt, path::Path, ptr, time::Duration, ffi::OsString};
use widestring::{WideCStr, WideCString};
use windows::{
    core::PCWSTR,
    Win32::{
        Foundation::{
            GetLastError, ERROR_INSUFFICIENT_BUFFER, ERROR_SERVICE_DOES_NOT_EXIST, HANDLE, ERROR_SERVICE_ALREADY_RUNNING,
        },
        Security::SC_HANDLE,
        System::{
            Services::{
                CloseServiceHandle, CreateServiceW, OpenSCManagerW, OpenServiceW,
                QueryServiceConfigW, QUERY_SERVICE_CONFIGW, SC_MANAGER_ALL_ACCESS,
                SERVICE_ALL_ACCESS, SERVICE_DEMAND_START, SERVICE_ERROR_NORMAL,
                SERVICE_KERNEL_DRIVER, DeleteService, StartServiceW, QueryServiceStatus,
                SERVICE_STATUS, SERVICE_RUNNING, SERVICE_STOPPED, ControlService, SERVICE_CONTROL_STOP, SERVICE_STATUS_CURRENT_STATE,
            },
            SystemServices::GENERIC_READ,
        },
    },
};
use talpid_types::ErrorExt;

const SPLIT_TUNNEL_SERVICE: &[u8] =
    b"m\0u\0l\0l\0v\0a\0d\0-\0s\0p\0l\0i\0t\0-\0t\0u\0n\0n\0e\0l\0\0\0";
const SERVICE_DISPLAY_NAME: &[u8] =
    b"M\0u\0l\0l\0v\0a\0d\0 \0S\0p\0l\0i\0t\0 \0T\0u\0n\0n\0e\0l\0 \0S\0e\0r\0v\0i\0c\0e\0\0\0";
const DRIVER_FILENAME: &str = "mullvad-split-tunnel.sys";

const WAIT_STATUS_TIMEOUT: Duration = Duration::from_secs(8);

#[derive(err_derive::Error, Debug)]
#[error(no_from)]
pub enum Error {
    /// Failed to open service control manager
    #[error(display = "Failed to connect to service control manager")]
    OpenServiceControlManager(#[error(source)] windows::core::Error),

    /// Failed to create a service handle
    #[error(display = "Failed to open service")]
    OpenServiceHandle(#[error(source)] windows::core::Error),

    /// Failed to start split tunnel service
    #[error(display = "Failed to start split tunnel device driver service")]
    StartService(#[error(source)] windows::core::Error),

    /// Failed to check service status
    #[error(display = "Failed to query service status")]
    QueryServiceStatus(#[error(source)] windows::core::Error),

    /// Failed to open service config
    #[error(display = "Failed to retrieve service config")]
    QueryServiceConfig(#[error(source)] windows::core::Error),

    /// Failed to install ST service
    #[error(display = "Failed to install split tunnel driver")]
    InstallService(#[error(source)] windows::core::Error),

    /// Failed to start ST service
    #[error(display = "Timed out waiting on service to start")]
    StartTimeout,

    /// Failed to connect to existing driver
    #[error(display = "Failed to connect to old service")]
    ConnectOldService(#[error(source)] super::driver::DeviceHandleError),

    /// Failed to reset existing driver
    #[error(display = "Failed to reset old service state")]
    ResetOldDriver(#[error(source)] io::Error),
}

struct ScopedServiceHandle(SC_HANDLE);

impl Drop for ScopedServiceHandle {
    fn drop(&mut self) {
        unsafe { CloseServiceHandle(self.0) };
    }
}

pub fn install_driver_if_required(resource_dir: &Path) -> Result<(), Error> {
    let scm =
        ScopedServiceHandle(unsafe { OpenSCManagerW(PCWSTR::default(), PCWSTR::default(), SC_MANAGER_ALL_ACCESS) }
            .map_err(Error::OpenServiceControlManager)?);

    let expected_syspath = resource_dir.join(DRIVER_FILENAME);

    let service = unsafe {
        OpenServiceW(
            scm.0,
            PCWSTR(SPLIT_TUNNEL_SERVICE as *const _ as *const u16),
            SERVICE_ALL_ACCESS,
        )
        .map(ScopedServiceHandle)
    };

    let service = match service {
        Ok(service) => service,
        Err(error) => {
            return if error.code() == ERROR_SERVICE_DOES_NOT_EXIST.to_hresult() {
                // TODO: could be marked for deletion
                unsafe { install_driver(scm.0, &expected_syspath) }
            } else {
                Err(Error::OpenServiceHandle(windows::core::Error::from(error)))
            };
        }
    };

    let binpath = unsafe { get_driver_binpath(service.0) }?;

    // Replace existing driver if its path is unexpected

    if expected_syspath != Path::new(&binpath) {
        log::debug!("The correct ST driver is already installed");
        return unsafe { start_and_wait_for_service(service.0) };
    }

    log::debug!("Replacing ST driver with unexpected path");

    unsafe { remove_device(service.0) }?;
    drop(service);

    unsafe { install_driver(scm.0, &expected_syspath) }
}

pub fn stop_driver_service() -> Result<(), Error> {
    let scm =
        ScopedServiceHandle(unsafe { OpenSCManagerW(PCWSTR::default(), PCWSTR::default(), SC_MANAGER_ALL_ACCESS) }
            .map_err(Error::OpenServiceControlManager)?);
    let service = unsafe {
        OpenServiceW(
            scm.0,
            PCWSTR(SPLIT_TUNNEL_SERVICE as *const _ as *const u16),
            SERVICE_ALL_ACCESS,
        )
        .map(ScopedServiceHandle)
    };

    let service = match service {
        Ok(service) => service,
        Err(error) => {
            return if error.code() == ERROR_SERVICE_DOES_NOT_EXIST.to_hresult() {
                return Ok(());
            } else {
                Err(Error::OpenServiceHandle(windows::core::Error::from(error)))
            };
        }
    };

    unsafe { stop_service(service.0) }
}

unsafe fn stop_service(service: SC_HANDLE) -> Result<(), Error> {
    let mut service_status = SERVICE_STATUS::default();
    ControlService(service, SERVICE_CONTROL_STOP, &mut service_status);
    wait_for_status(service, SERVICE_STOPPED)
}

unsafe fn remove_device(service: SC_HANDLE) -> Result<(), Error> {
    reset_driver(service)?;
    stop_service(service)?;
    DeleteService(service);
    Ok(())
}

unsafe fn reset_driver(service: SC_HANDLE) -> Result<(), Error> {
    let mut service_status = SERVICE_STATUS::default();

    if !QueryServiceStatus(service, &mut service_status).as_bool() {
        return Err(Error::QueryServiceStatus(windows::core::Error::from(
            GetLastError(),
        )));
    }

    if service_status.dwCurrentState == SERVICE_RUNNING {
        let old_handle = super::driver::DeviceHandle::new_handle_only()
            .map_err(Error::ConnectOldService)?;
        old_handle.reset().map_err(Error::ResetOldDriver)?;
    }

    Ok(())
}

unsafe fn install_driver(scm: SC_HANDLE, syspath: &Path) -> Result<(), Error> {
    log::debug!("Installing split tunnel driver");

    let binary_path: Vec<u16> = syspath
        .as_os_str()
        .encode_wide()
        .chain(std::iter::once(0u16))
        .collect();

    let service = CreateServiceW(
        scm,
        PCWSTR(SPLIT_TUNNEL_SERVICE as *const _ as *const u16),
        PCWSTR(SERVICE_DISPLAY_NAME as *const _ as *const u16),
        SERVICE_ALL_ACCESS,
        SERVICE_KERNEL_DRIVER,
        SERVICE_DEMAND_START,
        SERVICE_ERROR_NORMAL,
        PCWSTR(binary_path.as_ptr()),
        PCWSTR(ptr::null()),
        ptr::null_mut(),
        PCWSTR(ptr::null()),
        PCWSTR(ptr::null()),
        PCWSTR(ptr::null()),
    )
    .map_err(Error::InstallService)?;

    log::debug!("Created split tunnel service");

    let service = ScopedServiceHandle(service);
    start_and_wait_for_service(service.0)
}

unsafe fn start_and_wait_for_service(service: SC_HANDLE) -> Result<(), Error> {
    if !StartServiceW(service, &[]).as_bool() {
        let last_error = GetLastError();

        if last_error == ERROR_SERVICE_ALREADY_RUNNING {
            return Ok(());
        }

        return Err(Error::StartService(windows::core::Error::from(last_error)));
    }

    log::debug!("Starting split tunnel service");

    wait_for_status(service, SERVICE_RUNNING)
}

unsafe fn wait_for_status(service: SC_HANDLE, target_state: SERVICE_STATUS_CURRENT_STATE) -> Result<(), Error> {
    let mut service_status = SERVICE_STATUS::default();
    let initial_time = std::time::Instant::now();
    loop {
        if !QueryServiceStatus(service, &mut service_status).as_bool() {
            return Err(Error::QueryServiceStatus(windows::core::Error::from(
                GetLastError(),
            )));
        }

        if service_status.dwCurrentState == target_state {
            break;
        }

        if initial_time.elapsed() >= WAIT_STATUS_TIMEOUT {
            return Err(Error::StartTimeout);
        }

        std::thread::sleep(std::time::Duration::from_secs(1));
    }

    Ok(())
}

unsafe fn get_driver_binpath(service: SC_HANDLE) -> Result<OsString, Error> {
    let mut config_buf = vec![];
    let config;

    let mut bytes_needed = 0u32;

    let result = QueryServiceConfigW(service, ptr::null_mut(), 0, &mut bytes_needed);
    if !result.as_bool() {
        let last_error = GetLastError();
        if last_error != ERROR_INSUFFICIENT_BUFFER {
            return Err(Error::QueryServiceConfig(windows::core::Error::from(
                last_error,
            )));
        }
    }

    config_buf.resize(usize::try_from(bytes_needed).unwrap(), 0u8);

    let result = QueryServiceConfigW(
        service,
        config_buf.as_mut_ptr() as _,
        u32::try_from(config_buf.len()).unwrap(),
        &mut bytes_needed,
    );

    if !result.as_bool() {
        return Err(Error::QueryServiceConfig(windows::core::Error::from(
            GetLastError(),
        )));
    }

    config = &*(config_buf.as_ptr() as *const QUERY_SERVICE_CONFIGW);

    Ok(WideCStr::from_ptr_str(config.lpBinaryPathName.0).to_os_string())
}

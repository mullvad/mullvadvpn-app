use std::collections::HashMap;
use std::ffi::OsString;
use std::io;
use std::path::Path;
use test_rpc::mullvad_daemon::ServiceStatus;
use test_rpc::{meta::OsVersion, mullvad_daemon::Verbosity};
use windows_service::{
    service::{Service, ServiceAccess, ServiceInfo, ServiceState},
    service_manager::{ServiceManager, ServiceManagerAccess},
};
use windows_sys::Win32::{
    System::Shutdown::{
        EWX_REBOOT, ExitWindowsEx, SHTDN_REASON_FLAG_PLANNED, SHTDN_REASON_MAJOR_APPLICATION,
        SHTDN_REASON_MINOR_OTHER,
    },
    UI::WindowsAndMessaging::EWX_FORCEIFHUNG,
};

const MULLVAD_WIN_REGISTRY: &str = r"SYSTEM\CurrentControlSet\Services\Mullvad VPN";

pub fn reboot() -> Result<(), test_rpc::Error> {
    grant_shutdown_privilege()?;

    std::thread::spawn(|| {
        std::thread::sleep(std::time::Duration::from_secs(5));

        let shutdown_result = unsafe {
            ExitWindowsEx(
                EWX_FORCEIFHUNG | EWX_REBOOT,
                SHTDN_REASON_MAJOR_APPLICATION
                    | SHTDN_REASON_MINOR_OTHER
                    | SHTDN_REASON_FLAG_PLANNED,
            )
        };

        if shutdown_result == 0 {
            log::error!(
                "Failed to restart test machine: {}",
                io::Error::last_os_error()
            );
            std::process::exit(1);
        }

        std::process::exit(0);
    });

    // NOTE: We do not bother to revoke the privilege.

    Ok(())
}

fn grant_shutdown_privilege() -> Result<(), test_rpc::Error> {
    use windows_sys::Win32::{
        Foundation::{CloseHandle, HANDLE, LUID},
        Security::{
            AdjustTokenPrivileges, LUID_AND_ATTRIBUTES, LookupPrivilegeValueW,
            SE_PRIVILEGE_ENABLED, TOKEN_ADJUST_PRIVILEGES, TOKEN_PRIVILEGES,
        },
        System::{
            SystemServices::SE_SHUTDOWN_NAME,
            Threading::{GetCurrentProcess, OpenProcessToken},
        },
    };

    let mut privileges = TOKEN_PRIVILEGES {
        PrivilegeCount: 1,
        Privileges: [LUID_AND_ATTRIBUTES {
            Luid: LUID {
                HighPart: 0,
                LowPart: 0,
            },
            Attributes: SE_PRIVILEGE_ENABLED,
        }],
    };

    if unsafe {
        LookupPrivilegeValueW(
            std::ptr::null(),
            SE_SHUTDOWN_NAME,
            &raw mut privileges.Privileges[0].Luid,
        )
    } == 0
    {
        log::error!(
            "Failed to lookup shutdown privilege LUID: {}",
            io::Error::last_os_error()
        );
        return Err(test_rpc::Error::Syscall);
    }

    let mut token_handle: HANDLE = 0;

    if unsafe {
        OpenProcessToken(
            GetCurrentProcess(),
            TOKEN_ADJUST_PRIVILEGES,
            &raw mut token_handle,
        )
    } == 0
    {
        log::error!("OpenProcessToken() failed: {}", io::Error::last_os_error());
        return Err(test_rpc::Error::Syscall);
    }

    let result = unsafe {
        AdjustTokenPrivileges(
            token_handle,
            0,
            &raw const privileges,
            0,
            std::ptr::null_mut(),
            std::ptr::null_mut(),
        )
    };

    unsafe { CloseHandle(token_handle) };

    if result == 0 {
        log::error!(
            "Failed to enable SE_SHUTDOWN_NAME: {}",
            io::Error::last_os_error()
        );
        return Err(test_rpc::Error::Syscall);
    }

    Ok(())
}

/// Restart the Mullvad VPN application.
///
/// This function waits for the app to successfully start again.
pub async fn restart_app() -> Result<(), test_rpc::Error> {
    stop_app().await?;
    start_app().await?;
    Ok(())
}

/// Stop the Mullvad VPN application.
///
/// This function waits for the app to successfully shut down.
pub async fn stop_app() -> Result<(), test_rpc::Error> {
    let _ = tokio::process::Command::new("net")
        .args(["stop", "mullvadvpn"])
        .status()
        .await
        .map_err(|e| test_rpc::Error::ServiceStop(e.to_string()))?;
    Ok(())
}

/// Start the Mullvad VPN application.
///
/// This function waits for the app to successfully start again.
pub async fn start_app() -> Result<(), test_rpc::Error> {
    let _ = tokio::process::Command::new("net")
        .args(["start", "mullvadvpn"])
        .status()
        .await
        .map_err(|e| test_rpc::Error::ServiceStart(e.to_string()))?;
    Ok(())
}

/// Disable the Mullvad VPN system service startup. This will not trigger the service to stop
/// immediately, but it will prevent it from starting on the next system boot.
pub async fn disable_system_service_startup() -> Result<(), test_rpc::Error> {
    let status = tokio::process::Command::new("powershell")
        .args([
            "-NoProfile",
            "-Command",
            "Set-Service -Name MullvadVPN -StartupType Disabled",
        ])
        .status()
        .await
        .map_err(|e| test_rpc::Error::ServiceChange(e.to_string()))?;

    if !status.success() {
        return Err(test_rpc::Error::ServiceChange(
            "Failed to disable MullvadVPN service".to_string(),
        ));
    }

    Ok(())
}

/// Enable the Mullvad VPN system service startup. This will configure the service to start automatically on system boot.
pub async fn enable_system_service_startup() -> Result<(), test_rpc::Error> {
    let status = tokio::process::Command::new("powershell")
        .args([
            "-NoProfile",
            "-Command",
            "Set-Service -Name MullvadVPN -StartupType Automatic",
        ])
        .status()
        .await
        .map_err(|e| test_rpc::Error::ServiceChange(e.to_string()))?;

    if !status.success() {
        return Err(test_rpc::Error::ServiceChange(
            "Failed to enable MullvadVPN service".to_string(),
        ));
    }

    Ok(())
}

pub fn get_daemon_system_service_status() -> Result<ServiceStatus, test_rpc::Error> {
    let manager = ServiceManager::local_computer(None::<&str>, ServiceManagerAccess::CONNECT)
        .map_err(|e| test_rpc::Error::ServiceNotFound(e.to_string()))?;
    let service = manager
        .open_service("mullvadvpn", ServiceAccess::QUERY_STATUS)
        .map_err(|e| test_rpc::Error::ServiceNotFound(e.to_string()))?;

    get_daemon_system_service_status_inner(&service)
}

fn get_daemon_system_service_status_inner(
    service: &Service,
) -> Result<ServiceStatus, test_rpc::Error> {
    let status = service
        .query_status()
        .map_err(|e| test_rpc::Error::Other(e.to_string()))?;

    let status = match status.current_state {
        ServiceState::Running => ServiceStatus::Running,
        // NOTE: not counting pending start as running, since we cannot set log level then
        _ => ServiceStatus::NotRunning,
    };

    Ok(status)
}

async fn wait_for_service_status(
    service: &Service,
    accept_fn: impl Fn(&windows_service::service::ServiceStatus) -> bool,
) -> Result<(), test_rpc::Error> {
    const MAX_ATTEMPTS: usize = 10;
    const POLL_INTERVAL: std::time::Duration = std::time::Duration::from_secs(3);

    for _ in 0..MAX_ATTEMPTS {
        let status = service
            .query_status()
            .map_err(|e| test_rpc::Error::Other(e.to_string()))?;
        if accept_fn(&status) {
            return Ok(());
        }
        tokio::time::sleep(POLL_INTERVAL).await;
    }
    Err(test_rpc::Error::ServiceStart(
        "Awaiting new service state timed out".to_string(),
    ))
}

pub async fn set_daemon_log_level(verbosity_level: Verbosity) -> Result<(), test_rpc::Error> {
    use std::error::Error;

    fn error_with_source(e: &impl Error) -> String {
        if let Some(source) = e.source() {
            format!("{e}: {source}")
        } else {
            e.to_string()
        }
    }

    log::debug!("Setting log level");

    let verbosity = match verbosity_level {
        Verbosity::Info => "",
        Verbosity::Debug => "-v",
        Verbosity::Trace => "-vv",
    };

    let manager = ServiceManager::local_computer(None::<&str>, ServiceManagerAccess::CONNECT)
        .map_err(|e| test_rpc::Error::ServiceNotFound(e.to_string()))?;
    let service = manager
        .open_service(
            "mullvadvpn",
            ServiceAccess::QUERY_CONFIG
                | ServiceAccess::QUERY_STATUS
                | ServiceAccess::CHANGE_CONFIG
                | ServiceAccess::START
                | ServiceAccess::STOP,
        )
        .map_err(|e| {
            test_rpc::Error::ServiceNotFound(format!(
                "Failed to open service: {}",
                error_with_source(&e)
            ))
        })?;

    log::info!("Stopping service");

    // Stop the service
    service
        .stop()
        .map_err(|e| test_rpc::Error::ServiceStop(e.to_string()))?;

    // Wait until the service is fully stopped
    wait_for_service_status(&service, |status| {
        status.current_state == ServiceState::Stopped
    })
    .await?;

    // Get the current service configuration
    let config = service.query_config().map_err(|e| {
        test_rpc::Error::ServiceNotFound(format!(
            "Failed to query service config: {}",
            error_with_source(&e)
        ))
    })?;

    let executable_path = "C:\\Program Files\\Mullvad VPN\\resources\\mullvad-daemon.exe";
    let launch_arguments = vec![
        OsString::from("--run-as-service"),
        OsString::from(verbosity),
    ];

    // Update the service binary arguments
    let updated_config = ServiceInfo {
        name: config.display_name.clone(),
        display_name: config.display_name.clone(),
        service_type: config.service_type,
        start_type: config.start_type,
        error_control: config.error_control,
        executable_path: std::path::PathBuf::from(executable_path),
        launch_arguments,
        dependencies: config.dependencies.clone(),
        account_name: config.account_name.clone(),
        account_password: None,
    };

    // Apply the updated configuration
    service.change_config(&updated_config).map_err(|e| {
        test_rpc::Error::ServiceChange(format!("Update service config: {}", error_with_source(&e)))
    })?;

    // Start the service
    service.start::<String>(&[]).map_err(|e| {
        test_rpc::Error::ServiceNotFound(format!(
            "Failed to start service: {}",
            error_with_source(&e)
        ))
    })?;

    // Wait until the service is fully started
    wait_for_service_status(&service, |status| {
        status.current_state == ServiceState::Running
    })
    .await?;

    Ok(())
}

pub async fn set_daemon_environment(env: HashMap<String, String>) -> Result<(), test_rpc::Error> {
    // Set environment globally (not for service) to prevent it from being lost on upgrade
    for (k, v) in env.clone() {
        tokio::process::Command::new("setx")
            .arg("/m")
            .args([k, v])
            .status()
            .await
            .map_err(|e| test_rpc::Error::Registry(e.to_string()))?;
    }
    // Persist the changed environment variables, such that we can retrieve them at will.
    use winreg::{RegKey, enums::*};
    let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
    let path = Path::new(MULLVAD_WIN_REGISTRY).join("Environment");
    let (registry, _) = hklm.create_subkey(&path).map_err(|error| {
        test_rpc::Error::Registry(format!("Failed to open Mullvad VPN subkey: {error}"))
    })?;
    for (k, v) in env {
        registry.set_value(k, &v).map_err(|error| {
            test_rpc::Error::Registry(format!("Failed to set Environment var: {error}"))
        })?;
    }

    // Restart service
    stop_app().await?;
    start_app().await?;

    Ok(())
}

pub fn get_system_path_var() -> Result<String, test_rpc::Error> {
    use winreg::{enums::*, *};

    let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
    let key = hklm
        .open_subkey("SYSTEM\\CurrentControlSet\\Control\\Session Manager\\Environment")
        .map_err(|error| {
            test_rpc::Error::Registry(format!("Failed to open Environment subkey: {error}"))
        })?;

    let path: String = key
        .get_value("Path")
        .map_err(|error| test_rpc::Error::Registry(format!("Failed to get PATH: {error}")))?;

    Ok(path)
}

pub async fn get_daemon_environment() -> Result<HashMap<String, String>, test_rpc::Error> {
    use winreg::{RegKey, enums::*};

    let env =
        tokio::task::spawn_blocking(|| -> Result<HashMap<String, String>, test_rpc::Error> {
            let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
            let key = hklm.open_subkey(MULLVAD_WIN_REGISTRY).map_err(|error| {
                test_rpc::Error::Registry(format!("Failed to open Mullvad VPN subkey: {error}"))
            })?;

            // The Strings will be quoted (surrounded by ") when read from the registry - we should
            // trim that!
            let trim = |string: String| string.trim_matches('"').to_owned();
            let env = key
                .open_subkey("Environment")
                .map_err(|error| {
                    test_rpc::Error::Registry(
                        format!("Failed to open Environment subkey: {error}",),
                    )
                })?
                .enum_values()
                .filter_map(|x| x.inspect_err(|err| log::trace!("{err}")).ok())
                .map(|(k, v)| (trim(k), trim(v.to_string())))
                .collect();
            Ok(env)
        })
        .await
        .map_err(test_rpc::Error::from_tokio_join_error)??;

    Ok(env)
}

pub fn get_os_version() -> Result<OsVersion, test_rpc::Error> {
    use test_rpc::meta::WindowsVersion;

    let version = talpid_platform_metadata::WindowsVersion::new()
        .inspect_err(|error| {
            log::error!("Failed to obtain OS version: {error}");
        })
        .map_err(|_| test_rpc::Error::Syscall)?;

    Ok(OsVersion::Windows(WindowsVersion {
        major: version.release_version().0,
    }))
}

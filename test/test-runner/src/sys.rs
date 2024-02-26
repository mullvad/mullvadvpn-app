use std::collections::HashMap;
#[cfg(target_os = "windows")]
use std::io;
use test_rpc::mullvad_daemon::Verbosity;

#[cfg(target_os = "windows")]
use std::ffi::OsString;
#[cfg(target_os = "windows")]
use windows_service::{
    service::{ServiceAccess, ServiceInfo},
    service_manager::{ServiceManager, ServiceManagerAccess},
};

#[cfg(target_os = "windows")]
pub fn reboot() -> Result<(), test_rpc::Error> {
    use windows_sys::Win32::System::Shutdown::{
        ExitWindowsEx, EWX_REBOOT, SHTDN_REASON_FLAG_PLANNED, SHTDN_REASON_MAJOR_APPLICATION,
        SHTDN_REASON_MINOR_OTHER,
    };
    use windows_sys::Win32::UI::WindowsAndMessaging::EWX_FORCEIFHUNG;

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

#[cfg(target_os = "windows")]
fn grant_shutdown_privilege() -> Result<(), test_rpc::Error> {
    use windows_sys::Win32::Foundation::CloseHandle;
    use windows_sys::Win32::Foundation::HANDLE;
    use windows_sys::Win32::Foundation::LUID;
    use windows_sys::Win32::Security::AdjustTokenPrivileges;
    use windows_sys::Win32::Security::LookupPrivilegeValueW;
    use windows_sys::Win32::Security::LUID_AND_ATTRIBUTES;
    use windows_sys::Win32::Security::SE_PRIVILEGE_ENABLED;
    use windows_sys::Win32::Security::TOKEN_ADJUST_PRIVILEGES;
    use windows_sys::Win32::Security::TOKEN_PRIVILEGES;
    use windows_sys::Win32::System::SystemServices::SE_SHUTDOWN_NAME;
    use windows_sys::Win32::System::Threading::GetCurrentProcess;
    use windows_sys::Win32::System::Threading::OpenProcessToken;

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
            &mut privileges.Privileges[0].Luid,
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
            &mut token_handle,
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
            &privileges,
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

#[cfg(unix)]
pub fn reboot() -> Result<(), test_rpc::Error> {
    log::debug!("Rebooting system");

    std::thread::spawn(|| {
        #[cfg(target_os = "linux")]
        let mut cmd = std::process::Command::new("/usr/sbin/shutdown");
        #[cfg(target_os = "macos")]
        let mut cmd = std::process::Command::new("/sbin/shutdown");
        cmd.args(["-r", "now"]);

        std::thread::sleep(std::time::Duration::from_secs(5));

        let _ = cmd.spawn().map_err(|error| {
            log::error!("Failed to spawn shutdown command: {error}");
            error
        });
    });

    Ok(())
}

#[cfg(target_os = "linux")]
pub async fn set_daemon_log_level(verbosity_level: Verbosity) -> Result<(), test_rpc::Error> {
    use tokio::io::AsyncWriteExt;
    const SYSTEMD_OVERRIDE_FILE: &str =
        "/etc/systemd/system/mullvad-daemon.service.d/override.conf";

    log::debug!("Setting log level");

    let verbosity = match verbosity_level {
        Verbosity::Info => "",
        Verbosity::Debug => "-v",
        Verbosity::Trace => "-vv",
    };
    let systemd_service_file_content = format!(
        r#"[Service]
ExecStart=
ExecStart=/usr/bin/mullvad-daemon --disable-stdout-timestamps {verbosity}"#
    );

    let override_path = std::path::Path::new(SYSTEMD_OVERRIDE_FILE);
    if let Some(parent) = override_path.parent() {
        tokio::fs::create_dir_all(parent)
            .await
            .map_err(|e| test_rpc::Error::Service(e.to_string()))?;
    }

    let mut file = tokio::fs::OpenOptions::new()
        .create(true)
        .truncate(true)
        .write(true)
        .open(override_path)
        .await
        .map_err(|e| test_rpc::Error::Service(e.to_string()))?;

    file.write_all(systemd_service_file_content.as_bytes())
        .await
        .map_err(|e| test_rpc::Error::Service(e.to_string()))?;

    tokio::process::Command::new("systemctl")
        .args(["daemon-reload"])
        .status()
        .await
        .map_err(|e| test_rpc::Error::Service(e.to_string()))?;

    restart_app().await?;
    Ok(())
}

/// Restart the Mullvad VPN application.
///
/// This function waits for the app to successfully start again.
#[cfg(target_os = "linux")]
pub async fn restart_app() -> Result<(), test_rpc::Error> {
    tokio::process::Command::new("systemctl")
        .args(["restart", "mullvad-daemon"])
        .status()
        .await
        .map_err(|e| test_rpc::Error::Service(e.to_string()))?;
    wait_for_service_state(ServiceState::Running).await?;
    Ok(())
}

/// Stop the Mullvad VPN application.
///
/// This function waits for the app to successfully shut down.
#[cfg(target_os = "linux")]
pub async fn stop_app() -> Result<(), test_rpc::Error> {
    tokio::process::Command::new("systemctl")
        .args(["stop", "mullvad-daemon"])
        .status()
        .await
        .map_err(|e| test_rpc::Error::Service(e.to_string()))?;
    wait_for_service_state(ServiceState::Inactive).await?;

    Ok(())
}

/// Start the Mullvad VPN application.
///
/// This function waits for the app to successfully start again.
#[cfg(target_os = "linux")]
pub async fn start_app() -> Result<(), test_rpc::Error> {
    tokio::process::Command::new("systemctl")
        .args(["start", "mullvad-daemon"])
        .status()
        .await
        .map_err(|e| test_rpc::Error::Service(e.to_string()))?;
    wait_for_service_state(ServiceState::Running).await?;
    Ok(())
}

/// Restart the Mullvad VPN application.
///
/// This function waits for the app to successfully start again.
#[cfg(target_os = "windows")]
pub async fn restart_app() -> Result<(), test_rpc::Error> {
    stop_app().await?;
    start_app().await?;
    Ok(())
}

/// Stop the Mullvad VPN application.
///
/// This function waits for the app to successfully shut down.
#[cfg(target_os = "windows")]
pub async fn stop_app() -> Result<(), test_rpc::Error> {
    let _ = tokio::process::Command::new("net")
        .args(["stop", "mullvadvpn"])
        .status()
        .await
        .map_err(|e| test_rpc::Error::Service(e.to_string()))?;
    Ok(())
}

/// Start the Mullvad VPN application.
///
/// This function waits for the app to successfully start again.
#[cfg(target_os = "windows")]
pub async fn start_app() -> Result<(), test_rpc::Error> {
    let _ = tokio::process::Command::new("net")
        .args(["start", "mullvadvpn"])
        .status()
        .await
        .map_err(|e| test_rpc::Error::Service(e.to_string()))?;
    Ok(())
}

/// Restart the Mullvad VPN application.
///
/// This function waits for the app to successfully start again.
#[cfg(target_os = "macos")]
pub async fn restart_app() -> Result<(), test_rpc::Error> {
    stop_app().await?;
    start_app().await?;
    Ok(())
}

/// Stop the Mullvad VPN application.
///
/// This function waits for the app to successfully shut down.
#[cfg(target_os = "macos")]
pub async fn stop_app() -> Result<(), test_rpc::Error> {
    set_launch_daemon_state(false).await?;
    tokio::time::sleep(std::time::Duration::from_millis(1000)).await;
    Ok(())
}

/// Start the Mullvad VPN application.
///
/// This function waits for the app to successfully start again.
#[cfg(target_os = "macos")]
pub async fn start_app() -> Result<(), test_rpc::Error> {
    set_launch_daemon_state(true).await?;
    tokio::time::sleep(std::time::Duration::from_millis(1000)).await;
    Ok(())
}

#[cfg(target_os = "windows")]
pub async fn set_daemon_log_level(verbosity_level: Verbosity) -> Result<(), test_rpc::Error> {
    log::debug!("Setting log level");

    let verbosity = match verbosity_level {
        Verbosity::Info => "",
        Verbosity::Debug => "-v",
        Verbosity::Trace => "-vv",
    };

    let manager = ServiceManager::local_computer(None::<&str>, ServiceManagerAccess::CONNECT)
        .map_err(|e| test_rpc::Error::Service(e.to_string()))?;
    let service = manager
        .open_service(
            "mullvadvpn",
            ServiceAccess::QUERY_CONFIG
                | ServiceAccess::CHANGE_CONFIG
                | ServiceAccess::START
                | ServiceAccess::STOP,
        )
        .map_err(|e| test_rpc::Error::Service(e.to_string()))?;

    // Stop the service
    // TODO: Extract to separate function.
    service
        .stop()
        .map_err(|e| test_rpc::Error::Service(e.to_string()))?;
    tokio::process::Command::new("net")
        .args(["stop", "mullvadvpn"])
        .status()
        .await
        .map_err(|e| test_rpc::Error::Service(e.to_string()))?;

    // Get the current service configuration
    let config = service
        .query_config()
        .map_err(|e| test_rpc::Error::Service(e.to_string()))?;

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
    service
        .change_config(&updated_config)
        .map_err(|e| test_rpc::Error::Service(e.to_string()))?;

    // Start the service
    // TODO: Extract to separate function.
    service
        .start::<String>(&[])
        .map_err(|e| test_rpc::Error::Service(e.to_string()))?;

    Ok(())
}

#[cfg(target_os = "macos")]
#[allow(clippy::unused_async)]
pub async fn set_daemon_log_level(_verbosity_level: Verbosity) -> Result<(), test_rpc::Error> {
    // TODO: Not implemented
    log::warn!("Setting log level is not implemented on macOS");
    Ok(())
}

#[cfg(target_os = "linux")]
pub async fn set_daemon_environment(env: HashMap<String, String>) -> Result<(), test_rpc::Error> {
    use std::fmt::Write;
    use tokio::io::AsyncWriteExt;

    const SYSTEMD_OVERRIDE_FILE: &str = "/etc/systemd/system/mullvad-daemon.service.d/env.conf";

    let mut override_content = String::new();
    override_content.push_str("[Service]\n");

    for (k, v) in env {
        writeln!(&mut override_content, "Environment=\"{k}={v}\"").unwrap();
    }

    let override_path = std::path::Path::new(SYSTEMD_OVERRIDE_FILE);
    if let Some(parent) = override_path.parent() {
        tokio::fs::create_dir_all(parent)
            .await
            .map_err(|e| test_rpc::Error::Service(e.to_string()))?;
    }

    let mut file = tokio::fs::OpenOptions::new()
        .create(true)
        .truncate(true)
        .write(true)
        .open(override_path)
        .await
        .map_err(|e| test_rpc::Error::Service(e.to_string()))?;

    file.write_all(override_content.as_bytes())
        .await
        .map_err(|e| test_rpc::Error::Service(e.to_string()))?;

    tokio::process::Command::new("systemctl")
        .args(["daemon-reload"])
        .status()
        .await
        .map_err(|e| test_rpc::Error::Service(e.to_string()))?;

    tokio::process::Command::new("systemctl")
        .args(["restart", "mullvad-daemon"])
        .status()
        .await
        .map_err(|e| test_rpc::Error::Service(e.to_string()))?;

    wait_for_service_state(ServiceState::Running).await?;
    Ok(())
}

#[cfg(target_os = "windows")]
pub async fn set_daemon_environment(env: HashMap<String, String>) -> Result<(), test_rpc::Error> {
    // Set environment globally (not for service) to prevent it from being lost on upgrade
    for (k, v) in env {
        tokio::process::Command::new("setx")
            .arg("/m")
            .args([k, v])
            .status()
            .await
            .map_err(|e| test_rpc::Error::Registry(e.to_string()))?;
    }

    // Restart service
    stop_app().await?;
    start_app().await?;

    Ok(())
}

#[cfg(target_os = "windows")]
pub fn get_system_path_var() -> Result<String, test_rpc::Error> {
    use winreg::enums::*;
    use winreg::*;

    let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
    let key = hklm
        .open_subkey("SYSTEM\\CurrentControlSet\\Control\\Session Manager\\Environment")
        .map_err(|error| {
            test_rpc::Error::Registry(format!("Failed to open environment subkey: {}", error))
        })?;

    let path: String = key
        .get_value("Path")
        .map_err(|error| test_rpc::Error::Registry(format!("Failed to get PATH: {}", error)))?;

    Ok(path)
}

#[cfg(target_os = "macos")]
pub async fn set_daemon_environment(env: HashMap<String, String>) -> Result<(), test_rpc::Error> {
    const PLIST_PATH: &str = "/Library/LaunchDaemons/net.mullvad.daemon.plist";

    tokio::task::spawn_blocking(|| {
        let mut parsed_plist: plist::Value = plist::from_file(PLIST_PATH)
            .map_err(|error| test_rpc::Error::Service(format!("failed to parse plist: {error}")))?;

        let mut vars = plist::Dictionary::new();

        for (k, v) in env {
            // Set environment globally (not for service) to prevent it from being lost on upgrade
            std::process::Command::new("launchctl")
                .arg("setenv")
                .args([&k, &v])
                .status()
                .map_err(|e| test_rpc::Error::Service(e.to_string()))?;
            vars.insert(k, plist::Value::String(v));
        }

        // Add permanent env var
        parsed_plist
            .as_dictionary_mut()
            .ok_or_else(|| test_rpc::Error::Service("plist missing dict".to_owned()))?
            .insert(
                "EnvironmentVariables".to_owned(),
                plist::Value::Dictionary(vars),
            );

        let daemon_plist = std::fs::OpenOptions::new()
            .write(true)
            .truncate(true)
            .open(PLIST_PATH)
            .map_err(|e| test_rpc::Error::Service(format!("failed to open plist: {e}")))?;

        parsed_plist
            .to_writer_xml(daemon_plist)
            .map_err(|e| test_rpc::Error::Service(format!("failed to replace plist: {e}")))?;

        Ok::<(), test_rpc::Error>(())
    })
    .await
    .unwrap()?;

    // Restart service
    set_launch_daemon_state(false).await?;
    tokio::time::sleep(std::time::Duration::from_millis(1000)).await;
    set_launch_daemon_state(true).await?;
    tokio::time::sleep(std::time::Duration::from_millis(1000)).await;
    Ok(())
}

#[cfg(target_os = "macos")]
async fn set_launch_daemon_state(on: bool) -> Result<(), test_rpc::Error> {
    tokio::process::Command::new("launchctl")
        .args([
            if on { "load" } else { "unload" },
            "-w",
            "/Library/LaunchDaemons/net.mullvad.daemon.plist",
        ])
        .status()
        .await
        .map_err(|e| test_rpc::Error::Service(e.to_string()))?;
    Ok(())
}

#[cfg(target_os = "linux")]
enum ServiceState {
    Running,
    Inactive,
}

#[cfg(target_os = "linux")]
async fn wait_for_service_state(awaited_state: ServiceState) -> Result<(), test_rpc::Error> {
    const RETRY_ATTEMPTS: usize = 10;
    let mut attempt = 0;
    loop {
        attempt += 1;
        if attempt > RETRY_ATTEMPTS {
            return Err(test_rpc::Error::Service(String::from(
                "Awaiting new service state timed out",
            )));
        }

        let output = tokio::process::Command::new("systemctl")
            .args(["status", "mullvad-daemon"])
            .output()
            .await
            .map_err(|e| test_rpc::Error::Service(e.to_string()))?
            .stdout;
        let output = String::from_utf8_lossy(&output);

        match awaited_state {
            ServiceState::Running => {
                if output.contains("active (running)") {
                    break;
                }
            }
            ServiceState::Inactive => {
                if output.contains("inactive (dead)") {
                    break;
                }
            }
        }

        tokio::time::sleep(std::time::Duration::from_millis(1000)).await;
    }
    Ok(())
}

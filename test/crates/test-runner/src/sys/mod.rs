use std::path::Path;
use test_rpc::mullvad_daemon::SOCKET_PATH;
use test_rpc::mullvad_daemon::ServiceStatus;

#[cfg(target_os = "windows")]
mod windows;

#[cfg(target_os = "windows")]
pub use windows::*;

#[cfg(target_os = "linux")]
mod linux;

#[cfg(target_os = "linux")]
pub use linux::*;

#[cfg(target_os = "macos")]
mod macos;

#[cfg(target_os = "macos")]
pub use macos::*;

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

pub fn get_daemon_status() -> ServiceStatus {
    let rpc_socket_exists = Path::new(SOCKET_PATH).exists();

    // On Windows, we must also make sure service isn't in a pending state, since interacting with
    // the service may fail even if there is a working named pipe.
    #[cfg(target_os = "windows")]
    let service_is_started =
        get_daemon_system_service_status().unwrap_or(ServiceStatus::NotRunning);

    // NOTE: May not be necessary on non-Windows
    #[cfg(not(target_os = "windows"))]
    let service_is_started = ServiceStatus::Running;

    match (rpc_socket_exists, service_is_started) {
        (true, ServiceStatus::Running) => ServiceStatus::Running,
        _ => ServiceStatus::NotRunning,
    }
}

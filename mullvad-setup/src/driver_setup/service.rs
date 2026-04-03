use std::{
    thread,
    time::{Duration, Instant},
};
use windows_service::{
    service::{ServiceAccess, ServiceState},
    service_manager::{ServiceManager, ServiceManagerAccess},
};
use windows_sys::Win32::Foundation::ERROR_SERVICE_DOES_NOT_EXIST;

const MAX_WAIT: Duration = Duration::from_secs(5);
const POLL_INTERVAL: Duration = Duration::from_millis(100);

/// Returns `true` if the named service exists and is currently in the running state.
pub fn service_is_running(name: &str) -> Result<bool, windows_service::Error> {
    let scm = ServiceManager::local_computer(None::<&str>, ServiceManagerAccess::CONNECT)?;

    let service = match scm.open_service(name, ServiceAccess::QUERY_STATUS) {
        Ok(s) => s,
        Err(windows_service::Error::Winapi(e))
            if e.raw_os_error() == Some(ERROR_SERVICE_DOES_NOT_EXIST as i32) =>
        {
            return Ok(false);
        }
        Err(e) => return Err(e),
    };

    let status = service.query_status()?;
    Ok(status.current_state == ServiceState::Running)
}

/// Stop and delete the named service. Does nothing if the service does not exist.
/// May block the current thread for a prolonged duration defined by [`MAX_WAIT`].
pub fn stop_and_delete_service(name: &str) -> Result<(), windows_service::Error> {
    let scm = ServiceManager::local_computer(None::<&str>, ServiceManagerAccess::CONNECT)?;

    let service = match scm.open_service(
        name,
        ServiceAccess::STOP | ServiceAccess::QUERY_STATUS | ServiceAccess::DELETE,
    ) {
        Ok(s) => s,
        Err(windows_service::Error::Winapi(e))
            if e.raw_os_error() == Some(ERROR_SERVICE_DOES_NOT_EXIST as i32) =>
        {
            return Ok(());
        }
        Err(e) => return Err(e),
    };

    // Attempt to stop the service (ignore errors — it may already be stopped)
    let _ = service.stop();

    // Wait up to MAX_WAIT for it to stop
    let deadline = Instant::now() + MAX_WAIT;
    loop {
        match service.query_status() {
            Ok(status) if status.current_state == ServiceState::Stopped => break,
            _ => {}
        }
        if Instant::now() >= deadline {
            break;
        }
        thread::sleep(POLL_INTERVAL);
    }

    service.delete()
}

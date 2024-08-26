#![cfg(target_os = "linux")]
//! DBus system connection
pub use dbus;
use dbus::blocking::SyncConnection;
use std::sync::{Arc, LazyLock, Mutex};
pub mod network_manager;
pub mod systemd;
pub mod systemd_resolved;

static DBUS_CONNECTION: LazyLock<Mutex<Option<Arc<SyncConnection>>>> =
    LazyLock::new(|| Mutex::new(None));

/// Reuse or create a system DBus connection.
pub fn get_connection() -> Result<Arc<SyncConnection>, dbus::Error> {
    let mut connection = DBUS_CONNECTION.lock().expect("DBus lock poisoned");
    match &*connection {
        Some(existing_connection) => Ok(existing_connection.clone()),
        None => {
            let new_connection = Arc::new(SyncConnection::new_system()?);
            *connection = Some(new_connection.clone());
            Ok(new_connection)
        }
    }
}

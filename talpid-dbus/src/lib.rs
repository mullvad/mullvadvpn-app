//! DBus system connection
#![cfg(target_os = "linux")]

#[cfg(feature = "libdbus")] // TODO: implement network_manager using zbus
pub mod network_manager;
pub mod systemd;
pub mod systemd_resolved;

#[cfg(feature = "libdbus")]
pub use dbus;

#[cfg(feature = "libdbus")]
use std::sync::{Arc, LazyLock, Mutex};

/// Reuse or create a system DBus connection.
#[cfg(feature = "libdbus")]
pub fn get_connection() -> Result<Arc<dbus::blocking::SyncConnection>, dbus::Error> {
    use dbus::blocking::SyncConnection;

    static DBUS_CONNECTION: LazyLock<Mutex<Option<Arc<SyncConnection>>>> =
        LazyLock::new(|| Mutex::new(None));

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

#[cfg(feature = "zbus")]
/// Create a system DBus connection to the system-wide message bus.
pub fn get_connection_zbus() -> Result<zbus::blocking::Connection, zbus::Error> {
    // Create the socket once and then clone seems to be the deal with zbus.
    // No need to go super-fancy with the LazyLock<Mutex<..> shennanings.
    let system = zbus::blocking::Connection::system()?;
    Ok(system)
}

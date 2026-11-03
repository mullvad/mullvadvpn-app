//! DBus system connection
#![cfg(target_os = "linux")]

pub mod network_manager;
pub mod systemd;
pub mod systemd_resolved;

/// Create a system DBus connection to the system-wide message bus.
pub async fn get_connection_zbus() -> Result<zbus::Connection, zbus::Error> {
    // Create the socket once and then clone seems to be the deal with zbus.
    // No need to go super-fancy with the LazyLock<Mutex<..> shennanings.
    let system = zbus::Connection::system().await?;
    Ok(system)
}

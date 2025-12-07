#![cfg(target_os = "linux")]
//! DBus system connection
pub mod network_manager;
pub mod systemd;
pub mod systemd_resolved;

pub use zbus;
use zbus::blocking::Connection;

/// Reuse or create a system DBus connection to the system-wide message bus.
pub fn get_connection() -> Result<Connection, zbus::Error> {
    // TODO: Cache? Or I don't think it matters. Create it once and then clone seems to be the deal
    // with zbus. No need to go super-fancy with the LazyLock<Mutex<..> shennanings.
    let system = Connection::system()?;
    Ok(system)
}

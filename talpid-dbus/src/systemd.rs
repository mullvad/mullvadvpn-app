use zbus::blocking::{Connection, Proxy}; // TODO: async

use crate::get_connection;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Failed to create a DBus connection")]
    ConnectError(#[source] zbus::Error),

    #[error("Failed to read SystemState property")]
    ReadSystemStateError(#[source] zbus::Error),
}

// TODO: Maybe this is important, maybe it's not. I don't think it is if we make this module async.
// const RPC_TIMEOUT: std::duraction::Duration = std::duration::Duration::from_secs(1);

/// Returns true if the host is not shutting down or entering maintenance mode or some other weird
/// state.
pub fn is_host_running() -> Result<bool, Error> {
    Systemd::new()?
        .system_is_running()
        .map_err(Error::ReadSystemStateError)
}

struct Systemd {
    pub dbus_connection: Connection,
}

// TODO: Use proxy macro from zbus crate.
impl Systemd {
    /// Create a new systemd manager.
    fn new() -> Result<Self, Error> {
        Ok(Self {
            dbus_connection: get_connection().map_err(Error::ConnectError)?,
        })
    }

    /// TODO: Document me.
    fn system_is_running(&self) -> Result<bool, zbus::Error> {
        self.as_manager_object()?
            .get_property("SystemState")
            .map(|state: String| {
                !["starting", "initializing", "running", "degraded"].contains(&state.as_str())
            })
    }

    /// TODO: Document me.
    fn as_manager_object(&self) -> Result<Proxy<'_>, zbus::Error> {
        Proxy::new(
            &self.dbus_connection,
            "org.freedesktop.systemd1",
            "/org/freedesktop/systemd1",
            "org.freedesktop.systemd1.Manager",
        )
    }
}

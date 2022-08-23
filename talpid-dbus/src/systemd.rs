use dbus::blocking::{stdintf::org_freedesktop_dbus::Properties, Proxy, SyncConnection};
use std::{sync::Arc, time::Duration};

type Result<T> = std::result::Result<T, Error>;

#[derive(err_derive::Error, Debug)]
#[error(no_from)]
pub enum Error {
    #[error(display = "Failed to create a DBus connection")]
    ConnectError(#[error(source)] dbus::Error),

    #[error(display = "Failed to read SystemState property")]
    ReadSystemStateError(#[error(source)] dbus::Error),
}

const SYSTEMD_BUS: &str = "org.freedesktop.systemd1";
const SYSTEMD_PATH: &str = "/org/freedesktop/systemd1";
const MANAGER_INTERFACE: &str = "org.freedesktop.systemd1.Manager";
const SYSTEM_STATE: &str = "SystemState";
const SYSTEM_STATE_STOPPING: &str = "stopping";

const RPC_TIMEOUT: Duration = Duration::from_secs(1);

pub fn is_host_shutting_down() -> Result<bool> {
    Systemd::new()?.system_is_shutting_down()
}

struct Systemd {
    pub dbus_connection: Arc<SyncConnection>,
}

impl Systemd {
    fn new() -> Result<Self> {
        Ok(Self {
            dbus_connection: crate::get_connection().map_err(Error::ConnectError)?,
        })
    }

    fn system_is_shutting_down(&self) -> Result<bool> {
        self.as_manager_object()
            .get(MANAGER_INTERFACE, SYSTEM_STATE)
            .map(|state: String| state == SYSTEM_STATE_STOPPING)
            .map_err(Error::ReadSystemStateError)
    }

    fn as_manager_object(&self) -> Proxy<'_, &SyncConnection> {
        Proxy::new(
            SYSTEMD_BUS,
            SYSTEMD_PATH,
            RPC_TIMEOUT,
            &self.dbus_connection,
        )
    }
}

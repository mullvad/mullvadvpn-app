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
const SYSTEM_STATE_STARTING: &str = "starting";
const SYSTEM_STATE_INITIALIZING: &str = "initializing";
const SYSTEM_STATE_RUNNING: &str = "running";
const SYSTEM_STATE_DEGRADED: &str = "degraded";

const RPC_TIMEOUT: Duration = Duration::from_secs(1);

/// Returns true if the host is not shutting down or entering maintenance mode or some other weird
/// state.
pub fn is_host_running() -> Result<bool> {
    Systemd::new()?.system_is_running()
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

    fn system_is_running(&self) -> Result<bool> {
        self.as_manager_object()
            .get(MANAGER_INTERFACE, SYSTEM_STATE)
            .map(|state: String| {
                ![
                    SYSTEM_STATE_STARTING,
                    SYSTEM_STATE_INITIALIZING,
                    SYSTEM_STATE_RUNNING,
                    SYSTEM_STATE_DEGRADED,
                ]
                .contains(&state.as_str())
            })
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

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Failed to create a DBus connection")]
    ConnectError(#[source] zbus::Error),

    #[error("Failed to read SystemState property")]
    ReadSystemStateError(#[source] zbus::Error),

    #[error("Failed to parse SystemState value: {0}")]
    ParseSystemStateError(#[source] zvariant::Error),
}

/// Returns true if the host is in any running state. I.e. returns false if the host is shutting down,
/// entering maintenance mode or is in some other weird state.
pub fn is_host_running() -> Result<bool, Error> {
    use SystemState::*;
    let connection = crate::get_connection_zbus().map_err(Error::ConnectError)?;
    let proxy = SystemdManagerProxyBlocking::new(&connection).map_err(Error::ConnectError)?;
    let state = proxy
        .system_state()
        .map_err(Error::ReadSystemStateError)?
        .parse()
        .map_err(Error::ParseSystemStateError)?;
    log::trace!("SystemState: {state:#?}");
    let running = matches!(state, Starting | Initializing | Running | Degraded);
    Ok(running)
}

/// <https://www.freedesktop.org/software/systemd/man/latest/org.freedesktop.systemd1>
#[zbus::proxy(
    interface = "org.freedesktop.systemd1.Manager",
    default_service = "org.freedesktop.systemd1",
    default_path = "/org/freedesktop/systemd1"
)]
trait SystemdManager {
    /// See [`SystemState`].
    #[zbus(property)]
    fn system_state(&self) -> Result<String, zbus::Error>;
}

/// The current state of the system manager.
///
/// NOTE: This type can not be used as the return value of `SystemdManager::system_state` just yet: <https://github.com/z-galaxy/zbus/issues/1233>.
#[derive(Debug, serde::Deserialize, zvariant::Type, zvariant::OwnedValue)]
#[zvariant(signature = "s")]
pub enum SystemState {
    /// The system is booting, and basic.target has not been reached yet.
    #[serde(rename = "initializing")]
    Initializing,
    /// The system is booting, and basic.target has been reached.
    #[serde(rename = "starting")]
    Starting,
    /// The system has finished booting, and no units are in the failed state.
    #[serde(rename = "running")]
    Running,
    /// The system has finished booting, but some units are in the failed state.
    #[serde(rename = "degraded")]
    Degraded,
    /// The system has finished booting, but it has been put in rescue or maintenance mode.
    #[serde(rename = "maintenance")]
    Maintenance,
    /// The system is shutting down.
    #[serde(rename = "stopping")]
    Stopping,
}

// TODO: No longer need to implement this manually once
// https://github.com/z-galaxy/zbus/issues/234#issuecomment-2016855248 is fixed.
impl std::str::FromStr for SystemState {
    type Err = zvariant::Error;

    fn from_str(dbus_value: &str) -> Result<Self, <Self as std::str::FromStr>::Err> {
        use zvariant::serialized::Context;
        use zvariant::{LE, to_bytes};
        let ctxt = Context::new_dbus(LE, 0);

        let encoded = to_bytes(ctxt, &dbus_value)?;
        let state = encoded.deserialize()?.0;
        Ok(state)
    }
}

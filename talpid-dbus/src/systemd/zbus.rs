#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Failed to create a DBus connection")]
    ConnectError(#[source] zbus::Error),

    #[error("Failed to read SystemState property")]
    ReadSystemStateError(#[source] zbus::Error),
}

/// Returns true if the host is in any running state. I.e. returns false if the host is shutting down,
/// entering maintenance mode or is in some other weird state.
pub fn is_host_running() -> Result<bool, Error> {
    use systemd1::manager::SystemState::*;
    let dbus = crate::get_connection_zbus().map_err(Error::ConnectError)?;
    let state = systemd1::manager::proxy(&dbus)
        .map_err(Error::ConnectError)?
        .system_state()
        .map_err(Error::ReadSystemStateError)?;
    let running = matches!(state, Starting | Initializing | Running | Degraded);
    Ok(running)
}

mod systemd1 {
    //! https://www.freedesktop.org/software/systemd/man/latest/org.freedesktop.systemd1
    const DESTINATION: &str = "org.freedesktop.systemd1";
    const PATH: &str = "/org/freedesktop/systemd1";
    pub mod manager {
        //! https://www.freedesktop.org/software/systemd/man/latest/org.freedesktop.systemd1#The%20Manager%20Object
        use serde::Deserialize;
        use zbus::blocking::{Connection, Proxy};
        use zvariant;

        use crate::systemd::zbus::systemd1;

        const INTERFACE: &str = "org.freedesktop.systemd1.Manager";

        pub struct Manager<'a> {
            proxy: Proxy<'a>,
        }

        /// Return a systemd1.Manager proxy.
        pub fn proxy(dbus: &Connection) -> Result<Manager<'_>, zbus::Error> {
            let proxy = Proxy::new(dbus, systemd1::DESTINATION, systemd1::PATH, INTERFACE)?;
            Ok(Manager { proxy })
        }

        impl Manager<'_> {
            pub fn system_state(&self) -> Result<SystemState, zbus::Error> {
                self.proxy.get_property("SystemState")
            }
        }

        /// The current state of the system manager.
        #[derive(Deserialize, zvariant::OwnedValue)]
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
    }
}

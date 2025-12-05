#[cfg(not(feature = "zbus"))]
mod dbus_rs;
#[cfg(feature = "zbus")]
mod zbus;

#[cfg(not(feature = "zbus"))]
pub use dbus_rs::*;
#[cfg(feature = "zbus")]
pub use zbus::*;

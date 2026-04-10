//! NetworkManager is the one-stop-shop of network configuration on Linux.

#[cfg(feature = "libdbus")]
pub mod dbus_rs;
#[cfg(feature = "zbus")]
pub mod zbus;

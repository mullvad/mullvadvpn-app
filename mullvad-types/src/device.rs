use crate::account::AccountToken;
use chrono::{DateTime, Utc};
#[cfg(target_os = "android")]
use jnix::IntoJava;
use serde::{Deserialize, Serialize};
use std::fmt;
use talpid_types::net::wireguard::PublicKey;

/// UUID for a device.
pub type DeviceId = String;

/// Human-readable device identifier.
pub type DeviceName = String;

/// Contains data for a device returned by the API.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[cfg_attr(target_os = "android", derive(IntoJava))]
#[cfg_attr(target_os = "android", jnix(package = "net.mullvad.mullvadvpn.model"))]
pub struct Device {
    pub id: DeviceId,
    pub name: DeviceName,
    #[cfg_attr(target_os = "android", jnix(map = "|key| *key.as_bytes()"))]
    pub pubkey: PublicKey,
    pub ports: Vec<DevicePort>,
    #[cfg_attr(target_os = "android", jnix(skip))]
    pub hijack_dns: bool,
    #[cfg_attr(target_os = "android", jnix(map = "|expiry| expiry.to_string()"))]
    pub created: DateTime<Utc>,
}

impl Device {
    /// Return name with each word capitalized: "Happy Seagull" instead of "happy seagull"
    pub fn pretty_name(&self) -> String {
        self.name
            .split_whitespace()
            .map(|word| {
                let mut chars = word.chars();
                match chars.next() {
                    None => String::new(),
                    Some(c) => c.to_uppercase().chain(chars).collect(),
                }
            })
            .collect::<Vec<String>>()
            .join(" ")
    }

    pub fn eq_id(&self, other: &Device) -> bool {
        self.id == other.id
    }
}

/// Ports associated with a device.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[cfg_attr(target_os = "android", derive(IntoJava))]
#[cfg_attr(target_os = "android", jnix(package = "net.mullvad.mullvadvpn.model"))]
pub struct DevicePort {
    /// Port identifier.
    pub id: String,
}

impl fmt::Display for DevicePort {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.id)
    }
}

/// Contains a device state.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(target_os = "android", derive(IntoJava))]
#[cfg_attr(target_os = "android", jnix(package = "net.mullvad.mullvadvpn.model"))]
pub enum DeviceState {
    LoggedIn(AccountAndDevice),
    LoggedOut,
    Revoked,
}

impl DeviceState {
    pub fn into_device(self) -> Option<AccountAndDevice> {
        match self {
            DeviceState::LoggedIn(dev) => Some(dev),
            _ => None,
        }
    }

    pub fn is_logged_in(&self) -> bool {
        matches!(self, Self::LoggedIn(_))
    }

    pub fn get_account(&self) -> Option<&AccountAndDevice> {
        match self {
            DeviceState::LoggedIn(ref account) => Some(account),
            _ => None,
        }
    }
}

/// A [Device] and its associated account token.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[cfg_attr(target_os = "android", derive(IntoJava))]
#[cfg_attr(target_os = "android", jnix(package = "net.mullvad.mullvadvpn.model"))]
pub struct AccountAndDevice {
    pub account_token: AccountToken,
    pub device: Device,
}

impl AccountAndDevice {
    pub fn new(account_token: AccountToken, device: Device) -> Self {
        Self {
            account_token,
            device,
        }
    }
}

/// Reason why a [DeviceEvent] was emitted.
#[derive(Clone, Debug)]
#[cfg_attr(target_os = "android", derive(IntoJava))]
#[cfg_attr(target_os = "android", jnix(package = "net.mullvad.mullvadvpn.model"))]
pub enum DeviceEventCause {
    /// Logged in on a new device.
    LoggedIn,
    /// The device was removed due to user (or daemon) action.
    LoggedOut,
    /// Device was removed because it was not found remotely.
    Revoked,
    /// The device was updated, but not its key.
    Updated,
    /// The key was rotated.
    RotatedKey,
}

/// Emitted when logging in or out of an account, or when the device changes.
#[derive(Clone, Debug)]
#[cfg_attr(target_os = "android", derive(IntoJava))]
#[cfg_attr(target_os = "android", jnix(package = "net.mullvad.mullvadvpn.model"))]
pub struct DeviceEvent {
    pub cause: DeviceEventCause,
    pub new_state: DeviceState,
}

/// Emitted when a device is removed using the `RemoveDevice` RPC.
/// This is not sent by a normal logout or when it is revoked remotely.
#[derive(Clone, Debug)]
#[cfg_attr(target_os = "android", derive(IntoJava))]
#[cfg_attr(target_os = "android", jnix(package = "net.mullvad.mullvadvpn.model"))]
pub struct RemoveDeviceEvent {
    pub account_token: AccountToken,
    pub new_devices: Vec<Device>,
}

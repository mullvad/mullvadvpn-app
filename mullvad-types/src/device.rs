use crate::{account::AccountToken, wireguard};
use serde::{Deserialize, Serialize};
use talpid_types::net::wireguard::PublicKey;

/// UUID for a device.
pub type DeviceId = String;

/// Human-readable device identifier.
pub type DeviceName = String;

/// Contains data for a device returned by the API.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct Device {
    pub id: DeviceId,
    pub name: DeviceName,
    pub pubkey: PublicKey,
}

impl Eq for Device {}

/// A complete device configuration.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct DeviceData {
    pub token: AccountToken,
    pub device: Device,
    pub wg_data: wireguard::WireguardData,
}

impl From<DeviceData> for Device {
    fn from(data: DeviceData) -> Device {
        data.device
    }
}

/// Emitted when logging in or out of an account, or when the device changes.
#[derive(Clone, Debug)]
pub struct DeviceEvent(pub Option<Device>);

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
pub struct DeviceEvent(pub Option<(AccountToken, Device)>);

impl From<DeviceData> for DeviceEvent {
    fn from(data: DeviceData) -> DeviceEvent {
        DeviceEvent(Some((data.token, data.device)))
    }
}

impl From<Option<DeviceData>> for DeviceEvent {
    fn from(data: Option<DeviceData>) -> DeviceEvent {
        match data {
            Some(data) => DeviceEvent::from(data),
            None => DeviceEvent(None),
        }
    }
}

/// Emitted when a device is removed using the `RemoveDevice` RPC.
/// This is not sent by a normal logout or when it is revoked remotely.
#[derive(Clone, Debug)]
pub struct RemoveDeviceEvent {
    pub account_token: AccountToken,
    pub removed_device: Device,
    pub new_devices: Vec<Device>,
}

use crate::types::{conversions::bytes_to_pubkey, proto, FromProtobufTypeError};
use prost_types::Timestamp;

impl TryFrom<proto::Device> for mullvad_types::device::Device {
    type Error = FromProtobufTypeError;

    fn try_from(device: proto::Device) -> Result<Self, Self::Error> {
        Ok(mullvad_types::device::Device {
            id: device.id,
            name: device.name,
            pubkey: bytes_to_pubkey(&device.pubkey)?,
            ports: device
                .ports
                .into_iter()
                .map(mullvad_types::device::DevicePort::from)
                .collect(),
            hijack_dns: device.hijack_dns,
            created: chrono::DateTime::from_utc(
                chrono::NaiveDateTime::from_timestamp(
                    device
                        .created
                        .ok_or(FromProtobufTypeError::InvalidArgument(
                            "missing 'created' field",
                        ))?
                        .seconds,
                    0,
                ),
                chrono::Utc,
            ),
        })
    }
}

impl From<mullvad_types::device::Device> for proto::Device {
    fn from(device: mullvad_types::device::Device) -> Self {
        proto::Device {
            id: device.id,
            name: device.name,
            pubkey: device.pubkey.as_bytes().to_vec(),
            ports: device
                .ports
                .into_iter()
                .map(proto::DevicePort::from)
                .collect(),
            hijack_dns: device.hijack_dns,
            created: Some(Timestamp {
                seconds: device.created.timestamp(),
                nanos: 0,
            }),
        }
    }
}

impl From<mullvad_types::device::DevicePort> for proto::DevicePort {
    fn from(port: mullvad_types::device::DevicePort) -> Self {
        proto::DevicePort { id: port.id }
    }
}

impl From<mullvad_types::device::DeviceState> for proto::DeviceState {
    fn from(state: mullvad_types::device::DeviceState) -> Self {
        proto::DeviceState {
            state: proto::device_state::State::from(&state) as i32,
            device: state.into_device().map(|device| proto::AccountAndDevice {
                account_token: device.account_token,
                device: Some(proto::Device::from(device.device)),
            }),
        }
    }
}

impl From<&mullvad_types::device::DeviceState> for proto::device_state::State {
    fn from(state: &mullvad_types::device::DeviceState) -> Self {
        use mullvad_types::device::DeviceState as MullvadState;
        match state {
            MullvadState::LoggedIn(_) => proto::device_state::State::LoggedIn,
            MullvadState::LoggedOut => proto::device_state::State::LoggedOut,
            MullvadState::Revoked => proto::device_state::State::Revoked,
        }
    }
}

impl From<mullvad_types::device::DeviceEvent> for proto::DeviceEvent {
    fn from(event: mullvad_types::device::DeviceEvent) -> Self {
        proto::DeviceEvent {
            cause: proto::device_event::Cause::from(event.cause) as i32,
            new_state: Some(proto::DeviceState::from(event.new_state)),
        }
    }
}

impl From<mullvad_types::device::DeviceEventCause> for proto::device_event::Cause {
    fn from(cause: mullvad_types::device::DeviceEventCause) -> Self {
        use mullvad_types::device::DeviceEventCause as MullvadEvent;
        match cause {
            MullvadEvent::LoggedIn => proto::device_event::Cause::LoggedIn,
            MullvadEvent::LoggedOut => proto::device_event::Cause::LoggedOut,
            MullvadEvent::Revoked => proto::device_event::Cause::Revoked,
            MullvadEvent::Updated => proto::device_event::Cause::Updated,
            MullvadEvent::RotatedKey => proto::device_event::Cause::RotatedKey,
        }
    }
}

impl From<mullvad_types::device::RemoveDeviceEvent> for proto::RemoveDeviceEvent {
    fn from(event: mullvad_types::device::RemoveDeviceEvent) -> Self {
        proto::RemoveDeviceEvent {
            account_token: event.account_token,
            new_device_list: event
                .new_devices
                .into_iter()
                .map(proto::Device::from)
                .collect(),
        }
    }
}

impl From<mullvad_types::device::AccountAndDevice> for proto::AccountAndDevice {
    fn from(device: mullvad_types::device::AccountAndDevice) -> Self {
        proto::AccountAndDevice {
            account_token: device.account_token,
            device: Some(proto::Device::from(device.device)),
        }
    }
}

impl From<Vec<mullvad_types::device::Device>> for proto::DeviceList {
    fn from(devices: Vec<mullvad_types::device::Device>) -> Self {
        proto::DeviceList {
            devices: devices.into_iter().map(proto::Device::from).collect(),
        }
    }
}

impl From<proto::DevicePort> for mullvad_types::device::DevicePort {
    fn from(port: proto::DevicePort) -> Self {
        mullvad_types::device::DevicePort { id: port.id }
    }
}

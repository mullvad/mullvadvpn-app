use crate::types::{FromProtobufTypeError, conversions::bytes_to_pubkey, proto};
use chrono::DateTime;
use prost_types::Timestamp;

impl TryFrom<proto::Device> for mullvad_types::device::Device {
    type Error = FromProtobufTypeError;

    fn try_from(device: proto::Device) -> Result<Self, Self::Error> {
        let created_seconds = device
            .created
            .ok_or(FromProtobufTypeError::invalid_argument(
                "missing 'created' field",
            ))?
            .seconds;

        let created = DateTime::from_timestamp(created_seconds, 0)
            .ok_or(FromProtobufTypeError::invalid_argument("invalid timestamp"))?;

        Ok(mullvad_types::device::Device {
            id: device.id,
            name: device.name,
            pubkey: bytes_to_pubkey(&device.pubkey)?,
            hijack_dns: device.hijack_dns,
            created,
        })
    }
}

impl From<mullvad_types::device::Device> for proto::Device {
    fn from(device: mullvad_types::device::Device) -> Self {
        proto::Device {
            id: device.id,
            name: device.name,
            pubkey: device.pubkey.as_bytes().to_vec(),
            hijack_dns: device.hijack_dns,
            created: Some(Timestamp {
                seconds: device.created.timestamp(),
                nanos: 0,
            }),
        }
    }
}

impl TryFrom<proto::DeviceState> for mullvad_types::device::DeviceState {
    type Error = FromProtobufTypeError;

    fn try_from(state: proto::DeviceState) -> Result<Self, FromProtobufTypeError> {
        let Some(state) = state.state else {
            return Err(FromProtobufTypeError::invalid_argument(
                "invalid device state",
            ));
        };

        match state {
            proto::device_state::State::LoggedIn(proto::device_state::LoggedIn { state }) => {
                match state {
                    Some(proto::device_state::logged_in::State::Traditional(traditional)) => {
                        // TODO: Handle empty string!
                        let account_number = traditional.account_number;
                        let device =
                            traditional
                                .device
                                .ok_or(FromProtobufTypeError::invalid_argument(
                                    "missing device data",
                                ))?;

                        Ok(mullvad_types::device::DeviceState::LoggedIn(
                            mullvad_types::device::LoggedIn::Traditional(
                                mullvad_types::device::AccountAndDevice {
                                    account_number,
                                    device: mullvad_types::device::Device::try_from(device)?,
                                },
                            ),
                        ))
                    }
                    Some(proto::device_state::logged_in::State::Blind(device)) => {
                        Ok(mullvad_types::device::DeviceState::LoggedIn(
                            mullvad_types::device::LoggedIn::Blind(
                                mullvad_types::device::Device::try_from(device)?,
                            ),
                        ))
                    }
                    None => todo!(),
                }
            }
            proto::device_state::State::Revoked(proto::device_state::Revoked {}) => {
                Ok(mullvad_types::device::DeviceState::Revoked)
            }
            proto::device_state::State::LoggedOut(proto::device_state::LoggedOut {}) => {
                Ok(mullvad_types::device::DeviceState::LoggedOut)
            }
        }
    }
}

impl From<mullvad_types::device::DeviceState> for proto::DeviceState {
    fn from(state: mullvad_types::device::DeviceState) -> Self {
        proto::DeviceState {
            state: Some(proto::device_state::State::from(state)),
        }
    }
}

impl From<mullvad_types::device::DeviceState> for proto::device_state::State {
    fn from(state: mullvad_types::device::DeviceState) -> Self {
        use mullvad_types::device::{DeviceState, LoggedIn};
        match state {
            DeviceState::LoggedIn(LoggedIn::Traditional(traditional)) => {
                proto::device_state::State::LoggedIn(proto::device_state::LoggedIn {
                    state: Some(proto::device_state::logged_in::State::Traditional(
                        proto::AccountAndDevice::from(traditional),
                    )),
                })
            }
            DeviceState::LoggedIn(LoggedIn::Blind(device)) => {
                proto::device_state::State::LoggedIn(proto::device_state::LoggedIn {
                    state: Some(proto::device_state::logged_in::State::Blind(
                        proto::Device::from(device),
                    )),
                })
            }
            DeviceState::LoggedOut => {
                proto::device_state::State::LoggedOut(proto::device_state::LoggedOut {})
            }
            DeviceState::Revoked => {
                proto::device_state::State::Revoked(proto::device_state::Revoked {})
            }
        }
    }
}

impl From<mullvad_types::device::DeviceEvent> for proto::DeviceEvent {
    fn from(event: mullvad_types::device::DeviceEvent) -> Self {
        proto::DeviceEvent {
            cause: i32::from(proto::device_event::Cause::from(event.cause)),
            new_state: Some(proto::DeviceState::from(event.new_state)),
        }
    }
}

impl TryFrom<proto::DeviceEvent> for mullvad_types::device::DeviceEvent {
    type Error = FromProtobufTypeError;

    fn try_from(event: proto::DeviceEvent) -> Result<Self, Self::Error> {
        let cause = proto::device_event::Cause::try_from(event.cause)
            .map_err(|_| FromProtobufTypeError::invalid_argument("invalid event"))?;
        let cause = mullvad_types::device::DeviceEventCause::from(cause);

        let new_state = mullvad_types::device::DeviceState::try_from(event.new_state.ok_or(
            FromProtobufTypeError::invalid_argument("missing device state"),
        )?)?;

        Ok(mullvad_types::device::DeviceEvent { cause, new_state })
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

impl From<proto::device_event::Cause> for mullvad_types::device::DeviceEventCause {
    fn from(event: proto::device_event::Cause) -> Self {
        use mullvad_types::device::DeviceEventCause as MullvadEvent;
        match event {
            proto::device_event::Cause::LoggedIn => MullvadEvent::LoggedIn,
            proto::device_event::Cause::LoggedOut => MullvadEvent::LoggedOut,
            proto::device_event::Cause::Revoked => MullvadEvent::Revoked,
            proto::device_event::Cause::Updated => MullvadEvent::Updated,
            proto::device_event::Cause::RotatedKey => MullvadEvent::RotatedKey,
        }
    }
}

impl From<mullvad_types::device::RemoveDeviceEvent> for proto::RemoveDeviceEvent {
    fn from(event: mullvad_types::device::RemoveDeviceEvent) -> Self {
        proto::RemoveDeviceEvent {
            account_number: event.account_number,
            new_device_list: event
                .new_devices
                .into_iter()
                .map(proto::Device::from)
                .collect(),
        }
    }
}

impl TryFrom<proto::RemoveDeviceEvent> for mullvad_types::device::RemoveDeviceEvent {
    type Error = FromProtobufTypeError;

    fn try_from(event: proto::RemoveDeviceEvent) -> Result<Self, Self::Error> {
        let new_devices = event
            .new_device_list
            .into_iter()
            .map(mullvad_types::device::Device::try_from)
            .collect::<Result<Vec<_>, FromProtobufTypeError>>()?;
        Ok(mullvad_types::device::RemoveDeviceEvent {
            account_number: event.account_number,
            new_devices,
        })
    }
}

impl From<mullvad_types::device::AccountAndDevice> for proto::AccountAndDevice {
    fn from(device: mullvad_types::device::AccountAndDevice) -> Self {
        proto::AccountAndDevice {
            account_number: device.account_number,
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

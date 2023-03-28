use super::FromProtobufTypeError;
use crate::types::proto;
use prost_types::Timestamp;

impl From<mullvad_types::wireguard::PublicKey> for proto::PublicKey {
    fn from(public_key: mullvad_types::wireguard::PublicKey) -> Self {
        proto::PublicKey {
            key: public_key.key.as_bytes().to_vec(),
            created: Some(Timestamp {
                seconds: public_key.created.timestamp(),
                nanos: 0,
            }),
        }
    }
}

impl TryFrom<proto::PublicKey> for mullvad_types::wireguard::PublicKey {
    type Error = FromProtobufTypeError;

    fn try_from(public_key: proto::PublicKey) -> Result<Self, Self::Error> {
        let created = public_key
            .created
            .ok_or(FromProtobufTypeError::InvalidArgument(
                "missing 'created' timestamp",
            ))?;
        let ndt = chrono::NaiveDateTime::from_timestamp(created.seconds, created.nanos as u32);

        Ok(mullvad_types::wireguard::PublicKey {
            key: talpid_types::net::wireguard::PublicKey::try_from(public_key.key.as_slice())
                .map_err(|_| FromProtobufTypeError::InvalidArgument("invalid wireguard key"))?,
            created: chrono::DateTime::<chrono::Utc>::from_utc(ndt, chrono::Utc),
        })
    }
}

impl From<mullvad_types::wireguard::QuantumResistantState> for proto::QuantumResistantState {
    fn from(state: mullvad_types::wireguard::QuantumResistantState) -> Self {
        match state {
            mullvad_types::wireguard::QuantumResistantState::Auto => proto::QuantumResistantState {
                state: i32::from(proto::quantum_resistant_state::State::Auto),
            },
            mullvad_types::wireguard::QuantumResistantState::On => proto::QuantumResistantState {
                state: i32::from(proto::quantum_resistant_state::State::On),
            },
            mullvad_types::wireguard::QuantumResistantState::Off => proto::QuantumResistantState {
                state: i32::from(proto::quantum_resistant_state::State::Off),
            },
        }
    }
}

impl TryFrom<proto::QuantumResistantState> for mullvad_types::wireguard::QuantumResistantState {
    type Error = FromProtobufTypeError;

    fn try_from(state: proto::QuantumResistantState) -> Result<Self, Self::Error> {
        match proto::quantum_resistant_state::State::from_i32(state.state) {
            Some(proto::quantum_resistant_state::State::Auto) => {
                Ok(mullvad_types::wireguard::QuantumResistantState::Auto)
            }
            Some(proto::quantum_resistant_state::State::On) => {
                Ok(mullvad_types::wireguard::QuantumResistantState::On)
            }
            Some(proto::quantum_resistant_state::State::Off) => {
                Ok(mullvad_types::wireguard::QuantumResistantState::Off)
            }
            None => Err(FromProtobufTypeError::InvalidArgument(
                "invalid quantum resistance state",
            )),
        }
    }
}

use super::FromProtobufTypeError;
use crate::types::proto;
use chrono::DateTime;
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
            .ok_or(FromProtobufTypeError::invalid_argument(
                "missing 'created' timestamp",
            ))?;

        let created = DateTime::from_timestamp(created.seconds, created.nanos as u32)
            .ok_or(FromProtobufTypeError::invalid_argument("invalid timestamp"))?;

        Ok(mullvad_types::wireguard::PublicKey {
            key: talpid_types::net::wireguard::PublicKey::try_from(public_key.key.as_slice())
                .map_err(|_| FromProtobufTypeError::invalid_argument("invalid wireguard key"))?,
            created,
        })
    }
}

impl From<mullvad_types::wireguard::QuantumResistantState> for proto::QuantumResistantState {
    fn from(state: mullvad_types::wireguard::QuantumResistantState) -> Self {
        match state {
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
        match proto::quantum_resistant_state::State::try_from(state.state) {
            Ok(proto::quantum_resistant_state::State::On) => {
                Ok(mullvad_types::wireguard::QuantumResistantState::On)
            }
            Ok(proto::quantum_resistant_state::State::Off) => {
                Ok(mullvad_types::wireguard::QuantumResistantState::Off)
            }
            Err(_) => Err(FromProtobufTypeError::invalid_argument(
                "invalid quantum resistance state",
            )),
        }
    }
}

impl From<bool> for proto::DaitaSettings {
    fn from(enabled: bool) -> Self {
        proto::DaitaSettings { enabled }
    }
}

impl From<mullvad_types::wireguard::WireguardIdentityInfo> for proto::WireguardIdentity {
    fn from(info: mullvad_types::wireguard::WireguardIdentityInfo) -> Self {
        proto::WireguardIdentity {
            public_key: Some(proto::PublicKey::from(info.public_key)),
            ipv4_address: info.ipv4_address.to_string(),
            ipv6_address: info
                .ipv6_address
                .map(|address| address.to_string())
                .unwrap_or_default(),
        }
    }
}

impl TryFrom<proto::WireguardIdentity> for mullvad_types::wireguard::WireguardIdentityInfo {
    type Error = FromProtobufTypeError;

    fn try_from(identity: proto::WireguardIdentity) -> Result<Self, Self::Error> {
        let public_key = identity
            .public_key
            .ok_or(FromProtobufTypeError::invalid_argument("missing public key"))?;
        let ipv6_address = if identity.ipv6_address.is_empty() {
            None
        } else {
            Some(identity.ipv6_address.parse().map_err(|_| {
                FromProtobufTypeError::invalid_argument("invalid IPv6 tunnel address")
            })?)
        };
        Ok(mullvad_types::wireguard::WireguardIdentityInfo {
            public_key: mullvad_types::wireguard::PublicKey::try_from(public_key)?,
            ipv4_address: identity.ipv4_address.parse().map_err(|_| {
                FromProtobufTypeError::invalid_argument("invalid IPv4 tunnel address")
            })?,
            ipv6_address,
        })
    }
}

/// Convert the (possibly missing) identity into its protobuf representation.
pub fn identity_state_to_proto(
    info: Option<mullvad_types::wireguard::WireguardIdentityInfo>,
) -> proto::WireguardIdentityState {
    proto::WireguardIdentityState {
        identity: info.map(proto::WireguardIdentity::from),
    }
}

/// Convert the protobuf representation of the identity state into an optional identity.
pub fn identity_state_from_proto(
    state: proto::WireguardIdentityState,
) -> Result<Option<mullvad_types::wireguard::WireguardIdentityInfo>, FromProtobufTypeError> {
    state
        .identity
        .map(mullvad_types::wireguard::WireguardIdentityInfo::try_from)
        .transpose()
}

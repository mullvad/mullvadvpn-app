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

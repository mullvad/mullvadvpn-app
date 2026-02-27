use crate::types::{
    FromProtobufTypeError,
    conversions::{arg_from_str, bytes_to_privkey, bytes_to_pubkey},
    proto,
};
use mullvad_types::settings::{CustomVpnConfig, CustomVpnPeerConfig, CustomVpnTunnelConfig};

impl TryFrom<proto::CustomVpnConfig> for CustomVpnConfig {
    type Error = FromProtobufTypeError;

    fn try_from(config: proto::CustomVpnConfig) -> Result<Self, Self::Error> {
        let tunnel = config
            .tunnel
            .map(|t| {
                Ok::<_, FromProtobufTypeError>(CustomVpnTunnelConfig {
                    private_key: bytes_to_privkey(&t.private_key)?,
                    ip: arg_from_str(&t.ip, "invalid tunnel IP address")?,
                })
            })
            .ok_or(FromProtobufTypeError::InvalidArgument(
                "missing tunnel config",
            ))??;

        let peer = config
            .peer
            .map(|p| {
                Ok::<_, FromProtobufTypeError>(CustomVpnPeerConfig {
                    public_key: bytes_to_pubkey(&p.public_key)?,
                    allowed_ip: arg_from_str(&p.allowed_ip, "invalid allowed IP")?,
                    endpoint: arg_from_str(&p.endpoint, "invalid endpoint")?,
                })
            })
            .ok_or(FromProtobufTypeError::InvalidArgument(
                "missing peer config",
            ))??;

        Ok(CustomVpnConfig { tunnel, peer })
    }
}

impl From<CustomVpnConfig> for proto::CustomVpnConfig {
    fn from(config: CustomVpnConfig) -> Self {
        proto::CustomVpnConfig {
            tunnel: Some(proto::custom_vpn_config::TunnelConfig {
                private_key: config.tunnel.private_key.to_bytes().to_vec(),
                ip: config.tunnel.ip.to_string(),
            }),
            peer: Some(proto::custom_vpn_config::PeerConfig {
                public_key: config.peer.public_key.as_bytes().to_vec(),
                allowed_ip: config.peer.allowed_ip.to_string(),
                endpoint: config.peer.endpoint.to_string(),
            }),
        }
    }
}

use crate::types::{
    FromProtobufTypeError,
    conversions::{arg_from_str, bytes_to_privkey, bytes_to_pubkey},
    proto,
};
use talpid_types::net::wireguard::{
    PersonalVpnConfig, PersonalVpnPeerConfig, PersonalVpnTunnelConfig, UnresolvedPersonalVpnConfig,
    UnresolvedPersonalVpnPeerConfig,
};

impl TryFrom<proto::PersonalVpnConfig> for UnresolvedPersonalVpnConfig {
    type Error = FromProtobufTypeError;

    fn try_from(config: proto::PersonalVpnConfig) -> Result<Self, Self::Error> {
        let tunnel = config
            .tunnel
            .map(|t| {
                Ok::<_, FromProtobufTypeError>(PersonalVpnTunnelConfig {
                    private_key: bytes_to_privkey(&t.private_key)?,
                    ip: arg_from_str(&t.ip, "invalid tunnel IP address")?,
                })
            })
            .ok_or(FromProtobufTypeError::InvalidArgument(
                "missing tunnel config".to_owned(),
            ))??;

        let peer = config
            .peer
            .map(|p| {
                let endpoint = p.endpoint.trim().to_owned();
                if endpoint.is_empty() || !endpoint.contains(':') {
                    return Err(FromProtobufTypeError::InvalidArgument(
                        "invalid endpoint".to_owned(),
                    ));
                }
                Ok::<_, FromProtobufTypeError>(UnresolvedPersonalVpnPeerConfig {
                    public_key: bytes_to_pubkey(&p.public_key)?,
                    allowed_ip: p
                        .allowed_ip
                        .iter()
                        .map(|ip| arg_from_str(ip, "invalid allowed IP"))
                        .collect::<Result<_, _>>()?,
                    endpoint,
                })
            })
            .ok_or(FromProtobufTypeError::InvalidArgument(
                "missing peer config".to_owned(),
            ))??;

        Ok(UnresolvedPersonalVpnConfig { tunnel, peer })
    }
}

/// For the read path: a `proto::PersonalVpnConfig` that originated from a
/// previously-resolved settings blob always carries an `<ip>:<port>` string.
/// Parse it back into [`PersonalVpnConfig`] directly (no DNS needed).
impl TryFrom<proto::PersonalVpnConfig> for PersonalVpnConfig {
    type Error = FromProtobufTypeError;

    fn try_from(config: proto::PersonalVpnConfig) -> Result<Self, Self::Error> {
        let tunnel = config
            .tunnel
            .map(|t| {
                Ok::<_, FromProtobufTypeError>(PersonalVpnTunnelConfig {
                    private_key: bytes_to_privkey(&t.private_key)?,
                    ip: arg_from_str(&t.ip, "invalid tunnel IP address")?,
                })
            })
            .ok_or(FromProtobufTypeError::InvalidArgument(
                "missing tunnel config".to_owned(),
            ))??;

        let peer = config
            .peer
            .map(|p| {
                Ok::<_, FromProtobufTypeError>(PersonalVpnPeerConfig {
                    public_key: bytes_to_pubkey(&p.public_key)?,
                    allowed_ip: p
                        .allowed_ip
                        .iter()
                        .map(|ip| arg_from_str(ip, "invalid allowed IP"))
                        .collect::<Result<_, _>>()?,
                    endpoint: arg_from_str(&p.endpoint, "invalid endpoint")?,
                })
            })
            .ok_or(FromProtobufTypeError::InvalidArgument(
                "missing peer config".to_owned(),
            ))??;

        Ok(PersonalVpnConfig { tunnel, peer })
    }
}

impl From<PersonalVpnConfig> for proto::PersonalVpnConfig {
    fn from(config: PersonalVpnConfig) -> Self {
        proto::PersonalVpnConfig {
            tunnel: Some(proto::personal_vpn_config::TunnelConfig {
                private_key: config.tunnel.private_key.to_bytes().to_vec(),
                ip: config.tunnel.ip.to_string(),
            }),
            peer: Some(proto::personal_vpn_config::PeerConfig {
                public_key: config.peer.public_key.as_bytes().to_vec(),
                allowed_ip: config
                    .peer
                    .allowed_ip
                    .iter()
                    .map(|ip| ip.to_string())
                    .collect(),
                endpoint: config.peer.endpoint.to_string(),
            }),
        }
    }
}

impl From<UnresolvedPersonalVpnConfig> for proto::PersonalVpnConfig {
    fn from(config: UnresolvedPersonalVpnConfig) -> Self {
        proto::PersonalVpnConfig {
            tunnel: Some(proto::personal_vpn_config::TunnelConfig {
                private_key: config.tunnel.private_key.to_bytes().to_vec(),
                ip: config.tunnel.ip.to_string(),
            }),
            peer: Some(proto::personal_vpn_config::PeerConfig {
                public_key: config.peer.public_key.as_bytes().to_vec(),
                allowed_ip: config
                    .peer
                    .allowed_ip
                    .iter()
                    .map(|ip| ip.to_string())
                    .collect(),
                endpoint: config.peer.endpoint,
            }),
        }
    }
}

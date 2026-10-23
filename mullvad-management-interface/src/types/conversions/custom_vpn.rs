use crate::types::{
    FromProtobufTypeError,
    conversions::{arg_from_str, bytes_to_privkey, bytes_to_pubkey},
    proto,
};
use talpid_types::net::wireguard::{CustomVpnConfig, CustomVpnPeerConfig, CustomVpnTunnelConfig};

fn parse_tunnel_ips(ips: &[String]) -> Result<Vec<std::net::IpAddr>, FromProtobufTypeError> {
    if ips.is_empty() {
        return Err(FromProtobufTypeError::InvalidArgument(
            "tunnel requires at least one IP".to_owned(),
        ));
    }
    ips.iter()
        .map(|ip| arg_from_str(ip, "invalid tunnel IP address"))
        .collect()
}

/// For the read path: a `proto::CustomVpnConfig` that originated from a
/// previously-resolved settings blob always carries an `<ip>:<port>` string.
/// Parse it back into [`CustomVpnConfig`] directly (no DNS needed).
impl TryFrom<proto::CustomVpnConfig> for CustomVpnConfig {
    type Error = FromProtobufTypeError;

    fn try_from(config: proto::CustomVpnConfig) -> Result<Self, Self::Error> {
        let tunnel = config
            .tunnel
            .map(|t| {
                Ok::<_, FromProtobufTypeError>(CustomVpnTunnelConfig {
                    private_key: bytes_to_privkey(&t.private_key)?,
                    ips: parse_tunnel_ips(&t.ips)?,
                })
            })
            .ok_or(FromProtobufTypeError::InvalidArgument(
                "missing tunnel config".to_owned(),
            ))??;

        let peer = config
            .peer
            .map(|p| {
                Ok::<_, FromProtobufTypeError>(CustomVpnPeerConfig {
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

        Ok(CustomVpnConfig { tunnel, peer })
    }
}

impl From<CustomVpnConfig> for proto::CustomVpnConfig {
    fn from(config: CustomVpnConfig) -> Self {
        proto::CustomVpnConfig {
            tunnel: Some(proto::custom_vpn_config::TunnelConfig {
                private_key: config.tunnel.private_key.to_bytes().to_vec(),
                ips: config.tunnel.ips.iter().map(ToString::to_string).collect(),
            }),
            peer: Some(proto::custom_vpn_config::PeerConfig {
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

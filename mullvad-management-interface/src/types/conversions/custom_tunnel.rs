use crate::types::{
    FromProtobufTypeError,
    conversions::{bytes_to_privkey, bytes_to_pubkey},
    proto,
};
use talpid_types::net::wireguard;

impl TryFrom<proto::WireguardConfig> for wireguard::ConnectionConfig {
    type Error = FromProtobufTypeError;

    fn try_from(
        config: proto::WireguardConfig,
    ) -> Result<wireguard::ConnectionConfig, Self::Error> {
        let tunnel = config.tunnel.ok_or(FromProtobufTypeError::InvalidArgument(
            "missing tunnel config",
        ))?;

        let private_key = bytes_to_privkey(&tunnel.private_key)?;

        let peer = config.peer.ok_or(FromProtobufTypeError::InvalidArgument(
            "missing peer config",
        ))?;

        let public_key = bytes_to_pubkey(&peer.public_key)?;

        let ipv4_gateway = config
            .ipv4_gateway
            .parse()
            .map_err(|_err| FromProtobufTypeError::InvalidArgument("invalid IPv4 gateway"))?;
        let ipv6_gateway = config
            .ipv6_gateway
            .map(|addr| {
                addr.parse()
                    .map_err(|_err| FromProtobufTypeError::InvalidArgument("invalid IPv6 gateway"))
            })
            .transpose()?;

        let endpoint = peer
            .endpoint
            .parse()
            .map_err(|_err| FromProtobufTypeError::InvalidArgument("invalid peer address"))?;

        let mut tunnel_addresses = Vec::new();
        for address in tunnel.addresses {
            let address = address
                .parse()
                .map_err(|_| FromProtobufTypeError::InvalidArgument("invalid address"))?;
            tunnel_addresses.push(address);
        }

        let mut allowed_ips = Vec::new();
        for address in peer.allowed_ips {
            let address = address
                .parse()
                .map_err(|_| FromProtobufTypeError::InvalidArgument("invalid address"))?;
            allowed_ips.push(address);
        }

        Ok(wireguard::ConnectionConfig {
            tunnel: wireguard::TunnelConfig {
                private_key,
                addresses: tunnel_addresses,
            },
            peer: wireguard::PeerConfig {
                public_key,
                allowed_ips,
                endpoint,
                psk: None,
                #[cfg(daita)]
                constant_packet_size: false,
            },
            exit_peer: None,
            ipv4_gateway,
            ipv6_gateway,
            #[cfg(target_os = "linux")]
            fwmark: Some(mullvad_types::TUNNEL_FWMARK),
        })
    }
}

impl From<wireguard::ConnectionConfig> for proto::WireguardConfig {
    fn from(config: wireguard::ConnectionConfig) -> Self {
        proto::WireguardConfig {
            tunnel: Some(proto::wireguard_config::TunnelConfig {
                private_key: config.tunnel.private_key.to_bytes().to_vec(),
                addresses: config
                    .tunnel
                    .addresses
                    .iter()
                    .map(|address| address.to_string())
                    .collect(),
            }),
            peer: Some(proto::wireguard_config::PeerConfig {
                public_key: config.peer.public_key.as_bytes().to_vec(),
                allowed_ips: config
                    .peer
                    .allowed_ips
                    .iter()
                    .map(|address| address.to_string())
                    .collect(),
                endpoint: config.peer.endpoint.to_string(),
            }),
            ipv4_gateway: config.ipv4_gateway.to_string(),
            ipv6_gateway: config
                .ipv6_gateway
                .as_ref()
                .map(|address| address.to_string()),
        }
    }
}

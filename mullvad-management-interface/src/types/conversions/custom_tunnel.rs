use crate::types::{
    conversions::{bytes_to_privkey, bytes_to_pubkey, option_from_proto_string},
    proto, FromProtobufTypeError,
};
use talpid_types::net::wireguard;

impl TryFrom<proto::ConnectionConfig> for mullvad_types::ConnectionConfig {
    type Error = FromProtobufTypeError;

    fn try_from(
        config: proto::ConnectionConfig,
    ) -> Result<mullvad_types::ConnectionConfig, Self::Error> {
        use talpid_types::net::{self, openvpn};

        let config = config.config.ok_or(FromProtobufTypeError::InvalidArgument(
            "missing connection config",
        ))?;
        match config {
            proto::connection_config::Config::Openvpn(config) => {
                let address = match config.address.parse() {
                    Ok(address) => address,
                    Err(_) => {
                        return Err(FromProtobufTypeError::InvalidArgument("invalid address"))
                    }
                };

                Ok(mullvad_types::ConnectionConfig::OpenVpn(
                    openvpn::ConnectionConfig {
                        endpoint: net::Endpoint {
                            address,
                            protocol: super::net::try_transport_protocol_from_i32(config.protocol)?,
                        },
                        username: config.username,
                        password: config.password,
                    },
                ))
            }
            proto::connection_config::Config::Wireguard(config) => {
                let tunnel = config.tunnel.ok_or(FromProtobufTypeError::InvalidArgument(
                    "missing tunnel config",
                ))?;

                let private_key = bytes_to_privkey(&tunnel.private_key)?;

                let peer = config.peer.ok_or(FromProtobufTypeError::InvalidArgument(
                    "missing peer config",
                ))?;

                let public_key = bytes_to_pubkey(&peer.public_key)?;

                let ipv4_gateway = config.ipv4_gateway.parse().map_err(|_err| {
                    FromProtobufTypeError::InvalidArgument("invalid IPv4 gateway")
                })?;
                let ipv6_gateway = option_from_proto_string(config.ipv6_gateway)
                    .map(|addr| {
                        addr.parse().map_err(|_err| {
                            FromProtobufTypeError::InvalidArgument("invalid IPv6 gateway")
                        })
                    })
                    .transpose()?;

                let endpoint = peer.endpoint.parse().map_err(|_err| {
                    FromProtobufTypeError::InvalidArgument("invalid peer address")
                })?;

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

                Ok(mullvad_types::ConnectionConfig::Wireguard(
                    wireguard::ConnectionConfig {
                        tunnel: wireguard::TunnelConfig {
                            private_key,
                            addresses: tunnel_addresses,
                        },
                        peer: wireguard::PeerConfig {
                            public_key,
                            allowed_ips,
                            endpoint,
                            psk: None,
                        },
                        exit_peer: None,
                        ipv4_gateway,
                        ipv6_gateway,
                        #[cfg(target_os = "linux")]
                        fwmark: Some(mullvad_types::TUNNEL_FWMARK),
                    },
                ))
            }
        }
    }
}

impl From<mullvad_types::ConnectionConfig> for proto::ConnectionConfig {
    fn from(config: mullvad_types::ConnectionConfig) -> Self {
        use proto::connection_config;

        Self {
            config: Some(match config {
                mullvad_types::ConnectionConfig::OpenVpn(config) => {
                    connection_config::Config::Openvpn(connection_config::OpenvpnConfig {
                        address: config.endpoint.address.to_string(),
                        protocol: i32::from(proto::TransportProtocol::from(
                            config.endpoint.protocol,
                        )),
                        username: config.username,
                        password: config.password,
                    })
                }
                mullvad_types::ConnectionConfig::Wireguard(config) => {
                    connection_config::Config::Wireguard(connection_config::WireguardConfig {
                        tunnel: Some(connection_config::wireguard_config::TunnelConfig {
                            private_key: config.tunnel.private_key.to_bytes().to_vec(),
                            addresses: config
                                .tunnel
                                .addresses
                                .iter()
                                .map(|address| address.to_string())
                                .collect(),
                        }),
                        peer: Some(connection_config::wireguard_config::PeerConfig {
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
                            .map(|address| address.to_string())
                            .unwrap_or_default(),
                    })
                }
            }),
        }
    }
}

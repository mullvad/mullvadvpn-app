pub use prost_types::{Duration, Timestamp};

use mullvad_types::relay_constraints::Constraint;

tonic::include_proto!("mullvad_daemon.management_interface");

impl From<mullvad_types::version::AppVersionInfo> for AppVersionInfo {
    fn from(version_info: mullvad_types::version::AppVersionInfo) -> Self {
        Self {
            supported: version_info.supported,
            latest_stable: version_info.latest_stable,
            latest_beta: version_info.latest_beta,
            suggested_upgrade: version_info.suggested_upgrade.unwrap_or_default(),
        }
    }
}

impl From<&mullvad_types::ConnectionConfig> for ConnectionConfig {
    fn from(config: &mullvad_types::ConnectionConfig) -> Self {
        Self {
            config: Some(match config {
                mullvad_types::ConnectionConfig::OpenVpn(config) => {
                    connection_config::Config::Openvpn(connection_config::OpenvpnConfig {
                        address: config.endpoint.address.to_string(),
                        protocol: i32::from(TransportProtocol::from(config.endpoint.protocol)),
                        username: config.username.clone(),
                        password: config.password.clone(),
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
                            protocol: i32::from(TransportProtocol::from(config.peer.protocol)),
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

impl From<talpid_types::net::TransportProtocol> for TransportProtocol {
    fn from(protocol: talpid_types::net::TransportProtocol) -> Self {
        match protocol {
            talpid_types::net::TransportProtocol::Udp => TransportProtocol::Udp,
            talpid_types::net::TransportProtocol::Tcp => TransportProtocol::Tcp,
        }
    }
}

impl From<TransportProtocol> for TransportProtocolConstraint {
    fn from(protocol: TransportProtocol) -> Self {
        Self {
            protocol: i32::from(protocol),
        }
    }
}

impl From<talpid_types::net::IpVersion> for IpVersion {
    fn from(version: talpid_types::net::IpVersion) -> Self {
        match version {
            talpid_types::net::IpVersion::V4 => Self::V4,
            talpid_types::net::IpVersion::V6 => Self::V6,
        }
    }
}

impl From<IpVersion> for IpVersionConstraint {
    fn from(version: IpVersion) -> Self {
        Self {
            protocol: i32::from(version),
        }
    }
}

impl From<mullvad_types::relay_constraints::LocationConstraint> for RelayLocation {
    fn from(location: mullvad_types::relay_constraints::LocationConstraint) -> Self {
        use mullvad_types::relay_constraints::LocationConstraint;

        match location {
            LocationConstraint::Country(country) => Self {
                country,
                ..Default::default()
            },
            LocationConstraint::City(country, city) => Self {
                country,
                city,
                ..Default::default()
            },
            LocationConstraint::Hostname(country, city, hostname) => Self {
                country,
                city,
                hostname,
            },
        }
    }
}

impl From<TransportProtocol> for talpid_types::net::TransportProtocol {
    fn from(protocol: TransportProtocol) -> Self {
        match protocol {
            TransportProtocol::Udp => talpid_types::net::TransportProtocol::Udp,
            TransportProtocol::Tcp => talpid_types::net::TransportProtocol::Tcp,
        }
    }
}

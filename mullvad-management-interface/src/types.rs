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

impl From<mullvad_types::ConnectionConfig> for ConnectionConfig {
    fn from(config: mullvad_types::ConnectionConfig) -> Self {
        Self {
            config: Some(match config {
                mullvad_types::ConnectionConfig::OpenVpn(config) => {
                    connection_config::Config::Openvpn(connection_config::OpenvpnConfig {
                        address: config.endpoint.address.to_string(),
                        protocol: i32::from(TransportProtocol::from(config.endpoint.protocol)),
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

impl From<mullvad_types::relay_constraints::BridgeState> for BridgeState {
    fn from(state: mullvad_types::relay_constraints::BridgeState) -> Self {
        use mullvad_types::relay_constraints::BridgeState;
        Self {
            state: i32::from(match state {
                BridgeState::Auto => bridge_state::State::Auto,
                BridgeState::On => bridge_state::State::On,
                BridgeState::Off => bridge_state::State::Off,
            }),
        }
    }
}

impl From<mullvad_types::relay_constraints::BridgeSettings> for BridgeSettings {
    fn from(settings: mullvad_types::relay_constraints::BridgeSettings) -> Self {
        use mullvad_types::relay_constraints::BridgeSettings as MullvadBridgeSettings;
        use talpid_types::net as talpid_net;

        let settings = match settings {
            MullvadBridgeSettings::Normal(constraints) => {
                bridge_settings::Type::Normal(bridge_settings::BridgeConstraints {
                    location: constraints
                        .location
                        .clone()
                        .option()
                        .map(RelayLocation::from),
                    providers: convert_providers_constraint(&constraints.providers),
                })
            }
            MullvadBridgeSettings::Custom(proxy_settings) => match proxy_settings {
                talpid_net::openvpn::ProxySettings::Local(proxy_settings) => {
                    bridge_settings::Type::Local(bridge_settings::LocalProxySettings {
                        port: u32::from(proxy_settings.port),
                        peer: proxy_settings.peer.to_string(),
                    })
                }
                talpid_net::openvpn::ProxySettings::Remote(proxy_settings) => {
                    bridge_settings::Type::Remote(bridge_settings::RemoteProxySettings {
                        address: proxy_settings.address.to_string(),
                        auth: proxy_settings.auth.as_ref().map(|auth| {
                            bridge_settings::RemoteProxyAuth {
                                username: auth.username.clone(),
                                password: auth.password.clone(),
                            }
                        }),
                    })
                }
                talpid_net::openvpn::ProxySettings::Shadowsocks(proxy_settings) => {
                    bridge_settings::Type::Shadowsocks(bridge_settings::ShadowsocksProxySettings {
                        peer: proxy_settings.peer.to_string(),
                        password: proxy_settings.password.clone(),
                        cipher: proxy_settings.cipher.clone(),
                    })
                }
            },
        };

        BridgeSettings {
            r#type: Some(settings),
        }
    }
}

impl From<mullvad_types::relay_constraints::RelaySettings> for RelaySettings {
    fn from(settings: mullvad_types::relay_constraints::RelaySettings) -> Self {
        use mullvad_types::relay_constraints::RelaySettings as MullvadRelaySettings;
        use talpid_types::net as talpid_net;

        let endpoint = match settings {
            MullvadRelaySettings::CustomTunnelEndpoint(endpoint) => {
                relay_settings::Endpoint::Custom(CustomRelaySettings {
                    host: endpoint.host,
                    config: Some(ConnectionConfig::from(endpoint.config)),
                })
            }
            MullvadRelaySettings::Normal(constraints) => {
                relay_settings::Endpoint::Normal(NormalRelaySettings {
                    location: constraints.location.option().map(RelayLocation::from),
                    providers: convert_providers_constraint(&constraints.providers),
                    tunnel_type: match constraints.tunnel_protocol {
                        Constraint::Any => None,
                        Constraint::Only(talpid_net::TunnelType::Wireguard) => {
                            Some(TunnelType::Wireguard)
                        }
                        Constraint::Only(talpid_net::TunnelType::OpenVpn) => {
                            Some(TunnelType::Openvpn)
                        }
                    }
                    .map(|tunnel_type| TunnelTypeConstraint {
                        tunnel_type: i32::from(tunnel_type),
                    }),

                    wireguard_constraints: Some(WireguardConstraints {
                        port: u32::from(constraints.wireguard_constraints.port.unwrap_or(0)),
                        ip_version: constraints
                            .wireguard_constraints
                            .ip_version
                            .option()
                            .map(IpVersion::from)
                            .map(IpVersionConstraint::from),
                    }),

                    openvpn_constraints: Some(OpenvpnConstraints {
                        port: u32::from(constraints.openvpn_constraints.port.unwrap_or(0)),
                        protocol: constraints
                            .openvpn_constraints
                            .protocol
                            .as_ref()
                            .option()
                            .map(|protocol| TransportProtocol::from(*protocol))
                            .map(TransportProtocolConstraint::from),
                    }),
                })
            }
        };

        Self {
            endpoint: Some(endpoint),
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

fn convert_providers_constraint(
    providers: &Constraint<mullvad_types::relay_constraints::Providers>,
) -> Vec<String> {
    match providers.as_ref() {
        Constraint::Any => vec![],
        Constraint::Only(providers) => Vec::from(providers.clone()),
    }
}

use crate::types::{conversions::net::try_tunnel_type_from_i32, proto, FromProtobufTypeError};
use mullvad_types::{
    custom_list::Id,
    relay_constraints::{Constraint, GeographicLocationConstraint},
};
use std::str::FromStr;

impl TryFrom<&proto::WireguardConstraints>
    for mullvad_types::relay_constraints::WireguardConstraints
{
    type Error = FromProtobufTypeError;

    fn try_from(
        constraints: &proto::WireguardConstraints,
    ) -> Result<mullvad_types::relay_constraints::WireguardConstraints, Self::Error> {
        use mullvad_types::relay_constraints as mullvad_constraints;
        use talpid_types::net;

        let ip_version = match constraints.ip_version {
            Some(version) => Some(net::IpVersion::from(
                proto::IpVersion::try_from(version).map_err(|_| {
                    FromProtobufTypeError::InvalidArgument("invalid IP protocol version")
                })?,
            )),
            None => None,
        };

        Ok(mullvad_constraints::WireguardConstraints {
            port: Constraint::from(constraints.port.map(|port| port as u16)),
            ip_version: Constraint::from(ip_version),
            use_multihop: constraints.use_multihop,
            entry_location: constraints
                .entry_location
                .clone()
                .and_then(|loc| {
                    Constraint::<mullvad_types::relay_constraints::LocationConstraint>::try_from(
                        loc,
                    )
                    .ok()
                })
                .unwrap_or(Constraint::Any),
        })
    }
}

impl TryFrom<&proto::OpenvpnConstraints> for mullvad_types::relay_constraints::OpenVpnConstraints {
    type Error = FromProtobufTypeError;

    fn try_from(
        constraints: &proto::OpenvpnConstraints,
    ) -> Result<mullvad_types::relay_constraints::OpenVpnConstraints, Self::Error> {
        use mullvad_types::relay_constraints as mullvad_constraints;

        Ok(mullvad_constraints::OpenVpnConstraints {
            port: Constraint::from(match &constraints.port {
                Some(port) => Some(mullvad_constraints::TransportPort::try_from(port.clone())?),
                None => None,
            }),
        })
    }
}

impl TryFrom<proto::RelaySettings> for mullvad_types::relay_constraints::RelaySettings {
    type Error = FromProtobufTypeError;

    fn try_from(
        settings: proto::RelaySettings,
    ) -> Result<mullvad_types::relay_constraints::RelaySettings, Self::Error> {
        use mullvad_types::{relay_constraints as mullvad_constraints, CustomTunnelEndpoint};

        let update_value = settings
            .endpoint
            .ok_or(FromProtobufTypeError::InvalidArgument(
                "missing relay settings",
            ))?;

        match update_value {
            proto::relay_settings::Endpoint::Custom(settings) => {
                let config = settings
                    .config
                    .ok_or(FromProtobufTypeError::InvalidArgument(
                        "missing relay connection config",
                    ))?;
                let config = mullvad_types::ConnectionConfig::try_from(config)?;
                Ok(mullvad_constraints::RelaySettings::CustomTunnelEndpoint(
                    CustomTunnelEndpoint {
                        host: settings.host,
                        config,
                    },
                ))
            }

            proto::relay_settings::Endpoint::Normal(settings) => {
                let location = settings
                    .location
                    .and_then(|loc| Constraint::<mullvad_types::relay_constraints::LocationConstraint>::try_from(loc).ok())
                    .unwrap_or(Constraint::Any);
                let providers = try_providers_constraint_from_proto(&settings.providers)?;
                let ownership = try_ownership_constraint_from_i32(settings.ownership)?;
                let tunnel_protocol = Constraint::from(
                    settings
                        .tunnel_type
                        .map(try_tunnel_type_from_i32)
                        .transpose()?,
                );
                let openvpn_constraints =
                    mullvad_constraints::OpenVpnConstraints::try_from(
                        &settings.openvpn_constraints.ok_or(
                            FromProtobufTypeError::InvalidArgument("missing openvpn constraints"),
                        )?,
                    )?;
                let wireguard_constraints = mullvad_constraints::WireguardConstraints::try_from(
                    &settings.wireguard_constraints.ok_or(
                        FromProtobufTypeError::InvalidArgument("missing wireguard constraints"),
                    )?,
                )?;

                Ok(mullvad_constraints::RelaySettings::Normal(
                    mullvad_constraints::RelayConstraints {
                        location,
                        providers,
                        ownership,
                        tunnel_protocol,
                        wireguard_constraints,
                        openvpn_constraints,
                    },
                ))
            }
        }
    }
}

impl From<mullvad_types::relay_constraints::BridgeState> for proto::BridgeState {
    fn from(state: mullvad_types::relay_constraints::BridgeState) -> Self {
        use mullvad_types::relay_constraints::BridgeState;
        Self {
            state: i32::from(match state {
                BridgeState::Auto => proto::bridge_state::State::Auto,
                BridgeState::On => proto::bridge_state::State::On,
                BridgeState::Off => proto::bridge_state::State::Off,
            }),
        }
    }
}

impl From<&mullvad_types::relay_constraints::ObfuscationSettings> for proto::ObfuscationSettings {
    fn from(settings: &mullvad_types::relay_constraints::ObfuscationSettings) -> Self {
        use mullvad_types::relay_constraints::SelectedObfuscation;
        let selected_obfuscation = i32::from(match settings.selected_obfuscation {
            SelectedObfuscation::Auto => proto::obfuscation_settings::SelectedObfuscation::Auto,
            SelectedObfuscation::Off => proto::obfuscation_settings::SelectedObfuscation::Off,
            SelectedObfuscation::Udp2Tcp => {
                proto::obfuscation_settings::SelectedObfuscation::Udp2tcp
            }
        });
        Self {
            selected_obfuscation,
            udp2tcp: Some(proto::Udp2TcpObfuscationSettings::from(&settings.udp2tcp)),
        }
    }
}

impl From<mullvad_types::relay_constraints::ObfuscationSettings> for proto::ObfuscationSettings {
    fn from(settings: mullvad_types::relay_constraints::ObfuscationSettings) -> Self {
        proto::ObfuscationSettings::from(&settings)
    }
}

impl From<&mullvad_types::relay_constraints::Udp2TcpObfuscationSettings>
    for proto::Udp2TcpObfuscationSettings
{
    fn from(settings: &mullvad_types::relay_constraints::Udp2TcpObfuscationSettings) -> Self {
        Self {
            port: settings.port.map(u32::from).option(),
        }
    }
}

impl From<mullvad_types::relay_constraints::BridgeSettings> for proto::BridgeSettings {
    fn from(settings: mullvad_types::relay_constraints::BridgeSettings) -> Self {
        use mullvad_types::relay_constraints::BridgeSettings as MullvadBridgeSettings;
        use proto::bridge_settings;
        use talpid_types::net as talpid_net;

        let settings = match settings {
            MullvadBridgeSettings::Normal(constraints) => {
                bridge_settings::Type::Normal(bridge_settings::BridgeConstraints {
                    location: constraints
                        .location
                        .clone()
                        .option()
                        .map(proto::LocationConstraint::from),
                    providers: convert_providers_constraint(&constraints.providers),
                    ownership: convert_ownership_constraint(&constraints.ownership) as i32,
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
                        cipher: proxy_settings.cipher,
                    })
                }
            },
        };

        proto::BridgeSettings {
            r#type: Some(settings),
        }
    }
}

impl From<mullvad_types::relay_constraints::RelaySettings> for proto::RelaySettings {
    fn from(settings: mullvad_types::relay_constraints::RelaySettings) -> Self {
        use mullvad_types::relay_constraints::RelaySettings as MullvadRelaySettings;
        use proto::relay_settings;
        use talpid_types::net as talpid_net;

        let endpoint = match settings {
            MullvadRelaySettings::CustomTunnelEndpoint(endpoint) => {
                relay_settings::Endpoint::Custom(proto::CustomRelaySettings {
                    host: endpoint.host,
                    config: Some(proto::ConnectionConfig::from(endpoint.config)),
                })
            }
            MullvadRelaySettings::Normal(constraints) => {
                relay_settings::Endpoint::Normal(proto::NormalRelaySettings {
                    location: constraints
                        .location
                        .option()
                        .map(proto::LocationConstraint::from),
                    providers: convert_providers_constraint(&constraints.providers),
                    ownership: convert_ownership_constraint(&constraints.ownership) as i32,
                    tunnel_type: match constraints.tunnel_protocol {
                        Constraint::Any => None,
                        Constraint::Only(talpid_net::TunnelType::Wireguard) => {
                            Some(proto::TunnelType::Wireguard)
                        }
                        Constraint::Only(talpid_net::TunnelType::OpenVpn) => {
                            Some(proto::TunnelType::Openvpn)
                        }
                    }
                    .map(i32::from),

                    wireguard_constraints: Some(proto::WireguardConstraints {
                        port: constraints
                            .wireguard_constraints
                            .port
                            .map(u32::from)
                            .option(),
                        ip_version: constraints
                            .wireguard_constraints
                            .ip_version
                            .option()
                            .map(|ipv| i32::from(proto::IpVersion::from(ipv))),
                        use_multihop: constraints.wireguard_constraints.use_multihop,
                        entry_location: constraints
                            .wireguard_constraints
                            .entry_location
                            .option()
                            .map(proto::LocationConstraint::from),
                    }),

                    openvpn_constraints: Some(proto::OpenvpnConstraints {
                        port: constraints
                            .openvpn_constraints
                            .port
                            .option()
                            .map(proto::TransportPort::from),
                    }),
                })
            }
        };

        Self {
            endpoint: Some(endpoint),
        }
    }
}

impl From<mullvad_types::relay_constraints::TransportPort> for proto::TransportPort {
    fn from(port: mullvad_types::relay_constraints::TransportPort) -> Self {
        proto::TransportPort {
            protocol: proto::TransportProtocol::from(port.protocol) as i32,
            port: port.port.map(u32::from).option(),
        }
    }
}

impl From<mullvad_types::relay_constraints::LocationConstraint> for proto::LocationConstraint {
    fn from(location: mullvad_types::relay_constraints::LocationConstraint) -> Self {
        use mullvad_types::relay_constraints::LocationConstraint;
        match location {
            LocationConstraint::Location(location) => Self {
                r#type: Some(proto::location_constraint::Type::Location(
                    proto::GeographicLocationConstraint::from(location),
                )),
            },
            LocationConstraint::CustomList { list_id } => Self {
                r#type: Some(proto::location_constraint::Type::CustomList(
                    list_id.to_string(),
                )),
            },
        }
    }
}

impl TryFrom<proto::LocationConstraint>
    for Constraint<mullvad_types::relay_constraints::LocationConstraint>
{
    type Error = FromProtobufTypeError;

    fn try_from(location: proto::LocationConstraint) -> Result<Self, Self::Error> {
        use mullvad_types::relay_constraints::LocationConstraint;
        match location.r#type {
            Some(proto::location_constraint::Type::Location(location)) => Ok(Constraint::Only(
                LocationConstraint::Location(GeographicLocationConstraint::try_from(location)?),
            )),
            Some(proto::location_constraint::Type::CustomList(list_id)) => {
                let location = LocationConstraint::CustomList {
                    list_id: Id::from_str(&list_id).map_err(|_| {
                        FromProtobufTypeError::InvalidArgument("Id could not be parsed to a uuid")
                    })?,
                };
                Ok(Constraint::Only(location))
            }
            None => Ok(Constraint::Any),
        }
    }
}

impl From<GeographicLocationConstraint> for proto::GeographicLocationConstraint {
    fn from(location: mullvad_types::relay_constraints::GeographicLocationConstraint) -> Self {
        match location {
            GeographicLocationConstraint::Country(country) => Self {
                country,
                ..Default::default()
            },
            GeographicLocationConstraint::City(country, city) => Self {
                country,
                city: Some(city),
                hostname: None,
            },
            GeographicLocationConstraint::Hostname(country, city, hostname) => Self {
                country,
                city: Some(city),
                hostname: Some(hostname),
            },
        }
    }
}

impl TryFrom<proto::GeographicLocationConstraint> for GeographicLocationConstraint {
    type Error = FromProtobufTypeError;

    fn try_from(relay_location: proto::GeographicLocationConstraint) -> Result<Self, Self::Error> {
        match (
            relay_location.country,
            relay_location.city,
            relay_location.hostname,
        ) {
            (country, None, None) => Ok(GeographicLocationConstraint::Country(country)),
            (country, Some(city), None) => Ok(GeographicLocationConstraint::City(country, city)),
            (country, Some(city), Some(hostname)) => Ok(GeographicLocationConstraint::Hostname(
                country, city, hostname,
            )),
            (_country, None, Some(_hostname)) => Err(FromProtobufTypeError::InvalidArgument(
                "Relay location contains hostname but no city",
            )),
        }
    }
}

impl TryFrom<proto::BridgeSettings> for mullvad_types::relay_constraints::BridgeSettings {
    type Error = FromProtobufTypeError;

    fn try_from(settings: proto::BridgeSettings) -> Result<Self, Self::Error> {
        use mullvad_types::relay_constraints as mullvad_constraints;
        use talpid_types::net as talpid_net;

        match settings
            .r#type
            .ok_or(FromProtobufTypeError::InvalidArgument(
                "no settings provided",
            ))? {
            proto::bridge_settings::Type::Normal(constraints) => {
                let location = match constraints.location {
                    None => Constraint::Any,
                    Some(location) => Constraint::<
                        mullvad_types::relay_constraints::LocationConstraint,
                    >::try_from(location)?,
                };
                let providers = try_providers_constraint_from_proto(&constraints.providers)?;
                let ownership = try_ownership_constraint_from_i32(constraints.ownership)?;

                Ok(mullvad_constraints::BridgeSettings::Normal(
                    mullvad_constraints::BridgeConstraints {
                        location,
                        providers,
                        ownership,
                    },
                ))
            }
            proto::bridge_settings::Type::Local(proxy_settings) => {
                let peer = proxy_settings.peer.parse().map_err(|_| {
                    FromProtobufTypeError::InvalidArgument("failed to parse peer address")
                })?;
                let proxy_settings = talpid_net::openvpn::ProxySettings::Local(
                    talpid_net::openvpn::LocalProxySettings {
                        port: proxy_settings.port as u16,
                        peer,
                    },
                );
                Ok(mullvad_constraints::BridgeSettings::Custom(proxy_settings))
            }
            proto::bridge_settings::Type::Remote(proxy_settings) => {
                let address = proxy_settings.address.parse().map_err(|_| {
                    FromProtobufTypeError::InvalidArgument("failed to parse IP address")
                })?;
                let auth = proxy_settings
                    .auth
                    .map(|auth| talpid_net::openvpn::ProxyAuth {
                        username: auth.username,
                        password: auth.password,
                    });
                let proxy_settings = talpid_net::openvpn::ProxySettings::Remote(
                    talpid_net::openvpn::RemoteProxySettings { address, auth },
                );
                Ok(mullvad_constraints::BridgeSettings::Custom(proxy_settings))
            }
            proto::bridge_settings::Type::Shadowsocks(proxy_settings) => {
                let peer = proxy_settings.peer.parse().map_err(|_| {
                    FromProtobufTypeError::InvalidArgument("failed to parse peer address")
                })?;
                let proxy_settings = talpid_net::openvpn::ProxySettings::Shadowsocks(
                    talpid_net::openvpn::ShadowsocksProxySettings {
                        #[cfg(target_os = "linux")]
                        fwmark: Some(mullvad_types::TUNNEL_FWMARK),
                        peer,
                        password: proxy_settings.password,
                        cipher: proxy_settings.cipher,
                    },
                );
                Ok(mullvad_constraints::BridgeSettings::Custom(proxy_settings))
            }
        }
    }
}

impl TryFrom<proto::ObfuscationSettings> for mullvad_types::relay_constraints::ObfuscationSettings {
    type Error = FromProtobufTypeError;

    fn try_from(settings: proto::ObfuscationSettings) -> Result<Self, Self::Error> {
        use mullvad_types::relay_constraints::SelectedObfuscation;
        use proto::obfuscation_settings::SelectedObfuscation as IpcSelectedObfuscation;
        let selected_obfuscation =
            match IpcSelectedObfuscation::try_from(settings.selected_obfuscation) {
                Ok(IpcSelectedObfuscation::Auto) => SelectedObfuscation::Auto,
                Ok(IpcSelectedObfuscation::Off) => SelectedObfuscation::Off,
                Ok(IpcSelectedObfuscation::Udp2tcp) => SelectedObfuscation::Udp2Tcp,
                Err(_) => {
                    return Err(FromProtobufTypeError::InvalidArgument(
                        "invalid selected obfuscator",
                    ));
                }
            };

        let udp2tcp = match settings.udp2tcp {
            Some(settings) => {
                mullvad_types::relay_constraints::Udp2TcpObfuscationSettings::try_from(&settings)?
            }
            None => {
                return Err(FromProtobufTypeError::InvalidArgument(
                    "invalid selected obfuscator",
                ));
            }
        };

        Ok(Self {
            selected_obfuscation,
            udp2tcp,
        })
    }
}

impl TryFrom<&proto::Udp2TcpObfuscationSettings>
    for mullvad_types::relay_constraints::Udp2TcpObfuscationSettings
{
    type Error = FromProtobufTypeError;

    fn try_from(settings: &proto::Udp2TcpObfuscationSettings) -> Result<Self, Self::Error> {
        Ok(Self {
            port: Constraint::from(settings.port.map(|port| port as u16)),
        })
    }
}

impl TryFrom<proto::BridgeState> for mullvad_types::relay_constraints::BridgeState {
    type Error = FromProtobufTypeError;

    fn try_from(state: proto::BridgeState) -> Result<Self, Self::Error> {
        match proto::bridge_state::State::try_from(state.state) {
            Ok(proto::bridge_state::State::Auto) => {
                Ok(mullvad_types::relay_constraints::BridgeState::Auto)
            }
            Ok(proto::bridge_state::State::On) => {
                Ok(mullvad_types::relay_constraints::BridgeState::On)
            }
            Ok(proto::bridge_state::State::Off) => {
                Ok(mullvad_types::relay_constraints::BridgeState::Off)
            }
            Err(_) => Err(FromProtobufTypeError::InvalidArgument(
                "invalid bridge state",
            )),
        }
    }
}

impl TryFrom<proto::TransportPort> for mullvad_types::relay_constraints::TransportPort {
    type Error = FromProtobufTypeError;

    fn try_from(port: proto::TransportPort) -> Result<Self, Self::Error> {
        Ok(mullvad_types::relay_constraints::TransportPort {
            protocol: super::net::try_transport_protocol_from_i32(port.protocol)?,
            port: Constraint::from(port.port.map(|port| port as u16)),
        })
    }
}

pub fn try_providers_constraint_from_proto(
    providers: &[String],
) -> Result<Constraint<mullvad_types::relay_constraints::Providers>, FromProtobufTypeError> {
    if !providers.is_empty() {
        Ok(Constraint::Only(
            mullvad_types::relay_constraints::Providers::new(providers.iter().cloned()).map_err(
                |_| FromProtobufTypeError::InvalidArgument("must specify at least one provider"),
            )?,
        ))
    } else {
        Ok(Constraint::Any)
    }
}

pub fn try_ownership_constraint_from_i32(
    ownership: i32,
) -> Result<Constraint<mullvad_types::relay_constraints::Ownership>, FromProtobufTypeError> {
    proto::Ownership::try_from(ownership)
        .map(ownership_constraint_from_proto)
        .map_err(|_| FromProtobufTypeError::InvalidArgument("invalid ownership argument"))
}

pub fn ownership_constraint_from_proto(
    ownership: proto::Ownership,
) -> Constraint<mullvad_types::relay_constraints::Ownership> {
    use mullvad_types::relay_constraints::Ownership as MullvadOwnership;

    match ownership {
        proto::Ownership::Any => Constraint::Any,
        proto::Ownership::MullvadOwned => Constraint::Only(MullvadOwnership::MullvadOwned),
        proto::Ownership::Rented => Constraint::Only(MullvadOwnership::Rented),
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

fn convert_ownership_constraint(
    ownership: &Constraint<mullvad_types::relay_constraints::Ownership>,
) -> proto::Ownership {
    use mullvad_types::relay_constraints::Ownership as MullvadOwnership;

    match ownership.as_ref() {
        Constraint::Any => proto::Ownership::Any,
        Constraint::Only(ownership) => match ownership {
            MullvadOwnership::MullvadOwned => proto::Ownership::MullvadOwned,
            MullvadOwnership::Rented => proto::Ownership::Rented,
        },
    }
}

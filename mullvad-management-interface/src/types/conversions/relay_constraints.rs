use crate::types::{conversions::net::try_tunnel_type_from_i32, proto, FromProtobufTypeError};
use mullvad_types::{
    constraints::Constraint, custom_list::Id, relay_constraints::GeographicLocationConstraint,
};
use std::str::FromStr;
use talpid_types::net::proxy::CustomProxy;

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
            SelectedObfuscation::Shadowsocks => {
                proto::obfuscation_settings::SelectedObfuscation::Shadowsocks
            }
        });
        Self {
            selected_obfuscation,
            udp2tcp: Some(proto::Udp2TcpObfuscationSettings::from(&settings.udp2tcp)),
            shadowsocks: Some(proto::ShadowsocksSettings::from(&settings.shadowsocks)),
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

impl From<&mullvad_types::relay_constraints::ShadowsocksSettings> for proto::ShadowsocksSettings {
    fn from(settings: &mullvad_types::relay_constraints::ShadowsocksSettings) -> Self {
        Self {
            port: settings.port.map(u32::from).option(),
        }
    }
}

impl From<mullvad_types::relay_constraints::BridgeSettings> for proto::BridgeSettings {
    fn from(settings: mullvad_types::relay_constraints::BridgeSettings) -> Self {
        use proto::bridge_settings;

        let mode = match settings.bridge_type {
            mullvad_types::relay_constraints::BridgeType::Normal => {
                bridge_settings::BridgeType::Normal
            }
            mullvad_types::relay_constraints::BridgeType::Custom => {
                bridge_settings::BridgeType::Custom
            }
        };

        let normal = bridge_settings::BridgeConstraints {
            location: settings
                .normal
                .location
                .clone()
                .option()
                .map(proto::LocationConstraint::from),
            providers: convert_providers_constraint(&settings.normal.providers),
            ownership: i32::from(convert_ownership_constraint(&settings.normal.ownership)),
        };

        let custom = settings.custom.map(proto::CustomProxy::from);

        proto::BridgeSettings {
            bridge_type: i32::from(mode),
            normal: Some(normal),
            custom,
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
                        use_multihop: constraints.wireguard_constraints.multihop(),
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

pub fn try_bridge_mode_from_i32(
    mode: i32,
) -> Result<mullvad_types::relay_constraints::BridgeType, FromProtobufTypeError> {
    proto::bridge_settings::BridgeType::try_from(mode)
        .map(mullvad_types::relay_constraints::BridgeType::from)
        .map_err(|_| FromProtobufTypeError::InvalidArgument("invalid bridge mode argument"))
}

impl From<proto::bridge_settings::BridgeType> for mullvad_types::relay_constraints::BridgeType {
    fn from(value: proto::bridge_settings::BridgeType) -> Self {
        use mullvad_types::relay_constraints::BridgeType;

        match value {
            proto::bridge_settings::BridgeType::Normal => BridgeType::Normal,
            proto::bridge_settings::BridgeType::Custom => BridgeType::Custom,
        }
    }
}

impl TryFrom<proto::BridgeSettings> for mullvad_types::relay_constraints::BridgeSettings {
    type Error = FromProtobufTypeError;

    fn try_from(settings: proto::BridgeSettings) -> Result<Self, Self::Error> {
        use mullvad_types::relay_constraints::{BridgeConstraints, BridgeSettings};

        // convert normal bridge settings
        let constraints = settings
            .normal
            .ok_or(FromProtobufTypeError::InvalidArgument(
                "missing normal bridge constraints",
            ))?;
        let location = match constraints.location {
            None => Constraint::Any,
            Some(location) => {
                Constraint::<mullvad_types::relay_constraints::LocationConstraint>::try_from(
                    location,
                )?
            }
        };
        let normal = BridgeConstraints {
            location,
            providers: try_providers_constraint_from_proto(&constraints.providers)?,
            ownership: try_ownership_constraint_from_i32(constraints.ownership)?,
        };

        // convert custom bridge settings
        let custom = settings.custom.map(CustomProxy::try_from).transpose()?;

        Ok(BridgeSettings {
            bridge_type: try_bridge_mode_from_i32(settings.bridge_type)?,
            normal,
            custom,
        })
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
                Ok(IpcSelectedObfuscation::Shadowsocks) => SelectedObfuscation::Shadowsocks,
                Err(_) => {
                    return Err(FromProtobufTypeError::InvalidArgument(
                        "invalid obfuscation settings",
                    ));
                }
            };

        let udp2tcp = match settings.udp2tcp {
            Some(settings) => {
                mullvad_types::relay_constraints::Udp2TcpObfuscationSettings::try_from(&settings)?
            }
            None => {
                return Err(FromProtobufTypeError::InvalidArgument(
                    "invalid udp2tcp settings",
                ));
            }
        };
        let shadowsocks = match settings.shadowsocks {
            Some(settings) => {
                mullvad_types::relay_constraints::ShadowsocksSettings::try_from(&settings)?
            }
            None => {
                return Err(FromProtobufTypeError::InvalidArgument(
                    "invalid shadowsocks settings",
                ));
            }
        };

        Ok(Self {
            selected_obfuscation,
            udp2tcp,
            shadowsocks,
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

impl TryFrom<&proto::ShadowsocksSettings>
    for mullvad_types::relay_constraints::ShadowsocksSettings
{
    type Error = FromProtobufTypeError;

    fn try_from(settings: &proto::ShadowsocksSettings) -> Result<Self, Self::Error> {
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

impl From<mullvad_types::relay_constraints::RelayOverride> for proto::RelayOverride {
    fn from(r#override: mullvad_types::relay_constraints::RelayOverride) -> proto::RelayOverride {
        proto::RelayOverride {
            hostname: r#override.hostname,
            ipv4_addr_in: r#override.ipv4_addr_in.map(|addr| addr.to_string()),
            ipv6_addr_in: r#override.ipv6_addr_in.map(|addr| addr.to_string()),
        }
    }
}

impl TryFrom<proto::RelayOverride> for mullvad_types::relay_constraints::RelayOverride {
    type Error = FromProtobufTypeError;

    fn try_from(
        r#override: proto::RelayOverride,
    ) -> Result<mullvad_types::relay_constraints::RelayOverride, Self::Error> {
        Ok(mullvad_types::relay_constraints::RelayOverride {
            hostname: r#override.hostname,
            ipv4_addr_in: r#override
                .ipv4_addr_in
                .map(|addr| {
                    addr.parse()
                        .map_err(|_| FromProtobufTypeError::InvalidArgument("invalid IPv4 address"))
                })
                .transpose()?,
            ipv6_addr_in: r#override
                .ipv6_addr_in
                .map(|addr| {
                    addr.parse()
                        .map_err(|_| FromProtobufTypeError::InvalidArgument("invalid IPv6 address"))
                })
                .transpose()?,
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

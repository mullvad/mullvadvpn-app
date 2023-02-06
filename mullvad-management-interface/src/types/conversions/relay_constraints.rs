use crate::types::{conversions::option_from_proto_string, proto, FromProtobufTypeError};
use mullvad_types::relay_constraints::{Constraint, RelaySettingsUpdate};
use talpid_types::net::TunnelType;

impl TryFrom<&proto::WireguardConstraints>
    for mullvad_types::relay_constraints::WireguardConstraints
{
    type Error = FromProtobufTypeError;

    fn try_from(
        constraints: &proto::WireguardConstraints,
    ) -> Result<mullvad_types::relay_constraints::WireguardConstraints, Self::Error> {
        use mullvad_types::relay_constraints as mullvad_constraints;
        use talpid_types::net;

        let ip_version = match &constraints.ip_version {
            Some(constraint) => match proto::IpVersion::from_i32(constraint.protocol) {
                Some(proto::IpVersion::V4) => Some(net::IpVersion::V4),
                Some(proto::IpVersion::V6) => Some(net::IpVersion::V6),
                None => {
                    return Err(FromProtobufTypeError::InvalidArgument(
                        "invalid ip protocol version",
                    ))
                }
            },
            None => None,
        };

        Ok(mullvad_constraints::WireguardConstraints {
            port: if constraints.port == 0 {
                Constraint::Any
            } else {
                Constraint::Only(constraints.port as u16)
            },
            ip_version: Constraint::from(ip_version),
            use_multihop: constraints.use_multihop,
            entry_location: constraints
                .entry_location
                .clone()
                .map(Constraint::<mullvad_types::relay_constraints::LocationConstraint>::from)
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
        use talpid_types::net;

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
                    .map(Constraint::<mullvad_types::relay_constraints::LocationConstraint>::from)
                    .unwrap_or(Constraint::Any);
                let providers = try_providers_constraint_from_proto(&settings.providers)?;
                let ownership = try_ownership_constraint_from_i32(settings.ownership)?;
                let tunnel_protocol = settings
                    .tunnel_type
                    .map(Constraint::<net::TunnelType>::try_from)
                    .transpose()?
                    .unwrap_or(Constraint::Any);
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

impl From<RelaySettingsUpdate> for proto::RelaySettingsUpdate {
    fn from(relay_settings_update: RelaySettingsUpdate) -> Self {
        match relay_settings_update {
            RelaySettingsUpdate::Normal(constraints) => proto::RelaySettingsUpdate {
                r#type: Some(proto::relay_settings_update::Type::Normal(
                    proto::NormalRelaySettingsUpdate {
                        location: constraints.location.map(proto::RelayLocation::from),
                        providers: constraints
                            .providers
                            .map(|constraint| proto::ProviderUpdate {
                                providers: convert_providers_constraint(&constraint),
                            }),
                        ownership: constraints
                            .ownership
                            .map(|ownership| proto::OwnershipUpdate {
                                ownership: i32::from(convert_ownership_constraint(&ownership)),
                            }),
                        tunnel_type: constraints.tunnel_protocol.map(|protocol| {
                            proto::TunnelTypeUpdate {
                                tunnel_type: match protocol {
                                    Constraint::Any => None,
                                    Constraint::Only(protocol) => {
                                        Some(proto::TunnelTypeConstraint {
                                            tunnel_type: i32::from(match protocol {
                                                TunnelType::Wireguard => {
                                                    proto::TunnelType::Wireguard
                                                }
                                                TunnelType::OpenVpn => proto::TunnelType::Openvpn,
                                            }),
                                        })
                                    }
                                },
                            }
                        }),
                        wireguard_constraints: constraints.wireguard_constraints.map(
                            |wireguard_constraints| proto::WireguardConstraints {
                                port: u32::from(wireguard_constraints.port.unwrap_or(0)),
                                ip_version: wireguard_constraints
                                    .ip_version
                                    .option()
                                    .map(proto::IpVersion::from)
                                    .map(proto::IpVersionConstraint::from),
                                use_multihop: wireguard_constraints.use_multihop,
                                entry_location: wireguard_constraints
                                    .entry_location
                                    .option()
                                    .map(proto::RelayLocation::from),
                            },
                        ),
                        openvpn_constraints: constraints.openvpn_constraints.map(
                            |openvpn_constraints| proto::OpenvpnConstraints {
                                port: openvpn_constraints
                                    .port
                                    .option()
                                    .map(proto::TransportPort::from),
                            },
                        ),
                    },
                )),
            },
            RelaySettingsUpdate::CustomTunnelEndpoint(endpoint) => proto::RelaySettingsUpdate {
                r#type: Some(proto::relay_settings_update::Type::Custom(
                    proto::CustomRelaySettings {
                        host: endpoint.host.to_string(),
                        config: Some(proto::ConnectionConfig::from(endpoint.config)),
                    },
                )),
            },
        }
    }
}

impl TryFrom<proto::RelaySettingsUpdate> for mullvad_types::relay_constraints::RelaySettingsUpdate {
    type Error = FromProtobufTypeError;

    fn try_from(
        settings: proto::RelaySettingsUpdate,
    ) -> Result<mullvad_types::relay_constraints::RelaySettingsUpdate, Self::Error> {
        use mullvad_types::{relay_constraints as mullvad_constraints, CustomTunnelEndpoint};
        use talpid_types::net;

        let update_value = settings
            .r#type
            .ok_or(FromProtobufTypeError::InvalidArgument(
                "missing relay settings",
            ))?;

        match update_value {
            proto::relay_settings_update::Type::Custom(settings) => {
                let config = settings
                    .config
                    .ok_or(FromProtobufTypeError::InvalidArgument(
                        "missing relay connection config",
                    ))?;
                let config = mullvad_types::ConnectionConfig::try_from(config)?;
                Ok(
                    mullvad_constraints::RelaySettingsUpdate::CustomTunnelEndpoint(
                        CustomTunnelEndpoint {
                            host: settings.host,
                            config,
                        },
                    ),
                )
            }

            proto::relay_settings_update::Type::Normal(settings) => {
                // If `location` isn't provided, no changes are made.
                // If `location` is provided, but is an empty vector,
                // then the constraint is set to `Constraint::Any`.
                let location = settings
                    .location
                    .map(Constraint::<mullvad_types::relay_constraints::LocationConstraint>::from);
                let providers = if let Some(ref provider_update) = settings.providers {
                    Some(try_providers_constraint_from_proto(
                        &provider_update.providers,
                    )?)
                } else {
                    None
                };
                let ownership = if let Some(ref ownership_update) = settings.ownership {
                    Some(try_ownership_constraint_from_i32(
                        ownership_update.ownership,
                    )?)
                } else {
                    None
                };
                let tunnel_protocol = if let Some(update) = settings.tunnel_type {
                    Some(
                        update
                            .tunnel_type
                            .map(Constraint::<net::TunnelType>::try_from)
                            .transpose()?
                            .unwrap_or(Constraint::Any),
                    )
                } else {
                    None
                };
                let openvpn_constraints =
                    if let Some(ref constraints) = settings.openvpn_constraints {
                        Some(mullvad_constraints::OpenVpnConstraints::try_from(
                            constraints,
                        )?)
                    } else {
                        None
                    };
                let wireguard_constraints =
                    if let Some(ref constraints) = settings.wireguard_constraints {
                        Some(mullvad_constraints::WireguardConstraints::try_from(
                            constraints,
                        )?)
                    } else {
                        None
                    };
                Ok(mullvad_constraints::RelaySettingsUpdate::Normal(
                    mullvad_constraints::RelayConstraintsUpdate {
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

impl From<&mullvad_types::relay_constraints::Udp2TcpObfuscationSettings>
    for proto::Udp2TcpObfuscationSettings
{
    fn from(settings: &mullvad_types::relay_constraints::Udp2TcpObfuscationSettings) -> Self {
        Self {
            port: u32::from(settings.port.unwrap_or(0)),
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
                        .map(proto::RelayLocation::from),
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
                        .map(proto::RelayLocation::from),
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
                    .map(|tunnel_type| proto::TunnelTypeConstraint {
                        tunnel_type: i32::from(tunnel_type),
                    }),

                    wireguard_constraints: Some(proto::WireguardConstraints {
                        port: u32::from(constraints.wireguard_constraints.port.unwrap_or(0)),
                        ip_version: constraints
                            .wireguard_constraints
                            .ip_version
                            .option()
                            .map(proto::IpVersion::from)
                            .map(proto::IpVersionConstraint::from),
                        use_multihop: constraints.wireguard_constraints.use_multihop,
                        entry_location: constraints
                            .wireguard_constraints
                            .entry_location
                            .option()
                            .map(proto::RelayLocation::from),
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
            port: port.port.map(u32::from).unwrap_or(0),
        }
    }
}

impl
    From<
        mullvad_types::relay_constraints::Constraint<
            mullvad_types::relay_constraints::LocationConstraint,
        >,
    > for proto::RelayLocation
{
    fn from(
        location: mullvad_types::relay_constraints::Constraint<
            mullvad_types::relay_constraints::LocationConstraint,
        >,
    ) -> Self {
        location
            .option()
            .map(proto::RelayLocation::from)
            .unwrap_or_default()
    }
}

impl From<mullvad_types::relay_constraints::LocationConstraint> for proto::RelayLocation {
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

impl From<proto::RelayLocation>
    for Constraint<mullvad_types::relay_constraints::LocationConstraint>
{
    fn from(location: proto::RelayLocation) -> Self {
        use mullvad_types::relay_constraints::LocationConstraint;

        if let Some(hostname) = option_from_proto_string(location.hostname) {
            Constraint::Only(LocationConstraint::Hostname(
                location.country,
                location.city,
                hostname,
            ))
        } else if let Some(city) = option_from_proto_string(location.city) {
            Constraint::Only(LocationConstraint::City(location.country, city))
        } else if let Some(country) = option_from_proto_string(location.country) {
            Constraint::Only(LocationConstraint::Country(country))
        } else {
            Constraint::Any
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
                    Some(location) => {
                        Constraint::<mullvad_constraints::LocationConstraint>::from(location)
                    }
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
            match IpcSelectedObfuscation::from_i32(settings.selected_obfuscation) {
                Some(IpcSelectedObfuscation::Auto) => SelectedObfuscation::Auto,
                Some(IpcSelectedObfuscation::Off) => SelectedObfuscation::Off,
                Some(IpcSelectedObfuscation::Udp2tcp) => SelectedObfuscation::Udp2Tcp,
                None => {
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
            port: if settings.port == 0 {
                Constraint::Any
            } else {
                Constraint::Only(settings.port as u16)
            },
        })
    }
}

impl TryFrom<proto::BridgeState> for mullvad_types::relay_constraints::BridgeState {
    type Error = FromProtobufTypeError;

    fn try_from(state: proto::BridgeState) -> Result<Self, Self::Error> {
        match proto::bridge_state::State::from_i32(state.state) {
            Some(proto::bridge_state::State::Auto) => {
                Ok(mullvad_types::relay_constraints::BridgeState::Auto)
            }
            Some(proto::bridge_state::State::On) => {
                Ok(mullvad_types::relay_constraints::BridgeState::On)
            }
            Some(proto::bridge_state::State::Off) => {
                Ok(mullvad_types::relay_constraints::BridgeState::Off)
            }
            None => Err(FromProtobufTypeError::InvalidArgument(
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
            port: if port.port == 0 {
                Constraint::Any
            } else {
                Constraint::Only(port.port as u16)
            },
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
    proto::Ownership::from_i32(ownership)
        .map(ownership_constraint_from_proto)
        .ok_or(FromProtobufTypeError::InvalidArgument(
            "invalid ownership argument",
        ))
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

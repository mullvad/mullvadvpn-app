pub use prost_types::{Duration, Timestamp};

use mullvad_types::relay_constraints::Constraint;
use std::convert::TryFrom;

tonic::include_proto!("mullvad_daemon.management_interface");

impl From<mullvad_types::location::GeoIpLocation> for GeoIpLocation {
    fn from(geoip: mullvad_types::location::GeoIpLocation) -> GeoIpLocation {
        GeoIpLocation {
            ipv4: geoip.ipv4.map(|ip| ip.to_string()).unwrap_or_default(),
            ipv6: geoip.ipv6.map(|ip| ip.to_string()).unwrap_or_default(),
            country: geoip.country,
            city: geoip.city.unwrap_or_default(),
            latitude: geoip.latitude,
            longitude: geoip.longitude,
            mullvad_exit_ip: geoip.mullvad_exit_ip,
            hostname: geoip.hostname.unwrap_or_default(),
            bridge_hostname: geoip.bridge_hostname.unwrap_or_default(),
        }
    }
}

impl From<talpid_types::net::TunnelEndpoint> for TunnelEndpoint {
    fn from(endpoint: talpid_types::net::TunnelEndpoint) -> Self {
        use talpid_types::net;

        TunnelEndpoint {
            address: endpoint.endpoint.address.to_string(),
            protocol: i32::from(TransportProtocol::from(endpoint.endpoint.protocol)),
            tunnel_type: match endpoint.tunnel_type {
                net::TunnelType::Wireguard => i32::from(TunnelType::Wireguard),
                net::TunnelType::OpenVpn => i32::from(TunnelType::Openvpn),
            },
            proxy: endpoint.proxy.map(|proxy_ep| ProxyEndpoint {
                address: proxy_ep.endpoint.address.to_string(),
                protocol: i32::from(TransportProtocol::from(proxy_ep.endpoint.protocol)),
                proxy_type: match proxy_ep.proxy_type {
                    net::proxy::ProxyType::Shadowsocks => i32::from(ProxyType::Shadowsocks),
                    net::proxy::ProxyType::Custom => i32::from(ProxyType::Custom),
                },
            }),
            entry_endpoint: endpoint.entry_endpoint.map(|entry| Endpoint {
                address: entry.address.to_string(),
                protocol: i32::from(TransportProtocol::from(entry.protocol)),
            }),
        }
    }
}

impl From<mullvad_types::states::TunnelState> for TunnelState {
    fn from(state: mullvad_types::states::TunnelState) -> Self {
        use error_state::{
            firewall_policy_error::ErrorType as PolicyErrorType, Cause, FirewallPolicyError,
            GenerationError,
        };
        use mullvad_types::states::TunnelState as MullvadTunnelState;

        use talpid_types::tunnel as talpid_tunnel;

        let map_firewall_error =
            |firewall_error: &talpid_tunnel::FirewallPolicyError| match firewall_error {
                talpid_tunnel::FirewallPolicyError::Generic => FirewallPolicyError {
                    r#type: i32::from(PolicyErrorType::Generic),
                    ..Default::default()
                },
                #[cfg(windows)]
                talpid_tunnel::FirewallPolicyError::Locked(blocking_app) => {
                    let (lock_pid, lock_name) = match blocking_app {
                        Some(app) => (app.pid, app.name.clone()),
                        None => (0, "".to_string()),
                    };

                    FirewallPolicyError {
                        r#type: i32::from(PolicyErrorType::Locked),
                        lock_pid,
                        lock_name,
                    }
                }
            };

        let state = match state {
            MullvadTunnelState::Disconnected => {
                tunnel_state::State::Disconnected(tunnel_state::Disconnected {})
            }
            MullvadTunnelState::Connecting { endpoint, location } => {
                tunnel_state::State::Connecting(tunnel_state::Connecting {
                    relay_info: Some(TunnelStateRelayInfo {
                        tunnel_endpoint: Some(TunnelEndpoint::from(endpoint)),
                        location: location.map(GeoIpLocation::from),
                    }),
                })
            }
            MullvadTunnelState::Connected { endpoint, location } => {
                tunnel_state::State::Connected(tunnel_state::Connected {
                    relay_info: Some(TunnelStateRelayInfo {
                        tunnel_endpoint: Some(TunnelEndpoint::from(endpoint)),
                        location: location.map(GeoIpLocation::from),
                    }),
                })
            }
            MullvadTunnelState::Disconnecting(after_disconnect) => {
                tunnel_state::State::Disconnecting(tunnel_state::Disconnecting {
                    after_disconnect: match after_disconnect {
                        talpid_tunnel::ActionAfterDisconnect::Nothing => {
                            i32::from(AfterDisconnect::Nothing)
                        }
                        talpid_tunnel::ActionAfterDisconnect::Block => {
                            i32::from(AfterDisconnect::Block)
                        }
                        talpid_tunnel::ActionAfterDisconnect::Reconnect => {
                            i32::from(AfterDisconnect::Reconnect)
                        }
                    },
                })
            }
            MullvadTunnelState::Error(error_state) => {
                tunnel_state::State::Error(tunnel_state::Error {
                    error_state: Some(ErrorState {
                        cause: match error_state.cause() {
                            talpid_tunnel::ErrorStateCause::AuthFailed(_) => {
                                i32::from(Cause::AuthFailed)
                            }
                            talpid_tunnel::ErrorStateCause::Ipv6Unavailable => {
                                i32::from(Cause::Ipv6Unavailable)
                            }
                            talpid_tunnel::ErrorStateCause::SetFirewallPolicyError(_) => {
                                i32::from(Cause::SetFirewallPolicyError)
                            }
                            talpid_tunnel::ErrorStateCause::SetDnsError => {
                                i32::from(Cause::SetDnsError)
                            }
                            talpid_tunnel::ErrorStateCause::StartTunnelError => {
                                i32::from(Cause::StartTunnelError)
                            }
                            talpid_tunnel::ErrorStateCause::TunnelParameterError(_) => {
                                i32::from(Cause::TunnelParameterError)
                            }
                            talpid_tunnel::ErrorStateCause::IsOffline => {
                                i32::from(Cause::IsOffline)
                            }
                            #[cfg(target_os = "android")]
                            talpid_tunnel::ErrorStateCause::VpnPermissionDenied => {
                                i32::from(Cause::VpnPermissionDenied)
                            }
                        },
                        blocking_error: error_state.block_failure().map(map_firewall_error),
                        auth_fail_reason: if let talpid_tunnel::ErrorStateCause::AuthFailed(
                            reason,
                        ) = error_state.cause()
                        {
                            reason.clone().unwrap_or_default()
                        } else {
                            "".to_string()
                        },
                        parameter_error:
                            if let talpid_tunnel::ErrorStateCause::TunnelParameterError(reason) =
                                error_state.cause()
                            {
                                match reason {
                            talpid_tunnel::ParameterGenerationError::NoMatchingRelay => {
                                i32::from(GenerationError::NoMatchingRelay)
                            }
                            talpid_tunnel::ParameterGenerationError::NoMatchingBridgeRelay => {
                                i32::from(GenerationError::NoMatchingBridgeRelay)
                            }
                            talpid_tunnel::ParameterGenerationError::NoWireguardKey => {
                                i32::from(GenerationError::NoWireguardKey)
                            }
                            talpid_tunnel::ParameterGenerationError::CustomTunnelHostResultionError => {
                                i32::from(GenerationError::CustomTunnelHostResolutionError)
                            }
                        }
                            } else {
                                0
                            },
                        policy_error:
                            if let talpid_tunnel::ErrorStateCause::SetFirewallPolicyError(reason) =
                                error_state.cause()
                            {
                                Some(map_firewall_error(reason))
                            } else {
                                None
                            },
                    }),
                })
            }
        };

        TunnelState { state: Some(state) }
    }
}

impl From<mullvad_types::wireguard::KeygenEvent> for KeygenEvent {
    fn from(event: mullvad_types::wireguard::KeygenEvent) -> Self {
        use keygen_event::KeygenEvent as Event;
        use mullvad_types::wireguard::KeygenEvent as MullvadEvent;

        KeygenEvent {
            event: match event {
                MullvadEvent::NewKey(_) => i32::from(Event::NewKey),
                MullvadEvent::TooManyKeys => i32::from(Event::TooManyKeys),
                MullvadEvent::GenerationFailure => i32::from(Event::GenerationFailure),
            },
            new_key: if let MullvadEvent::NewKey(key) = event {
                Some(PublicKey::from(key))
            } else {
                None
            },
        }
    }
}

impl From<mullvad_types::wireguard::PublicKey> for PublicKey {
    fn from(public_key: mullvad_types::wireguard::PublicKey) -> Self {
        PublicKey {
            key: public_key.key.as_bytes().to_vec(),
            created: Some(Timestamp {
                seconds: public_key.created.timestamp(),
                nanos: 0,
            }),
        }
    }
}

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

impl
    From<
        mullvad_types::relay_constraints::Constraint<
            mullvad_types::relay_constraints::LocationConstraint,
        >,
    > for RelayLocation
{
    fn from(
        location: mullvad_types::relay_constraints::Constraint<
            mullvad_types::relay_constraints::LocationConstraint,
        >,
    ) -> Self {
        location
            .option()
            .map(RelayLocation::from)
            .unwrap_or_default()
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

impl From<&mullvad_types::settings::Settings> for Settings {
    fn from(settings: &mullvad_types::settings::Settings) -> Self {
        Self {
            account_token: settings.get_account_token().unwrap_or_default(),
            relay_settings: Some(RelaySettings::from(settings.get_relay_settings())),
            bridge_settings: Some(BridgeSettings::from(settings.bridge_settings.clone())),
            bridge_state: Some(BridgeState::from(settings.get_bridge_state())),
            allow_lan: settings.allow_lan,
            block_when_disconnected: settings.block_when_disconnected,
            auto_connect: settings.auto_connect,
            tunnel_options: Some(TunnelOptions::from(&settings.tunnel_options)),
            show_beta_releases: settings.show_beta_releases,
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
                        entry_location: constraints
                            .wireguard_constraints
                            .entry_location
                            .map(RelayLocation::from),
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

impl From<&mullvad_types::settings::DnsOptions> for DnsOptions {
    fn from(options: &mullvad_types::settings::DnsOptions) -> Self {
        DnsOptions {
            state: match options.state {
                mullvad_types::settings::DnsState::Default => dns_options::DnsState::Default as i32,
                mullvad_types::settings::DnsState::Custom => dns_options::DnsState::Custom as i32,
            },
            default_options: Some(DefaultDnsOptions {
                block_ads: options.default_options.block_ads,
                block_trackers: options.default_options.block_trackers,
            }),
            custom_options: Some(CustomDnsOptions {
                addresses: options
                    .custom_options
                    .addresses
                    .iter()
                    .map(|addr| addr.to_string())
                    .collect(),
            }),
        }
    }
}

impl From<&mullvad_types::settings::TunnelOptions> for TunnelOptions {
    fn from(options: &mullvad_types::settings::TunnelOptions) -> Self {
        Self {
            openvpn: Some(tunnel_options::OpenvpnOptions {
                mssfix: u32::from(options.openvpn.mssfix.unwrap_or_default()),
            }),
            wireguard: Some(tunnel_options::WireguardOptions {
                mtu: u32::from(options.wireguard.options.mtu.unwrap_or_default()),
                rotation_interval: options
                    .wireguard
                    .rotation_interval
                    .map(|ivl| Duration::from(std::time::Duration::from(ivl))),
            }),
            generic: Some(tunnel_options::GenericOptions {
                enable_ipv6: options.generic.enable_ipv6,
            }),
            #[cfg(not(target_os = "android"))]
            dns_options: Some(DnsOptions::from(&options.dns_options)),
            #[cfg(target_os = "android")]
            dns_options: None,
        }
    }
}

impl From<mullvad_types::relay_list::RelayListCountry> for RelayListCountry {
    fn from(country: mullvad_types::relay_list::RelayListCountry) -> Self {
        let mut proto_country = RelayListCountry {
            name: country.name,
            code: country.code,
            cities: Vec::with_capacity(country.cities.len()),
        };

        for city in country.cities.into_iter() {
            proto_country.cities.push(RelayListCity {
                name: city.name,
                code: city.code,
                latitude: city.latitude,
                longitude: city.longitude,
                relays: city.relays.into_iter().map(Relay::from).collect(),
            });
        }

        proto_country
    }
}

impl From<mullvad_types::relay_list::Relay> for Relay {
    fn from(relay: mullvad_types::relay_list::Relay) -> Self {
        Self {
            hostname: relay.hostname,
            ipv4_addr_in: relay.ipv4_addr_in.to_string(),
            ipv6_addr_in: relay
                .ipv6_addr_in
                .map(|addr| addr.to_string())
                .unwrap_or_default(),
            include_in_country: relay.include_in_country,
            active: relay.active,
            owned: relay.owned,
            provider: relay.provider,
            weight: relay.weight,
            tunnels: Some(RelayTunnels {
                openvpn: relay
                    .tunnels
                    .openvpn
                    .iter()
                    .map(|endpoint| OpenVpnEndpointData {
                        port: u32::from(endpoint.port),
                        protocol: i32::from(TransportProtocol::from(endpoint.protocol)),
                    })
                    .collect(),
                wireguard: relay
                    .tunnels
                    .wireguard
                    .iter()
                    .map(|endpoint| {
                        let port_ranges = endpoint
                            .port_ranges
                            .iter()
                            .map(|range| PortRange {
                                first: u32::from(range.0),
                                last: u32::from(range.1),
                            })
                            .collect();
                        WireguardEndpointData {
                            port_ranges,
                            ipv4_gateway: endpoint.ipv4_gateway.to_string(),
                            ipv6_gateway: endpoint.ipv6_gateway.to_string(),
                            public_key: endpoint.public_key.as_bytes().to_vec(),
                        }
                    })
                    .collect(),
            }),
            bridges: Some(RelayBridges {
                shadowsocks: relay
                    .bridges
                    .shadowsocks
                    .into_iter()
                    .map(|endpoint| ShadowsocksEndpointData {
                        port: u32::from(endpoint.port),
                        cipher: endpoint.cipher,
                        password: endpoint.password,
                        protocol: i32::from(TransportProtocol::from(endpoint.protocol)),
                    })
                    .collect(),
            }),
            location: relay.location.map(|location| Location {
                country: location.country,
                country_code: location.country_code,
                city: location.city,
                city_code: location.city_code,
                latitude: location.latitude,
                longitude: location.longitude,
            }),
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

#[derive(Debug)]
pub enum FromProtobufTypeError {
    InvalidArgument(&'static str),
}

impl TryFrom<RelaySettingsUpdate> for mullvad_types::relay_constraints::RelaySettingsUpdate {
    type Error = FromProtobufTypeError;

    fn try_from(
        settings: RelaySettingsUpdate,
    ) -> Result<mullvad_types::relay_constraints::RelaySettingsUpdate, Self::Error> {
        use mullvad_types::CustomTunnelEndpoint;
        use talpid_types::net::{self, openvpn, wireguard};

        use mullvad_types::relay_constraints as mullvad_constraints;

        let update_value =
            settings
                .r#type
                .clone()
                .ok_or(FromProtobufTypeError::InvalidArgument(
                    "missing relay settings",
                ))?;

        match update_value {
            relay_settings_update::Type::Custom(settings) => {
                let config = settings
                    .config
                    .ok_or(FromProtobufTypeError::InvalidArgument(
                        "missing relay settings",
                    ))?;
                let config = config.config.ok_or(FromProtobufTypeError::InvalidArgument(
                    "missing relay settings",
                ))?;
                let config = match config {
                    connection_config::Config::Openvpn(config) => {
                        let address = match config.address.parse() {
                            Ok(address) => address,
                            Err(_) => {
                                return Err(FromProtobufTypeError::InvalidArgument(
                                    "invalid address",
                                ))
                            }
                        };

                        mullvad_types::ConnectionConfig::OpenVpn(openvpn::ConnectionConfig {
                            endpoint: net::Endpoint {
                                address,
                                protocol: TransportProtocol::from_i32(config.protocol)
                                    .ok_or(FromProtobufTypeError::InvalidArgument(
                                        "invalid transport protocol",
                                    ))?
                                    .into(),
                            },
                            username: config.username,
                            password: config.password,
                        })
                    }
                    connection_config::Config::Wireguard(config) => {
                        let tunnel = config.tunnel.ok_or(
                            FromProtobufTypeError::InvalidArgument("missing tunnel config"),
                        )?;

                        // Copy the private key to an array
                        if tunnel.private_key.len() != 32 {
                            return Err(FromProtobufTypeError::InvalidArgument(
                                "invalid private key",
                            ));
                        }

                        let mut private_key = [0; 32];
                        let buffer = &tunnel.private_key[..private_key.len()];
                        private_key.copy_from_slice(buffer);

                        let peer = config.peer.ok_or(FromProtobufTypeError::InvalidArgument(
                            "missing peer config",
                        ))?;

                        // Copy the public key to an array
                        if peer.public_key.len() != 32 {
                            return Err(FromProtobufTypeError::InvalidArgument(
                                "invalid public key",
                            ));
                        }

                        let mut public_key = [0; 32];
                        let buffer = &peer.public_key[..public_key.len()];
                        public_key.copy_from_slice(buffer);

                        let ipv4_gateway = match config.ipv4_gateway.parse() {
                            Ok(address) => address,
                            Err(_) => {
                                return Err(FromProtobufTypeError::InvalidArgument(
                                    "invalid IPv4 gateway",
                                ))
                            }
                        };
                        let ipv6_gateway = if !config.ipv6_gateway.is_empty() {
                            let address = match config.ipv6_gateway.parse() {
                                Ok(address) => address,
                                Err(_) => {
                                    return Err(FromProtobufTypeError::InvalidArgument(
                                        "invalid IPv6 gateway",
                                    ))
                                }
                            };
                            Some(address)
                        } else {
                            None
                        };

                        let endpoint = match peer.endpoint.parse() {
                            Ok(address) => address,
                            Err(_) => {
                                return Err(FromProtobufTypeError::InvalidArgument(
                                    "invalid peer address",
                                ))
                            }
                        };

                        let mut tunnel_addresses = Vec::new();
                        for address in tunnel.addresses {
                            let address = address.parse().map_err(|_| {
                                FromProtobufTypeError::InvalidArgument("invalid address")
                            })?;
                            tunnel_addresses.push(address);
                        }

                        let mut allowed_ips = Vec::new();
                        for address in peer.allowed_ips {
                            let address = address.parse().map_err(|_| {
                                FromProtobufTypeError::InvalidArgument("invalid address")
                            })?;
                            allowed_ips.push(address);
                        }

                        mullvad_types::ConnectionConfig::Wireguard(wireguard::ConnectionConfig {
                            tunnel: wireguard::TunnelConfig {
                                private_key: wireguard::PrivateKey::from(private_key),
                                addresses: tunnel_addresses,
                            },
                            peer: wireguard::PeerConfig {
                                public_key: wireguard::PublicKey::from(public_key),
                                allowed_ips,
                                endpoint,
                                protocol: TransportProtocol::from_i32(peer.protocol)
                                    .ok_or(FromProtobufTypeError::InvalidArgument(
                                        "invalid transport protocol",
                                    ))?
                                    .into(),
                            },
                            exit_peer: None,
                            ipv4_gateway,
                            ipv6_gateway,
                        })
                    }
                };

                Ok(
                    mullvad_constraints::RelaySettingsUpdate::CustomTunnelEndpoint(
                        CustomTunnelEndpoint {
                            host: settings.host,
                            config,
                        },
                    ),
                )
            }

            relay_settings_update::Type::Normal(settings) => {
                // If `location` isn't provided, no changes are made.
                // If `location` is provided, but is an empty vector,
                // then the constraint is set to `Constraint::Any`.
                let location = settings
                    .location
                    .map(Constraint::<mullvad_types::relay_constraints::LocationConstraint>::from);

                let tunnel_protocol = if let Some(update) = settings.tunnel_type {
                    match update.tunnel_type {
                        Some(constraint) => match TunnelType::from_i32(constraint.tunnel_type) {
                            Some(TunnelType::Openvpn) => {
                                Some(Constraint::Only(net::TunnelType::OpenVpn))
                            }
                            Some(TunnelType::Wireguard) => {
                                Some(Constraint::Only(net::TunnelType::Wireguard))
                            }
                            None => {
                                return Err(FromProtobufTypeError::InvalidArgument(
                                    "invalid tunnel protocol",
                                ))
                            }
                        },
                        None => Some(Constraint::Any),
                    }
                } else {
                    None
                };

                let transport_protocol = if let Some(ref constraints) = settings.openvpn_constraints
                {
                    match &constraints.protocol {
                        Some(constraint) => Some(
                            TransportProtocol::from_i32(constraint.protocol)
                                .ok_or(FromProtobufTypeError::InvalidArgument(
                                    "invalid transport protocol",
                                ))?
                                .into(),
                        ),
                        None => None,
                    }
                } else {
                    None
                };

                let providers = if let Some(ref provider_update) = settings.providers {
                    if !provider_update.providers.is_empty() {
                        Some(Constraint::Only(
                            mullvad_constraints::Providers::new(
                                provider_update.providers.clone().into_iter(),
                            )
                            .map_err(|_| {
                                FromProtobufTypeError::InvalidArgument(
                                    "must specify at least one provider",
                                )
                            })?,
                        ))
                    } else {
                        Some(Constraint::Any)
                    }
                } else {
                    None
                };
                let ip_version = if let Some(ref constraints) = settings.wireguard_constraints {
                    match &constraints.ip_version {
                        Some(constraint) => match IpVersion::from_i32(constraint.protocol) {
                            Some(IpVersion::V4) => Some(net::IpVersion::V4),
                            Some(IpVersion::V6) => Some(net::IpVersion::V6),
                            None => {
                                return Err(FromProtobufTypeError::InvalidArgument(
                                    "invalid ip protocol version",
                                ))
                            }
                        },
                        None => None,
                    }
                } else {
                    None
                };

                Ok(mullvad_constraints::RelaySettingsUpdate::Normal(
                    mullvad_constraints::RelayConstraintsUpdate {
                        location,
                        providers,
                        tunnel_protocol,
                        wireguard_constraints: settings.wireguard_constraints.map(|constraints| {
                            mullvad_constraints::WireguardConstraints {
                                port: if constraints.port != 0 {
                                    Constraint::Only(constraints.port as u16)
                                } else {
                                    Constraint::Any
                                },
                                ip_version: Constraint::from(ip_version),
                                entry_location: constraints.entry_location.map(
                                    Constraint::<
                                        mullvad_types::relay_constraints::LocationConstraint,
                                    >::from,
                                ),
                            }
                        }),
                        openvpn_constraints: settings.openvpn_constraints.map(|constraints| {
                            mullvad_constraints::OpenVpnConstraints {
                                port: if constraints.port != 0 {
                                    Constraint::Only(constraints.port as u16)
                                } else {
                                    Constraint::Any
                                },
                                protocol: Constraint::from(transport_protocol),
                            }
                        }),
                    },
                ))
            }
        }
    }
}

impl From<RelayLocation> for Constraint<mullvad_types::relay_constraints::LocationConstraint> {
    fn from(location: RelayLocation) -> Self {
        use mullvad_types::relay_constraints::LocationConstraint;

        if !location.hostname.is_empty() {
            Constraint::Only(LocationConstraint::Hostname(
                location.country,
                location.city,
                location.hostname,
            ))
        } else if !location.city.is_empty() {
            Constraint::Only(LocationConstraint::City(location.country, location.city))
        } else if !location.country.is_empty() {
            Constraint::Only(LocationConstraint::Country(location.country))
        } else {
            Constraint::Any
        }
    }
}

impl TryFrom<BridgeSettings> for mullvad_types::relay_constraints::BridgeSettings {
    type Error = FromProtobufTypeError;

    fn try_from(settings: BridgeSettings) -> Result<Self, Self::Error> {
        use mullvad_types::relay_constraints as mullvad_constraints;
        use talpid_types::net as talpid_net;

        match settings
            .r#type
            .ok_or(FromProtobufTypeError::InvalidArgument(
                "no settings provided",
            ))? {
            bridge_settings::Type::Normal(constraints) => {
                let location = match constraints.location {
                    None => Constraint::Any,
                    Some(location) => {
                        Constraint::<mullvad_constraints::LocationConstraint>::from(location)
                    }
                };
                let providers = if constraints.providers.is_empty() {
                    Constraint::Any
                } else {
                    Constraint::Only(
                        mullvad_constraints::Providers::new(
                            constraints.providers.clone().into_iter(),
                        )
                        .map_err(|_| {
                            FromProtobufTypeError::InvalidArgument(
                                "must specify at least one provider",
                            )
                        })?,
                    )
                };

                Ok(mullvad_constraints::BridgeSettings::Normal(
                    mullvad_constraints::BridgeConstraints {
                        location,
                        providers,
                    },
                ))
            }
            bridge_settings::Type::Local(proxy_settings) => {
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
            bridge_settings::Type::Remote(proxy_settings) => {
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
            bridge_settings::Type::Shadowsocks(proxy_settings) => {
                let peer = proxy_settings.peer.parse().map_err(|_| {
                    FromProtobufTypeError::InvalidArgument("failed to parse peer address")
                })?;
                let proxy_settings = talpid_net::openvpn::ProxySettings::Shadowsocks(
                    talpid_net::openvpn::ShadowsocksProxySettings {
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

impl TryFrom<BridgeState> for mullvad_types::relay_constraints::BridgeState {
    type Error = FromProtobufTypeError;

    fn try_from(state: BridgeState) -> Result<Self, Self::Error> {
        match bridge_state::State::from_i32(state.state) {
            Some(bridge_state::State::Auto) => {
                Ok(mullvad_types::relay_constraints::BridgeState::Auto)
            }
            Some(bridge_state::State::On) => Ok(mullvad_types::relay_constraints::BridgeState::On),
            Some(bridge_state::State::Off) => {
                Ok(mullvad_types::relay_constraints::BridgeState::Off)
            }
            None => Err(FromProtobufTypeError::InvalidArgument(
                "invalid bridge state",
            )),
        }
    }
}

impl TryFrom<DnsOptions> for mullvad_types::settings::DnsOptions {
    type Error = FromProtobufTypeError;

    fn try_from(options: DnsOptions) -> Result<Self, Self::Error> {
        use mullvad_types::settings::{
            CustomDnsOptions as MullvadCustomDnsOptions,
            DefaultDnsOptions as MullvadDefaultDnsOptions, DnsOptions as MullvadDnsOptions,
            DnsState as MullvadDnsState,
        };

        let state = match dns_options::DnsState::from_i32(options.state) {
            Some(dns_options::DnsState::Default) => MullvadDnsState::Default,
            Some(dns_options::DnsState::Custom) => MullvadDnsState::Custom,
            None => {
                return Err(FromProtobufTypeError::InvalidArgument(
                    "invalid DNS options state",
                ))
            }
        };

        let default_options =
            options
                .default_options
                .ok_or(FromProtobufTypeError::InvalidArgument(
                    "missing default DNS options",
                ))?;
        let custom_options =
            options
                .custom_options
                .ok_or(FromProtobufTypeError::InvalidArgument(
                    "missing default DNS options",
                ))?;

        Ok(MullvadDnsOptions {
            state,
            default_options: MullvadDefaultDnsOptions {
                block_ads: default_options.block_ads,
                block_trackers: default_options.block_trackers,
            },
            custom_options: MullvadCustomDnsOptions {
                addresses: custom_options
                    .addresses
                    .into_iter()
                    .map(|addr| {
                        addr.parse().map_err(|_| {
                            FromProtobufTypeError::InvalidArgument("invalid IP address")
                        })
                    })
                    .collect::<Result<Vec<_>, _>>()?,
            },
        })
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

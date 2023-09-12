use crate::types::{proto, FromProtobufTypeError};
use mullvad_types::settings::CURRENT_SETTINGS_VERSION;
use talpid_types::ErrorExt;

impl From<&mullvad_types::settings::Settings> for proto::Settings {
    fn from(settings: &mullvad_types::settings::Settings) -> Self {
        #[cfg(windows)]
        let split_tunnel = {
            let mut converted_list = vec![];
            for path in settings.split_tunnel.apps.clone().iter() {
                match path.as_path().as_os_str().to_str() {
                    Some(path) => converted_list.push(path.to_string()),
                    None => {
                        log::error!("failed to convert OS string: {:?}", path);
                    }
                }
            }

            Some(proto::SplitTunnelSettings {
                enable_exclusions: settings.split_tunnel.enable_exclusions,
                apps: converted_list,
            })
        };
        #[cfg(not(windows))]
        let split_tunnel = None;

        Self {
            relay_settings: Some(proto::RelaySettings::from(settings.get_relay_settings())),
            bridge_settings: Some(proto::BridgeSettings::from(
                settings.bridge_settings.clone(),
            )),
            bridge_state: Some(proto::BridgeState::from(settings.bridge_state)),
            allow_lan: settings.allow_lan,
            block_when_disconnected: settings.block_when_disconnected,
            auto_connect: settings.auto_connect,
            tunnel_options: Some(proto::TunnelOptions::from(&settings.tunnel_options)),
            show_beta_releases: settings.show_beta_releases,
            obfuscation_settings: Some(proto::ObfuscationSettings::from(
                &settings.obfuscation_settings,
            )),
            split_tunnel,
            custom_lists: Some(proto::CustomListSettings::from(&settings.custom_lists)),
            api_access_methods: Some(proto::ApiAccessMethodSettings::from(
                &settings.api_access_methods,
            )),
        }
    }
}

impl From<&mullvad_types::settings::DnsOptions> for proto::DnsOptions {
    fn from(options: &mullvad_types::settings::DnsOptions) -> Self {
        use proto::dns_options;

        proto::DnsOptions {
            state: match options.state {
                mullvad_types::settings::DnsState::Default => dns_options::DnsState::Default as i32,
                mullvad_types::settings::DnsState::Custom => dns_options::DnsState::Custom as i32,
            },
            default_options: Some(proto::DefaultDnsOptions {
                block_ads: options.default_options.block_ads,
                block_trackers: options.default_options.block_trackers,
                block_malware: options.default_options.block_malware,
                block_adult_content: options.default_options.block_adult_content,
                block_gambling: options.default_options.block_gambling,
                block_social_media: options.default_options.block_social_media,
            }),
            custom_options: Some(proto::CustomDnsOptions {
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

impl From<&mullvad_types::settings::TunnelOptions> for proto::TunnelOptions {
    fn from(options: &mullvad_types::settings::TunnelOptions) -> Self {
        Self {
            openvpn: Some(proto::tunnel_options::OpenvpnOptions {
                mssfix: u32::from(options.openvpn.mssfix.unwrap_or_default()),
            }),
            wireguard: Some(proto::tunnel_options::WireguardOptions {
                mtu: u32::from(options.wireguard.mtu.unwrap_or_default()),
                rotation_interval: options.wireguard.rotation_interval.map(|ivl| {
                    prost_types::Duration::try_from(std::time::Duration::from(ivl))
                        .expect("Failed to convert std::time::Duration to prost_types::Duration for tunnel_options.wireguard.rotation_interval")
                }),
                quantum_resistant: Some(proto::QuantumResistantState::from(options.wireguard.quantum_resistant)),
            }),
            generic: Some(proto::tunnel_options::GenericOptions {
                enable_ipv6: options.generic.enable_ipv6,
            }),
            #[cfg(not(target_os = "android"))]
            dns_options: Some(proto::DnsOptions::from(&options.dns_options)),
            #[cfg(target_os = "android")]
            dns_options: None,
        }
    }
}

impl TryFrom<proto::Settings> for mullvad_types::settings::Settings {
    type Error = FromProtobufTypeError;

    fn try_from(settings: proto::Settings) -> Result<Self, Self::Error> {
        let relay_settings =
            settings
                .relay_settings
                .ok_or(FromProtobufTypeError::InvalidArgument(
                    "missing relay settings",
                ))?;
        let bridge_settings =
            settings
                .bridge_settings
                .ok_or(FromProtobufTypeError::InvalidArgument(
                    "missing bridge settings",
                ))?;
        let bridge_state = settings
            .bridge_state
            .ok_or(FromProtobufTypeError::InvalidArgument(
                "missing bridge state",
            ))
            .and_then(|state| try_bridge_state_from_i32(state.state))?;
        let tunnel_options =
            settings
                .tunnel_options
                .ok_or(FromProtobufTypeError::InvalidArgument(
                    "missing tunnel options",
                ))?;
        let obfuscation_settings =
            settings
                .obfuscation_settings
                .ok_or(FromProtobufTypeError::InvalidArgument(
                    "missing obfuscation settings",
                ))?;
        let custom_lists_settings =
            settings
                .custom_lists
                .ok_or(FromProtobufTypeError::InvalidArgument(
                    "missing custom lists settings",
                ))?;
        let api_access_methods_settings =
            settings
                .api_access_methods
                .ok_or(FromProtobufTypeError::InvalidArgument(
                    "missing api access methods settings",
                ))?;
        #[cfg(windows)]
        let split_tunnel = settings
            .split_tunnel
            .ok_or(FromProtobufTypeError::InvalidArgument(
                "missing split tunnel options",
            ))?;

        Ok(Self {
            relay_settings: mullvad_types::relay_constraints::RelaySettings::try_from(
                relay_settings,
            )?,
            bridge_settings: mullvad_types::relay_constraints::BridgeSettings::try_from(
                bridge_settings,
            )?,
            bridge_state,
            allow_lan: settings.allow_lan,
            block_when_disconnected: settings.block_when_disconnected,
            auto_connect: settings.auto_connect,
            tunnel_options: mullvad_types::settings::TunnelOptions::try_from(tunnel_options)?,
            show_beta_releases: settings.show_beta_releases,
            #[cfg(windows)]
            split_tunnel: mullvad_types::settings::SplitTunnelSettings::from(split_tunnel),
            obfuscation_settings: mullvad_types::relay_constraints::ObfuscationSettings::try_from(
                obfuscation_settings,
            )?,
            // NOTE: This field is set based on mullvad-types. It's not based on the actual settings
            // version.
            settings_version: CURRENT_SETTINGS_VERSION,
            custom_lists: mullvad_types::custom_list::CustomListsSettings::try_from(
                custom_lists_settings,
            )?,
            api_access_methods: mullvad_types::api_access_method::Settings::try_from(
                api_access_methods_settings,
            )?,
        })
    }
}

pub fn try_bridge_state_from_i32(
    bridge_state: i32,
) -> Result<mullvad_types::relay_constraints::BridgeState, FromProtobufTypeError> {
    match proto::bridge_state::State::try_from(bridge_state) {
        Ok(proto::bridge_state::State::Auto) => {
            Ok(mullvad_types::relay_constraints::BridgeState::Auto)
        }
        Ok(proto::bridge_state::State::On) => Ok(mullvad_types::relay_constraints::BridgeState::On),
        Ok(proto::bridge_state::State::Off) => {
            Ok(mullvad_types::relay_constraints::BridgeState::Off)
        }
        Err(_) => Err(FromProtobufTypeError::InvalidArgument(
            "invalid bridge state",
        )),
    }
}

#[cfg(windows)]
impl From<proto::SplitTunnelSettings> for mullvad_types::settings::SplitTunnelSettings {
    fn from(value: proto::SplitTunnelSettings) -> Self {
        mullvad_types::settings::SplitTunnelSettings {
            enable_exclusions: value.enable_exclusions,
            apps: value
                .apps
                .into_iter()
                .map(std::path::PathBuf::from)
                .collect(),
        }
    }
}

impl TryFrom<proto::TunnelOptions> for mullvad_types::settings::TunnelOptions {
    type Error = FromProtobufTypeError;

    fn try_from(options: proto::TunnelOptions) -> Result<Self, Self::Error> {
        use talpid_types::net;

        let openvpn_options = options
            .openvpn
            .ok_or(FromProtobufTypeError::InvalidArgument(
                "missing openvpn tunnel options",
            ))?;
        let wireguard_options = options
            .wireguard
            .ok_or(FromProtobufTypeError::InvalidArgument(
                "missing wireguard tunnel options",
            ))?;
        let generic_options = options
            .generic
            .ok_or(FromProtobufTypeError::InvalidArgument(
                "missing generic tunnel options",
            ))?;
        let dns_options = options
            .dns_options
            .ok_or(FromProtobufTypeError::InvalidArgument(
                "missing tunnel DNS options",
            ))?;

        Ok(Self {
            openvpn: net::openvpn::TunnelOptions {
                mssfix: if openvpn_options.mssfix != 0 {
                    Some(openvpn_options.mssfix as u16)
                } else {
                    None
                },
            },
            wireguard: mullvad_types::wireguard::TunnelOptions {
                mtu: if wireguard_options.mtu != 0 {
                    Some(wireguard_options.mtu as u16)
                } else {
                    None
                },
                rotation_interval: wireguard_options
                    .rotation_interval
                    .map(std::time::Duration::try_from)
                    .transpose()
                    .map_err(|_| FromProtobufTypeError::InvalidArgument("invalid duration"))?
                    .map(mullvad_types::wireguard::RotationInterval::try_from)
                    .transpose()
                    .map_err(|error: mullvad_types::wireguard::RotationIntervalError| {
                        log::error!(
                            "{}",
                            error.display_chain_with_msg("Invalid rotation interval")
                        );
                        FromProtobufTypeError::InvalidArgument("invalid rotation interval")
                    })?,
                quantum_resistant: wireguard_options
                    .quantum_resistant
                    .map(mullvad_types::wireguard::QuantumResistantState::try_from)
                    .ok_or(FromProtobufTypeError::InvalidArgument(
                        "missing quantum resistant state",
                    ))??,
            },
            generic: net::GenericTunnelOptions {
                enable_ipv6: generic_options.enable_ipv6,
            },
            #[cfg(not(target_os = "android"))]
            dns_options: mullvad_types::settings::DnsOptions::try_from(dns_options)?,
        })
    }
}

impl TryFrom<proto::DnsOptions> for mullvad_types::settings::DnsOptions {
    type Error = FromProtobufTypeError;

    fn try_from(options: proto::DnsOptions) -> Result<Self, Self::Error> {
        use mullvad_types::settings::{
            CustomDnsOptions as MullvadCustomDnsOptions,
            DefaultDnsOptions as MullvadDefaultDnsOptions, DnsOptions as MullvadDnsOptions,
            DnsState as MullvadDnsState,
        };

        let state = match proto::dns_options::DnsState::try_from(options.state) {
            Ok(proto::dns_options::DnsState::Default) => MullvadDnsState::Default,
            Ok(proto::dns_options::DnsState::Custom) => MullvadDnsState::Custom,
            Err(_) => {
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
                block_malware: default_options.block_malware,
                block_adult_content: default_options.block_adult_content,
                block_gambling: default_options.block_gambling,
                block_social_media: default_options.block_social_media,
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

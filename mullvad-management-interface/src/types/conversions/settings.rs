use crate::types::{proto, FromProtobufTypeError};
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
            bridge_state: Some(proto::BridgeState::from(settings.get_bridge_state())),
            allow_lan: settings.allow_lan,
            block_when_disconnected: settings.block_when_disconnected,
            auto_connect: settings.auto_connect,
            tunnel_options: Some(proto::TunnelOptions::from(&settings.tunnel_options)),
            show_beta_releases: settings.show_beta_releases,
            obfuscation_settings: Some(proto::ObfuscationSettings::from(
                &settings.obfuscation_settings,
            )),
            split_tunnel,
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
                #[cfg(windows)]
                use_wireguard_nt: options.wireguard.use_wireguard_nt,
                #[cfg(not(windows))]
                use_wireguard_nt: false,
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
                #[cfg(windows)]
                use_wireguard_nt: wireguard_options.use_wireguard_nt,
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

        let state = match proto::dns_options::DnsState::from_i32(options.state) {
            Some(proto::dns_options::DnsState::Default) => MullvadDnsState::Default,
            Some(proto::dns_options::DnsState::Custom) => MullvadDnsState::Custom,
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
                block_malware: default_options.block_malware,
                block_adult_content: default_options.block_adult_content,
                block_gambling: default_options.block_gambling,
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

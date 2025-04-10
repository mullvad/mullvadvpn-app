use crate::types::{proto, FromProtobufTypeError};
use talpid_types::net::IpVersion;

impl From<mullvad_types::states::TunnelState> for proto::TunnelState {
    fn from(state: mullvad_types::states::TunnelState) -> Self {
        use mullvad_types::states::TunnelState as MullvadTunnelState;
        use proto::error_state::{
            firewall_policy_error::ErrorType as PolicyErrorType, Cause, FirewallPolicyError,
            GenerationError,
        };

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
                        Some(app) => (app.pid, Some(app.name.clone())),
                        None => (0, None),
                    };

                    FirewallPolicyError {
                        r#type: i32::from(PolicyErrorType::Locked),
                        lock_pid,
                        lock_name,
                    }
                }
            };

        let state = match state {
            MullvadTunnelState::Disconnected {
                location: disconnected_location,
                #[cfg(not(target_os = "android"))]
                locked_down,
            } => proto::tunnel_state::State::Disconnected(proto::tunnel_state::Disconnected {
                disconnected_location: disconnected_location.map(proto::GeoIpLocation::from),
                #[cfg(not(target_os = "android"))]
                locked_down,
                #[cfg(target_os = "android")]
                locked_down: false,
            }),
            MullvadTunnelState::Connecting {
                endpoint,
                location,
                feature_indicators,
            } => proto::tunnel_state::State::Connecting(proto::tunnel_state::Connecting {
                relay_info: Some(proto::TunnelStateRelayInfo {
                    tunnel_endpoint: Some(proto::TunnelEndpoint::from(endpoint)),
                    location: location.map(proto::GeoIpLocation::from),
                }),
                feature_indicators: Some(proto::FeatureIndicators::from(feature_indicators)),
            }),
            MullvadTunnelState::Connected {
                endpoint,
                location,
                feature_indicators,
            } => proto::tunnel_state::State::Connected(proto::tunnel_state::Connected {
                relay_info: Some(proto::TunnelStateRelayInfo {
                    tunnel_endpoint: Some(proto::TunnelEndpoint::from(endpoint)),
                    location: location.map(proto::GeoIpLocation::from),
                }),
                feature_indicators: Some(proto::FeatureIndicators::from(feature_indicators)),
            }),
            MullvadTunnelState::Disconnecting(after_disconnect) => {
                proto::tunnel_state::State::Disconnecting(proto::tunnel_state::Disconnecting {
                    after_disconnect: match after_disconnect {
                        talpid_tunnel::ActionAfterDisconnect::Nothing => {
                            i32::from(proto::AfterDisconnect::Nothing)
                        }
                        talpid_tunnel::ActionAfterDisconnect::Block => {
                            i32::from(proto::AfterDisconnect::Block)
                        }
                        talpid_tunnel::ActionAfterDisconnect::Reconnect => {
                            i32::from(proto::AfterDisconnect::Reconnect)
                        }
                    },
                })
            }
            MullvadTunnelState::Error(error_state) => {
                proto::tunnel_state::State::Error(proto::tunnel_state::Error {
                    error_state: Some(proto::ErrorState {
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
                            #[cfg(target_os = "windows")]
                            talpid_tunnel::ErrorStateCause::CreateTunnelDevice { os_error: _ } => {
                                i32::from(Cause::CreateTunnelDevice)
                            }
                            talpid_tunnel::ErrorStateCause::TunnelParameterError(_) => {
                                i32::from(Cause::TunnelParameterError)
                            }
                            talpid_tunnel::ErrorStateCause::IsOffline => {
                                i32::from(Cause::IsOffline)
                            }
                            #[cfg(target_os = "android")]
                            talpid_tunnel::ErrorStateCause::NotPrepared => {
                                i32::from(Cause::NotPrepared)
                            }
                            #[cfg(target_os = "android")]
                            talpid_tunnel::ErrorStateCause::OtherAlwaysOnApp { .. } => {
                                i32::from(Cause::OtherAlwaysOnApp)
                            }
                            #[cfg(target_os = "android")]
                            talpid_tunnel::ErrorStateCause::OtherLegacyAlwaysOnVpn => {
                                i32::from(Cause::OtherLegacyAlwaysOnVpn)
                            }
                            #[cfg(target_os = "android")]
                            talpid_tunnel::ErrorStateCause::InvalidDnsServers(_) => {
                                i32::from(Cause::InvalidDnsServers)
                            }
                            #[cfg(any(
                                target_os = "windows",
                                target_os = "macos",
                                target_os = "android"
                            ))]
                            talpid_tunnel::ErrorStateCause::SplitTunnelError => {
                                i32::from(Cause::SplitTunnelError)
                            }
                            #[cfg(target_os = "macos")]
                            talpid_tunnel::ErrorStateCause::NeedFullDiskPermissions => {
                                i32::from(Cause::NeedFullDiskPermissions)
                            }
                        },
                        blocking_error: error_state.block_failure().map(map_firewall_error),
                        #[cfg(not(target_os = "android"))]
                        other_always_on_app_error: None,
                        #[cfg(target_os = "android")]
                        other_always_on_app_error:
                            if let talpid_tunnel::ErrorStateCause::OtherAlwaysOnApp { app_name } =
                                error_state.cause()
                            {
                                Some(proto::error_state::OtherAlwaysOnAppError {
                                    app_name: app_name.to_string(),
                                })
                            } else {
                                None
                            },
                        #[cfg(not(target_os = "android"))]
                        invalid_dns_servers_error: None,
                        #[cfg(target_os = "android")]
                        invalid_dns_servers_error:
                            if let talpid_tunnel::ErrorStateCause::InvalidDnsServers(ip_addrs) =
                                error_state.cause()
                            {
                                Some(proto::error_state::InvalidDnsServersError {
                                    ip_addrs: ip_addrs.iter().map(|ip| ip.to_string()).collect(),
                                })
                            } else {
                                None
                            },
                        auth_failed_error: mullvad_types::auth_failed::AuthFailed::try_from(
                            error_state.cause(),
                        )
                        .ok()
                        .map(|auth_failed| {
                            i32::from(proto::error_state::AuthFailedError::from(auth_failed))
                        })
                        .unwrap_or(0i32),
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
                                talpid_tunnel::ParameterGenerationError::CustomTunnelHostResolutionError => {
                                    i32::from(GenerationError::CustomTunnelHostResolutionError)
                                }
                                talpid_tunnel::ParameterGenerationError::IpVersionUnavailable { family: IpVersion::V4 } => {
                                    i32::from(GenerationError::NetworkIpv4Unavailable)
                                }
                                talpid_tunnel::ParameterGenerationError::IpVersionUnavailable { family: IpVersion::V6 } => {
                                    i32::from(GenerationError::NetworkIpv6Unavailable)
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
                        #[cfg(not(target_os = "windows"))]
                        create_tunnel_error: None,
                        #[cfg(target_os = "windows")]
                        create_tunnel_error: match error_state.cause() {
                            talpid_tunnel::ErrorStateCause::CreateTunnelDevice { os_error } => {
                                *os_error
                            }
                            _ => None,
                        },
                    }),
                })
            }
        };

        proto::TunnelState { state: Some(state) }
    }
}

impl From<mullvad_types::auth_failed::AuthFailed> for proto::error_state::AuthFailedError {
    fn from(auth_failed: mullvad_types::auth_failed::AuthFailed) -> Self {
        use mullvad_types::auth_failed::AuthFailed;
        use proto::error_state;
        match auth_failed {
            AuthFailed::InvalidAccount => error_state::AuthFailedError::InvalidAccount,
            AuthFailed::ExpiredAccount => error_state::AuthFailedError::ExpiredAccount,
            AuthFailed::TooManyConnections => error_state::AuthFailedError::TooManyConnections,
            AuthFailed::Unknown => error_state::AuthFailedError::Unknown,
        }
    }
}

fn try_auth_failed_from_i32(
    auth_failed_error: i32,
) -> Result<mullvad_types::auth_failed::AuthFailed, FromProtobufTypeError> {
    proto::error_state::AuthFailedError::try_from(auth_failed_error)
        .map(mullvad_types::auth_failed::AuthFailed::from)
        .map_err(|_| FromProtobufTypeError::InvalidArgument("invalid auth failed error"))
}

impl From<proto::error_state::AuthFailedError> for mullvad_types::auth_failed::AuthFailed {
    fn from(auth_failed: proto::error_state::AuthFailedError) -> Self {
        use mullvad_types::auth_failed::AuthFailed;
        use proto::error_state;
        match auth_failed {
            error_state::AuthFailedError::InvalidAccount => AuthFailed::InvalidAccount,
            error_state::AuthFailedError::ExpiredAccount => AuthFailed::ExpiredAccount,
            error_state::AuthFailedError::TooManyConnections => AuthFailed::TooManyConnections,
            error_state::AuthFailedError::Unknown => AuthFailed::Unknown,
        }
    }
}

impl TryFrom<proto::TunnelState> for mullvad_types::states::TunnelState {
    type Error = FromProtobufTypeError;

    fn try_from(state: proto::TunnelState) -> Result<Self, FromProtobufTypeError> {
        use mullvad_types::states::TunnelState as MullvadState;
        use talpid_types::{net as talpid_net, tunnel as talpid_tunnel};

        let state = match state.state {
            #[cfg_attr(target_os = "android", allow(unused_variables))]
            Some(proto::tunnel_state::State::Disconnected(proto::tunnel_state::Disconnected {
                disconnected_location,
                locked_down,
            })) => MullvadState::Disconnected {
                location: disconnected_location
                    .map(mullvad_types::location::GeoIpLocation::try_from)
                    .transpose()?,
                #[cfg(not(target_os = "android"))]
                locked_down,
            },
            Some(proto::tunnel_state::State::Connecting(proto::tunnel_state::Connecting {
                relay_info:
                    Some(proto::TunnelStateRelayInfo {
                        tunnel_endpoint: Some(tunnel_endpoint),
                        location,
                    }),
                feature_indicators,
            })) => MullvadState::Connecting {
                endpoint: talpid_net::TunnelEndpoint::try_from(tunnel_endpoint)?,
                location: location
                    .map(mullvad_types::location::GeoIpLocation::try_from)
                    .transpose()?,
                feature_indicators: feature_indicators
                    .map(mullvad_types::features::FeatureIndicators::from)
                    .ok_or(FromProtobufTypeError::InvalidArgument(
                        "Missing feature indicators",
                    ))?,
            },
            Some(proto::tunnel_state::State::Connected(proto::tunnel_state::Connected {
                relay_info:
                    Some(proto::TunnelStateRelayInfo {
                        tunnel_endpoint: Some(tunnel_endpoint),
                        location,
                    }),
                feature_indicators,
            })) => MullvadState::Connected {
                endpoint: talpid_net::TunnelEndpoint::try_from(tunnel_endpoint)?,
                location: location
                    .map(mullvad_types::location::GeoIpLocation::try_from)
                    .transpose()?,
                feature_indicators: feature_indicators
                    .map(mullvad_types::features::FeatureIndicators::from)
                    .ok_or(FromProtobufTypeError::InvalidArgument(
                        "Missing feature indicators",
                    ))?,
            },
            Some(proto::tunnel_state::State::Disconnecting(
                proto::tunnel_state::Disconnecting { after_disconnect },
            )) => MullvadState::Disconnecting(
                match proto::AfterDisconnect::try_from(after_disconnect) {
                    Ok(proto::AfterDisconnect::Nothing) => {
                        talpid_tunnel::ActionAfterDisconnect::Nothing
                    }
                    Ok(proto::AfterDisconnect::Block) => {
                        talpid_tunnel::ActionAfterDisconnect::Block
                    }
                    Ok(proto::AfterDisconnect::Reconnect) => {
                        talpid_tunnel::ActionAfterDisconnect::Reconnect
                    }
                    _ => {
                        return Err(FromProtobufTypeError::InvalidArgument(
                            "invalid \"after_disconnect\" action",
                        ))
                    }
                },
            ),
            Some(proto::tunnel_state::State::Error(proto::tunnel_state::Error {
                error_state:
                    Some(proto::ErrorState {
                        cause,
                        blocking_error,
                        auth_failed_error,
                        parameter_error,
                        policy_error,
                        create_tunnel_error,
                        ..
                    }),
            })) => {
                #[cfg(not(target_os = "windows"))]
                let _ = create_tunnel_error;

                let cause = match proto::error_state::Cause::try_from(cause) {
                    Ok(proto::error_state::Cause::AuthFailed) => {
                        let auth_failed = try_auth_failed_from_i32(auth_failed_error)?;
                        talpid_tunnel::ErrorStateCause::AuthFailed(Some(
                            auth_failed.as_str().to_string(),
                        ))
                    }
                    Ok(proto::error_state::Cause::Ipv6Unavailable) => {
                        talpid_tunnel::ErrorStateCause::Ipv6Unavailable
                    }
                    Ok(proto::error_state::Cause::IsOffline) => {
                        talpid_tunnel::ErrorStateCause::IsOffline
                    }
                    Ok(proto::error_state::Cause::SetDnsError) => {
                        talpid_tunnel::ErrorStateCause::SetDnsError
                    }
                    Ok(proto::error_state::Cause::SetFirewallPolicyError) => {
                        let policy_error = policy_error.ok_or(
                            FromProtobufTypeError::InvalidArgument("missing firewall policy error"),
                        )?;
                        let policy_error = try_firewall_policy_error_from_i32(
                            policy_error.r#type,
                            policy_error.lock_pid,
                            policy_error.lock_name,
                        )?;
                        talpid_tunnel::ErrorStateCause::SetFirewallPolicyError(policy_error)
                    }
                    Ok(proto::error_state::Cause::StartTunnelError) => {
                        talpid_tunnel::ErrorStateCause::StartTunnelError
                    }
                    #[cfg(target_os = "windows")]
                    Ok(proto::error_state::Cause::CreateTunnelDevice) => {
                        talpid_tunnel::ErrorStateCause::CreateTunnelDevice {
                            os_error: create_tunnel_error,
                        }
                    }
                    Ok(proto::error_state::Cause::TunnelParameterError) => {
                        let parameter_error = match proto::error_state::GenerationError::try_from(parameter_error) {
                            Ok(proto::error_state::GenerationError::CustomTunnelHostResolutionError) => talpid_tunnel::ParameterGenerationError::CustomTunnelHostResolutionError,
                            Ok(proto::error_state::GenerationError::NoMatchingBridgeRelay) => talpid_tunnel::ParameterGenerationError::NoMatchingBridgeRelay,
                            Ok(proto::error_state::GenerationError::NoMatchingRelay) => talpid_tunnel::ParameterGenerationError::NoMatchingRelay,
                            Ok(proto::error_state::GenerationError::NoWireguardKey) => talpid_tunnel::ParameterGenerationError::NoWireguardKey,
                            Ok(proto::error_state::GenerationError::NetworkIpv4Unavailable) => talpid_tunnel::ParameterGenerationError::IpVersionUnavailable { family: IpVersion::V4 },
                            Ok(proto::error_state::GenerationError::NetworkIpv6Unavailable) => talpid_tunnel::ParameterGenerationError::IpVersionUnavailable { family: IpVersion::V6 },
                            _ => return Err(FromProtobufTypeError::InvalidArgument(
                                "invalid parameter error",
                            )),
                        };
                        talpid_tunnel::ErrorStateCause::TunnelParameterError(parameter_error)
                    }
                    #[cfg(any(target_os = "windows", target_os = "macos"))]
                    Ok(proto::error_state::Cause::SplitTunnelError) => {
                        talpid_tunnel::ErrorStateCause::SplitTunnelError
                    }
                    #[cfg(target_os = "macos")]
                    Ok(proto::error_state::Cause::NeedFullDiskPermissions) => {
                        talpid_tunnel::ErrorStateCause::NeedFullDiskPermissions
                    }
                    _ => {
                        return Err(FromProtobufTypeError::InvalidArgument(
                            "invalid error cause",
                        ))
                    }
                };

                let block_failure = blocking_error
                    .map(|blocking_error| {
                        try_firewall_policy_error_from_i32(
                            blocking_error.r#type,
                            blocking_error.lock_pid,
                            blocking_error.lock_name,
                        )
                    })
                    .transpose()?;

                MullvadState::Error(talpid_tunnel::ErrorState::new(cause, block_failure))
            }
            _ => {
                return Err(FromProtobufTypeError::InvalidArgument(
                    "invalid tunnel state",
                ))
            }
        };

        Ok(state)
    }
}

#[cfg_attr(not(target_os = "windows"), allow(unused_variables))]
fn try_firewall_policy_error_from_i32(
    policy_error: i32,
    lock_pid: u32,
    lock_name: Option<String>,
) -> Result<talpid_types::tunnel::FirewallPolicyError, FromProtobufTypeError> {
    match proto::error_state::firewall_policy_error::ErrorType::try_from(policy_error) {
        Ok(proto::error_state::firewall_policy_error::ErrorType::Generic) => {
            Ok(talpid_types::tunnel::FirewallPolicyError::Generic)
        }
        #[cfg(windows)]
        Ok(proto::error_state::firewall_policy_error::ErrorType::Locked) => {
            let blocking_app = lock_name.map(|name| talpid_types::tunnel::BlockingApplication {
                pid: lock_pid,
                name,
            });
            Ok(talpid_types::tunnel::FirewallPolicyError::Locked(
                blocking_app,
            ))
        }
        _ => Err(FromProtobufTypeError::InvalidArgument(
            "invalid firewall policy error",
        )),
    }
}

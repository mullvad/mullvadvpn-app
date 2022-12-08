#[cfg(windows)]
use crate::types::conversions::option_from_proto_string;
use crate::types::{proto, FromProtobufTypeError};

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
                proto::tunnel_state::State::Disconnected(proto::tunnel_state::Disconnected {})
            }
            MullvadTunnelState::Connecting { endpoint, location } => {
                proto::tunnel_state::State::Connecting(proto::tunnel_state::Connecting {
                    relay_info: Some(proto::TunnelStateRelayInfo {
                        tunnel_endpoint: Some(proto::TunnelEndpoint::from(endpoint)),
                        location: location.map(proto::GeoIpLocation::from),
                    }),
                })
            }
            MullvadTunnelState::Connected { endpoint, location } => {
                proto::tunnel_state::State::Connected(proto::tunnel_state::Connected {
                    relay_info: Some(proto::TunnelStateRelayInfo {
                        tunnel_endpoint: Some(proto::TunnelEndpoint::from(endpoint)),
                        location: location.map(proto::GeoIpLocation::from),
                    }),
                })
            }
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
                            #[cfg(target_os = "windows")]
                            talpid_tunnel::ErrorStateCause::SplitTunnelError => {
                                i32::from(Cause::SplitTunnelError)
                            }
                        },
                        blocking_error: error_state.block_failure().map(map_firewall_error),
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
    proto::error_state::AuthFailedError::from_i32(auth_failed_error)
        .map(mullvad_types::auth_failed::AuthFailed::from)
        .ok_or(FromProtobufTypeError::InvalidArgument(
            "invalid auth failed error",
        ))
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
            Some(proto::tunnel_state::State::Disconnected(_)) => MullvadState::Disconnected,
            Some(proto::tunnel_state::State::Connecting(proto::tunnel_state::Connecting {
                relay_info:
                    Some(proto::TunnelStateRelayInfo {
                        tunnel_endpoint: Some(tunnel_endpoint),
                        location,
                    }),
            })) => MullvadState::Connecting {
                endpoint: talpid_net::TunnelEndpoint::try_from(tunnel_endpoint)?,
                location: location
                    .map(mullvad_types::location::GeoIpLocation::try_from)
                    .transpose()?,
            },
            Some(proto::tunnel_state::State::Connected(proto::tunnel_state::Connected {
                relay_info:
                    Some(proto::TunnelStateRelayInfo {
                        tunnel_endpoint: Some(tunnel_endpoint),
                        location,
                    }),
            })) => MullvadState::Connected {
                endpoint: talpid_net::TunnelEndpoint::try_from(tunnel_endpoint)?,
                location: location
                    .map(mullvad_types::location::GeoIpLocation::try_from)
                    .transpose()?,
            },
            Some(proto::tunnel_state::State::Disconnecting(
                proto::tunnel_state::Disconnecting { after_disconnect },
            )) => MullvadState::Disconnecting(
                match proto::AfterDisconnect::from_i32(after_disconnect) {
                    Some(proto::AfterDisconnect::Nothing) => {
                        talpid_tunnel::ActionAfterDisconnect::Nothing
                    }
                    Some(proto::AfterDisconnect::Block) => {
                        talpid_tunnel::ActionAfterDisconnect::Block
                    }
                    Some(proto::AfterDisconnect::Reconnect) => {
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
                    }),
            })) => {
                let cause = match proto::error_state::Cause::from_i32(cause) {
                    Some(proto::error_state::Cause::AuthFailed) => {
                        let auth_failed = try_auth_failed_from_i32(auth_failed_error)?;
                        talpid_tunnel::ErrorStateCause::AuthFailed(Some(
                            auth_failed.as_str().to_string(),
                        ))
                    }
                    Some(proto::error_state::Cause::Ipv6Unavailable) => {
                        talpid_tunnel::ErrorStateCause::Ipv6Unavailable
                    }
                    Some(proto::error_state::Cause::IsOffline) => {
                        talpid_tunnel::ErrorStateCause::IsOffline
                    }
                    Some(proto::error_state::Cause::SetDnsError) => {
                        talpid_tunnel::ErrorStateCause::SetDnsError
                    }
                    Some(proto::error_state::Cause::SetFirewallPolicyError) => {
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
                    Some(proto::error_state::Cause::StartTunnelError) => {
                        talpid_tunnel::ErrorStateCause::StartTunnelError
                    }
                    Some(proto::error_state::Cause::TunnelParameterError) => {
                        let parameter_error = match proto::error_state::GenerationError::from_i32(parameter_error) {
                            Some(proto::error_state::GenerationError::CustomTunnelHostResolutionError) => talpid_tunnel::ParameterGenerationError::CustomTunnelHostResultionError,
                            Some(proto::error_state::GenerationError::NoMatchingBridgeRelay) => talpid_tunnel::ParameterGenerationError::NoMatchingBridgeRelay,
                            Some(proto::error_state::GenerationError::NoMatchingRelay) => talpid_tunnel::ParameterGenerationError::NoMatchingRelay,
                            Some(proto::error_state::GenerationError::NoWireguardKey) => talpid_tunnel::ParameterGenerationError::NoWireguardKey,
                            _ => return Err(FromProtobufTypeError::InvalidArgument(
                                "invalid parameter error",
                            )),
                        };
                        talpid_tunnel::ErrorStateCause::TunnelParameterError(parameter_error)
                    }
                    #[cfg(target_os = "android")]
                    Some(proto::error_state::Cause::VpnPermissionDenied) => {
                        talpid_tunnel::ErrorStateCause::VpnPermissionDenied
                    }
                    #[cfg(target_os = "windows")]
                    Some(proto::error_state::Cause::SplitTunnelError) => {
                        talpid_tunnel::ErrorStateCause::SplitTunnelError
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
    lock_name: String,
) -> Result<talpid_types::tunnel::FirewallPolicyError, FromProtobufTypeError> {
    match proto::error_state::firewall_policy_error::ErrorType::from_i32(policy_error) {
        Some(proto::error_state::firewall_policy_error::ErrorType::Generic) => {
            Ok(talpid_types::tunnel::FirewallPolicyError::Generic)
        }
        #[cfg(windows)]
        Some(proto::error_state::firewall_policy_error::ErrorType::Locked) => {
            let blocking_app = option_from_proto_string(lock_name).map(|name| {
                talpid_types::tunnel::BlockingApplication {
                    pid: lock_pid,
                    name,
                }
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

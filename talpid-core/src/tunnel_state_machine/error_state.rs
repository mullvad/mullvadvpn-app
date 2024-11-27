use super::{
    ConnectingState, DisconnectedState, EventConsequence, SharedTunnelStateValues, TunnelCommand,
    TunnelCommandReceiver, TunnelState, TunnelStateTransition,
};
#[cfg(target_os = "macos")]
use crate::dns::DnsConfig;
#[cfg(not(target_os = "android"))]
use crate::firewall::FirewallPolicy;
use futures::StreamExt;
#[cfg(target_os = "macos")]
use std::net::Ipv4Addr;
use talpid_types::{
    tunnel::{ErrorStateCause, FirewallPolicyError},
    ErrorExt,
};

/// No tunnel is running and all network connections are blocked.
pub struct ErrorState {
    block_reason: ErrorStateCause,
}

impl ErrorState {
    pub(super) fn enter(
        shared_values: &mut SharedTunnelStateValues,
        block_reason: ErrorStateCause,
    ) -> (Box<dyn TunnelState>, TunnelStateTransition) {
        #[cfg(windows)]
        if let Err(error) = shared_values.split_tunnel.set_tunnel_addresses(None) {
            log::error!(
                "{}",
                error.display_chain_with_msg(
                    "Failed to register addresses with split tunnel driver"
                )
            );
        }

        #[cfg(target_os = "macos")]
        if !block_reason.prevents_filtering_resolver() {
            // Set system DNS to our local DNS resolver
            let system_dns = DnsConfig::default().resolve(
                &[Ipv4Addr::LOCALHOST.into()],
                shared_values.filtering_resolver.listening_port(),
            );
            if let Err(err) = shared_values.dns_monitor.set("lo", system_dns) {
                log::error!(
                    "{}",
                    err.display_chain_with_msg(
                        "Failed to configure system to use filtering resolver"
                    )
                );
                return Self::enter(shared_values, ErrorStateCause::SetDnsError);
            }
        };

        #[cfg(not(target_os = "android"))]
        let block_failure = Self::set_firewall_policy(shared_values).err();

        #[cfg(target_os = "android")]
        let block_failure = if shared_values.restart_tunnel(true).is_err() {
            Some(FirewallPolicyError::Generic)
        } else {
            None
        };

        (
            Box::new(ErrorState {
                block_reason: block_reason.clone(),
            }),
            TunnelStateTransition::Error(talpid_types::tunnel::ErrorState::new(
                block_reason,
                block_failure,
            )),
        )
    }

    #[cfg(not(target_os = "android"))]
    fn set_firewall_policy(
        shared_values: &mut SharedTunnelStateValues,
    ) -> Result<(), FirewallPolicyError> {
        let policy = FirewallPolicy::Blocked {
            allow_lan: shared_values.allow_lan,
            allowed_endpoint: Some(shared_values.allowed_endpoint.clone()),
            #[cfg(target_os = "macos")]
            dns_redirect_port: shared_values.filtering_resolver.listening_port(),
        };

        #[cfg(target_os = "linux")]
        shared_values.disable_connectivity_check();

        shared_values
            .firewall
            .apply_policy(policy)
            .map_err(|error| {
                log::error!(
                    "{}",
                    error.display_chain_with_msg(
                        "Failed to apply firewall policy for blocked state"
                    )
                );
                match error {
                    #[cfg(windows)]
                    crate::firewall::Error::ApplyingBlockedPolicy(policy_error) => policy_error,
                    _ => FirewallPolicyError::Generic,
                }
            })
    }

    fn reset_dns(shared_values: &mut SharedTunnelStateValues) {
        if let Err(error) = shared_values.dns_monitor.reset() {
            log::error!("{}", error.display_chain_with_msg("Unable to reset DNS"));
        }
    }
}

impl TunnelState for ErrorState {
    #[cfg_attr(not(target_os = "macos"), allow(unused_mut))]
    fn handle_event(
        self: Box<Self>,
        runtime: &tokio::runtime::Handle,
        commands: &mut TunnelCommandReceiver,
        shared_values: &mut SharedTunnelStateValues,
    ) -> EventConsequence {
        use self::EventConsequence::*;

        match runtime.block_on(commands.next()) {
            Some(TunnelCommand::AllowLan(allow_lan, complete_tx)) => {
                let consequence = if shared_values.set_allow_lan(allow_lan) {
                    #[cfg(target_os = "android")]
                    if let Err(_err) = shared_values.restart_tunnel(true) {
                        NewState(Self::enter(
                            shared_values,
                            ErrorStateCause::StartTunnelError,
                        ))
                    } else {
                        SameState(self)
                    }
                    #[cfg(not(target_os = "android"))]
                    {
                        let _ = Self::set_firewall_policy(shared_values);
                        SameState(self)
                    }
                } else {
                    SameState(self)
                };

                let _ = complete_tx.send(());
                consequence
            }
            #[cfg(not(target_os = "android"))]
            Some(TunnelCommand::AllowEndpoint(endpoint, tx)) => {
                if shared_values.allowed_endpoint != endpoint {
                    shared_values.allowed_endpoint = endpoint;
                    let _ = Self::set_firewall_policy(shared_values);
                }
                let _ = tx.send(());
                SameState(self)
            }
            Some(TunnelCommand::Dns(servers, complete_tx)) => {
                let consequence = if shared_values.set_dns_config(servers) {
                    #[cfg(target_os = "android")]
                    {
                        // DNS is blocked in the error state, so only update tun config
                        shared_values.prepare_tun_config(true);
                        if let ErrorStateCause::InvalidDnsServers(_) = self.block_reason {
                            NewState(ConnectingState::enter(shared_values, 0))
                        } else {
                            SameState(self)
                        }
                    }
                    #[cfg(not(target_os = "android"))]
                    {
                        let _ = Self::set_firewall_policy(shared_values);
                        SameState(self)
                    }
                } else {
                    SameState(self)
                };
                let _ = complete_tx.send(());
                consequence
            }
            #[cfg(not(target_os = "android"))]
            Some(TunnelCommand::BlockWhenDisconnected(block_when_disconnected, complete_tx)) => {
                shared_values.block_when_disconnected = block_when_disconnected;
                let _ = complete_tx.send(());
                SameState(self)
            }
            Some(TunnelCommand::Connectivity(connectivity)) => {
                shared_values.connectivity = connectivity;
                if !connectivity.is_offline()
                    && matches!(self.block_reason, ErrorStateCause::IsOffline)
                {
                    Self::reset_dns(shared_values);
                    NewState(ConnectingState::enter(shared_values, 0))
                } else {
                    SameState(self)
                }
            }
            Some(TunnelCommand::Connect) => {
                Self::reset_dns(shared_values);

                NewState(ConnectingState::enter(shared_values, 0))
            }
            Some(TunnelCommand::Disconnect) | None => {
                #[cfg(target_os = "linux")]
                shared_values.reset_connectivity_check();
                Self::reset_dns(shared_values);
                NewState(DisconnectedState::enter(shared_values, true))
            }
            Some(TunnelCommand::Block(reason)) => {
                NewState(ErrorState::enter(shared_values, reason))
            }
            #[cfg(target_os = "android")]
            Some(TunnelCommand::BypassSocket(fd, done_tx)) => {
                shared_values.bypass_socket(fd, done_tx);
                SameState(self)
            }
            #[cfg(target_os = "android")]
            Some(TunnelCommand::SetExcludedApps(result_tx, paths)) => {
                if shared_values.set_excluded_paths(paths) {
                    if let Err(err) = shared_values.restart_tunnel(true) {
                        let _ =
                            result_tx.send(Err(crate::split_tunnel::Error::SetExcludedApps(err)));
                    }
                } else {
                    let _ = result_tx.send(Ok(()));
                }
                SameState(self)
            }
            #[cfg(windows)]
            Some(TunnelCommand::SetExcludedApps(result_tx, paths)) => {
                shared_values.exclude_paths(paths, result_tx);
                SameState(self)
            }
            #[cfg(target_os = "macos")]
            Some(TunnelCommand::SetExcludedApps(result_tx, paths)) => {
                let _ = result_tx.send(shared_values.set_exclude_paths(paths).map(|_| ()));
                SameState(self)
            }
        }
    }
}

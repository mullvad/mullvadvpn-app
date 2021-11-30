use super::{
    ConnectingState, DisconnectedState, EventConsequence, SharedTunnelStateValues, TunnelCommand,
    TunnelCommandReceiver, TunnelState, TunnelStateTransition, TunnelStateWrapper,
};
use crate::firewall::FirewallPolicy;
#[cfg(target_os = "macos")]
use crate::resolver;
use futures::StreamExt;
#[cfg(target_os = "macos")]
use std::{
    collections::BTreeSet,
    net::{IpAddr, Ipv4Addr},
};
use talpid_types::{
    tunnel::{self as talpid_tunnel, ErrorStateCause, FirewallPolicyError},
    ErrorExt,
};

/// No tunnel is running and all network connections are blocked.
pub struct ErrorState {
    #[cfg(target_os = "macos")]
    allowed_ips: BTreeSet<IpAddr>,
    block_reason: ErrorStateCause,
}

impl ErrorState {
    fn set_firewall(
        &self,
        shared_values: &mut SharedTunnelStateValues,
    ) -> Result<(), FirewallPolicyError> {
        Self::set_firewall_policy(
            shared_values,
            #[cfg(target_os = "macos")]
            self.allowed_ips.clone(),
            #[cfg(target_os = "macos")]
            shared_values.enable_custom_resolver,
        )
    }

    fn set_firewall_policy(
        shared_values: &mut SharedTunnelStateValues,
        #[cfg(target_os = "macos")] allowed_ips: BTreeSet<IpAddr>,
        #[cfg(target_os = "macos")] allow_custom_resolver: bool,
    ) -> Result<(), FirewallPolicyError> {
        let policy = FirewallPolicy::Blocked {
            allow_lan: shared_values.allow_lan,
            allowed_endpoint: shared_values.allowed_endpoint.clone(),
            #[cfg(target_os = "macos")]
            allowed_ips,
            #[cfg(target_os = "macos")]
            allow_custom_resolver,
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

    /// Returns true if a new tunnel device was successfully created.
    #[cfg(target_os = "android")]
    fn create_blocking_tun(shared_values: &mut SharedTunnelStateValues) -> bool {
        match shared_values.tun_provider.create_blocking_tun() {
            Ok(()) => true,
            Err(error) => {
                log::error!(
                    "{}",
                    error.display_chain_with_msg(
                        "Failed to open tunnel adapter to drop packets for blocked state"
                    )
                );
                false
            }
        }
    }

    fn reset_dns(shared_values: &mut SharedTunnelStateValues) {
        if let Err(error) = shared_values.dns_monitor.reset() {
            log::error!("{}", error.display_chain_with_msg("Unable to reset DNS"));
        }
    }
}

impl TunnelState for ErrorState {
    type Bootstrap = ErrorStateCause;

    fn enter(
        shared_values: &mut SharedTunnelStateValues,
        block_reason: Self::Bootstrap,
    ) -> (TunnelStateWrapper, TunnelStateTransition) {
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
        let host_config =
            if shared_values.enable_custom_resolver && !block_reason.prevents_custom_resolver() {
                if let Err(err) = shared_values
                    .dns_monitor
                    .set("lo", &[Ipv4Addr::LOCALHOST.into()])
                {
                    log::error!(
                        "{}",
                        err.display_chain_with_msg("Failed to configure custom resolver")
                    );
                    return Self::enter(shared_values, ErrorStateCause::SetDnsError);
                }
                match shared_values.get_custom_resolver_config() {
                    Ok(host_config) => host_config,
                    Err(err) => {
                        log::error!(
                            "{}",
                            err.display_chain_with_msg("Failed to start custom resolver")
                        );
                        return Self::enter(shared_values, ErrorStateCause::CustomResolverError);
                    }
                }
            } else {
                None
            };

        #[cfg(not(target_os = "android"))]
        let block_failure = Self::set_firewall_policy(
            shared_values,
            #[cfg(target_os = "macos")]
            BTreeSet::new(),
            #[cfg(target_os = "macos")]
            shared_values.enable_custom_resolver,
        )
        .err();

        #[cfg(target_os = "macos")]
        if let Some(dns_config) = host_config {
            if let Err(err) = shared_values
                .runtime
                .block_on(shared_values.custom_resolver.set_active(Some(dns_config)))
            {
                log::error!(
                    "{}",
                    err.display_chain_with_msg("Failed to activate custom resolver")
                );
                return Self::enter(shared_values, ErrorStateCause::CustomResolverError);
            }
        }

        #[cfg(target_os = "android")]
        let block_failure = if !Self::create_blocking_tun(shared_values) {
            Some(FirewallPolicyError::Generic)
        } else {
            None
        };
        (
            TunnelStateWrapper::from(ErrorState {
                block_reason: block_reason.clone(),
                #[cfg(target_os = "macos")]
                allowed_ips: BTreeSet::new(),
            }),
            TunnelStateTransition::Error(talpid_tunnel::ErrorState::new(
                block_reason,
                block_failure,
            )),
        )
    }

    #[cfg_attr(not(target_os = "macos"), allow(unused_mut))]
    fn handle_event(
        mut self,
        runtime: &tokio::runtime::Handle,
        commands: &mut TunnelCommandReceiver,
        shared_values: &mut SharedTunnelStateValues,
    ) -> EventConsequence {
        use self::EventConsequence::*;

        match runtime.block_on(commands.next()) {
            #[cfg(target_os = "macos")]
            Some(TunnelCommand::AddAllowedIps(allowed_ips, done_tx)) => {
                let new_addresses = allowed_ips.iter().any(|ip| self.allowed_ips.insert(*ip));
                if new_addresses {
                    if let Err(err) = self.set_firewall(shared_values) {
                        return NewState(Self::enter(
                            shared_values,
                            ErrorStateCause::SetFirewallPolicyError(err),
                        ));
                    }
                }
                let _ = done_tx.send(());
                SameState(self.into())
            }

            #[cfg(target_os = "macos")]
            Some(TunnelCommand::SetCustomResolver(enable, done_tx)) => {
                let result = if enable && !shared_values.enable_custom_resolver {
                    shared_values.enable_custom_resolver = enable;
                    if let Err(err) = self.set_firewall(shared_values) {
                        return NewState(ErrorState::enter(
                            shared_values,
                            ErrorStateCause::SetFirewallPolicyError(err),
                        ));
                    }

                    match shared_values.dns_monitor.get_system_config() {
                        Ok(current_system_config) => {
                            match shared_values.runtime.block_on(
                                shared_values
                                    .custom_resolver
                                    .set_active(current_system_config),
                            ) {
                                Ok(_) => {
                                    if let Err(err) = shared_values
                                        .dns_monitor
                                        .set("lo", &[Ipv4Addr::LOCALHOST.into()])
                                    {
                                        log::error!(
                                            "{}",
                                            err.display_chain_with_msg(
                                                "Failed to configure system to use custom resolver"
                                            )
                                        );
                                        let _ =
                                            done_tx.send(Err(resolver::Error::SystemDnsError(err)));
                                        return NewState(ErrorState::enter(
                                            shared_values,
                                            ErrorStateCause::SetDnsError,
                                        ));
                                    }
                                    Ok(())
                                }

                                Err(err) => {
                                    log::error!(
                                        "{}",
                                        err.display_chain_with_msg(
                                            "Failed to start custom resolver"
                                        )
                                    );
                                    Err(err)
                                }
                            }
                        }
                        Err(err) => {
                            log::error!(
                                "{}",
                                err.display_chain_with_msg("Failed to obtain system DNS config")
                            );

                            let _ = done_tx.send(Err(resolver::Error::SystemDnsError(err)));
                            return NewState(ErrorState::enter(
                                shared_values,
                                ErrorStateCause::ReadSystemDnsConfig,
                            ));
                        }
                    }
                } else {
                    shared_values.deactivate_custom_resolver(enable)
                };
                let _ = done_tx.send(result);
                SameState(self.into())
            }

            #[cfg(target_os = "macos")]
            Some(TunnelCommand::HostDnsConfig(host_config)) => {
                if shared_values.enable_custom_resolver {
                    if let Err(err) = shared_values
                        .runtime
                        .block_on(shared_values.custom_resolver.set_active(host_config))
                    {
                        log::error!(
                            "Failed to set apply new DNS config to custom resolver: {}",
                            err
                        );
                        return NewState(Self::enter(
                            shared_values,
                            ErrorStateCause::CustomResolverError,
                        ));
                    }
                }
                SameState(self.into())
            }
            Some(TunnelCommand::AllowLan(allow_lan)) => {
                if let Err(error_state_cause) = shared_values.set_allow_lan(allow_lan) {
                    NewState(Self::enter(shared_values, error_state_cause))
                } else {
                    let _ = self.set_firewall(shared_values);
                    SameState(self.into())
                }
            }
            Some(TunnelCommand::AllowEndpoint(endpoint, tx)) => {
                if shared_values.set_allowed_endpoint(endpoint) {
                    let _ = self.set_firewall(shared_values);

                    #[cfg(target_os = "android")]
                    if !Self::create_blocking_tun(shared_values) {
                        return NewState(Self::enter(
                            shared_values,
                            ErrorStateCause::SetFirewallPolicyError(FirewallPolicyError::Generic),
                        ));
                    }
                }
                if let Err(_) = tx.send(()) {
                    log::error!("The AllowEndpoint receiver was dropped");
                }
                SameState(self.into())
            }
            Some(TunnelCommand::Dns(servers)) => {
                if let Err(error_state_cause) = shared_values.set_dns_servers(servers) {
                    NewState(Self::enter(shared_values, error_state_cause))
                } else {
                    SameState(self.into())
                }
            }
            Some(TunnelCommand::BlockWhenDisconnected(block_when_disconnected)) => {
                shared_values.block_when_disconnected = block_when_disconnected;
                SameState(self.into())
            }
            Some(TunnelCommand::IsOffline(is_offline)) => {
                shared_values.is_offline = is_offline;
                if !is_offline && self.block_reason == ErrorStateCause::IsOffline {
                    Self::reset_dns(shared_values);
                    NewState(ConnectingState::enter(shared_values, 0))
                } else {
                    SameState(self.into())
                }
            }
            Some(TunnelCommand::Connect) => {
                Self::reset_dns(shared_values);

                NewState(ConnectingState::enter(shared_values, 0))
            }
            Some(TunnelCommand::Disconnect) | None => {
                #[cfg(target_os = "linux")]
                shared_values.reset_connectivity_check();
                #[cfg(target_os = "macos")]
                if !shared_values.block_when_disconnected {
                    if let Err(err) = shared_values.disable_custom_resolver() {
                        log::error!("Failed to disable custom resolver: {}", err);
                    }
                }
                Self::reset_dns(shared_values);
                NewState(DisconnectedState::enter(shared_values, true))
            }
            Some(TunnelCommand::Block(reason)) => {
                NewState(ErrorState::enter(shared_values, reason))
            }
            #[cfg(target_os = "android")]
            Some(TunnelCommand::BypassSocket(fd, done_tx)) => {
                shared_values.bypass_socket(fd, done_tx);
                SameState(self.into())
            }
            #[cfg(windows)]
            Some(TunnelCommand::SetExcludedApps(result_tx, paths)) => {
                shared_values.split_tunnel.set_paths(&paths, result_tx);
                SameState(self.into())
            }
        }
    }
}

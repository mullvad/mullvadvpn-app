use super::{
    ConnectingState, ErrorState, EventConsequence, SharedTunnelStateValues, TunnelCommand,
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
#[cfg(target_os = "macos")]
use talpid_types::tunnel::ErrorStateCause;
use talpid_types::ErrorExt;

/// No tunnel is running.
pub struct DisconnectedState {
    #[cfg(target_os = "macos")]
    allowed_ips: BTreeSet<IpAddr>,
    #[cfg(target_os = "macos")]
    allowed_resolvers: BTreeSet<IpAddr>,
}

impl DisconnectedState {
    #[cfg(target_os = "macos")]
    fn reset_allowed_resolvers(
        &mut self,
        resolver_config: &Option<(String, Vec<IpAddr>)>,
        shared_values: &mut SharedTunnelStateValues,
    ) {
        if let Some((_interface, resolver_ips)) = &resolver_config {
            self.allowed_resolvers = resolver_ips.iter().cloned().collect();
        } else {
            self.allowed_resolvers = BTreeSet::new();
        }
        self.set_firewall_policy(shared_values, false);
    }

    fn set_firewall_policy(
        &mut self,
        shared_values: &mut SharedTunnelStateValues,
        should_reset_firewall: bool,
    ) {
        let result = if shared_values.block_when_disconnected {
            let policy = FirewallPolicy::Blocked {
                allow_lan: shared_values.allow_lan,
                allowed_endpoint: shared_values.allowed_endpoint.clone(),
                #[cfg(target_os = "macos")]
                allowed_ips: self.allowed_ips.clone(),
                #[cfg(target_os = "macos")]
                allowed_resolvers: self.allowed_resolvers.clone(),
            };

            let firewall_result = shared_values.firewall.apply_policy(policy).map_err(|e| {
                e.display_chain_with_msg(
                    "Failed to apply blocking firewall policy for disconnected state",
                )
            });

            firewall_result
        } else if should_reset_firewall {
            shared_values
                .firewall
                .reset_policy()
                .map_err(|e| e.display_chain_with_msg("Failed to reset firewall policy"))
        } else {
            Ok(())
        };
        if let Err(error_chain) = result {
            log::error!("{}", error_chain);
        }
    }

    #[cfg(windows)]
    fn register_split_tunnel_addresses(
        shared_values: &mut SharedTunnelStateValues,
        should_reset_firewall: bool,
    ) {
        if should_reset_firewall && !shared_values.block_when_disconnected {
            if let Err(error) = shared_values.split_tunnel.clear_tunnel_addresses() {
                log::error!(
                    "{}",
                    error.display_chain_with_msg(
                        "Failed to unregister addresses with split tunnel driver"
                    )
                );
            }
        } else {
            if let Err(error) = shared_values.split_tunnel.set_tunnel_addresses(None) {
                log::error!(
                    "{}",
                    error
                        .display_chain_with_msg("Failed to reset addresses in split tunnel driver")
                );
            }
        }
    }

    #[cfg(target_os = "macos")]
    fn set_dns(shared_values: &mut SharedTunnelStateValues) {
        if let Some(ref dns_servers) = shared_values.dns_servers {
            if let Err(err) = shared_values.dns_monitor.set("lo0", &dns_servers) {
                log::error!("failed to set custom DNS servers: {}", err);
            }
        }
    }

    fn reset_dns(shared_values: &mut SharedTunnelStateValues) {
        if let Err(error) = shared_values.dns_monitor.reset() {
            log::error!("{}", error.display_chain_with_msg("Unable to reset DNS"));
        }
    }
}

impl TunnelState for DisconnectedState {
    type Bootstrap = bool;

    fn enter(
        shared_values: &mut SharedTunnelStateValues,
        should_reset_firewall: Self::Bootstrap,
    ) -> (TunnelStateWrapper, TunnelStateTransition) {
        let mut disconnected_state = DisconnectedState {
            #[cfg(target_os = "macos")]
            allowed_ips: BTreeSet::new(),
            #[cfg(target_os = "macos")]
            allowed_resolvers: BTreeSet::new(),
        };

        #[cfg(target_os = "macos")]
        if shared_values.enable_custom_resolver {
            if let Err(err) = shared_values
                .dns_monitor
                .set("lo", &[Ipv4Addr::LOCALHOST.into()])
            {
                log::error!(
                    "{}",
                    err.display_chain_with_msg("Failed to configure system to use custom resolver")
                );
            }
        } else {
            if let Err(error) = shared_values.disable_custom_resolver() {
                log::error!(
                    "{}",
                    error.display_chain_with_msg("Unable to disable custom resolver")
                );
            }
        }

        #[cfg(windows)]
        Self::register_split_tunnel_addresses(shared_values, should_reset_firewall);
        disconnected_state.set_firewall_policy(shared_values, should_reset_firewall);
        #[cfg(target_os = "linux")]
        shared_values.reset_connectivity_check();
        #[cfg(target_os = "android")]
        shared_values.tun_provider.close_tun();

        (
            TunnelStateWrapper::from(disconnected_state),
            TunnelStateTransition::Disconnected,
        )
    }

    fn handle_event(
        mut self,
        runtime: &tokio::runtime::Handle,
        commands: &mut TunnelCommandReceiver,
        shared_values: &mut SharedTunnelStateValues,
    ) -> EventConsequence {
        use self::EventConsequence::*;

        match runtime.block_on(commands.next()) {
            Some(TunnelCommand::AllowLan(allow_lan)) => {
                if shared_values.allow_lan != allow_lan {
                    // The only platform that can fail is Android, but Android doesn't support the
                    // "block when disconnected" option, so the following call never fails.
                    shared_values
                        .set_allow_lan(allow_lan)
                        .expect("Failed to set allow LAN parameter");

                    self.set_firewall_policy(shared_values, true);
                }
                SameState(self.into())
            }
            Some(TunnelCommand::AllowEndpoint(endpoint, tx)) => {
                if shared_values.set_allowed_endpoint(endpoint) {
                    self.set_firewall_policy(shared_values, true);
                }
                if let Err(_) = tx.send(()) {
                    log::error!("The AllowEndpoint receiver was dropped");
                }
                SameState(self.into())
            }
            Some(TunnelCommand::Dns(servers)) => {
                // Same situation as allow LAN above.
                shared_values
                    .set_dns_servers(servers)
                    .expect("Failed to reconnect after changing custom DNS servers");
                Self::set_dns(shared_values);

                SameState(self.into())
            }
            Some(TunnelCommand::BlockWhenDisconnected(block_when_disconnected)) => {
                if shared_values.block_when_disconnected != block_when_disconnected {
                    shared_values.block_when_disconnected = block_when_disconnected;
                    #[cfg(windows)]
                    Self::register_split_tunnel_addresses(shared_values, true);
                    #[cfg(target_os = "macos")]
                    if block_when_disconnected {
                        Self::set_dns(shared_values);
                    } else {
                        Self::reset_dns(shared_values);
                    }
                    self.set_firewall_policy(shared_values, true);
                }
                SameState(self.into())
            }
            Some(TunnelCommand::IsOffline(is_offline)) => {
                shared_values.is_offline = is_offline;
                SameState(self.into())
            }
            Some(TunnelCommand::Connect) => NewState(ConnectingState::enter(shared_values, 0)),
            Some(TunnelCommand::Block(reason)) => {
                Self::reset_dns(shared_values);
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
            #[cfg(target_os = "macos")]
            Some(TunnelCommand::SetCustomResolver(enable, done_tx)) => {
                if let Err(err) = shared_values.deactivate_custom_resolver(enable) {
                    let _ = done_tx.send(Err(err));
                    return SameState(self.into());
                };

                if shared_values.block_when_disconnected && enable {
                    match shared_values.dns_monitor.get_system_config() {
                        Ok(system_config) => {
                            self.reset_allowed_resolvers(&system_config, shared_values);

                            match shared_values
                                .runtime
                                .block_on(shared_values.custom_resolver.set_active(system_config))
                            {
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
                                        return NewState(ErrorState::enter(
                                            shared_values,
                                            ErrorStateCause::SetDnsError,
                                        ));
                                    }
                                }
                                Err(err) => {
                                    let _ = done_tx.send(Err(err));
                                }
                            }
                        }
                        Err(err) => {
                            log::error!(
                                "{}",
                                err.display_chain_with_msg("Failed to obtain system DNS config")
                            );

                            let _ = done_tx.send(Err(resolver::Error::SystemDnsError(err)));
                        }
                    }
                } else {
                    let _ = done_tx.send(Ok(()));
                }
                SameState(self.into())
            }
            #[cfg(target_os = "macos")]
            Some(TunnelCommand::HostDnsConfig(host_config)) => {
                if shared_values.block_when_disconnected && shared_values.enable_custom_resolver {
                    self.reset_allowed_resolvers(&host_config, shared_values);
                    if let Err(err) = shared_values
                        .runtime
                        .block_on(shared_values.custom_resolver.set_active(host_config))
                    {
                        log::error!(
                            "{}",
                            err.display_chain_with_msg("Failed to activate custom resolver")
                        );
                        return NewState(ErrorState::enter(
                            shared_values,
                            ErrorStateCause::CustomResolverError,
                        ));
                    }
                }
                SameState(self.into())
            }
            #[cfg(target_os = "macos")]
            Some(TunnelCommand::AddAllowedIps(allowed_ips, done_tx)) => {
                let new_addresses = allowed_ips.iter().any(|ip| self.allowed_ips.insert(*ip));
                if new_addresses {
                    let _ = self.set_firewall_policy(shared_values, false);
                }
                let _ = done_tx.send(());

                SameState(self.into())
            }

            None => {
                Self::reset_dns(shared_values);
                Finished
            }
            Some(_) => SameState(self.into()),
        }
    }
}

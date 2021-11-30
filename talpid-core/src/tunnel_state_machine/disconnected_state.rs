use super::{
    ConnectingState, ErrorState, EventConsequence, SharedTunnelStateValues, TunnelCommand,
    TunnelCommandReceiver, TunnelState, TunnelStateTransition, TunnelStateWrapper,
};
use crate::firewall::FirewallPolicy;
#[cfg(target_os = "macos")]
use crate::{dns, resolver};
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
}

impl DisconnectedState {
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
                allow_custom_resolver: shared_values.enable_custom_resolver,
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

    fn reset_dns(shared_values: &mut SharedTunnelStateValues) {
        if let Err(error) = shared_values.dns_monitor.reset() {
            log::error!("{}", error.display_chain_with_msg("Unable to reset DNS"));
        }
    }

    #[cfg(target_os = "macos")]
    fn start_custom_resolver(
        &mut self,
        shared_values: &mut SharedTunnelStateValues,
    ) -> Result<(), either::Either<resolver::Error, dns::Error>> {
        use either::Either;
        let system_config = shared_values
            .dns_monitor
            .get_system_config()
            .map_err(Either::Right)?;

        shared_values
            .runtime
            .block_on(shared_values.custom_resolver.set_active(system_config))
            .map_err(Either::Left)?;
        shared_values
            .dns_monitor
            .set("lo", &[Ipv4Addr::LOCALHOST.into()])
            .map_err(resolver::Error::SystemDnsError)
            .map_err(Either::Left)
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
        };

        #[cfg(target_os = "macos")]
        if shared_values.enable_custom_resolver {
            if let Err(err) = disconnected_state.start_custom_resolver(shared_values) {
                log::error!(
                    "{}",
                    err.display_chain_with_msg("Failed to start custom resolver:")
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

                SameState(self.into())
            }
            Some(TunnelCommand::BlockWhenDisconnected(block_when_disconnected)) => {
                if shared_values.block_when_disconnected != block_when_disconnected {
                    shared_values.block_when_disconnected = block_when_disconnected;
                    self.set_firewall_policy(shared_values, true);
                    #[cfg(windows)]
                    Self::register_split_tunnel_addresses(shared_values, true);
                    #[cfg(target_os = "macos")]
                    if block_when_disconnected {
                        if let Err(err) = self.start_custom_resolver(shared_values) {
                            let block_reason = map_custom_resolver_start(&err);
                            return NewState(ErrorState::enter(shared_values, block_reason));
                        }
                    } else {
                        Self::reset_dns(shared_values);
                    }
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
                if !enable {
                    if let Err(err) = shared_values.deactivate_custom_resolver(enable) {
                        let _ = done_tx.send(Err(err));
                        if shared_values.enable_custom_resolver {
                            self.set_firewall_policy(shared_values, false);
                        }
                        return SameState(self.into());
                    };
                }
                if shared_values.enable_custom_resolver != enable {
                    shared_values.enable_custom_resolver = enable;
                    self.set_firewall_policy(shared_values, false);
                    if shared_values.block_when_disconnected && enable {
                        if let Err(err) = self.start_custom_resolver(shared_values) {
                            log::error!(
                                "{}",
                                err.display_chain_with_msg("Failed to start custom resolver:")
                            );

                            let error_cause = map_custom_resolver_start(&err);
                            let _ = done_tx.send(Err(err.left_or_else(resolver::Error::from)));
                            return NewState(ErrorState::enter(shared_values, error_cause));
                        }
                    }
                }
                let _ = done_tx.send(Ok(()));
                SameState(self.into())
            }
            #[cfg(target_os = "macos")]
            Some(TunnelCommand::HostDnsConfig(host_config)) => {
                if shared_values.block_when_disconnected && shared_values.enable_custom_resolver {
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

#[cfg(target_os = "macos")]
fn map_custom_resolver_start(err: &either::Either<resolver::Error, dns::Error>) -> ErrorStateCause {
    match err {
        either::Either::Right(_dns_err) => ErrorStateCause::SetDnsError,
        either::Either::Left(resolver::Error::SystemDnsError(_)) => {
            ErrorStateCause::ReadSystemDnsConfig
        }
        either::Either::Left(_other_err) => ErrorStateCause::CustomResolverError,
    }
}

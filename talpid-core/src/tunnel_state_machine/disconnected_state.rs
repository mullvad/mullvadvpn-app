use super::{
    ConnectingState, ErrorState, EventConsequence, SharedTunnelStateValues, TunnelCommand,
    TunnelCommandReceiver, TunnelState, TunnelStateTransition,
};
#[cfg(target_os = "macos")]
use crate::dns;
use crate::firewall::FirewallPolicy;
use futures::StreamExt;
#[cfg(target_os = "macos")]
use std::net::Ipv4Addr;
#[cfg(target_os = "macos")]
use talpid_types::tunnel::ErrorStateCause;
use talpid_types::ErrorExt;

/// No tunnel is running.
pub struct DisconnectedState(());

impl DisconnectedState {
    pub(super) fn enter(
        shared_values: &mut SharedTunnelStateValues,
        should_reset_firewall: bool,
    ) -> (Box<dyn TunnelState>, TunnelStateTransition) {
        #[cfg(target_os = "macos")]
        if shared_values.block_when_disconnected {
            if let Err(err) = Self::setup_local_dns_config(shared_values) {
                log::error!(
                    "{}",
                    err.display_chain_with_msg("Failed to start filtering resolver:")
                );
            }
        } else if let Err(error) = shared_values.dns_monitor.reset() {
            log::error!(
                "{}",
                error.display_chain_with_msg("Unable to disable filtering resolver")
            );
        }
        #[cfg(windows)]
        Self::register_split_tunnel_addresses(shared_values, should_reset_firewall);
        Self::set_firewall_policy(shared_values, should_reset_firewall);
        #[cfg(target_os = "linux")]
        shared_values.reset_connectivity_check();
        #[cfg(target_os = "android")]
        shared_values.tun_provider.lock().unwrap().close_tun();

        Self::construct_state_transition(shared_values)
    }

    fn construct_state_transition(
        shared_values: &mut SharedTunnelStateValues,
    ) -> (Box<dyn TunnelState>, TunnelStateTransition) {
        (
            Box::new(DisconnectedState(())),
            TunnelStateTransition::Disconnected {
                // Being disconnected and having lockdown mode enabled implies that your internet
                // access is locked down
                locked_down: shared_values.block_when_disconnected,
            },
        )
    }

    fn set_firewall_policy(
        shared_values: &mut SharedTunnelStateValues,
        should_reset_firewall: bool,
    ) {
        let result = if shared_values.block_when_disconnected {
            let policy = FirewallPolicy::Blocked {
                allow_lan: shared_values.allow_lan,
                allowed_endpoint: Some(shared_values.allowed_endpoint.clone()),
                #[cfg(target_os = "macos")]
                dns_redirect_port: shared_values.filtering_resolver.listening_port(),
            };

            shared_values.firewall.apply_policy(policy).map_err(|e| {
                e.display_chain_with_msg(
                    "Failed to apply blocking firewall policy for disconnected state",
                )
            })
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
        } else if let Err(error) = shared_values.split_tunnel.set_tunnel_addresses(None) {
            log::error!(
                "{}",
                error.display_chain_with_msg("Failed to reset addresses in split tunnel driver")
            );
        }
    }

    fn reset_dns(shared_values: &mut SharedTunnelStateValues) {
        if let Err(error) = shared_values.dns_monitor.reset() {
            log::error!("{}", error.display_chain_with_msg("Unable to reset DNS"));
        }
    }

    /// Configures host to use a localhost resolver
    #[cfg(target_os = "macos")]
    fn setup_local_dns_config(
        shared_values: &mut SharedTunnelStateValues,
    ) -> Result<(), dns::Error> {
        shared_values
            .dns_monitor
            .set("lo", &[Ipv4Addr::LOCALHOST.into()])
    }
}

impl TunnelState for DisconnectedState {
    fn handle_event(
        self: Box<Self>,
        runtime: &tokio::runtime::Handle,
        commands: &mut TunnelCommandReceiver,
        shared_values: &mut SharedTunnelStateValues,
    ) -> EventConsequence {
        use self::EventConsequence::*;

        match runtime.block_on(commands.next()) {
            Some(TunnelCommand::AllowLan(allow_lan, complete_tx)) => {
                if shared_values.allow_lan != allow_lan {
                    // The only platform that can fail is Android, but Android doesn't support the
                    // "block when disconnected" option, so the following call never fails.
                    shared_values
                        .set_allow_lan(allow_lan)
                        .expect("Failed to set allow LAN parameter");

                    Self::set_firewall_policy(shared_values, false);
                }
                let _ = complete_tx.send(());
                SameState(self)
            }
            Some(TunnelCommand::AllowEndpoint(endpoint, tx)) => {
                if shared_values.allowed_endpoint != endpoint {
                    shared_values.allowed_endpoint = endpoint;
                    Self::set_firewall_policy(shared_values, false);
                }
                let _ = tx.send(());
                SameState(self)
            }
            Some(TunnelCommand::Dns(servers, complete_tx)) => {
                // Same situation as allow LAN above.
                shared_values
                    .set_dns_servers(servers)
                    .expect("Failed to reconnect after changing custom DNS servers");
                let _ = complete_tx.send(());
                SameState(self)
            }
            Some(TunnelCommand::BlockWhenDisconnected(block_when_disconnected, complete_tx)) => {
                if shared_values.block_when_disconnected != block_when_disconnected {
                    shared_values.block_when_disconnected = block_when_disconnected;

                    // TODO: Investigate if we can simply return
                    // `NewState(Self::enter(shared_values, true))`.
                    // The logic for updating the firewall in `DisconnectedState::enter` is
                    // identical but it does not enter the error state if setting the local DNS
                    // fails.
                    Self::set_firewall_policy(shared_values, true);
                    #[cfg(windows)]
                    Self::register_split_tunnel_addresses(shared_values, true);
                    #[cfg(target_os = "macos")]
                    if block_when_disconnected {
                        if let Err(err) = Self::setup_local_dns_config(shared_values) {
                            log::error!(
                                "{}",
                                err.display_chain_with_msg("Failed to configure host DNS")
                            );
                            return NewState(ErrorState::enter(
                                shared_values,
                                ErrorStateCause::SetDnsError,
                            ));
                        }
                    } else {
                        Self::reset_dns(shared_values);
                    }
                    let _ = complete_tx.send(());
                    NewState(Self::construct_state_transition(shared_values))
                } else {
                    let _ = complete_tx.send(());
                    SameState(self)
                }
            }
            Some(TunnelCommand::Connectivity(connectivity)) => {
                shared_values.connectivity = connectivity;
                SameState(self)
            }
            Some(TunnelCommand::Connect) => NewState(ConnectingState::enter(shared_values, 0)),
            Some(TunnelCommand::Block(reason)) => {
                Self::reset_dns(shared_values);
                NewState(ErrorState::enter(shared_values, reason))
            }
            #[cfg(target_os = "android")]
            Some(TunnelCommand::BypassSocket(fd, done_tx)) => {
                shared_values.bypass_socket(fd, done_tx);
                SameState(self)
            }
            #[cfg(windows)]
            Some(TunnelCommand::SetExcludedApps(result_tx, paths)) => {
                shared_values.split_tunnel.set_paths(&paths, result_tx);
                SameState(self)
            }
            None => {
                Self::reset_dns(shared_values);
                Finished
            }
            Some(_) => SameState(self),
        }
    }
}

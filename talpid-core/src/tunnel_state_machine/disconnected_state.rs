use super::{
    ConnectingState, ErrorState, EventConsequence, SharedTunnelStateValues, TunnelCommand,
    TunnelCommandReceiver, TunnelState, TunnelStateTransition, TunnelStateWrapper,
};
use crate::firewall::FirewallPolicy;
#[cfg(windows)]
use crate::split_tunnel;
use futures::StreamExt;
#[cfg(windows)]
use std::ffi::OsStr;
use talpid_types::ErrorExt;

/// No tunnel is running.
pub struct DisconnectedState;

impl DisconnectedState {
    fn set_firewall_policy(
        shared_values: &mut SharedTunnelStateValues,
        should_reset_firewall: bool,
    ) {
        let result = if shared_values.block_when_disconnected {
            let policy = FirewallPolicy::Blocked {
                allow_lan: shared_values.allow_lan,
                allowed_endpoint: shared_values.allowed_endpoint.clone(),
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
    fn apply_split_tunnel_config<T: AsRef<OsStr>>(
        shared_values: &SharedTunnelStateValues,
        paths: &[T],
    ) -> Result<(), split_tunnel::Error> {
        let split_tunnel = shared_values
            .split_tunnel
            .lock()
            .expect("Thread unexpectedly panicked while holding the mutex");
        split_tunnel.set_paths(paths)
    }
}

impl TunnelState for DisconnectedState {
    type Bootstrap = bool;

    fn enter(
        shared_values: &mut SharedTunnelStateValues,
        should_reset_firewall: Self::Bootstrap,
    ) -> (TunnelStateWrapper, TunnelStateTransition) {
        Self::set_firewall_policy(shared_values, should_reset_firewall);
        #[cfg(target_os = "linux")]
        shared_values.reset_connectivity_check();
        #[cfg(target_os = "android")]
        shared_values.tun_provider.close_tun();

        (
            TunnelStateWrapper::from(DisconnectedState),
            TunnelStateTransition::Disconnected,
        )
    }

    fn handle_event(
        self,
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

                    Self::set_firewall_policy(shared_values, true);
                }
                SameState(self.into())
            }
            Some(TunnelCommand::AllowEndpoint(endpoint, tx)) => {
                if shared_values.set_allowed_endpoint(endpoint) {
                    Self::set_firewall_policy(shared_values, true);
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
                    Self::set_firewall_policy(shared_values, true);
                }
                SameState(self.into())
            }
            Some(TunnelCommand::IsOffline(is_offline)) => {
                shared_values.is_offline = is_offline;
                SameState(self.into())
            }
            Some(TunnelCommand::Connect) => NewState(ConnectingState::enter(shared_values, 0)),
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
                let _ = result_tx.send(Self::apply_split_tunnel_config(shared_values, &paths));
                SameState(self.into())
            }
            Some(_) => SameState(self.into()),
            None => Finished,
        }
    }
}

use super::{
    ConnectingState, ErrorState, EventConsequence, SharedTunnelStateValues, TunnelCommand,
    TunnelState, TunnelStateTransition, TunnelStateWrapper,
};
use crate::firewall::FirewallPolicy;
use futures::{sync::mpsc, Stream};
use talpid_types::ErrorExt;

/// No tunnel is running.
pub struct DisconnectedState;

impl DisconnectedState {
    fn set_firewall_policy(shared_values: &mut SharedTunnelStateValues) {
        let result = if shared_values.block_when_disconnected {
            let policy = FirewallPolicy::Blocked {
                allow_lan: shared_values.allow_lan,
            };
            shared_values.firewall.apply_policy(policy).map_err(|e| {
                e.display_chain_with_msg(
                    "Failed to apply blocking firewall policy for disconnected state",
                )
            })
        } else {
            shared_values
                .firewall
                .reset_policy()
                .map_err(|e| e.display_chain_with_msg("Failed to reset firewall policy"))
        };
        if let Err(error_chain) = result {
            log::error!("{}", error_chain);
        }
    }
}

impl TunnelState for DisconnectedState {
    type Bootstrap = ();

    fn enter(
        shared_values: &mut SharedTunnelStateValues,
        _: Self::Bootstrap,
    ) -> (TunnelStateWrapper, TunnelStateTransition) {
        if let Err(error) = shared_values.split_tunnel.disable_routing() {
            log::error!(
                "{}",
                error.display_chain_with_msg("Failed to update routing")
            );
        }

        Self::set_firewall_policy(shared_values);
        #[cfg(target_os = "android")]
        shared_values.tun_provider.close_tun();

        (
            TunnelStateWrapper::from(DisconnectedState),
            TunnelStateTransition::Disconnected,
        )
    }

    fn handle_event(
        self,
        commands: &mut mpsc::UnboundedReceiver<TunnelCommand>,
        shared_values: &mut SharedTunnelStateValues,
    ) -> EventConsequence<Self> {
        use self::EventConsequence::*;

        match try_handle_event!(self, commands.poll()) {
            Ok(TunnelCommand::AllowLan(allow_lan)) => {
                if shared_values.allow_lan != allow_lan {
                    // The only platform that can fail is Android, but Android doesn't support the
                    // "block when disconnected" option, so the following call never fails.
                    shared_values
                        .set_allow_lan(allow_lan)
                        .expect("Failed to set allow LAN parameter");

                    Self::set_firewall_policy(shared_values);
                }
                SameState(self)
            }
            Ok(TunnelCommand::BlockWhenDisconnected(block_when_disconnected)) => {
                if shared_values.block_when_disconnected != block_when_disconnected {
                    shared_values.block_when_disconnected = block_when_disconnected;
                    Self::set_firewall_policy(shared_values);
                }
                SameState(self)
            }
            Ok(TunnelCommand::IsOffline(is_offline)) => {
                shared_values.is_offline = is_offline;
                SameState(self)
            }
            Ok(TunnelCommand::Connect) => NewState(ConnectingState::enter(shared_values, 0)),
            Ok(TunnelCommand::Block(reason)) => NewState(ErrorState::enter(shared_values, reason)),
            Ok(_) => SameState(self),
            Err(_) => Finished,
        }
    }
}

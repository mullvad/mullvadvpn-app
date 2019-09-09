use super::{
    ConnectingState, DisconnectedState, EventConsequence, SharedTunnelStateValues, TunnelCommand,
    TunnelState, TunnelStateTransition, TunnelStateWrapper,
};
use crate::firewall::FirewallPolicy;
use futures::{sync::mpsc, Stream};
use talpid_types::{tunnel::BlockReason, ErrorExt};

/// No tunnel is running and all network connections are blocked.
pub struct BlockedState {
    block_reason: BlockReason,
}

impl BlockedState {
    fn set_firewall_policy(shared_values: &mut SharedTunnelStateValues) -> Option<BlockReason> {
        let policy = FirewallPolicy::Blocked {
            allow_lan: shared_values.allow_lan,
        };

        match shared_values.firewall.apply_policy(policy) {
            Ok(()) => None,
            Err(error) => {
                log::error!(
                    "{}",
                    error.display_chain_with_msg(
                        "Failed to apply firewall policy for blocked state"
                    )
                );
                Some(BlockReason::SetFirewallPolicyError)
            }
        }
    }

    #[cfg(target_os = "android")]
    fn create_blocking_tun(shared_values: &mut SharedTunnelStateValues) -> Option<BlockReason> {
        match shared_values.tun_provider.create_tun_if_closed() {
            Ok(()) => None,
            Err(error) => {
                log::error!(
                    "{}",
                    error.display_chain_with_msg(
                        "Failed to open tunnel adapter to drop packets for blocked state"
                    )
                );
                Some(BlockReason::SetFirewallPolicyError)
            }
        }
    }
}

impl TunnelState for BlockedState {
    type Bootstrap = BlockReason;

    fn enter(
        shared_values: &mut SharedTunnelStateValues,
        block_reason: Self::Bootstrap,
    ) -> (TunnelStateWrapper, TunnelStateTransition) {
        let block_reason = Self::set_firewall_policy(shared_values).unwrap_or_else(|| block_reason);
        #[cfg(target_os = "android")]
        let block_reason = Self::create_blocking_tun(shared_values).unwrap_or_else(|| block_reason);

        (
            TunnelStateWrapper::from(BlockedState {
                block_reason: block_reason.clone(),
            }),
            TunnelStateTransition::Blocked(block_reason),
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
                shared_values.allow_lan = allow_lan;
                Self::set_firewall_policy(shared_values);
                SameState(self)
            }
            Ok(TunnelCommand::BlockWhenDisconnected(block_when_disconnected)) => {
                shared_values.block_when_disconnected = block_when_disconnected;
                SameState(self)
            }
            Ok(TunnelCommand::IsOffline(is_offline)) => {
                shared_values.is_offline = is_offline;
                if !is_offline && self.block_reason == BlockReason::IsOffline {
                    NewState(ConnectingState::enter(shared_values, 0))
                } else {
                    SameState(self)
                }
            }
            Ok(TunnelCommand::Connect) => NewState(ConnectingState::enter(shared_values, 0)),
            Ok(TunnelCommand::Disconnect) | Err(_) => {
                NewState(DisconnectedState::enter(shared_values, ()))
            }
            Ok(TunnelCommand::Block(reason)) => {
                NewState(BlockedState::enter(shared_values, reason))
            }
        }
    }
}

use error_chain::ChainedError;
use futures::sync::mpsc;
use futures::Stream;
use talpid_types::tunnel::BlockReason;

use super::{
    ConnectingState, DisconnectedState, EventConsequence, ResultExt, SharedTunnelStateValues,
    TunnelCommand, TunnelState, TunnelStateTransition, TunnelStateWrapper,
};
use security::SecurityPolicy;

/// No tunnel is running and all network connections are blocked.
pub struct BlockedState;

impl BlockedState {
    fn set_security_policy(shared_values: &mut SharedTunnelStateValues) {
        let policy = SecurityPolicy::Blocked {
            allow_lan: shared_values.allow_lan,
        };
        if let Err(error) = shared_values
            .security
            .apply_policy(policy)
            .chain_err(|| "Failed to apply security policy for blocked state")
        {
            log::error!("{}", error.display_chain());
        }
    }
}

impl TunnelState for BlockedState {
    type Bootstrap = BlockReason;

    fn enter(
        shared_values: &mut SharedTunnelStateValues,
        block_reason: Self::Bootstrap,
    ) -> (TunnelStateWrapper, TunnelStateTransition) {
        Self::set_security_policy(shared_values);
        (
            TunnelStateWrapper::from(BlockedState),
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
                Self::set_security_policy(shared_values);
                SameState(self)
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

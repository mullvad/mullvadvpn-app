use error_chain::ChainedError;
use futures::sync::mpsc;
use futures::Stream;

use talpid_types::tunnel::BlockReason;

use super::{
    ConnectingState, DisconnectedState, EventConsequence, ResultExt, SharedTunnelStateValues,
    TunnelCommand, TunnelState, TunnelStateTransition, TunnelStateWrapper,
};
use security::{NetworkSecurity, SecurityPolicy};

/// No tunnel is running and all network connections are blocked.
pub struct BlockedState;

impl BlockedState {
    fn set_security_policy(shared_values: &mut SharedTunnelStateValues, allow_lan: bool) {
        let policy = SecurityPolicy::Blocked { allow_lan };
        debug!("Setting security policy: {:?}", policy);
        if let Err(error) = shared_values
            .security
            .apply_policy(policy)
            .chain_err(|| "Failed to apply security policy for blocked state")
        {
            error!("{}", error.display_chain());
        }
    }
}

impl TunnelState for BlockedState {
    type Bootstrap = (BlockReason, bool);

    fn enter(
        shared_values: &mut SharedTunnelStateValues,
        (block_reason, allow_lan): Self::Bootstrap,
    ) -> (TunnelStateWrapper, TunnelStateTransition) {
        Self::set_security_policy(shared_values, allow_lan);
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
                Self::set_security_policy(shared_values, allow_lan);
                SameState(self)
            }
            Ok(TunnelCommand::Connect(parameters)) => {
                NewState(ConnectingState::enter(shared_values, parameters))
            }
            Ok(TunnelCommand::Disconnect) | Err(_) => {
                NewState(DisconnectedState::enter(shared_values, ()))
            }
            Ok(TunnelCommand::Block(reason, allow_lan)) => {
                NewState(BlockedState::enter(shared_values, (reason, allow_lan)))
            }
        }
    }
}

use futures::sync::mpsc;
use futures::Stream;

use talpid_types::tunnel::BlockReason;

use super::{
    ConnectingState, DisconnectedState, EventConsequence, SharedTunnelStateValues, TunnelCommand,
    TunnelState, TunnelStateTransition, TunnelStateWrapper,
};

/// No tunnel is running and all network connections are blocked.
pub struct BlockedState;

impl TunnelState for BlockedState {
    type Bootstrap = BlockReason;

    fn enter(
        _: &mut SharedTunnelStateValues,
        block_reason: Self::Bootstrap,
    ) -> (TunnelStateWrapper, TunnelStateTransition) {
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
            Ok(TunnelCommand::Connect(parameters)) => {
                NewState(ConnectingState::enter(shared_values, parameters))
            }
            Ok(TunnelCommand::Disconnect) | Err(_) => {
                NewState(DisconnectedState::enter(shared_values, ()))
            }
            _ => SameState(self),
        }
    }
}

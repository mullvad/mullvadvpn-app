use futures::sync::mpsc;
use futures::Stream;

use super::{
    ConnectingState, DisconnectedState, EventConsequence, SharedTunnelStateValues, TunnelCommand,
    TunnelState, TunnelStateTransition, TunnelStateWrapper,
};

mod block_cause {
    error_chain!{}
}

pub use self::block_cause::Error as BlockCause;
pub use self::block_cause::ErrorKind as BlockReason;

/// No tunnel is running.
pub struct BlockedState;

impl TunnelState for BlockedState {
    type Bootstrap = BlockCause;

    fn enter(
        _: &mut SharedTunnelStateValues,
        block_cause: Self::Bootstrap,
    ) -> (TunnelStateWrapper, TunnelStateTransition) {
        (
            TunnelStateWrapper::from(BlockedState),
            TunnelStateTransition::Blocked(block_cause),
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

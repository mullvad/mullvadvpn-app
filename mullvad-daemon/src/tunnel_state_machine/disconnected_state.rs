use futures::sync::mpsc;
use futures::Stream;

use super::{
    ConnectingState, EventConsequence, StateEntryResult, TunnelCommand, TunnelState,
    TunnelStateWrapper,
};

/// No tunnel is running.
pub struct DisconnectedState;

impl TunnelState for DisconnectedState {
    type Bootstrap = ();

    fn enter(_: Self::Bootstrap) -> StateEntryResult {
        Ok(TunnelStateWrapper::from(DisconnectedState))
    }

    fn handle_event(
        self,
        commands: &mut mpsc::UnboundedReceiver<TunnelCommand>,
    ) -> EventConsequence<Self> {
        use self::EventConsequence::*;

        match try_handle_event!(self, commands.poll()) {
            Ok(TunnelCommand::Connect(parameters)) => NewState(ConnectingState::enter(parameters)),
            Ok(TunnelCommand::Disconnect) | Err(_) => SameState(self),
        }
    }
}

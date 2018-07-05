use futures::future::Shared;
use futures::sync::{mpsc, oneshot};
use futures::{Async, Future, Stream};

use super::{
    DisconnectedState, EventConsequence, ReconnectingState, SharedTunnelStateValues,
    StateEntryResult, TunnelCommand, TunnelState, TunnelStateWrapper,
};

/// This state is active from when we manually trigger a tunnel kill until the tunnel wait
/// operation (TunnelExit) returned.
pub struct DisconnectingState {
    exited: Shared<oneshot::Receiver<()>>,
}

impl DisconnectingState {
    fn handle_commands(
        self,
        commands: &mut mpsc::UnboundedReceiver<TunnelCommand>,
        shared_values: &mut SharedTunnelStateValues,
    ) -> EventConsequence<Self> {
        use self::EventConsequence::*;

        match try_handle_event!(self, commands.poll()) {
            Ok(TunnelCommand::Connect(parameters)) => NewState(ReconnectingState::enter(
                shared_values,
                (self.exited, parameters),
            )),
            _ => SameState(self),
        }
    }

    fn handle_exit_event(
        mut self,
        shared_values: &mut SharedTunnelStateValues,
    ) -> EventConsequence<Self> {
        use self::EventConsequence::*;

        match self.exited.poll() {
            Ok(Async::NotReady) => NoEvents(self),
            Ok(Async::Ready(_)) | Err(_) => NewState(DisconnectedState::enter(shared_values, ())),
        }
    }
}

impl TunnelState for DisconnectingState {
    type Bootstrap = Shared<oneshot::Receiver<()>>;

    fn enter(_: &mut SharedTunnelStateValues, exited: Self::Bootstrap) -> StateEntryResult {
        Ok(TunnelStateWrapper::from(DisconnectingState { exited }))
    }

    fn handle_event(
        self,
        commands: &mut mpsc::UnboundedReceiver<TunnelCommand>,
        shared_values: &mut SharedTunnelStateValues,
    ) -> EventConsequence<Self> {
        self.handle_commands(commands, shared_values)
            .or_else(Self::handle_exit_event, shared_values)
    }
}

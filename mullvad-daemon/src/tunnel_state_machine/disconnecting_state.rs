use std::io;

use futures::sync::{mpsc, oneshot};
use futures::{Async, Future, Stream};

use super::{
    DisconnectedState, EventConsequence, ReconnectingState, StateEntryResult, TunnelCommand,
    TunnelState, TunnelStateWrapper,
};

/// This state is active from when we manually trigger a tunnel kill until the tunnel wait
/// operation (TunnelExit) returned.
pub struct DisconnectingState {
    exited: oneshot::Receiver<io::Result<()>>,
}

impl DisconnectingState {
    fn handle_commands(
        self,
        commands: &mut mpsc::UnboundedReceiver<TunnelCommand>,
    ) -> EventConsequence<Self> {
        use self::EventConsequence::*;

        match try_handle_event!(self, commands.poll()) {
            Ok(TunnelCommand::Connect(parameters)) => {
                NewState(ReconnectingState::enter((self.exited, parameters)))
            }
            _ => SameState(self),
        }
    }

    fn handle_exit_event(mut self) -> EventConsequence<Self> {
        use self::EventConsequence::*;

        match self.exited.poll() {
            Ok(Async::NotReady) => NoEvents(self),
            Ok(Async::Ready(_)) | Err(_) => NewState(DisconnectedState::enter(())),
        }
    }
}

impl TunnelState for DisconnectingState {
    type Bootstrap = oneshot::Receiver<io::Result<()>>;

    fn enter(exited: Self::Bootstrap) -> StateEntryResult {
        Ok(TunnelStateWrapper::from(DisconnectingState { exited }))
    }

    fn handle_event(
        self,
        commands: &mut mpsc::UnboundedReceiver<TunnelCommand>,
    ) -> EventConsequence<Self> {
        self.handle_commands(commands)
            .or_else(Self::handle_exit_event)
    }
}

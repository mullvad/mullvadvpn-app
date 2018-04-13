use std::io;

use futures::sync::{mpsc, oneshot};
use futures::{Async, Future, Stream};

use super::{
    ConnectingState, DisconnectingState, EventConsequence, StateEntryResult, TunnelCommand,
    TunnelParameters, TunnelState, TunnelStateWrapper,
};

/// This state is active when the tunnel is being closed but will be reopened shortly afterwards.
pub struct ReconnectingState {
    exited: oneshot::Receiver<io::Result<()>>,
    parameters: TunnelParameters,
}

impl ReconnectingState {
    fn handle_commands(
        self,
        commands: &mut mpsc::UnboundedReceiver<TunnelCommand>,
    ) -> EventConsequence<Self> {
        use self::EventConsequence::*;

        match try_handle_event!(self, commands.poll()) {
            Ok(TunnelCommand::Connect(_)) => SameState(self),
            Ok(TunnelCommand::Disconnect) | Err(_) => {
                NewState(DisconnectingState::enter(self.exited))
            }
        }
    }

    fn handle_exit_event(mut self) -> EventConsequence<Self> {
        use self::EventConsequence::*;

        match self.exited.poll() {
            Ok(Async::NotReady) => NoEvents(self),
            Ok(Async::Ready(_)) | Err(_) => NewState(ConnectingState::enter(self.parameters)),
        }
    }
}

impl TunnelState for ReconnectingState {
    type Bootstrap = (oneshot::Receiver<io::Result<()>>, TunnelParameters);

    fn enter((exited, parameters): Self::Bootstrap) -> StateEntryResult {
        Ok(TunnelStateWrapper::from(ReconnectingState {
            exited,
            parameters,
        }))
    }

    fn handle_event(
        self,
        commands: &mut mpsc::UnboundedReceiver<TunnelCommand>,
    ) -> EventConsequence<Self> {
        self.handle_commands(commands)
            .or_else(Self::handle_exit_event)
    }
}

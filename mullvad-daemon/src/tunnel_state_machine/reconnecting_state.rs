use futures::future::Shared;
use futures::sync::{mpsc, oneshot};
use futures::{Async, Future, Stream};

use super::{
    ConnectingState, DisconnectingState, EventConsequence, SharedTunnelStateValues,
    StateEntryResult, TunnelCommand, TunnelParameters, TunnelState, TunnelStateWrapper,
};

/// This state is active when the tunnel is being closed but will be reopened shortly afterwards.
pub struct ReconnectingState {
    exited: Shared<oneshot::Receiver<()>>,
    parameters: TunnelParameters,
}

impl ReconnectingState {
    fn handle_commands(
        mut self,
        commands: &mut mpsc::UnboundedReceiver<TunnelCommand>,
        shared_values: &mut SharedTunnelStateValues,
    ) -> EventConsequence<Self> {
        use self::EventConsequence::*;

        match try_handle_event!(self, commands.poll()) {
            Ok(TunnelCommand::Connect(parameters)) | Ok(TunnelCommand::Reconnect(parameters)) => {
                self.parameters = parameters;
                SameState(self)
            }
            Ok(TunnelCommand::Disconnect) | Err(_) => {
                NewState(DisconnectingState::enter(shared_values, self.exited))
            }
            Ok(TunnelCommand::AllowLan(allow_lan)) => {
                self.parameters.allow_lan = allow_lan;
                SameState(self)
            }
        }
    }

    fn handle_exit_event(
        mut self,
        shared_values: &mut SharedTunnelStateValues,
    ) -> EventConsequence<Self> {
        use self::EventConsequence::*;

        match self.exited.poll() {
            Ok(Async::NotReady) => NoEvents(self),
            Ok(Async::Ready(_)) | Err(_) => {
                NewState(ConnectingState::enter(shared_values, self.parameters))
            }
        }
    }
}

impl TunnelState for ReconnectingState {
    type Bootstrap = (Shared<oneshot::Receiver<()>>, TunnelParameters);

    fn enter(
        _: &mut SharedTunnelStateValues,
        (exited, parameters): Self::Bootstrap,
    ) -> StateEntryResult {
        Ok(TunnelStateWrapper::from(ReconnectingState {
            exited,
            parameters,
        }))
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

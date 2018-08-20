use error_chain::ChainedError;
use futures::sync::{mpsc, oneshot};
use futures::{Async, Future, Stream};

use talpid_core::tunnel::CloseHandle;

use super::{
    ConnectingState, DisconnectedState, EventConsequence, ResultExt, StateEntryResult,
    TunnelCommand, TunnelParameters, TunnelState, TunnelStateWrapper,
};

/// This state is active from when we manually trigger a tunnel kill until the tunnel wait
/// operation (TunnelExit) returned.
pub struct DisconnectingState {
    exited: oneshot::Receiver<()>,
    after_disconnect: AfterDisconnect,
}

impl DisconnectingState {
    fn handle_commands(
        mut self,
        commands: &mut mpsc::UnboundedReceiver<TunnelCommand>,
    ) -> EventConsequence<Self> {
        use self::AfterDisconnect::*;

        self.after_disconnect = match try_handle_event!(self, commands.poll()) {
            Ok(TunnelCommand::Connect(parameters)) => Reconnect(parameters),
            Ok(TunnelCommand::Disconnect) | Err(_) => Nothing,
        };

        EventConsequence::SameState(self)
    }

    fn handle_exit_event(mut self) -> EventConsequence<Self> {
        use self::EventConsequence::*;

        match self.exited.poll() {
            Ok(Async::NotReady) => NoEvents(self),
            Ok(Async::Ready(_)) | Err(_) => NewState(self.after_disconnect()),
        }
    }

    fn after_disconnect(self) -> StateEntryResult {
        match self.after_disconnect {
            AfterDisconnect::Nothing => DisconnectedState::enter(()),
            AfterDisconnect::Reconnect(tunnel_parameters) => {
                ConnectingState::enter(tunnel_parameters)
            }
        }
    }
}

impl TunnelState for DisconnectingState {
    type Bootstrap = (CloseHandle, oneshot::Receiver<()>, AfterDisconnect);

    fn enter((close_handle, exited, after_disconnect): Self::Bootstrap) -> StateEntryResult {
        let close_result = close_handle
            .close()
            .chain_err(|| "Failed to request tunnel monitor to close the tunnel");

        if let Err(error) = close_result {
            error!("{}", error.display_chain());
        }

        Ok(TunnelStateWrapper::from(DisconnectingState {
            exited,
            after_disconnect,
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

/// Which state should be transitioned to after disconnection is complete.
pub enum AfterDisconnect {
    Nothing,
    Reconnect(TunnelParameters),
}

use futures::sync::mpsc;
use futures::Stream;

use talpid_core::tunnel::TunnelEvent;

use super::{
    AfterDisconnect, CloseHandle, DisconnectingState, EventConsequence, StateEntryResult,
    TunnelCommand, TunnelState, TunnelStateWrapper,
};

pub struct ConnectedStateBootstrap {
    pub tunnel_events: mpsc::UnboundedReceiver<TunnelEvent>,
    pub close_handle: CloseHandle,
}

/// The tunnel is up and working.
pub struct ConnectedState {
    close_handle: CloseHandle,
    tunnel_events: mpsc::UnboundedReceiver<TunnelEvent>,
}

impl ConnectedState {
    fn from(bootstrap: ConnectedStateBootstrap) -> Self {
        ConnectedState {
            close_handle: bootstrap.close_handle,
            tunnel_events: bootstrap.tunnel_events,
        }
    }

    fn handle_commands(
        self,
        commands: &mut mpsc::UnboundedReceiver<TunnelCommand>,
    ) -> EventConsequence<Self> {
        use self::EventConsequence::*;

        match try_handle_event!(self, commands.poll()) {
            Ok(TunnelCommand::Connect(_)) => SameState(self),
            Ok(TunnelCommand::Disconnect) | Err(_) => NewState(DisconnectingState::enter((
                self.close_handle.close(),
                AfterDisconnect::Nothing,
            ))),
        }
    }

    fn handle_tunnel_events(mut self) -> EventConsequence<Self> {
        use self::EventConsequence::*;

        match try_handle_event!(self, self.tunnel_events.poll()) {
            Ok(TunnelEvent::Down) | Err(_) => NewState(DisconnectingState::enter((
                self.close_handle.close(),
                AfterDisconnect::Nothing,
            ))),
            Ok(_) => SameState(self),
        }
    }
}

impl TunnelState for ConnectedState {
    type Bootstrap = ConnectedStateBootstrap;

    fn enter(bootstrap: Self::Bootstrap) -> StateEntryResult {
        Ok(TunnelStateWrapper::from(ConnectedState::from(bootstrap)))
    }

    fn handle_event(
        self,
        commands: &mut mpsc::UnboundedReceiver<TunnelCommand>,
    ) -> EventConsequence<Self> {
        self.handle_commands(commands)
            .or_else(Self::handle_tunnel_events)
    }
}

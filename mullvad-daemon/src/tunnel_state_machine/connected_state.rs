use futures::sync::{mpsc, oneshot};
use futures::{Async, Future, Stream};

use talpid_core::tunnel::{CloseHandle, TunnelEvent, TunnelMetadata};
use talpid_types::net::TunnelEndpoint;

use super::{
    AfterDisconnect, ConnectingState, DisconnectingState, EventConsequence, StateEntryResult,
    TunnelCommand, TunnelParameters, TunnelState, TunnelStateTransition, TunnelStateWrapper,
};

pub struct ConnectedStateBootstrap {
    pub metadata: TunnelMetadata,
    pub tunnel_events: mpsc::UnboundedReceiver<TunnelEvent>,
    pub tunnel_endpoint: TunnelEndpoint,
    pub tunnel_parameters: TunnelParameters,
    pub tunnel_close_event: oneshot::Receiver<()>,
    pub close_handle: CloseHandle,
}

/// The tunnel is up and working.
pub struct ConnectedState {
    metadata: TunnelMetadata,
    tunnel_events: mpsc::UnboundedReceiver<TunnelEvent>,
    tunnel_endpoint: TunnelEndpoint,
    tunnel_parameters: TunnelParameters,
    tunnel_close_event: oneshot::Receiver<()>,
    close_handle: CloseHandle,
}

impl ConnectedState {
    fn from(bootstrap: ConnectedStateBootstrap) -> Self {
        ConnectedState {
            metadata: bootstrap.metadata,
            tunnel_events: bootstrap.tunnel_events,
            tunnel_endpoint: bootstrap.tunnel_endpoint,
            tunnel_parameters: bootstrap.tunnel_parameters,
            tunnel_close_event: bootstrap.tunnel_close_event,
            close_handle: bootstrap.close_handle,
        }
    }

    pub fn info(&self) -> TunnelStateTransition {
        TunnelStateTransition::Connected(self.tunnel_endpoint, self.metadata.clone())
    }

    fn handle_commands(
        self,
        commands: &mut mpsc::UnboundedReceiver<TunnelCommand>,
    ) -> EventConsequence<Self> {
        use self::EventConsequence::*;

        match try_handle_event!(self, commands.poll()) {
            Ok(TunnelCommand::Connect(parameters)) => {
                if parameters != self.tunnel_parameters {
                    NewState(DisconnectingState::enter((
                        self.close_handle,
                        self.tunnel_close_event,
                        AfterDisconnect::Reconnect(parameters),
                    )))
                } else {
                    SameState(self)
                }
            }
            Ok(TunnelCommand::Disconnect) | Err(_) => NewState(DisconnectingState::enter((
                self.close_handle,
                self.tunnel_close_event,
                AfterDisconnect::Nothing,
            ))),
        }
    }

    fn handle_tunnel_events(mut self) -> EventConsequence<Self> {
        use self::EventConsequence::*;

        match try_handle_event!(self, self.tunnel_events.poll()) {
            Ok(TunnelEvent::Down) | Err(_) => NewState(DisconnectingState::enter((
                self.close_handle,
                self.tunnel_close_event,
                AfterDisconnect::Reconnect(self.tunnel_parameters),
            ))),
            Ok(_) => SameState(self),
        }
    }

    fn handle_tunnel_close_event(mut self) -> EventConsequence<Self> {
        use self::EventConsequence::*;

        match self.tunnel_close_event.poll() {
            Ok(Async::Ready(_)) => {}
            Ok(Async::NotReady) => return NoEvents(self),
            Err(_cancelled) => warn!("Tunnel monitor thread has stopped unexpectedly"),
        }

        info!("Tunnel closed. Reconnecting.");
        NewState(ConnectingState::enter(self.tunnel_parameters))
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
            .or_else(Self::handle_tunnel_close_event)
    }
}

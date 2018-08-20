use futures::future::Shared;
use futures::sync::{mpsc, oneshot};
use futures::{Async, Future, Stream};

use talpid_core::firewall::{Firewall, SecurityPolicy};
use talpid_core::tunnel::{TunnelEvent, TunnelMetadata};
use talpid_types::net::TunnelEndpoint;

use super::{
    CloseHandle, ConnectingState, DisconnectingState, EventConsequence, ReconnectingState, Result,
    ResultExt, SharedTunnelStateValues, StateEntryResult, TunnelCommand, TunnelParameters,
    TunnelState, TunnelStateWrapper,
};

pub struct ConnectedStateBootstrap {
    pub metadata: TunnelMetadata,
    pub tunnel_events: mpsc::UnboundedReceiver<TunnelEvent>,
    pub tunnel_endpoint: TunnelEndpoint,
    pub tunnel_parameters: TunnelParameters,
    pub tunnel_close_event: Shared<oneshot::Receiver<()>>,
    pub close_handle: CloseHandle,
}

/// The tunnel is up and working.
pub struct ConnectedState {
    metadata: TunnelMetadata,
    tunnel_events: mpsc::UnboundedReceiver<TunnelEvent>,
    tunnel_endpoint: TunnelEndpoint,
    tunnel_parameters: TunnelParameters,
    tunnel_close_event: Shared<oneshot::Receiver<()>>,
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

    fn set_security_policy(
        shared_values: &mut SharedTunnelStateValues,
        endpoint: TunnelEndpoint,
        metadata: TunnelMetadata,
        allow_lan: bool,
    ) -> Result<()> {
        let policy = SecurityPolicy::Connected {
            relay_endpoint: endpoint.to_endpoint(),
            tunnel: metadata,
            allow_lan,
        };

        debug!("Set security policy: {:?}", policy);
        shared_values
            .firewall
            .apply_policy(policy)
            .chain_err(|| "Failed to apply security policy for connected state")
    }

    fn handle_commands(
        mut self,
        commands: &mut mpsc::UnboundedReceiver<TunnelCommand>,
        shared_values: &mut SharedTunnelStateValues,
    ) -> EventConsequence<Self> {
        use self::EventConsequence::*;

        match try_handle_event!(self, commands.poll()) {
            Ok(TunnelCommand::Connect(parameters)) => {
                if parameters != self.tunnel_parameters {
                    NewState(ReconnectingState::enter(
                        shared_values,
                        (self.close_handle.close(), parameters),
                    ))
                } else {
                    SameState(self)
                }
            }
            Ok(TunnelCommand::Reconnect(parameters)) => NewState(ReconnectingState::enter(
                shared_values,
                (self.close_handle.close(), parameters),
            )),
            Ok(TunnelCommand::Disconnect) | Err(_) => NewState(DisconnectingState::enter(
                shared_values,
                self.close_handle.close(),
            )),
            Ok(TunnelCommand::AllowLan(allow_lan)) => {
                self.tunnel_parameters.allow_lan = allow_lan;

                let set_security_policy_result = Self::set_security_policy(
                    shared_values,
                    self.tunnel_endpoint,
                    self.metadata.clone(),
                    allow_lan,
                );

                match set_security_policy_result {
                    Ok(()) => SameState(self),
                    Err(error) => {
                        error!("{}", error.chain_err(|| "Failed to update security policy"));
                        NewState(DisconnectingState::enter(
                            shared_values,
                            self.close_handle.close(),
                        ))
                    }
                }
            }
        }
    }

    fn handle_tunnel_events(
        mut self,
        shared_values: &mut SharedTunnelStateValues,
    ) -> EventConsequence<Self> {
        use self::EventConsequence::*;

        match try_handle_event!(self, self.tunnel_events.poll()) {
            Ok(TunnelEvent::Down) | Err(_) => NewState(ReconnectingState::enter(
                shared_values,
                (self.close_handle.close(), self.tunnel_parameters),
            )),
            Ok(_) => SameState(self),
        }
    }

    fn handle_tunnel_close_event(
        mut self,
        shared_values: &mut SharedTunnelStateValues,
    ) -> EventConsequence<Self> {
        use self::EventConsequence::*;

        match self.tunnel_close_event.poll() {
            Ok(Async::Ready(_)) => {}
            Ok(Async::NotReady) => return NoEvents(self),
            Err(_cancelled) => warn!("Tunnel monitor thread has stopped unexpectedly"),
        }

        info!("Tunnel closed. Reconnecting.");
        NewState(ConnectingState::enter(
            shared_values,
            self.tunnel_parameters,
        ))
    }
}

impl TunnelState for ConnectedState {
    type Bootstrap = ConnectedStateBootstrap;

    fn enter(
        shared_values: &mut SharedTunnelStateValues,
        bootstrap: Self::Bootstrap,
    ) -> StateEntryResult {
        let endpoint = bootstrap.tunnel_endpoint;
        let metadata = bootstrap.metadata.clone();
        let allow_lan = bootstrap.tunnel_parameters.allow_lan;

        match Self::set_security_policy(shared_values, endpoint, metadata, allow_lan) {
            Ok(()) => Ok(TunnelStateWrapper::from(ConnectedState::from(bootstrap))),
            Err(error) => Err((
                error,
                DisconnectingState::enter(shared_values, bootstrap.close_handle.close())
                    .expect("Failed to disconnect after failed transition to connected state"),
            )),
        }
    }

    fn handle_event(
        self,
        commands: &mut mpsc::UnboundedReceiver<TunnelCommand>,
        shared_values: &mut SharedTunnelStateValues,
    ) -> EventConsequence<Self> {
        self.handle_commands(commands, shared_values)
            .or_else(Self::handle_tunnel_events, shared_values)
            .or_else(Self::handle_tunnel_close_event, shared_values)
    }
}

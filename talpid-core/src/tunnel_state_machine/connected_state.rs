use error_chain::ChainedError;
use futures::sync::{mpsc, oneshot};
use futures::{Async, Future, Stream};
use talpid_types::net::{Endpoint, OpenVpnProxySettings, TransportProtocol};
use talpid_types::tunnel::BlockReason;

use super::{
    AfterDisconnect, ConnectingState, DisconnectingState, EventConsequence, Result, ResultExt,
    SharedTunnelStateValues, TunnelCommand, TunnelParameters, TunnelState, TunnelStateTransition,
    TunnelStateWrapper,
};
use crate::{
    security::SecurityPolicy,
    tunnel::{CloseHandle, TunnelEvent, TunnelMetadata},
};

pub struct ConnectedStateBootstrap {
    pub metadata: TunnelMetadata,
    pub tunnel_events: mpsc::UnboundedReceiver<TunnelEvent>,
    pub tunnel_parameters: TunnelParameters,
    pub tunnel_close_event: oneshot::Receiver<()>,
    pub close_handle: CloseHandle,
}

/// The tunnel is up and working.
pub struct ConnectedState {
    metadata: TunnelMetadata,
    tunnel_events: mpsc::UnboundedReceiver<TunnelEvent>,
    tunnel_parameters: TunnelParameters,
    tunnel_close_event: oneshot::Receiver<()>,
    close_handle: CloseHandle,
}

impl ConnectedState {
    fn from(bootstrap: ConnectedStateBootstrap) -> Self {
        ConnectedState {
            metadata: bootstrap.metadata,
            tunnel_events: bootstrap.tunnel_events,
            tunnel_parameters: bootstrap.tunnel_parameters,
            tunnel_close_event: bootstrap.tunnel_close_event,
            close_handle: bootstrap.close_handle,
        }
    }

    fn set_security_policy(&self, shared_values: &mut SharedTunnelStateValues) -> Result<()> {
        // If a proxy is specified we need to pass it on as the peer endpoint.
        let peer_endpoint = match self.tunnel_parameters.options.openvpn.proxy {
            Some(OpenVpnProxySettings::Local(ref local_proxy)) => Endpoint {
                address: local_proxy.peer,
                protocol: TransportProtocol::Tcp,
            },
            Some(OpenVpnProxySettings::Remote(ref remote_proxy)) => Endpoint {
                address: remote_proxy.address,
                protocol: TransportProtocol::Tcp,
            },
            _ => self.tunnel_parameters.endpoint.to_endpoint(),
        };

        let policy = SecurityPolicy::Connected {
            peer_endpoint,
            tunnel: self.metadata.clone(),
            allow_lan: shared_values.allow_lan,
        };
        shared_values
            .security
            .apply_policy(policy)
            .chain_err(|| "Failed to apply security policy for connected state")
    }

    fn handle_commands(
        self,
        commands: &mut mpsc::UnboundedReceiver<TunnelCommand>,
        shared_values: &mut SharedTunnelStateValues,
    ) -> EventConsequence<Self> {
        use self::EventConsequence::*;

        match try_handle_event!(self, commands.poll()) {
            Ok(TunnelCommand::AllowLan(allow_lan)) => {
                shared_values.allow_lan = allow_lan;

                match self.set_security_policy(shared_values) {
                    Ok(()) => SameState(self),
                    Err(error) => {
                        log::error!("{}", error.display_chain());

                        NewState(DisconnectingState::enter(
                            shared_values,
                            (
                                self.close_handle,
                                self.tunnel_close_event,
                                AfterDisconnect::Block(BlockReason::SetSecurityPolicyError),
                            ),
                        ))
                    }
                }
            }
            Ok(TunnelCommand::BlockWhenDisconnected(block_when_disconnected)) => {
                shared_values.block_when_disconnected = block_when_disconnected;
                SameState(self)
            }
            Ok(TunnelCommand::IsOffline(is_offline)) => {
                shared_values.is_offline = is_offline;
                if is_offline {
                    NewState(DisconnectingState::enter(
                        shared_values,
                        (
                            self.close_handle,
                            self.tunnel_close_event,
                            AfterDisconnect::Block(BlockReason::IsOffline),
                        ),
                    ))
                } else {
                    SameState(self)
                }
            }
            Ok(TunnelCommand::Connect) => NewState(DisconnectingState::enter(
                shared_values,
                (
                    self.close_handle,
                    self.tunnel_close_event,
                    AfterDisconnect::Reconnect(0),
                ),
            )),
            Ok(TunnelCommand::Disconnect) | Err(_) => NewState(DisconnectingState::enter(
                shared_values,
                (
                    self.close_handle,
                    self.tunnel_close_event,
                    AfterDisconnect::Nothing,
                ),
            )),
            Ok(TunnelCommand::Block(reason)) => NewState(DisconnectingState::enter(
                shared_values,
                (
                    self.close_handle,
                    self.tunnel_close_event,
                    AfterDisconnect::Block(reason),
                ),
            )),
        }
    }

    fn handle_tunnel_events(
        mut self,
        shared_values: &mut SharedTunnelStateValues,
    ) -> EventConsequence<Self> {
        use self::EventConsequence::*;

        match try_handle_event!(self, self.tunnel_events.poll()) {
            Ok(TunnelEvent::Down) | Err(_) => NewState(DisconnectingState::enter(
                shared_values,
                (
                    self.close_handle,
                    self.tunnel_close_event,
                    AfterDisconnect::Reconnect(0),
                ),
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
            Err(_cancelled) => log::warn!("Tunnel monitor thread has stopped unexpectedly"),
        }

        log::info!("Tunnel closed. Reconnecting.");
        NewState(ConnectingState::enter(shared_values, 0))
    }
}

impl TunnelState for ConnectedState {
    type Bootstrap = ConnectedStateBootstrap;

    fn enter(
        shared_values: &mut SharedTunnelStateValues,
        bootstrap: Self::Bootstrap,
    ) -> (TunnelStateWrapper, TunnelStateTransition) {
        let tunnel_endpoint = bootstrap.tunnel_parameters.endpoint;
        let connected_state = ConnectedState::from(bootstrap);

        match connected_state.set_security_policy(shared_values) {
            Ok(()) => (
                TunnelStateWrapper::from(connected_state),
                TunnelStateTransition::Connected(tunnel_endpoint),
            ),
            Err(error) => {
                log::error!("{}", error.display_chain());

                DisconnectingState::enter(
                    shared_values,
                    (
                        connected_state.close_handle,
                        connected_state.tunnel_close_event,
                        AfterDisconnect::Block(BlockReason::SetSecurityPolicyError),
                    ),
                )
            }
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

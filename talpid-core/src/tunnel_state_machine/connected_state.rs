use error_chain::ChainedError;
use futures::{
    sync::{mpsc, oneshot},
    Async, Future, Stream,
};
use talpid_types::{
    net::{Endpoint, OpenVpnProxySettings, TransportProtocol},
    tunnel::BlockReason,
};

use super::{
    AfterDisconnect, BlockedState, ConnectingState, DisconnectingState, EventConsequence, Result,
    ResultExt, SharedTunnelStateValues, TunnelCommand, TunnelParameters, TunnelState,
    TunnelStateTransition, TunnelStateWrapper,
};
use crate::{
    security::SecurityPolicy,
    tunnel::{CloseHandle, TunnelEvent, TunnelMetadata},
};

pub struct ConnectedStateBootstrap {
    pub metadata: TunnelMetadata,
    pub tunnel_events: mpsc::UnboundedReceiver<TunnelEvent>,
    pub tunnel_parameters: TunnelParameters,
    pub tunnel_close_event: oneshot::Receiver<Option<BlockReason>>,
    pub close_handle: CloseHandle,
}

/// The tunnel is up and working.
pub struct ConnectedState {
    metadata: TunnelMetadata,
    tunnel_events: mpsc::UnboundedReceiver<TunnelEvent>,
    tunnel_parameters: TunnelParameters,
    tunnel_close_event: oneshot::Receiver<Option<BlockReason>>,
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

    fn set_dns(&self, shared_values: &mut SharedTunnelStateValues) -> Result<()> {
        shared_values
            .dns_monitor
            .set(&self.metadata.interface, &[self.metadata.gateway.into()])
            .chain_err(|| "Failed to set system DNS settings")
    }

    fn reset_dns(shared_values: &mut SharedTunnelStateValues) {
        if let Err(error) = shared_values
            .dns_monitor
            .reset()
            .chain_err(|| "Unable to reset DNS")
        {
            log::error!("{}", error.display_chain());
        }
    }

    fn disconnect(
        self,
        shared_values: &mut SharedTunnelStateValues,
        after_disconnect: AfterDisconnect,
    ) -> EventConsequence<Self> {
        Self::reset_dns(shared_values);
        EventConsequence::NewState(DisconnectingState::enter(
            shared_values,
            (self.close_handle, self.tunnel_close_event, after_disconnect),
        ))
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
                        self.disconnect(
                            shared_values,
                            AfterDisconnect::Block(BlockReason::SetSecurityPolicyError),
                        )
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
                    self.disconnect(
                        shared_values,
                        AfterDisconnect::Block(BlockReason::IsOffline),
                    )
                } else {
                    SameState(self)
                }
            }
            Ok(TunnelCommand::Connect) => {
                self.disconnect(shared_values, AfterDisconnect::Reconnect(0))
            }
            Ok(TunnelCommand::Disconnect) | Err(_) => {
                self.disconnect(shared_values, AfterDisconnect::Nothing)
            }
            Ok(TunnelCommand::Block(reason)) => {
                self.disconnect(shared_values, AfterDisconnect::Block(reason))
            }
        }
    }

    fn handle_tunnel_events(
        mut self,
        shared_values: &mut SharedTunnelStateValues,
    ) -> EventConsequence<Self> {
        use self::EventConsequence::*;

        match try_handle_event!(self, self.tunnel_events.poll()) {
            Ok(TunnelEvent::Down) | Err(_) => {
                self.disconnect(shared_values, AfterDisconnect::Reconnect(0))
            }
            Ok(_) => SameState(self),
        }
    }

    fn handle_tunnel_close_event(
        mut self,
        shared_values: &mut SharedTunnelStateValues,
    ) -> EventConsequence<Self> {
        use self::EventConsequence::*;

        match self.tunnel_close_event.poll() {
            Ok(Async::Ready(block_reason)) => {
                if let Some(reason) = block_reason {
                    return NewState(BlockedState::enter(shared_values, reason));
                }
            }
            Ok(Async::NotReady) => return NoEvents(self),
            Err(_cancelled) => log::warn!("Tunnel monitor thread has stopped unexpectedly"),
        }

        log::info!("Tunnel closed. Reconnecting.");
        Self::reset_dns(shared_values);
        NewState(ConnectingState::enter(shared_values, 0))
    }
}

impl TunnelState for ConnectedState {
    type Bootstrap = ConnectedStateBootstrap;

    fn enter(
        shared_values: &mut SharedTunnelStateValues,
        bootstrap: Self::Bootstrap,
    ) -> (TunnelStateWrapper, TunnelStateTransition) {
        let tunnel_endpoint = bootstrap.tunnel_parameters.endpoint.clone();
        let connected_state = ConnectedState::from(bootstrap);

        if let Err(error) = connected_state.set_security_policy(shared_values) {
            log::error!("{}", error.display_chain());
            DisconnectingState::enter(
                shared_values,
                (
                    connected_state.close_handle,
                    connected_state.tunnel_close_event,
                    AfterDisconnect::Block(BlockReason::SetSecurityPolicyError),
                ),
            )
        } else if let Err(error) = connected_state.set_dns(shared_values) {
            log::error!("{}", error.display_chain());
            DisconnectingState::enter(
                shared_values,
                (
                    connected_state.close_handle,
                    connected_state.tunnel_close_event,
                    AfterDisconnect::Block(BlockReason::SetDnsError),
                ),
            )
        } else {
            (
                TunnelStateWrapper::from(connected_state),
                TunnelStateTransition::Connected(tunnel_endpoint),
            )
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

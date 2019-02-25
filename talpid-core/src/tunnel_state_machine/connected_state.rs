use error_chain::ChainedError;
use futures::{
    sync::{mpsc, oneshot},
    Async, Future, Stream,
};
use talpid_types::{
    net::{Endpoint, TunnelParameters},
    tunnel::BlockReason,
};

use super::{
    AfterDisconnect, BlockedState, ConnectingState, DisconnectingState, EventConsequence, Result,
    ResultExt, SharedTunnelStateValues, TunnelCommand, TunnelState, TunnelStateTransition,
    TunnelStateWrapper,
};
use crate::{
    firewall::FirewallPolicy,
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

    fn set_firewall_policy(&self, shared_values: &mut SharedTunnelStateValues) -> Result<()> {
        // If a proxy is specified we need to pass it on as the peer endpoint.
        let peer_endpoint = self.get_endpoint_from_params();

        let policy = FirewallPolicy::Connected {
            peer_endpoint,
            tunnel: self.metadata.clone(),
            allow_lan: shared_values.allow_lan,
        };
        shared_values
            .firewall
            .apply_policy(policy)
            .chain_err(|| "Failed to apply firewall policy for connected state")
    }

    fn get_endpoint_from_params(&self) -> Endpoint {
        match self.tunnel_parameters {
            TunnelParameters::OpenVpn(ref config) => match config.options.proxy {
                Some(ref proxy_settings) => proxy_settings.get_endpoint(),
                None => self.tunnel_parameters.get_tunnel_endpoint().endpoint,
            },
            _ => self.tunnel_parameters.get_tunnel_endpoint().endpoint,
        }
    }

    fn set_dns(&self, shared_values: &mut SharedTunnelStateValues) -> Result<()> {
        let mut dns_ips = vec![self.metadata.ipv4_gateway.into()];
        if let Some(ipv6_gateway) = self.metadata.ipv6_gateway {
            dns_ips.push(ipv6_gateway.into());
        };

        shared_values
            .dns_monitor
            .set(&self.metadata.interface, &dns_ips)
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

                match self.set_firewall_policy(shared_values) {
                    Ok(()) => SameState(self),
                    Err(error) => {
                        log::error!("{}", error.display_chain());
                        self.disconnect(
                            shared_values,
                            AfterDisconnect::Block(BlockReason::SetFirewallPolicyError),
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
        let connected_state = ConnectedState::from(bootstrap);
        let tunnel_endpoint = connected_state.tunnel_parameters.get_tunnel_endpoint();

        if let Err(error) = connected_state.set_firewall_policy(shared_values) {
            log::error!("{}", error.display_chain());
            DisconnectingState::enter(
                shared_values,
                (
                    connected_state.close_handle,
                    connected_state.tunnel_close_event,
                    AfterDisconnect::Block(BlockReason::SetFirewallPolicyError),
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

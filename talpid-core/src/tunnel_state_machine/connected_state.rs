use super::{
    AfterDisconnect, ConnectingState, DisconnectingState, ErrorState, EventConsequence,
    SharedTunnelStateValues, TunnelCommand, TunnelState, TunnelStateTransition, TunnelStateWrapper,
};
use crate::{
    firewall::FirewallPolicy,
    tunnel::{CloseHandle, TunnelEvent, TunnelMetadata},
};
use futures::{
    sync::{mpsc, oneshot},
    Async, Future, Stream,
};
use talpid_types::{
    net::{Endpoint, TunnelParameters},
    tunnel::ErrorStateCause,
    BoxedError, ErrorExt,
};

pub struct ConnectedStateBootstrap {
    pub metadata: TunnelMetadata,
    pub tunnel_events: mpsc::UnboundedReceiver<TunnelEvent>,
    pub tunnel_parameters: TunnelParameters,
    pub tunnel_close_event: Option<oneshot::Receiver<Option<ErrorStateCause>>>,
    pub close_handle: Option<CloseHandle>,
}

/// The tunnel is up and working.
pub struct ConnectedState {
    metadata: TunnelMetadata,
    tunnel_events: mpsc::UnboundedReceiver<TunnelEvent>,
    tunnel_parameters: TunnelParameters,
    tunnel_close_event: Option<oneshot::Receiver<Option<ErrorStateCause>>>,
    close_handle: Option<CloseHandle>,
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

    fn set_firewall_policy(
        &self,
        shared_values: &mut SharedTunnelStateValues,
    ) -> Result<(), crate::firewall::Error> {
        // If a proxy is specified we need to pass it on as the peer endpoint.
        let peer_endpoint = self.get_endpoint_from_params();

        let policy = FirewallPolicy::Connected {
            peer_endpoint,
            tunnel: self.metadata.clone(),
            allow_lan: shared_values.allow_lan,
        };
        shared_values.firewall.apply_policy(policy)
    }

    fn get_endpoint_from_params(&self) -> Endpoint {
        match self.tunnel_parameters {
            TunnelParameters::OpenVpn(ref params) => match params.proxy {
                Some(ref proxy_settings) => proxy_settings.get_endpoint().endpoint,
                None => params.config.endpoint,
            },
            TunnelParameters::Wireguard(ref params) => params.connection.get_endpoint(),
        }
    }

    fn set_dns(&self, shared_values: &mut SharedTunnelStateValues) -> Result<(), BoxedError> {
        let mut dns_ips = vec![self.metadata.ipv4_gateway.into()];
        if let Some(ipv6_gateway) = self.metadata.ipv6_gateway {
            dns_ips.push(ipv6_gateway.into());
        };

        #[cfg(target_os = "linux")]
        {
            shared_values
                .split_tunnel
                .route_dns(&self.metadata.interface, &dns_ips)
                .map_err(BoxedError::new)?;
        }

        shared_values
            .dns_monitor
            .set(&self.metadata.interface, &dns_ips)
            .map_err(BoxedError::new)
    }

    fn reset_dns(shared_values: &mut SharedTunnelStateValues) {
        if let Err(error) = shared_values.dns_monitor.reset() {
            log::error!("{}", error.display_chain_with_msg("Unable to reset DNS"));
        }

        #[cfg(target_os = "linux")]
        {
            if let Err(error) = shared_values.split_tunnel.flush_dns() {
                log::error!(
                    "{}",
                    error.display_chain_with_msg("Unable to update split-tunnel route")
                );
            }
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
                if let Err(error_cause) = shared_values.set_allow_lan(allow_lan) {
                    self.disconnect(shared_values, AfterDisconnect::Block(error_cause))
                } else {
                    match self.set_firewall_policy(shared_values) {
                        Ok(()) => SameState(self),
                        Err(error) => {
                            log::error!(
                                "{}",
                                error.display_chain_with_msg(
                                    "Failed to apply firewall policy for connected state"
                                )
                            );
                            self.disconnect(
                                shared_values,
                                AfterDisconnect::Block(ErrorStateCause::SetFirewallPolicyError),
                            )
                        }
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
                        AfterDisconnect::Block(ErrorStateCause::IsOffline),
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

        let poll_result = match &mut self.tunnel_close_event {
            Some(tunnel_close_event) => tunnel_close_event.poll(),
            None => Ok(Async::NotReady),
        };

        match poll_result {
            Ok(Async::Ready(block_reason)) => {
                if let Some(reason) = block_reason {
                    return NewState(ErrorState::enter(shared_values, reason));
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

    #[cfg_attr(target_os = "android", allow(unused_variables))]
    fn enter(
        shared_values: &mut SharedTunnelStateValues,
        bootstrap: Self::Bootstrap,
    ) -> (TunnelStateWrapper, TunnelStateTransition) {
        let connected_state = ConnectedState::from(bootstrap);
        let tunnel_endpoint = connected_state.tunnel_parameters.get_tunnel_endpoint();

        if let Err(error) = connected_state.set_firewall_policy(shared_values) {
            log::error!(
                "{}",
                error.display_chain_with_msg("Failed to apply firewall policy for connected state")
            );
            DisconnectingState::enter(
                shared_values,
                (
                    connected_state.close_handle,
                    connected_state.tunnel_close_event,
                    AfterDisconnect::Block(ErrorStateCause::SetFirewallPolicyError),
                ),
            )
        } else if let Err(error) = connected_state.set_dns(shared_values) {
            log::error!("{}", error.display_chain_with_msg("Failed to set DNS"));
            DisconnectingState::enter(
                shared_values,
                (
                    connected_state.close_handle,
                    connected_state.tunnel_close_event,
                    AfterDisconnect::Block(ErrorStateCause::SetDnsError),
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

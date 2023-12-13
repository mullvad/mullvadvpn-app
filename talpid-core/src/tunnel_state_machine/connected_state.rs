use super::{
    AfterDisconnect, ConnectingState, DisconnectingState, ErrorState, EventConsequence,
    EventResult, SharedTunnelStateValues, TunnelCommand, TunnelCommandReceiver, TunnelState,
    TunnelStateTransition,
};
use crate::{
    firewall::FirewallPolicy,
    tunnel::{TunnelEvent, TunnelMetadata},
};
use futures::{
    channel::{mpsc, oneshot},
    stream::Fuse,
    StreamExt,
};
use std::net::IpAddr;
use talpid_types::{
    net::TunnelParameters,
    tunnel::{ErrorStateCause, FirewallPolicyError},
    BoxedError, ErrorExt,
};

#[cfg(windows)]
use crate::tunnel::TunnelMonitor;

use super::connecting_state::TunnelCloseEvent;

pub(crate) type TunnelEventsReceiver =
    Fuse<mpsc::UnboundedReceiver<(TunnelEvent, oneshot::Sender<()>)>>;

/// The tunnel is up and working.
pub struct ConnectedState {
    metadata: TunnelMetadata,
    tunnel_events: TunnelEventsReceiver,
    tunnel_parameters: TunnelParameters,
    tunnel_close_event: TunnelCloseEvent,
    tunnel_close_tx: oneshot::Sender<()>,
}

impl ConnectedState {
    #[cfg_attr(target_os = "android", allow(unused_variables))]
    pub(super) fn enter(
        shared_values: &mut SharedTunnelStateValues,
        metadata: TunnelMetadata,
        tunnel_events: TunnelEventsReceiver,
        tunnel_parameters: TunnelParameters,
        tunnel_close_event: TunnelCloseEvent,
        tunnel_close_tx: oneshot::Sender<()>,
    ) -> (Box<dyn TunnelState>, TunnelStateTransition) {
        let connected_state = ConnectedState {
            metadata,
            tunnel_events,
            tunnel_parameters,
            tunnel_close_event,
            tunnel_close_tx,
        };

        let tunnel_interface = Some(connected_state.metadata.interface.clone());
        let tunnel_endpoint = talpid_types::net::TunnelEndpoint {
            tunnel_interface,
            ..connected_state.tunnel_parameters.get_tunnel_endpoint()
        };

        if let Err(error) = connected_state.set_firewall_policy(shared_values) {
            DisconnectingState::enter(
                connected_state.tunnel_close_tx,
                connected_state.tunnel_close_event,
                AfterDisconnect::Block(ErrorStateCause::SetFirewallPolicyError(error)),
            )
        } else if let Err(error) = connected_state.set_dns(shared_values) {
            log::error!("{}", error.display_chain_with_msg("Failed to set DNS"));
            DisconnectingState::enter(
                connected_state.tunnel_close_tx,
                connected_state.tunnel_close_event,
                AfterDisconnect::Block(ErrorStateCause::SetDnsError),
            )
        } else {
            (
                Box::new(connected_state),
                TunnelStateTransition::Connected(tunnel_endpoint),
            )
        }
    }

    fn set_firewall_policy(
        &self,
        shared_values: &mut SharedTunnelStateValues,
    ) -> Result<(), FirewallPolicyError> {
        let policy = self.get_firewall_policy(shared_values);
        shared_values
            .firewall
            .apply_policy(policy)
            .map_err(|error| {
                log::error!(
                    "{}",
                    error.display_chain_with_msg(
                        "Failed to apply firewall policy for connected state"
                    )
                );
                #[cfg(windows)]
                match error {
                    crate::firewall::Error::ApplyingConnectedPolicy(policy_error) => policy_error,
                    _ => FirewallPolicyError::Generic,
                }
                #[cfg(not(windows))]
                FirewallPolicyError::Generic
            })
    }

    #[allow(unused_variables)]
    fn get_dns_servers(&self, shared_values: &SharedTunnelStateValues) -> Vec<IpAddr> {
        #[cfg(not(target_os = "android"))]
        if let Some(ref servers) = shared_values.dns_servers {
            servers.clone()
        } else {
            let mut dns_ips = vec![self.metadata.ipv4_gateway.into()];
            if let Some(ipv6_gateway) = self.metadata.ipv6_gateway {
                dns_ips.push(ipv6_gateway.into());
            };
            dns_ips
        }
        #[cfg(target_os = "android")]
        {
            let mut dns_ips = vec![];
            dns_ips.push(self.metadata.ipv4_gateway.into());
            if let Some(ipv6_gateway) = self.metadata.ipv6_gateway {
                dns_ips.push(ipv6_gateway.into());
            };
            dns_ips
        }
    }

    fn get_firewall_policy(&self, shared_values: &SharedTunnelStateValues) -> FirewallPolicy {
        let peer_endpoint = self.tunnel_parameters.get_next_hop_endpoint();
        let allow_all_traffic_to_peer = self.tunnel_parameters.get_openvpn_local_proxy_settings().is_some();

        FirewallPolicy::Connected {
            peer_endpoint,
            tunnel: self.metadata.clone(),
            allow_lan: shared_values.allow_lan,
            #[cfg(target_os = "linux")]
            allow_all_traffic_to_peer,
            #[cfg(not(target_os = "android"))]
            dns_servers: self.get_dns_servers(shared_values),
            #[cfg(windows)]
            relay_client: TunnelMonitor::get_relay_client(
                &shared_values.resource_dir,
                &self.tunnel_parameters,
            ),
        }
    }

    fn set_dns(&self, shared_values: &mut SharedTunnelStateValues) -> Result<(), BoxedError> {
        let dns_ips = self.get_dns_servers(shared_values);

        #[cfg(any(target_os = "linux", target_os = "windows"))]
        let dns_ips = dns_ips
            .into_iter()
            .filter(|ip| {
                !crate::firewall::is_local_address(ip)
                    || IpAddr::V4(self.metadata.ipv4_gateway) == *ip
                    || self.metadata.ipv6_gateway.map(IpAddr::V6) == Some(*ip)
            })
            .collect::<Vec<_>>();

        shared_values
            .dns_monitor
            .set(&self.metadata.interface, &dns_ips)
            .map_err(BoxedError::new)?;

        Ok(())
    }

    fn reset_dns(shared_values: &mut SharedTunnelStateValues) {
        if let Err(error) = shared_values.dns_monitor.reset_before_interface_removal() {
            log::error!("{}", error.display_chain_with_msg("Unable to reset DNS"));
        }
    }

    fn reset_routes(
        #[cfg(target_os = "windows")] shared_values: &SharedTunnelStateValues,
        #[cfg(not(target_os = "windows"))] shared_values: &mut SharedTunnelStateValues,
    ) {
        if let Err(error) = shared_values.route_manager.clear_routes() {
            log::error!("{}", error.display_chain_with_msg("Failed to clear routes"));
        }
        #[cfg(target_os = "linux")]
        if let Err(error) = shared_values
            .runtime
            .block_on(shared_values.route_manager.clear_routing_rules())
        {
            log::error!(
                "{}",
                error.display_chain_with_msg("Failed to clear routing rules")
            );
        }
    }

    fn disconnect(
        self,
        shared_values: &mut SharedTunnelStateValues,
        after_disconnect: AfterDisconnect,
    ) -> EventConsequence {
        Self::reset_dns(shared_values);
        Self::reset_routes(shared_values);

        EventConsequence::NewState(DisconnectingState::enter(
            self.tunnel_close_tx,
            self.tunnel_close_event,
            after_disconnect,
        ))
    }

    fn handle_commands(
        self: Box<Self>,
        command: Option<TunnelCommand>,
        shared_values: &mut SharedTunnelStateValues,
    ) -> EventConsequence {
        use self::EventConsequence::*;

        match command {
            Some(TunnelCommand::AllowLan(allow_lan)) => {
                if let Err(error_cause) = shared_values.set_allow_lan(allow_lan) {
                    self.disconnect(shared_values, AfterDisconnect::Block(error_cause))
                } else {
                    match self.set_firewall_policy(shared_values) {
                        Ok(()) => {
                            if cfg!(target_os = "android") {
                                self.disconnect(shared_values, AfterDisconnect::Reconnect(0))
                            } else {
                                SameState(self)
                            }
                        }
                        Err(error) => self.disconnect(
                            shared_values,
                            AfterDisconnect::Block(ErrorStateCause::SetFirewallPolicyError(error)),
                        ),
                    }
                }
            }
            Some(TunnelCommand::AllowEndpoint(endpoint, tx)) => {
                shared_values.allowed_endpoint = endpoint;
                let _ = tx.send(());
                SameState(self)
            }
            Some(TunnelCommand::Dns(servers)) => match shared_values.set_dns_servers(servers) {
                Ok(true) => {
                    if let Err(error) = self.set_firewall_policy(shared_values) {
                        return self.disconnect(
                            shared_values,
                            AfterDisconnect::Block(ErrorStateCause::SetFirewallPolicyError(error)),
                        );
                    }

                    match self.set_dns(shared_values) {
                        #[cfg(target_os = "android")]
                        Ok(()) => self.disconnect(shared_values, AfterDisconnect::Reconnect(0)),
                        #[cfg(not(target_os = "android"))]
                        Ok(()) => SameState(self),
                        Err(error) => {
                            log::error!("{}", error.display_chain_with_msg("Failed to set DNS"));
                            self.disconnect(
                                shared_values,
                                AfterDisconnect::Block(ErrorStateCause::SetDnsError),
                            )
                        }
                    }
                }
                Ok(false) => SameState(self),
                Err(error_cause) => {
                    self.disconnect(shared_values, AfterDisconnect::Block(error_cause))
                }
            },
            Some(TunnelCommand::BlockWhenDisconnected(block_when_disconnected)) => {
                shared_values.block_when_disconnected = block_when_disconnected;
                SameState(self)
            }
            Some(TunnelCommand::IsOffline(is_offline)) => {
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
            Some(TunnelCommand::Connect) => {
                self.disconnect(shared_values, AfterDisconnect::Reconnect(0))
            }
            Some(TunnelCommand::Disconnect) | None => {
                self.disconnect(shared_values, AfterDisconnect::Nothing)
            }
            Some(TunnelCommand::Block(reason)) => {
                self.disconnect(shared_values, AfterDisconnect::Block(reason))
            }
            #[cfg(target_os = "android")]
            Some(TunnelCommand::BypassSocket(fd, done_tx)) => {
                shared_values.bypass_socket(fd, done_tx);
                SameState(self)
            }
            #[cfg(windows)]
            Some(TunnelCommand::SetExcludedApps(result_tx, paths)) => {
                shared_values.split_tunnel.set_paths(&paths, result_tx);
                SameState(self)
            }
        }
    }

    fn handle_tunnel_events(
        self: Box<Self>,
        event: Option<(TunnelEvent, oneshot::Sender<()>)>,
        shared_values: &mut SharedTunnelStateValues,
    ) -> EventConsequence {
        use self::EventConsequence::*;

        match event {
            Some((TunnelEvent::Down, _)) | None => {
                self.disconnect(shared_values, AfterDisconnect::Reconnect(0))
            }
            Some(_) => SameState(self),
        }
    }

    fn handle_tunnel_close_event(
        self,
        block_reason: Option<ErrorStateCause>,
        shared_values: &mut SharedTunnelStateValues,
    ) -> EventConsequence {
        use self::EventConsequence::*;

        if let Some(block_reason) = block_reason {
            Self::reset_dns(shared_values);
            Self::reset_routes(shared_values);
            return NewState(ErrorState::enter(shared_values, block_reason));
        }

        log::info!("Tunnel closed. Reconnecting.");
        Self::reset_dns(shared_values);
        Self::reset_routes(shared_values);
        NewState(ConnectingState::enter(shared_values, 0))
    }
}

impl TunnelState for ConnectedState {
    fn handle_event(
        mut self: Box<Self>,
        runtime: &tokio::runtime::Handle,
        commands: &mut TunnelCommandReceiver,
        shared_values: &mut SharedTunnelStateValues,
    ) -> EventConsequence {
        let result = runtime.block_on(async {
            futures::select! {
                command = commands.next() => EventResult::Command(command),
                event = self.tunnel_events.next() => EventResult::Event(event),
                result = &mut self.tunnel_close_event => EventResult::Close(result),
            }
        });

        match result {
            EventResult::Command(command) => self.handle_commands(command, shared_values),
            EventResult::Event(event) => self.handle_tunnel_events(event, shared_values),
            EventResult::Close(result) => {
                if result.is_err() {
                    log::warn!("Tunnel monitor thread has stopped unexpectedly");
                }
                let block_reason = result.unwrap_or(None);
                self.handle_tunnel_close_event(block_reason, shared_values)
            }
        }
    }
}

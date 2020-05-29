use super::{
    AfterDisconnect, ConnectingState, DisconnectingState, ErrorState, EventConsequence,
    EventResult, SharedTunnelStateValues, TunnelCommand, TunnelCommandReceiver, TunnelState,
    TunnelStateTransition, TunnelStateWrapper,
};
use crate::{
    firewall::FirewallPolicy,
    tunnel::{CloseHandle, TunnelEvent, TunnelMetadata},
};
#[cfg(windows)]
use crate::{
    split_tunnel::{self, SplitTunnel},
    winnet::{self, get_best_default_route, interface_luid_to_ip, WinNetAddrFamily},
};
use cfg_if::cfg_if;
use futures::{channel::mpsc, stream::Fuse, StreamExt};
use std::net::IpAddr;
#[cfg(windows)]
use std::{
    ffi::OsStr,
    net::{Ipv4Addr, Ipv6Addr},
    sync::{Arc, Mutex},
};
use talpid_types::{
    net::TunnelParameters,
    tunnel::{ErrorStateCause, FirewallPolicyError},
    BoxedError, ErrorExt,
};

#[cfg(windows)]
use crate::tunnel::TunnelMonitor;

use super::connecting_state::TunnelCloseEvent;

pub(crate) type TunnelEventsReceiver = Fuse<mpsc::UnboundedReceiver<TunnelEvent>>;


pub struct ConnectedStateBootstrap {
    pub metadata: TunnelMetadata,
    pub tunnel_events: TunnelEventsReceiver,
    pub tunnel_parameters: TunnelParameters,
    pub tunnel_close_event: TunnelCloseEvent,
    pub close_handle: Option<CloseHandle>,
}

/// The tunnel is up and working.
pub struct ConnectedState {
    metadata: TunnelMetadata,
    tunnel_events: TunnelEventsReceiver,
    tunnel_parameters: TunnelParameters,
    tunnel_close_event: TunnelCloseEvent,
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
            let mut dns_ips = vec![];
            dns_ips.push(self.metadata.ipv4_gateway.into());
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
        FirewallPolicy::Connected {
            peer_endpoint: self.tunnel_parameters.get_next_hop_endpoint(),
            tunnel: self.metadata.clone(),
            allow_lan: shared_values.allow_lan,
            #[cfg(not(target_os = "android"))]
            dns_servers: self.get_dns_servers(shared_values),
            #[cfg(windows)]
            relay_client: TunnelMonitor::get_relay_client(
                &shared_values.resource_dir,
                &self.tunnel_parameters,
            ),
        }
    }

    #[cfg(target_os = "windows")]
    pub unsafe extern "system" fn split_tunnel_default_route_change_handler(
        event_type: winnet::WinNetDefaultRouteChangeEventType,
        address_family: WinNetAddrFamily,
        default_route: winnet::WinNetDefaultRoute,
        ctx: *mut libc::c_void,
    ) {
        // Update the "internet interface" IP when best default route changes
        let ctx = &mut *(ctx as *mut SplitTunnelDefaultRouteChangeHandlerContext);

        let result = match event_type {
            winnet::WinNetDefaultRouteChangeEventType::DefaultRouteChanged => {
                let ip = interface_luid_to_ip(address_family.clone(), default_route.interface_luid);

                // TODO: Should we block here?
                let ip = match ip {
                    Ok(Some(ip)) => ip,
                    Ok(None) => {
                        log::error!("Failed to obtain new default route address: none found",);
                        // Early return
                        return;
                    }
                    Err(error) => {
                        log::error!(
                            "{}",
                            error.display_chain_with_msg(
                                "Failed to obtain new default route address"
                            )
                        );
                        // Early return
                        return;
                    }
                };

                match address_family {
                    WinNetAddrFamily::IPV4 => {
                        let ip = Ipv4Addr::from(ip);
                        ctx.internet_ipv4 = ip;
                    }
                    WinNetAddrFamily::IPV6 => {
                        let ip = Ipv6Addr::from(ip);
                        ctx.internet_ipv6 = Some(ip);
                    }
                }

                ctx.register_ips()
            }
            // no default route
            winnet::WinNetDefaultRouteChangeEventType::DefaultRouteRemoved => {
                match address_family {
                    WinNetAddrFamily::IPV4 => {
                        ctx.internet_ipv4 = Ipv4Addr::new(0, 0, 0, 0);
                    }
                    WinNetAddrFamily::IPV6 => {
                        ctx.internet_ipv6 = None;
                    }
                }
                ctx.register_ips()
            }
        };

        if let Err(error) = result {
            // TODO: Should we block here?
            log::error!(
                "{}",
                error.display_chain_with_msg(
                    "Failed to register new addresses in split tunnel driver"
                )
            );
        }
    }

    #[cfg(windows)]
    fn update_split_tunnel_addresses(
        &self,
        shared_values: &mut SharedTunnelStateValues,
    ) -> Result<(), BoxedError> {
        // Identify tunnel IP addresses
        // TODO: Multiple IP addresses?
        let mut tunnel_ipv4 = None;
        let mut tunnel_ipv6 = None;

        for ip in &self.metadata.ips {
            match ip {
                IpAddr::V4(address) => tunnel_ipv4 = Some(address.clone()),
                IpAddr::V6(address) => tunnel_ipv6 = Some(address.clone()),
            }
        }

        // Identify IP address that gives us Internet access
        let internet_ipv4 = get_best_default_route(WinNetAddrFamily::IPV4)
            .map_err(BoxedError::new)?
            .map(|route| interface_luid_to_ip(WinNetAddrFamily::IPV4, route.interface_luid))
            .transpose()
            .map_err(BoxedError::new)?
            .flatten();
        let internet_ipv6 = get_best_default_route(WinNetAddrFamily::IPV6)
            .map_err(BoxedError::new)?
            .map(|route| interface_luid_to_ip(WinNetAddrFamily::IPV6, route.interface_luid))
            .transpose()
            .map_err(BoxedError::new)?
            .flatten();

        let tunnel_ipv4 = tunnel_ipv4.unwrap_or(Ipv4Addr::new(0, 0, 0, 0));
        let internet_ipv4 = Ipv4Addr::from(internet_ipv4.unwrap_or_default());
        let internet_ipv6 = internet_ipv6.map(|addr| Ipv6Addr::from(addr));

        let context = SplitTunnelDefaultRouteChangeHandlerContext::new(
            shared_values.split_tunnel.clone(),
            tunnel_ipv4,
            tunnel_ipv6,
            internet_ipv4,
            internet_ipv6,
        );

        shared_values
            .split_tunnel
            .lock()
            .expect("Thread unexpectedly panicked while holding the mutex")
            .register_ips(tunnel_ipv4, tunnel_ipv6, internet_ipv4, internet_ipv6)
            .map_err(BoxedError::new)?;

        #[cfg(target_os = "windows")]
        shared_values.route_manager.add_default_route_callback(
            Some(Self::split_tunnel_default_route_change_handler),
            context,
        );

        Ok(())
    }

    fn set_dns(&self, shared_values: &mut SharedTunnelStateValues) -> Result<(), BoxedError> {
        let dns_ips = self.get_dns_servers(shared_values);
        shared_values
            .dns_monitor
            .set(&self.metadata.interface, &dns_ips)
            .map_err(BoxedError::new)?;

        Ok(())
    }

    fn reset_dns(shared_values: &mut SharedTunnelStateValues) {
        if let Err(error) = shared_values.dns_monitor.reset() {
            log::error!("{}", error.display_chain_with_msg("Unable to reset DNS"));
        }
    }

    fn reset_routes(shared_values: &mut SharedTunnelStateValues) {
        #[cfg(windows)]
        shared_values.route_manager.clear_default_route_callbacks();
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

    #[cfg(windows)]
    fn apply_split_tunnel_config<T: AsRef<OsStr>>(
        shared_values: &SharedTunnelStateValues,
        paths: &[T],
    ) -> Result<(), split_tunnel::Error> {
        let split_tunnel = shared_values
            .split_tunnel
            .lock()
            .expect("Thread unexpectedly panicked while holding the mutex");
        split_tunnel.set_paths(paths)
    }

    fn disconnect(
        self,
        shared_values: &mut SharedTunnelStateValues,
        after_disconnect: AfterDisconnect,
    ) -> EventConsequence {
        Self::reset_dns(shared_values);
        Self::reset_routes(shared_values);

        #[cfg(windows)]
        if let Err(error) = shared_values
            .split_tunnel
            .lock()
            .expect("Thread unexpectedly panicked while holding the mutex")
            .register_ips(
                Ipv4Addr::new(0, 0, 0, 0),
                None,
                Ipv4Addr::new(0, 0, 0, 0),
                None,
            )
        {
            log::error!(
                "{}",
                error.display_chain_with_msg("Failed to unregister IP addresses")
            );
        }

        EventConsequence::NewState(DisconnectingState::enter(
            shared_values,
            (self.close_handle, self.tunnel_close_event, after_disconnect),
        ))
    }

    fn handle_commands(
        self,
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
                            cfg_if! {
                                if #[cfg(target_os = "android")] {
                                    self.disconnect(shared_values, AfterDisconnect::Reconnect(0))
                                } else {
                                    SameState(self.into())
                                }
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
                let _ = shared_values.set_allowed_endpoint(endpoint);
                if let Err(_) = tx.send(()) {
                    log::error!("The AllowEndpoint receiver was dropped");
                }
                SameState(self.into())
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
                        Ok(()) => SameState(self.into()),
                        Err(error) => {
                            log::error!("{}", error.display_chain_with_msg("Failed to set DNS"));
                            self.disconnect(
                                shared_values,
                                AfterDisconnect::Block(ErrorStateCause::SetDnsError),
                            )
                        }
                    }
                }
                Ok(false) => SameState(self.into()),
                Err(error_cause) => {
                    self.disconnect(shared_values, AfterDisconnect::Block(error_cause))
                }
            },
            Some(TunnelCommand::BlockWhenDisconnected(block_when_disconnected)) => {
                shared_values.block_when_disconnected = block_when_disconnected;
                SameState(self.into())
            }
            Some(TunnelCommand::IsOffline(is_offline)) => {
                shared_values.is_offline = is_offline;
                if is_offline {
                    self.disconnect(
                        shared_values,
                        AfterDisconnect::Block(ErrorStateCause::IsOffline),
                    )
                } else {
                    SameState(self.into())
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
                SameState(self.into())
            }
            #[cfg(windows)]
            Some(TunnelCommand::SetExcludedApps(result_tx, paths)) => {
                let _ = result_tx.send(Self::apply_split_tunnel_config(shared_values, &paths));
                SameState(self.into())
            }
        }
    }

    fn handle_tunnel_events(
        self,
        event: Option<TunnelEvent>,
        shared_values: &mut SharedTunnelStateValues,
    ) -> EventConsequence {
        use self::EventConsequence::*;

        match event {
            Some(TunnelEvent::Down) | None => {
                self.disconnect(shared_values, AfterDisconnect::Reconnect(0))
            }
            Some(_) => SameState(self.into()),
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
    type Bootstrap = ConnectedStateBootstrap;

    #[cfg_attr(target_os = "android", allow(unused_variables))]
    fn enter(
        shared_values: &mut SharedTunnelStateValues,
        bootstrap: Self::Bootstrap,
    ) -> (TunnelStateWrapper, TunnelStateTransition) {
        let connected_state = ConnectedState::from(bootstrap);
        let tunnel_endpoint = connected_state.tunnel_parameters.get_tunnel_endpoint();

        if let Err(error) = connected_state.set_firewall_policy(shared_values) {
            DisconnectingState::enter(
                shared_values,
                (
                    connected_state.close_handle,
                    connected_state.tunnel_close_event,
                    AfterDisconnect::Block(ErrorStateCause::SetFirewallPolicyError(error)),
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
            #[cfg(windows)]
            if let Err(error) = connected_state.update_split_tunnel_addresses(shared_values) {
                log::error!("{}", error.display_chain());
                return DisconnectingState::enter(
                    shared_values,
                    (
                        connected_state.close_handle,
                        connected_state.tunnel_close_event,
                        AfterDisconnect::Block(ErrorStateCause::StartTunnelError),
                    ),
                );
            }

            (
                TunnelStateWrapper::from(connected_state),
                TunnelStateTransition::Connected(tunnel_endpoint),
            )
        }
    }

    fn handle_event(
        mut self,
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

#[cfg(target_os = "windows")]
struct SplitTunnelDefaultRouteChangeHandlerContext {
    split_tunnel: Arc<Mutex<SplitTunnel>>,
    pub tunnel_ipv4: Ipv4Addr,
    pub tunnel_ipv6: Option<Ipv6Addr>,
    pub internet_ipv4: Ipv4Addr,
    pub internet_ipv6: Option<Ipv6Addr>,
}

#[cfg(target_os = "windows")]
impl SplitTunnelDefaultRouteChangeHandlerContext {
    pub fn new(
        split_tunnel: Arc<Mutex<SplitTunnel>>,
        tunnel_ipv4: Ipv4Addr,
        tunnel_ipv6: Option<Ipv6Addr>,
        internet_ipv4: Ipv4Addr,
        internet_ipv6: Option<Ipv6Addr>,
    ) -> Self {
        SplitTunnelDefaultRouteChangeHandlerContext {
            split_tunnel,
            tunnel_ipv4,
            tunnel_ipv6,
            internet_ipv4,
            internet_ipv6,
        }
    }

    pub fn register_ips(&self) -> Result<(), split_tunnel::Error> {
        let split_tunnel = self
            .split_tunnel
            .lock()
            .expect("Thread unexpectedly panicked while holding the mutex");
        split_tunnel.register_ips(
            self.tunnel_ipv4,
            self.tunnel_ipv6,
            self.internet_ipv4,
            self.internet_ipv6,
        )
    }
}

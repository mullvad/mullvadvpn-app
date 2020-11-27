use super::{
    AfterDisconnect, ConnectedState, ConnectedStateBootstrap, DisconnectingState, ErrorState,
    EventConsequence, EventResult, SharedTunnelStateValues, TunnelCommand, TunnelCommandReceiver,
    TunnelState, TunnelStateTransition, TunnelStateWrapper,
};
use crate::{
    firewall::FirewallPolicy,
    routing::RouteManager,
    tunnel::{
        self, tun_provider::TunProvider, CloseHandle, TunnelEvent, TunnelMetadata, TunnelMonitor,
    },
};
use cfg_if::cfg_if;
use futures::{
    channel::{mpsc, oneshot},
    future::Fuse,
    FutureExt, StreamExt,
};
use log::{debug, error, info, trace, warn};
use std::{
    net::IpAddr,
    path::{Path, PathBuf},
    thread,
    time::{Duration, Instant},
};
use talpid_types::{
    net::TunnelParameters,
    tunnel::{ErrorStateCause, FirewallPolicyError},
    ErrorExt,
};

#[cfg(target_os = "android")]
use crate::tunnel::tun_provider;

use super::connected_state::TunnelEventsReceiver;

pub(crate) type TunnelCloseEvent = Fuse<oneshot::Receiver<Option<ErrorStateCause>>>;

#[cfg(target_os = "android")]
const MAX_ATTEMPTS_WITH_SAME_TUN: u32 = 5;
const MIN_TUNNEL_ALIVE_TIME: Duration = Duration::from_millis(1000);

/// The tunnel has been started, but it is not established/functional.
pub struct ConnectingState {
    tunnel_events: TunnelEventsReceiver,
    tunnel_parameters: TunnelParameters,
    tunnel_close_event: TunnelCloseEvent,
    close_handle: Option<CloseHandle>,
    retry_attempt: u32,
}

impl ConnectingState {
    fn set_firewall_policy(
        shared_values: &mut SharedTunnelStateValues,
        params: &TunnelParameters,
    ) -> Result<(), FirewallPolicyError> {
        #[cfg(target_os = "linux")]
        shared_values.disable_connectivity_check();

        let peer_endpoint = params.get_next_hop_endpoint();

        let policy = FirewallPolicy::Connecting {
            peer_endpoint,
            pingable_hosts: gateway_list_from_params(params),
            allow_lan: shared_values.allow_lan,
            #[cfg(windows)]
            relay_client: TunnelMonitor::get_relay_client(&shared_values.resource_dir, &params),
            #[cfg(target_os = "linux")]
            use_fwmark: params.get_proxy_endpoint().is_none(),
        };
        shared_values
            .firewall
            .apply_policy(policy)
            .map_err(|error| {
                error!(
                    "{}",
                    error.display_chain_with_msg(
                        "Failed to apply firewall policy for connecting state"
                    )
                );
                match error {
                    #[cfg(windows)]
                    crate::firewall::Error::ApplyingConnectingPolicy(policy_error) => policy_error,
                    _ => FirewallPolicyError::Generic,
                }
            })
    }

    fn start_tunnel(
        parameters: TunnelParameters,
        log_dir: &Option<PathBuf>,
        resource_dir: &Path,
        tun_provider: &mut TunProvider,
        route_manager: &mut RouteManager,
        retry_attempt: u32,
    ) -> crate::tunnel::Result<Self> {
        let (event_tx, event_rx) = mpsc::unbounded();
        let on_tunnel_event = move |event| {
            let _ = event_tx.unbounded_send(event);
        };

        let monitor = TunnelMonitor::start(
            &parameters,
            log_dir,
            resource_dir,
            on_tunnel_event,
            tun_provider,
            route_manager,
        )?;
        let close_handle = Some(monitor.close_handle());
        let tunnel_close_event = Self::spawn_tunnel_monitor_wait_thread(monitor);

        Ok(ConnectingState {
            tunnel_events: event_rx.fuse(),
            tunnel_parameters: parameters,
            tunnel_close_event,
            close_handle,
            retry_attempt,
        })
    }

    fn spawn_tunnel_monitor_wait_thread(tunnel_monitor: TunnelMonitor) -> TunnelCloseEvent {
        let (tunnel_close_event_tx, tunnel_close_event_rx) = oneshot::channel();

        thread::spawn(move || {
            let start = Instant::now();

            let block_reason = Self::wait_for_tunnel_monitor(tunnel_monitor);
            debug!(
                "Tunnel monitor exited with block reason: {:?}",
                block_reason
            );

            if block_reason.is_none() {
                if let Some(remaining_time) = MIN_TUNNEL_ALIVE_TIME.checked_sub(start.elapsed()) {
                    thread::sleep(remaining_time);
                }
            }

            if tunnel_close_event_tx.send(block_reason).is_err() {
                warn!("Tunnel state machine stopped before receiving tunnel closed event");
            }

            trace!("Tunnel monitor thread exit");
        });

        tunnel_close_event_rx.fuse()
    }

    fn wait_for_tunnel_monitor(tunnel_monitor: TunnelMonitor) -> Option<ErrorStateCause> {
        match tunnel_monitor.wait() {
            Ok(_) => None,
            Err(error) => match error {
                #[cfg(windows)]
                error
                @
                tunnel::Error::OpenVpnTunnelMonitoringError(
                    tunnel::openvpn::Error::DisabledVirtualAdapter,
                )
                | error
                @
                tunnel::Error::OpenVpnTunnelMonitoringError(
                    tunnel::openvpn::Error::MissingVirtualAdapter,
                ) => {
                    warn!(
                        "{}",
                        error.display_chain_with_msg("Virtual adapter problem detected")
                    );
                    Some(ErrorStateCause::VirtualAdapterProblem)
                }
                tunnel::Error::WireguardTunnelMonitoringError(
                    tunnel::wireguard::Error::TimeoutError,
                ) => {
                    log::debug!("WireGuard tunnel timed out");
                    None
                }
                error => {
                    warn!(
                        "{}",
                        error.display_chain_with_msg("Tunnel has stopped unexpectedly")
                    );
                    None
                }
            },
        }
    }

    fn into_connected_state_bootstrap(self, metadata: TunnelMetadata) -> ConnectedStateBootstrap {
        ConnectedStateBootstrap {
            metadata,
            tunnel_events: self.tunnel_events,
            tunnel_parameters: self.tunnel_parameters,
            tunnel_close_event: self.tunnel_close_event,
            close_handle: self.close_handle,
        }
    }

    fn reset_routes(shared_values: &mut SharedTunnelStateValues) {
        #[cfg(windows)]
        shared_values.route_manager.clear_default_route_callbacks();
        if let Err(error) = shared_values.route_manager.clear_routes() {
            log::error!("{}", error.display_chain_with_msg("Failed to clear routes"));
        }
        #[cfg(target_os = "linux")]
        if let Err(error) = shared_values.route_manager.clear_routing_rules() {
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
        Self::reset_routes(shared_values);

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
                    match Self::set_firewall_policy(shared_values, &self.tunnel_parameters) {
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
            #[cfg(not(target_os = "android"))]
            Some(TunnelCommand::CustomDns(servers)) => {
                shared_values.custom_dns = servers;
                SameState(self.into())
            }
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
        }
    }

    fn handle_tunnel_events(
        self,
        event: Option<tunnel::TunnelEvent>,
        shared_values: &mut SharedTunnelStateValues,
    ) -> EventConsequence {
        use self::EventConsequence::*;

        match event {
            Some(TunnelEvent::AuthFailed(reason)) => self.disconnect(
                shared_values,
                AfterDisconnect::Block(ErrorStateCause::AuthFailed(reason)),
            ),
            Some(TunnelEvent::Up(metadata)) => NewState(ConnectedState::enter(
                shared_values,
                self.into_connected_state_bootstrap(metadata),
            )),
            Some(TunnelEvent::Down) => SameState(self.into()),
            None => {
                // The channel was closed
                debug!("The tunnel disconnected unexpectedly");
                let retry_attempt = self.retry_attempt + 1;
                self.disconnect(shared_values, AfterDisconnect::Reconnect(retry_attempt))
            }
        }
    }

    fn handle_tunnel_close_event(
        self,
        block_reason: Option<ErrorStateCause>,
        shared_values: &mut SharedTunnelStateValues,
    ) -> EventConsequence {
        use self::EventConsequence::*;

        if let Some(block_reason) = block_reason {
            Self::reset_routes(shared_values);
            return NewState(ErrorState::enter(shared_values, block_reason));
        }

        info!(
            "Tunnel closed. Reconnecting, attempt {}.",
            self.retry_attempt + 1
        );
        Self::reset_routes(shared_values);
        EventConsequence::NewState(ConnectingState::enter(
            shared_values,
            self.retry_attempt + 1,
        ))
    }
}

fn should_retry(error: &tunnel::Error) -> bool {
    #[cfg(not(windows))]
    use tunnel::wireguard::{Error, TunnelError};
    match error {
        #[cfg(not(windows))]
        tunnel::Error::WireguardTunnelMonitoringError(Error::TunnelError(
            TunnelError::RecoverableStartWireguardError,
        )) => true,

        #[cfg(target_os = "android")]
        tunnel::Error::WireguardTunnelMonitoringError(Error::TunnelError(
            TunnelError::BypassError(_),
        )) => true,

        _ => false,
    }
}

impl TunnelState for ConnectingState {
    type Bootstrap = u32;

    fn enter(
        shared_values: &mut SharedTunnelStateValues,
        retry_attempt: u32,
    ) -> (TunnelStateWrapper, TunnelStateTransition) {
        if shared_values.is_offline {
            return ErrorState::enter(shared_values, ErrorStateCause::IsOffline);
        }
        match shared_values
            .tunnel_parameters_generator
            .generate(retry_attempt)
        {
            Err(err) => {
                ErrorState::enter(shared_values, ErrorStateCause::TunnelParameterError(err))
            }
            Ok(tunnel_parameters) => {
                if let Err(error) = Self::set_firewall_policy(shared_values, &tunnel_parameters) {
                    ErrorState::enter(
                        shared_values,
                        ErrorStateCause::SetFirewallPolicyError(error),
                    )
                } else {
                    #[cfg(target_os = "android")]
                    {
                        if retry_attempt > 0 && retry_attempt % MAX_ATTEMPTS_WITH_SAME_TUN == 0 {
                            if let Err(error) = shared_values.tun_provider.create_tun() {
                                error!(
                                    "{}",
                                    error.display_chain_with_msg("Failed to recreate tun device")
                                );
                            }
                        }
                    }

                    match Self::start_tunnel(
                        tunnel_parameters,
                        &shared_values.log_dir,
                        &shared_values.resource_dir,
                        &mut shared_values.tun_provider,
                        &mut shared_values.route_manager,
                        retry_attempt,
                    ) {
                        Ok(connecting_state) => {
                            let params = connecting_state.tunnel_parameters.clone();
                            (
                                TunnelStateWrapper::from(connecting_state),
                                TunnelStateTransition::Connecting(params.get_tunnel_endpoint()),
                            )
                        }
                        Err(error) => {
                            if should_retry(&error) {
                                log::warn!(
                                    "{}",
                                    error.display_chain_with_msg(
                                        "Retrying to connect after failing to start tunnel"
                                    )
                                );
                                DisconnectingState::enter(
                                    shared_values,
                                    (
                                        None,
                                        Fuse::terminated(),
                                        AfterDisconnect::Reconnect(retry_attempt + 1),
                                    ),
                                )
                            } else {
                                log::error!(
                                    "{}",
                                    error.display_chain_with_msg("Failed to start tunnel")
                                );
                                let block_reason = match error {
                                    tunnel::Error::EnableIpv6Error => {
                                        ErrorStateCause::Ipv6Unavailable
                                    }
                                    #[cfg(windows)]
                                    tunnel::Error::OpenVpnTunnelMonitoringError(
                                        tunnel::openvpn::Error::WinnetError(
                                            crate::winnet::Error::GetVirtualAdapterAlias,
                                        ),
                                    )
                                    | tunnel::Error::WinnetError(
                                        crate::winnet::Error::GetVirtualAdapterAlias,
                                    ) => ErrorStateCause::VirtualAdapterProblem,
                                    #[cfg(target_os = "android")]
                                    tunnel::Error::WireguardTunnelMonitoringError(
                                        tunnel::wireguard::Error::TunnelError(
                                            tunnel::wireguard::TunnelError::SetupTunnelDeviceError(
                                                tun_provider::Error::PermissionDenied,
                                            ),
                                        ),
                                    ) => ErrorStateCause::VpnPermissionDenied,
                                    _ => ErrorStateCause::StartTunnelError,
                                };
                                ErrorState::enter(shared_values, block_reason)
                            }
                        }
                    }
                }
            }
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

fn gateway_list_from_params(params: &TunnelParameters) -> Vec<IpAddr> {
    match params {
        TunnelParameters::Wireguard(params) => {
            let mut gateways = vec![params.connection.ipv4_gateway.into()];
            if let Some(ipv6_gateway) = params.connection.ipv6_gateway {
                gateways.push(ipv6_gateway.into())
            };
            gateways
        }
        // No gateway list required when connecting to openvpn
        TunnelParameters::OpenVpn(_) => vec![],
    }
}

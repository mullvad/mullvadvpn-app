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
use std::{
    path::{Path, PathBuf},
    thread,
    time::{Duration, Instant},
};
use talpid_types::{
    net::TunnelParameters,
    tunnel::{ErrorStateCause, FirewallPolicyError},
    ErrorExt,
};

#[cfg(windows)]
use crate::{routing, winnet};

#[cfg(target_os = "android")]
use crate::tunnel::tun_provider;

use super::connected_state::TunnelEventsReceiver;

pub(crate) type TunnelCloseEvent = Fuse<oneshot::Receiver<Option<ErrorStateCause>>>;

#[cfg(target_os = "android")]
const MAX_ATTEMPTS_WITH_SAME_TUN: u32 = 5;
const MIN_TUNNEL_ALIVE_TIME: Duration = Duration::from_millis(1000);
#[cfg(target_os = "windows")]
const MAX_ADAPTER_FAIL_RETRIES: u32 = 4;

/// The tunnel has been started, but it is not established/functional.
pub struct ConnectingState {
    tunnel_events: TunnelEventsReceiver,
    tunnel_parameters: TunnelParameters,
    tunnel_metadata: Option<TunnelMetadata>,
    tunnel_close_event: TunnelCloseEvent,
    close_handle: Option<CloseHandle>,
    retry_attempt: u32,
}

impl ConnectingState {
    fn set_firewall_policy(
        shared_values: &mut SharedTunnelStateValues,
        params: &TunnelParameters,
        tunnel_metadata: &Option<TunnelMetadata>,
    ) -> Result<(), FirewallPolicyError> {
        #[cfg(target_os = "linux")]
        shared_values.disable_connectivity_check();

        let peer_endpoint = params.get_next_hop_endpoint();

        let policy = FirewallPolicy::Connecting {
            peer_endpoint,
            tunnel: tunnel_metadata.clone(),
            allow_lan: shared_values.allow_lan,
            allowed_endpoint: shared_values.allowed_endpoint.clone(),
            #[cfg(windows)]
            relay_client: TunnelMonitor::get_relay_client(&shared_values.resource_dir, &params),
        };
        shared_values
            .firewall
            .apply_policy(policy)
            .map_err(|error| {
                log::error!(
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
        runtime: tokio::runtime::Handle,
        parameters: TunnelParameters,
        log_dir: &Option<PathBuf>,
        resource_dir: &Path,
        tun_provider: &mut TunProvider,
        route_manager: &mut RouteManager,
        retry_attempt: u32,
    ) -> crate::tunnel::Result<Self> {
        let (event_tx, event_rx) = mpsc::unbounded();
        let on_tunnel_event =
            move |event| -> std::pin::Pin<Box<dyn std::future::Future<Output = ()> + Send>> {
                let (tx, rx) = oneshot::channel();
                let _ = event_tx.unbounded_send((event, tx));
                Box::pin(async move {
                    let _ = rx.await;
                })
            };

        let monitor = TunnelMonitor::start(
            runtime,
            &parameters,
            log_dir,
            resource_dir,
            on_tunnel_event,
            tun_provider,
            route_manager,
            retry_attempt,
        )?;
        let close_handle = Some(monitor.close_handle());
        let tunnel_close_event =
            Self::spawn_tunnel_monitor_wait_thread(Some(monitor), retry_attempt);

        Ok(ConnectingState {
            tunnel_events: event_rx.fuse(),
            tunnel_parameters: parameters,
            tunnel_metadata: None,
            tunnel_close_event,
            close_handle,
            retry_attempt,
        })
    }

    fn spawn_tunnel_monitor_wait_thread(
        tunnel_monitor: Option<TunnelMonitor>,
        retry_attempt: u32,
    ) -> TunnelCloseEvent {
        let (tunnel_close_event_tx, tunnel_close_event_rx) = oneshot::channel();

        thread::spawn(move || {
            let start = Instant::now();

            let block_reason = if let Some(monitor) = tunnel_monitor {
                let reason = Self::wait_for_tunnel_monitor(monitor, retry_attempt);
                log::debug!("Tunnel monitor exited with block reason: {:?}", reason);
                reason
            } else {
                None
            };

            if block_reason.is_none() {
                if let Some(remaining_time) = MIN_TUNNEL_ALIVE_TIME.checked_sub(start.elapsed()) {
                    thread::sleep(remaining_time);
                }
            }

            if tunnel_close_event_tx.send(block_reason).is_err() {
                log::warn!("Tunnel state machine stopped before receiving tunnel closed event");
            }

            log::trace!("Tunnel monitor thread exit");
        });

        tunnel_close_event_rx.fuse()
    }

    fn wait_for_tunnel_monitor(
        tunnel_monitor: TunnelMonitor,
        retry_attempt: u32,
    ) -> Option<ErrorStateCause> {
        match tunnel_monitor.wait() {
            Ok(_) => None,
            Err(error) => match error {
                tunnel::Error::WireguardTunnelMonitoringError(
                    tunnel::wireguard::Error::TimeoutError,
                ) => {
                    log::debug!("WireGuard tunnel timed out");
                    None
                }
                error @ tunnel::Error::WireguardTunnelMonitoringError(..)
                    if !should_retry(&error, retry_attempt) =>
                {
                    log::error!(
                        "{}",
                        error.display_chain_with_msg("Tunnel has stopped unexpectedly")
                    );
                    Some(ErrorStateCause::StartTunnelError)
                }
                error => {
                    log::warn!(
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
        Self::reset_routes(shared_values);

        EventConsequence::NewState(DisconnectingState::enter(
            shared_values,
            (self.close_handle, self.tunnel_close_event, after_disconnect),
        ))
    }

    fn reset_firewall(self, shared_values: &mut SharedTunnelStateValues) -> EventConsequence {
        match Self::set_firewall_policy(
            shared_values,
            &self.tunnel_parameters,
            &self.tunnel_metadata,
        ) {
            Ok(()) => {
                cfg_if! {
                    if #[cfg(target_os = "android")] {
                        self.disconnect(shared_values, AfterDisconnect::Reconnect(0))
                    } else {
                        EventConsequence::SameState(self.into())
                    }
                }
            }
            Err(error) => self.disconnect(
                shared_values,
                AfterDisconnect::Block(ErrorStateCause::SetFirewallPolicyError(error)),
            ),
        }
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
                    let next_state = self.reset_firewall(shared_values);
                    return next_state;
                }
            }
            Some(TunnelCommand::AllowEndpoint(endpoint, tx)) => {
                if shared_values.set_allowed_endpoint(endpoint) {
                    if let Err(error) = Self::set_firewall_policy(
                        shared_values,
                        &self.tunnel_parameters,
                        &self.tunnel_metadata,
                    ) {
                        return self.disconnect(
                            shared_values,
                            AfterDisconnect::Block(ErrorStateCause::SetFirewallPolicyError(error)),
                        );
                    }
                    let next_state = self.reset_firewall(shared_values);
                    let _ = tx.send(());
                    return next_state;
                }
                if let Err(_) = tx.send(()) {
                    log::error!("The AllowEndpoint receiver was dropped");
                }
                SameState(self.into())
            }
            Some(TunnelCommand::Dns(servers)) => match shared_values.set_dns_servers(servers) {
                #[cfg(target_os = "android")]
                Ok(true) => self.disconnect(shared_values, AfterDisconnect::Reconnect(0)),
                Ok(_) => SameState(self.into()),
                Err(cause) => self.disconnect(shared_values, AfterDisconnect::Block(cause)),
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
                shared_values.split_tunnel.set_paths(&paths, result_tx);
                SameState(self.into())
            }
        }
    }

    fn handle_tunnel_events(
        mut self,
        event: Option<(tunnel::TunnelEvent, oneshot::Sender<()>)>,
        shared_values: &mut SharedTunnelStateValues,
    ) -> EventConsequence {
        use self::EventConsequence::*;

        match event {
            Some((TunnelEvent::AuthFailed(reason), _)) => self.disconnect(
                shared_values,
                AfterDisconnect::Block(ErrorStateCause::AuthFailed(reason)),
            ),
            Some((TunnelEvent::InterfaceUp(metadata), _done_tx)) => {
                #[cfg(windows)]
                if let Err(error) = shared_values
                    .split_tunnel
                    .set_tunnel_addresses(Some(&metadata))
                {
                    log::error!(
                        "{}",
                        error.display_chain_with_msg(
                            "Failed to register addresses with split tunnel driver"
                        )
                    );
                    return self.disconnect(
                        shared_values,
                        AfterDisconnect::Block(ErrorStateCause::SplitTunnelError),
                    );
                }
                self.tunnel_metadata = Some(metadata);
                match Self::set_firewall_policy(
                    shared_values,
                    &self.tunnel_parameters,
                    &self.tunnel_metadata,
                ) {
                    Ok(()) => SameState(self.into()),
                    Err(error) => self.disconnect(
                        shared_values,
                        AfterDisconnect::Block(ErrorStateCause::SetFirewallPolicyError(error)),
                    ),
                }
            }
            Some((TunnelEvent::Up(metadata), _)) => NewState(ConnectedState::enter(
                shared_values,
                self.into_connected_state_bootstrap(metadata),
            )),
            Some((TunnelEvent::Down, _)) => SameState(self.into()),
            None => {
                // The channel was closed
                log::debug!("The tunnel disconnected unexpectedly");
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

        log::info!(
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

#[cfg_attr(not(target_os = "windows"), allow(unused_variables))]
fn should_retry(error: &tunnel::Error, retry_attempt: u32) -> bool {
    #[cfg(windows)]
    use tunnel::openvpn;
    use tunnel::wireguard::{Error, TunnelError};

    match error {
        tunnel::Error::WireguardTunnelMonitoringError(Error::Udp2TcpError(_)) => true,

        #[cfg(not(windows))]
        tunnel::Error::WireguardTunnelMonitoringError(Error::TunnelError(
            TunnelError::RecoverableStartWireguardError,
        )) => true,

        #[cfg(target_os = "android")]
        tunnel::Error::WireguardTunnelMonitoringError(Error::TunnelError(
            TunnelError::BypassError(_),
        )) => true,

        #[cfg(windows)]
        tunnel::Error::WireguardTunnelMonitoringError(Error::SetupRoutingError(error)) => {
            is_recoverable_routing_error(error)
        }

        #[cfg(windows)]
        tunnel::Error::WireguardTunnelMonitoringError(Error::TunnelError(
            TunnelError::RecoverableStartWireguardError,
        )) if retry_attempt < MAX_ADAPTER_FAIL_RETRIES => true,

        #[cfg(windows)]
        tunnel::Error::OpenVpnTunnelMonitoringError(openvpn::Error::WintunCreateAdapterError(
            _,
        )) if retry_attempt < MAX_ADAPTER_FAIL_RETRIES => true,

        _ => false,
    }
}

#[cfg(windows)]
fn is_recoverable_routing_error(error: &crate::routing::Error) -> bool {
    match error {
        routing::Error::AddRoutesFailed(route_error) => match route_error {
            winnet::Error::GetDefaultRoute
            | winnet::Error::GetDeviceByName
            | winnet::Error::GetDeviceByGateway => true,
            _ => false,
        },
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
                #[cfg(windows)]
                if let Err(error) = shared_values.split_tunnel.set_tunnel_addresses(None) {
                    log::error!(
                        "{}",
                        error.display_chain_with_msg(
                            "Failed to reset addresses in split tunnel driver"
                        )
                    );

                    return ErrorState::enter(shared_values, ErrorStateCause::SplitTunnelError);
                }

                if let Err(error) =
                    Self::set_firewall_policy(shared_values, &tunnel_parameters, &None)
                {
                    ErrorState::enter(
                        shared_values,
                        ErrorStateCause::SetFirewallPolicyError(error),
                    )
                } else {
                    #[cfg(target_os = "android")]
                    {
                        if retry_attempt > 0 && retry_attempt % MAX_ATTEMPTS_WITH_SAME_TUN == 0 {
                            if let Err(error) = shared_values.tun_provider.create_tun() {
                                log::error!(
                                    "{}",
                                    error.display_chain_with_msg("Failed to recreate tun device")
                                );
                            }
                        }
                    }

                    match Self::start_tunnel(
                        shared_values.runtime.clone(),
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
                            if should_retry(&error, retry_attempt) {
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
                                        Self::spawn_tunnel_monitor_wait_thread(None, retry_attempt),
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
                                    #[cfg(target_os = "android")]
                                    tunnel::Error::WireguardTunnelMonitoringError(
                                        tunnel::wireguard::Error::TunnelError(
                                            tunnel::wireguard::TunnelError::SetupTunnelDeviceError(
                                                tun_provider::Error::PermissionDenied,
                                            ),
                                        ),
                                    ) => ErrorStateCause::VpnPermissionDenied,
                                    #[cfg(target_os = "android")]
                                    tunnel::Error::WireguardTunnelMonitoringError(
                                        tunnel::wireguard::Error::TunnelError(
                                            tunnel::wireguard::TunnelError::SetupTunnelDeviceError(
                                                tun_provider::Error::InvalidDnsServers(addresses),
                                            ),
                                        ),
                                    ) => ErrorStateCause::InvalidDnsServers(addresses),
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

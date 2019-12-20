use super::{
    AfterDisconnect, ConnectedState, ConnectedStateBootstrap, DisconnectingState, ErrorState,
    EventConsequence, SharedTunnelStateValues, TunnelCommand, TunnelState, TunnelStateTransition,
    TunnelStateWrapper,
};
use crate::{
    firewall::FirewallPolicy,
    tunnel::{
        self, tun_provider::TunProvider, CloseHandle, TunnelEvent, TunnelMetadata, TunnelMonitor,
    },
};
use futures::{
    sync::{mpsc, oneshot},
    Async, Future, Stream,
};
use log::{debug, error, info, trace, warn};
use std::{
    net::IpAddr,
    path::{Path, PathBuf},
    thread,
    time::{Duration, Instant},
};
use talpid_types::{
    net::{openvpn, TunnelParameters},
    tunnel::ErrorStateCause,
    ErrorExt,
};

#[cfg(target_os = "android")]
const MAX_ATTEMPTS_WITH_SAME_TUN: u32 = 5;
const MIN_TUNNEL_ALIVE_TIME: Duration = Duration::from_millis(1000);

/// The tunnel has been started, but it is not established/functional.
pub struct ConnectingState {
    tunnel_events: mpsc::UnboundedReceiver<TunnelEvent>,
    tunnel_parameters: TunnelParameters,
    tunnel_close_event: Option<oneshot::Receiver<Option<ErrorStateCause>>>,
    close_handle: Option<CloseHandle>,
    retry_attempt: u32,
}

impl ConnectingState {
    fn set_firewall_policy(
        shared_values: &mut SharedTunnelStateValues,
        params: &TunnelParameters,
    ) -> Result<(), crate::firewall::Error> {
        let proxy = &get_openvpn_proxy_settings(&params);
        let endpoint = params.get_tunnel_endpoint().endpoint;

        let peer_endpoint = match proxy {
            Some(proxy_settings) => proxy_settings.get_endpoint().endpoint,
            None => endpoint,
        };

        let policy = FirewallPolicy::Connecting {
            peer_endpoint,
            pingable_hosts: gateway_list_from_params(params),
            allow_lan: shared_values.allow_lan,
        };
        shared_values.firewall.apply_policy(policy)
    }

    fn start_tunnel(
        parameters: TunnelParameters,
        log_dir: &Option<PathBuf>,
        resource_dir: &Path,
        tun_provider: &mut TunProvider,
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
        )?;
        let close_handle = Some(monitor.close_handle());
        let tunnel_close_event = Self::spawn_tunnel_monitor_wait_thread(monitor);

        Ok(ConnectingState {
            tunnel_events: event_rx,
            tunnel_parameters: parameters,
            tunnel_close_event,
            close_handle,
            retry_attempt,
        })
    }

    fn spawn_tunnel_monitor_wait_thread(
        tunnel_monitor: TunnelMonitor,
    ) -> Option<oneshot::Receiver<Option<ErrorStateCause>>> {
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

        Some(tunnel_close_event_rx)
    }

    fn wait_for_tunnel_monitor(tunnel_monitor: TunnelMonitor) -> Option<ErrorStateCause> {
        match tunnel_monitor.wait() {
            Ok(_) => None,
            Err(error) => match error {
                #[cfg(windows)]
                error @ tunnel::Error::OpenVpnTunnelMonitoringError(
                    tunnel::openvpn::Error::DisabledTapAdapter,
                )
                | error @ tunnel::Error::OpenVpnTunnelMonitoringError(
                    tunnel::openvpn::Error::MissingTapAdapter,
                ) => {
                    warn!(
                        "{}",
                        error.display_chain_with_msg("TAP adapter problem detected")
                    );
                    Some(ErrorStateCause::TapAdapterProblem)
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

    fn handle_commands(
        self,
        commands: &mut mpsc::UnboundedReceiver<TunnelCommand>,
        shared_values: &mut SharedTunnelStateValues,
    ) -> EventConsequence<Self> {
        use self::EventConsequence::*;

        match try_handle_event!(self, commands.poll()) {
            Ok(TunnelCommand::AllowLan(allow_lan)) => {
                shared_values.allow_lan = allow_lan;
                match Self::set_firewall_policy(shared_values, &self.tunnel_parameters) {
                    Ok(()) => SameState(self),
                    Err(error) => {
                        error!(
                            "{}",
                            error.display_chain_with_msg(
                                "Failed to apply firewall policy for connecting state"
                            )
                        );

                        NewState(DisconnectingState::enter(
                            shared_values,
                            (
                                self.close_handle,
                                self.tunnel_close_event,
                                AfterDisconnect::Block(ErrorStateCause::SetFirewallPolicyError),
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
                            AfterDisconnect::Block(ErrorStateCause::IsOffline),
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
            Ok(TunnelEvent::AuthFailed(reason)) => NewState(DisconnectingState::enter(
                shared_values,
                (
                    self.close_handle,
                    self.tunnel_close_event,
                    AfterDisconnect::Block(ErrorStateCause::AuthFailed(reason)),
                ),
            )),
            Ok(TunnelEvent::Up(metadata)) => NewState(ConnectedState::enter(
                shared_values,
                self.into_connected_state_bootstrap(metadata),
            )),
            Ok(_) => SameState(self),
            Err(_) => {
                debug!("The tunnel disconnected unexpectedly");
                NewState(DisconnectingState::enter(
                    shared_values,
                    (
                        self.close_handle,
                        self.tunnel_close_event,
                        AfterDisconnect::Reconnect(self.retry_attempt + 1),
                    ),
                ))
            }
        }
    }

    fn handle_tunnel_close_event(
        mut self,
        shared_values: &mut SharedTunnelStateValues,
    ) -> EventConsequence<Self> {
        let poll_result = match &mut self.tunnel_close_event {
            Some(tunnel_close_event) => tunnel_close_event.poll(),
            None => Ok(Async::NotReady),
        };

        match poll_result {
            Ok(Async::Ready(block_reason)) => {
                if let Some(reason) = block_reason {
                    return EventConsequence::NewState(ErrorState::enter(shared_values, reason));
                }
            }
            Ok(Async::NotReady) => return EventConsequence::NoEvents(self),
            Err(_cancelled) => warn!("Tunnel monitor thread has stopped unexpectedly"),
        }

        info!(
            "Tunnel closed. Reconnecting, attempt {}.",
            self.retry_attempt + 1
        );
        EventConsequence::NewState(ConnectingState::enter(
            shared_values,
            self.retry_attempt + 1,
        ))
    }
}

fn get_openvpn_proxy_settings(
    tunnel_parameters: &TunnelParameters,
) -> &Option<openvpn::ProxySettings> {
    match tunnel_parameters {
        TunnelParameters::OpenVpn(ref config) => &config.proxy,
        _ => &None,
    }
}

fn should_retry(error: &tunnel::Error) -> bool {
    match error {
        #[cfg(not(windows))]
        tunnel::Error::WireguardTunnelMonitoringError(
            tunnel::wireguard::Error::RecoverableStartWireguardError,
        ) => true,

        #[cfg(target_os = "android")]
        tunnel::Error::WireguardTunnelMonitoringError(tunnel::wireguard::Error::BypassError(_)) => {
            true
        }

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
                    error!(
                        "{}",
                        error.display_chain_with_msg(
                            "Failed to apply firewall policy for connecting state"
                        )
                    );
                    ErrorState::enter(shared_values, ErrorStateCause::StartTunnelError)
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
                                    (None, None, AfterDisconnect::Reconnect(retry_attempt + 1)),
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
                                            crate::winnet::Error::GetTapAlias,
                                        ),
                                    )
                                    | tunnel::Error::WinnetError(
                                        crate::winnet::Error::GetTapAlias,
                                    ) => ErrorStateCause::TapAdapterProblem,
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
        self,
        commands: &mut mpsc::UnboundedReceiver<TunnelCommand>,
        shared_values: &mut SharedTunnelStateValues,
    ) -> EventConsequence<Self> {
        self.handle_commands(commands, shared_values)
            .or_else(Self::handle_tunnel_events, shared_values)
            .or_else(Self::handle_tunnel_close_event, shared_values)
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

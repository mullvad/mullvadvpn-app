use std::{
    ffi::OsString,
    path::{Path, PathBuf},
    thread,
    time::{Duration, Instant},
};

use error_chain::ChainedError;
use futures::{
    sync::{mpsc, oneshot},
    Async, Future, Stream,
};
use log::{debug, error, info, trace, warn};
use talpid_types::{
    net::{openvpn, Endpoint, TunnelParameters},
    tunnel::BlockReason,
};

use super::{
    AfterDisconnect, BlockedState, ConnectedState, ConnectedStateBootstrap, DisconnectingState,
    EventConsequence, SharedTunnelStateValues, TunnelCommand, TunnelState, TunnelStateTransition,
    TunnelStateWrapper,
};
use crate::{
    firewall::FirewallPolicy,
    logging,
    tunnel::{self, CloseHandle, TunnelEvent, TunnelMetadata, TunnelMonitor},
};


const MIN_TUNNEL_ALIVE_TIME: Duration = Duration::from_millis(1000);

const OPENVPN_LOG_FILENAME: &str = "openvpn.log";
const WIREGUARD_LOG_FILENAME: &str = "wireguard.log";

#[cfg(windows)]
const TUNNEL_INTERFACE_ALIAS: Option<&str> = Some("Mullvad");
#[cfg(not(windows))]
const TUNNEL_INTERFACE_ALIAS: Option<&str> = None;

error_chain! {
    errors {
        RotateLogError {
            description("Failed to rotate tunnel log file")
        }
    }

    links {
        TunnelMonitorError(tunnel::Error, tunnel::ErrorKind);
    }
}

/// The tunnel has been started, but it is not established/functional.
pub struct ConnectingState {
    tunnel_events: mpsc::UnboundedReceiver<TunnelEvent>,
    tunnel_parameters: TunnelParameters,
    tunnel_close_event: oneshot::Receiver<Option<BlockReason>>,
    close_handle: CloseHandle,
    retry_attempt: u32,
}

impl ConnectingState {
    fn set_firewall_policy(
        shared_values: &mut SharedTunnelStateValues,
        proxy: &Option<openvpn::ProxySettings>,
        endpoint: Endpoint,
    ) -> Result<()> {
        // If a proxy is specified we need to pass it on as the peer endpoint.
        let peer_endpoint = match proxy {
            Some(proxy_settings) => proxy_settings.get_endpoint(),
            None => endpoint,
        };

        let policy = FirewallPolicy::Connecting {
            peer_endpoint,
            allow_lan: shared_values.allow_lan,
        };
        shared_values
            .firewall
            .apply_policy(policy)
            .chain_err(|| "Failed to apply firewall policy for connecting state")
    }

    fn start_tunnel(
        parameters: TunnelParameters,
        log_dir: &Option<PathBuf>,
        resource_dir: &Path,
        retry_attempt: u32,
    ) -> Result<Self> {
        let (event_tx, event_rx) = mpsc::unbounded();
        let monitor = Self::spawn_tunnel_monitor(&parameters, log_dir, resource_dir, event_tx)?;
        let close_handle = monitor.close_handle();
        let tunnel_close_event = Self::spawn_tunnel_monitor_wait_thread(monitor);

        Ok(ConnectingState {
            tunnel_events: event_rx,
            tunnel_parameters: parameters,
            tunnel_close_event,
            close_handle,
            retry_attempt,
        })
    }

    fn spawn_tunnel_monitor(
        parameters: &TunnelParameters,
        log_dir: &Option<PathBuf>,
        resource_dir: &Path,
        events: mpsc::UnboundedSender<TunnelEvent>,
    ) -> Result<TunnelMonitor> {
        let on_tunnel_event = move |event| {
            let _ = events.unbounded_send(event);
        };
        let log_file = Self::prepare_tunnel_log_file(&parameters, log_dir)?;

        Ok(TunnelMonitor::start(
            &parameters,
            TUNNEL_INTERFACE_ALIAS.to_owned().map(OsString::from),
            log_file.clone(),
            resource_dir,
            on_tunnel_event,
        )?)
    }

    fn prepare_tunnel_log_file(
        parameters: &TunnelParameters,
        log_dir: &Option<PathBuf>,
    ) -> Result<Option<PathBuf>> {
        if let Some(ref log_dir) = log_dir {
            let filename = match parameters {
                TunnelParameters::OpenVpn(_) => OPENVPN_LOG_FILENAME,
                TunnelParameters::Wireguard(_) => WIREGUARD_LOG_FILENAME,
            };
            let tunnel_log = log_dir.join(filename);
            logging::rotate_log(&tunnel_log).chain_err(|| ErrorKind::RotateLogError)?;
            Ok(Some(tunnel_log))
        } else {
            Ok(None)
        }
    }

    fn spawn_tunnel_monitor_wait_thread(
        tunnel_monitor: TunnelMonitor,
    ) -> oneshot::Receiver<Option<BlockReason>> {
        let (tunnel_close_event_tx, tunnel_close_event_rx) = oneshot::channel();

        thread::spawn(move || {
            let start = Instant::now();

            let block_reason = Self::wait_for_tunnel_monitor(tunnel_monitor);

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

        tunnel_close_event_rx
    }

    fn wait_for_tunnel_monitor(tunnel_monitor: TunnelMonitor) -> Option<BlockReason> {
        match tunnel_monitor.wait() {
            Ok(_) => {
                debug!("Tunnel has finished without errors");
                None
            }
            Err(error) => match error {
                #[cfg(windows)]
                error @ tunnel::Error(
                    tunnel::ErrorKind::OpenVpnTunnelMonitoringError(
                        tunnel::openvpn::ErrorKind::DisabledTapAdapter,
                    ),
                    _,
                )
                | error @ tunnel::Error(
                    tunnel::ErrorKind::OpenVpnTunnelMonitoringError(
                        tunnel::openvpn::ErrorKind::MissingTapAdapter,
                    ),
                    _,
                ) => {
                    let chained_error = error.chain_err(|| "TAP adapter problem detected");
                    warn!("{}", chained_error.display_chain());
                    Some(BlockReason::TapAdapterProblem)
                }
                error => {
                    let chained_error = error.chain_err(|| "Tunnel has stopped unexpectedly");
                    warn!("{}", chained_error.display_chain());
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
                match Self::set_firewall_policy(
                    shared_values,
                    &get_openvpn_proxy_settings(&self.tunnel_parameters),
                    self.tunnel_parameters.get_tunnel_endpoint().endpoint,
                ) {
                    Ok(()) => SameState(self),
                    Err(error) => {
                        error!("{}", error.display_chain());

                        NewState(DisconnectingState::enter(
                            shared_values,
                            (
                                self.close_handle,
                                self.tunnel_close_event,
                                AfterDisconnect::Block(BlockReason::SetFirewallPolicyError),
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
            Ok(TunnelEvent::AuthFailed(reason)) => NewState(DisconnectingState::enter(
                shared_values,
                (
                    self.close_handle,
                    self.tunnel_close_event,
                    AfterDisconnect::Block(BlockReason::AuthFailed(reason)),
                ),
            )),
            Ok(TunnelEvent::Up(metadata)) => NewState(ConnectedState::enter(
                shared_values,
                self.into_connected_state_bootstrap(metadata),
            )),
            Ok(_) => SameState(self),
            Err(_) => {
                debug!("The OpenVPN tunnel event plugin disconnected");
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
        match self.tunnel_close_event.poll() {
            Ok(Async::Ready(block_reason)) => {
                if let Some(reason) = block_reason {
                    return EventConsequence::NewState(BlockedState::enter(shared_values, reason));
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
        TunnelParameters::OpenVpn(ref config) => &config.options.proxy,
        _ => &None,
    }
}

impl TunnelState for ConnectingState {
    type Bootstrap = u32;

    fn enter(
        shared_values: &mut SharedTunnelStateValues,
        retry_attempt: u32,
    ) -> (TunnelStateWrapper, TunnelStateTransition) {
        if shared_values.is_offline {
            return BlockedState::enter(shared_values, BlockReason::IsOffline);
        }
        match shared_values
            .tunnel_parameters_generator
            .generate(retry_attempt)
        {
            None => BlockedState::enter(shared_values, BlockReason::NoMatchingRelay),
            Some(tunnel_parameters) => {
                let endpoint = tunnel_parameters.get_tunnel_endpoint().endpoint;
                if let Err(error) = Self::set_firewall_policy(
                    shared_values,
                    &get_openvpn_proxy_settings(&tunnel_parameters),
                    endpoint,
                ) {
                    error!("{}", error.display_chain());
                    BlockedState::enter(shared_values, BlockReason::StartTunnelError)
                } else {
                    match Self::start_tunnel(
                        tunnel_parameters,
                        &shared_values.log_dir,
                        &shared_values.resource_dir,
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
                            let block_reason = match *error.kind() {
                                ErrorKind::TunnelMonitorError(
                                    tunnel::ErrorKind::EnableIpv6Error,
                                ) => BlockReason::Ipv6Unavailable,
                                _ => BlockReason::StartTunnelError,
                            };

                            let chained_error = error.chain_err(|| "Failed to start tunnel");
                            error!("{}", chained_error.display_chain());

                            BlockedState::enter(shared_values, block_reason)
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

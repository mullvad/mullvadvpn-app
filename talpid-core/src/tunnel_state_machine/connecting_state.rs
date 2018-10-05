use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::thread;
use std::time::{Duration, Instant};

use error_chain::ChainedError;
use futures::sync::{mpsc, oneshot};
use futures::{Async, Future, Stream};

use talpid_types::net::{TunnelEndpoint, TunnelEndpointData};
use talpid_types::tunnel::BlockReason;

use super::{
    tunnel_event_channel, AfterDisconnect, BlockedState, ConnectedState, ConnectedStateBootstrap,
    DisconnectingState, EventConsequence, SharedTunnelStateValues, TunnelCommand,
    TunnelEventReceiver, TunnelEventSender, TunnelParameters, TunnelState, TunnelStateTransition,
    TunnelStateWrapper,
};
use logging;
use security::SecurityPolicy;
use tunnel::{self, CloseHandle, TunnelEvent, TunnelMetadata, TunnelMonitor};

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
    tunnel_events: TunnelEventReceiver,
    tunnel_parameters: TunnelParameters,
    tunnel_close_event: oneshot::Receiver<()>,
    close_handle: CloseHandle,
}

impl ConnectingState {
    fn set_security_policy(
        shared_values: &mut SharedTunnelStateValues,
        endpoint: TunnelEndpoint,
    ) -> Result<()> {
        let policy = SecurityPolicy::Connecting {
            relay_endpoint: endpoint.to_endpoint(),
            allow_lan: shared_values.allow_lan,
        };
        shared_values
            .security
            .apply_policy(policy)
            .chain_err(|| "Failed to apply security policy for connecting state")
    }

    fn start_tunnel(
        parameters: TunnelParameters,
        log_dir: &Option<PathBuf>,
        resource_dir: &Path,
    ) -> Result<Self> {
        let (event_tx, event_rx) = tunnel_event_channel();
        let monitor = Self::spawn_tunnel_monitor(&parameters, log_dir, resource_dir, event_tx)?;
        let close_handle = monitor.close_handle();
        let tunnel_close_event = Self::spawn_tunnel_monitor_wait_thread(monitor);

        Ok(ConnectingState {
            tunnel_events: event_rx,
            tunnel_parameters: parameters,
            tunnel_close_event,
            close_handle,
        })
    }

    fn spawn_tunnel_monitor(
        parameters: &TunnelParameters,
        log_dir: &Option<PathBuf>,
        resource_dir: &Path,
        events: TunnelEventSender,
    ) -> Result<TunnelMonitor> {
        let on_tunnel_event = move |event| {
            if events.send(event).is_err() {
                warn!("Tunnel state machine stopped before tunnel event was received");
            }
        };
        let log_file = Self::prepare_tunnel_log_file(&parameters, log_dir)?;

        Ok(TunnelMonitor::new(
            parameters.endpoint,
            &parameters.options,
            TUNNEL_INTERFACE_ALIAS.to_owned().map(OsString::from),
            &parameters.username,
            log_file.as_ref().map(PathBuf::as_path),
            resource_dir,
            on_tunnel_event,
        )?)
    }

    fn prepare_tunnel_log_file(
        parameters: &TunnelParameters,
        log_dir: &Option<PathBuf>,
    ) -> Result<Option<PathBuf>> {
        if let Some(ref log_dir) = log_dir {
            let filename = match parameters.endpoint.tunnel {
                TunnelEndpointData::OpenVpn(_) => OPENVPN_LOG_FILENAME,
                TunnelEndpointData::Wireguard(_) => WIREGUARD_LOG_FILENAME,
            };
            let tunnel_log = log_dir.join(filename);
            logging::rotate_log(&tunnel_log).chain_err(|| ErrorKind::RotateLogError)?;
            Ok(Some(tunnel_log))
        } else {
            Ok(None)
        }
    }

    fn spawn_tunnel_monitor_wait_thread(tunnel_monitor: TunnelMonitor) -> oneshot::Receiver<()> {
        let (tunnel_close_event_tx, tunnel_close_event_rx) = oneshot::channel();

        thread::spawn(move || {
            let start = Instant::now();

            match tunnel_monitor.wait() {
                Ok(_) => debug!("Tunnel has finished without errors"),
                Err(error) => {
                    let chained_error = error.chain_err(|| "Tunnel has stopped unexpectedly");
                    warn!("{}", chained_error.display_chain());
                }
            }

            if let Some(remaining_time) = MIN_TUNNEL_ALIVE_TIME.checked_sub(start.elapsed()) {
                thread::sleep(remaining_time);
            }

            if tunnel_close_event_tx.send(()).is_err() {
                warn!("Tunnel state machine stopped before receiving tunnel closed event");
            }

            trace!("Tunnel monitor thread exit");
        });

        tunnel_close_event_rx
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
                match Self::set_security_policy(shared_values, self.tunnel_parameters.endpoint) {
                    Ok(()) => SameState(self),
                    Err(error) => {
                        error!("{}", error.display_chain());

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
            Ok(TunnelCommand::Connect(parameters)) => {
                if parameters != self.tunnel_parameters {
                    NewState(DisconnectingState::enter(
                        shared_values,
                        (
                            self.close_handle,
                            self.tunnel_close_event,
                            AfterDisconnect::Reconnect(parameters),
                        ),
                    ))
                } else {
                    SameState(self)
                }
            }
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
                        AfterDisconnect::Reconnect(self.tunnel_parameters),
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
            Ok(Async::Ready(_)) => {}
            Ok(Async::NotReady) => return EventConsequence::NoEvents(self),
            Err(_cancelled) => warn!("Tunnel monitor thread has stopped unexpectedly"),
        }

        info!("Tunnel closed. Reconnecting.");
        EventConsequence::NewState(ConnectingState::enter(
            shared_values,
            self.tunnel_parameters,
        ))
    }
}

impl TunnelState for ConnectingState {
    type Bootstrap = TunnelParameters;

    fn enter(
        shared_values: &mut SharedTunnelStateValues,
        parameters: Self::Bootstrap,
    ) -> (TunnelStateWrapper, TunnelStateTransition) {
        if let Err(error) = Self::set_security_policy(shared_values, parameters.endpoint) {
            error!("{}", error.display_chain());
            return BlockedState::enter(shared_values, BlockReason::StartTunnelError);
        }

        match Self::start_tunnel(
            parameters,
            &shared_values.log_dir,
            &shared_values.resource_dir,
        ) {
            Ok(connecting_state) => (
                TunnelStateWrapper::from(connecting_state),
                TunnelStateTransition::Connecting,
            ),
            Err(error) => {
                let block_reason = match *error.kind() {
                    ErrorKind::TunnelMonitorError(tunnel::ErrorKind::EnableIpv6Error) => {
                        BlockReason::Ipv6Unavailable
                    }
                    _ => BlockReason::StartTunnelError,
                };

                let chained_error = error.chain_err(|| "Failed to start tunnel");
                error!("{}", chained_error.display_chain());

                BlockedState::enter(shared_values, block_reason)
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

use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::sync::{mpsc as sync_mpsc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

use error_chain::ChainedError;
use futures::future::Shared;
use futures::sink::Wait;
use futures::sync::{mpsc, oneshot};
use futures::{Async, Future, Poll, Sink, Stream};
use tokio_core::reactor::Core;

use mullvad_types::account::AccountToken;
use talpid_core::firewall::{Firewall, FirewallProxy, SecurityPolicy};
use talpid_core::mpsc::IntoSender;
use talpid_core::tunnel::{self, TunnelEvent, TunnelMetadata, TunnelMonitor};
use talpid_types::net::{TunnelEndpoint, TunnelEndpointData, TunnelOptions};

use super::{OPENVPN_LOG_FILENAME, WIREGUARD_LOG_FILENAME};
use logging;

error_chain! {
    errors {
        FirewallError {
            description("Firewall error")
        }
        ReactorError {
            description("Failed to initialize tunnel state machine event loop executor")
        }
        UnknownStartUpError {
            description("Tunnel state machine thread failed to start for an unknown reason")
        }
    }
}

const MIN_TUNNEL_ALIVE_TIME: Duration = Duration::from_millis(1000);

#[cfg(windows)]
const TUNNEL_INTERFACE_ALIAS: Option<&str> = Some("Mullvad");
#[cfg(not(windows))]
const TUNNEL_INTERFACE_ALIAS: Option<&str> = None;

/// Spawn the tunnel state machine thread, returning a channel for sending tunnel requests.
pub fn spawn<P, T>(
    cache_dir: P,
    state_change_listener: IntoSender<TunnelStateInfo, T>,
) -> Result<mpsc::UnboundedSender<TunnelRequest>>
where
    P: AsRef<Path> + Send + 'static,
    T: From<TunnelStateInfo> + Send + 'static,
{
    let (request_tx, request_rx) = mpsc::unbounded();
    let (startup_result_tx, startup_result_rx) = sync_mpsc::channel();

    thread::spawn(
        move || match create_event_loop(cache_dir, request_rx, state_change_listener) {
            Ok((mut reactor, event_loop)) => {
                startup_result_tx.send(Ok(())).expect(
                    "Tunnel state machine won't be started because the owner thread crashed",
                );

                if let Err(error) = reactor.run(event_loop) {
                    let chained_error =
                        Error::with_chain(error, "Tunnel state machine exited with an error");
                    error!("{}", chained_error.display_chain());
                }
            }
            Err(startup_error) => {
                startup_result_tx
                    .send(Err(startup_error))
                    .expect("Failed to send startup error");
            }
        },
    );

    startup_result_rx
        .recv()
        .expect("Failed to start tunnel state machine thread")
        .map(|_| request_tx)
}

fn create_event_loop<P, T>(
    cache_dir: P,
    requests: mpsc::UnboundedReceiver<TunnelRequest>,
    state_change_listener: IntoSender<TunnelStateInfo, T>,
) -> Result<(Core, impl Future<Item = (), Error = Error>)>
where
    P: AsRef<Path>,
    T: From<TunnelStateInfo> + Send + 'static,
{
    let reactor = Core::new().chain_err(|| ErrorKind::ReactorError)?;
    let state_machine = TunnelStateMachine::new(&cache_dir, requests)?;

    let future = state_machine.for_each(move |state_change_event| {
        state_change_listener
            .send(state_change_event)
            .chain_err(|| "Failed to send state change event to listener")
    });

    Ok((reactor, future))
}

/// Representation of external requests for the tunnel state machine.
pub enum TunnelRequest {
    /// Request to enable or disable LAN access in the firewall.
    AllowLan(bool),
    /// Request a tunnel to be opened.
    Connect(TunnelParameters),
    /// Request the tunnel to restart if it has been previously requested to be opened.
    Reconnect(TunnelParameters),
    /// Request a tunnel to be closed.
    Disconnect,
}

/// Information necessary to open a tunnel.
#[derive(Debug, PartialEq)]
pub struct TunnelParameters {
    pub endpoint: TunnelEndpoint,
    pub options: TunnelOptions,
    pub log_dir: Option<PathBuf>,
    pub resource_dir: PathBuf,
    pub account_token: AccountToken,
    pub allow_lan: bool,
}

/// Description of the tunnel states.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum TunnelStateInfo {
    Disconnected,
    Connecting,
    Connected,
    Disconnecting,
    Reconnecting,
}

/// Asynchronous handling of the tunnel state machine.
///
/// This type implements `Stream`, and attempts to advance the state machine based on the events
/// received on the requests stream and possibly on events that specific states are also listening
/// to. Every time it successfully advances the state machine a `TunnelStateInfo` is emitted by the
/// stream.
struct TunnelStateMachine {
    current_state: Option<TunnelState>,
    requests: mpsc::UnboundedReceiver<TunnelRequest>,
    shared_values: SharedTunnelStateValues,
}

impl TunnelStateMachine {
    fn new<P: AsRef<Path>>(
        cache_dir: P,
        requests: mpsc::UnboundedReceiver<TunnelRequest>,
    ) -> Result<Self> {
        let firewall = FirewallProxy::new(cache_dir).chain_err(|| ErrorKind::FirewallError)?;
        let shared_values = SharedTunnelStateValues { firewall };

        Ok(TunnelStateMachine {
            current_state: Some(TunnelState::from(DisconnectedState)),
            requests,
            shared_values,
        })
    }
}

impl Stream for TunnelStateMachine {
    type Item = TunnelStateInfo;
    type Error = Error;

    fn poll(&mut self) -> Poll<Option<Self::Item>, Self::Error> {
        use self::TunnelStateTransition::*;

        let mut state = self
            .current_state
            .take()
            .ok_or_else(|| Error::from("State machine lost track of its state!"))?;
        let mut result = Ok(Async::Ready(None));
        let mut event_was_ignored = true;

        while event_was_ignored {
            let transition = state.handle_event(&mut self.requests, &mut self.shared_values);

            event_was_ignored = match transition {
                SameState(_) => true,
                NewState(_) | NoEvents(_) => false,
            };

            result = match transition {
                NewState(ref state) => Ok(Async::Ready(Some(state.info()))),
                SameState(_) => result,
                NoEvents(_) => Ok(Async::NotReady),
            };

            state = transition.into_tunnel_state();
        }

        self.current_state = Some(state);

        result
    }
}

/// Values that are common to all tunnel states.
struct SharedTunnelStateValues {
    firewall: FirewallProxy,
}

/// Asynchronous result of an attempt to progress a state.
enum TunnelStateTransition<T: TunnelStateProgress> {
    /// Transition to a new state.
    NewState(TunnelState),
    /// An event was received, but it was ignored by the state so no transition is performed.
    SameState(T),
    /// No events were received, the event loop should block until one becomes available.
    NoEvents(T),
}

impl<T: TunnelStateProgress> TunnelStateTransition<T> {
    /// Helper method to chain handling multiple different event types.
    ///
    /// The `handle_event` is only called if no events were handled so far.
    pub fn or_else<F>(self, handle_event: F, shared_values: &mut SharedTunnelStateValues) -> Self
    where
        F: FnOnce(T, &mut SharedTunnelStateValues) -> Self,
    {
        use self::TunnelStateTransition::*;

        match self {
            NewState(state) => NewState(state),
            SameState(state) => SameState(state),
            NoEvents(state) => handle_event(state, shared_values),
        }
    }
}

impl<T> TunnelStateTransition<T>
where
    T: TunnelStateProgress,
    TunnelState: From<T>,
{
    /// Extracts the destination state as a `TunnelState`.
    pub fn into_tunnel_state(self) -> TunnelState {
        use self::TunnelStateTransition::*;

        match self {
            NewState(tunnel_state) => tunnel_state,
            SameState(state) | NoEvents(state) => TunnelState::from(state),
        }
    }
}

/// Trait that contains the method all states should implement to handle an event and advance the
/// state machine.
trait TunnelStateProgress: Sized {
    /// Main state function.
    ///
    /// This is the state entry point. It consumes itself and returns the next state to advance to
    /// when it has completed, or itself if it wants to ignore a received event or if no events were
    /// ready to be received. See [`TunnelStateTransition`] for more details.
    ///
    /// An implementation can handle events from many sources, but it should also handle request
    /// events received through the provided `requests` stream.
    ///
    /// [`TunnelStateTransition`]: enum.TunnelStateTransition.html
    fn handle_event(
        self,
        requests: &mut mpsc::UnboundedReceiver<TunnelRequest>,
        shared_values: &mut SharedTunnelStateValues,
    ) -> TunnelStateTransition<Self>;
}

/// Try to receive an event from a `Stream`'s asynchronous poll expression.
///
/// This macro is similar to the `try_ready!` macro provided in `futures`. If there is an event
/// ready, it will be returned wrapped in a `Result`. If there are no events ready to be received,
/// the outer function will return with a transition that indicates that no events were received,
/// which is analogous to `Async::NotReady`.
///
/// When the asynchronous event indicates that the stream has finished or that it has failed, an
/// error type is returned so that either close scenario can be handled in a similar way.
macro_rules! try_handle_event {
    ($same_state:expr, $event:expr) => {
        match $event {
            Ok(Async::Ready(Some(event))) => Ok(event),
            Ok(Async::Ready(None)) => Err(None),
            Ok(Async::NotReady) => return TunnelStateTransition::NoEvents($same_state),
            Err(error) => Err(Some(error)),
        }
    };
}

/// Valid states of the tunnel.
///
/// All implementations must implement `TunnelStateProgress` so that they can handle events and
/// requests in order to advance the state machine.
enum TunnelState {
    Disconnected(DisconnectedState),
    Connecting(ConnectingState),
    Connected(ConnectedState),
    Disconnecting(DisconnectingState),
    Reconnecting(ReconnectingState),
}

impl TunnelState {
    /// Returns information describing the state.
    fn info(&self) -> TunnelStateInfo {
        match *self {
            TunnelState::Disconnected(_) => TunnelStateInfo::Disconnected,
            TunnelState::Connecting(_) => TunnelStateInfo::Connecting,
            TunnelState::Connected(_) => TunnelStateInfo::Connected,
            TunnelState::Disconnecting(_) => TunnelStateInfo::Disconnecting,
            TunnelState::Reconnecting(_) => TunnelStateInfo::Reconnecting,
        }
    }
}

macro_rules! impl_from_for_tunnel_state {
    ($state_variant:ident($state_type:ident)) => {
        impl From<$state_type> for TunnelState {
            fn from(state: $state_type) -> Self {
                TunnelState::$state_variant(state)
            }
        }
    };
}

impl_from_for_tunnel_state!(Disconnected(DisconnectedState));
impl_from_for_tunnel_state!(Connecting(ConnectingState));
impl_from_for_tunnel_state!(Connected(ConnectedState));
impl_from_for_tunnel_state!(Disconnecting(DisconnectingState));
impl_from_for_tunnel_state!(Reconnecting(ReconnectingState));

impl TunnelStateProgress for TunnelState {
    /// Main state function.
    ///
    /// This is the state entry point. It consumes itself and returns the next state to advance to
    /// when it has completed, or `None` if the requests channel has closed. The requests channel
    /// contains `TunnelRequest` events that are handled by the state to advance the state machine.
    fn handle_event(
        self,
        requests: &mut mpsc::UnboundedReceiver<TunnelRequest>,
        shared_values: &mut SharedTunnelStateValues,
    ) -> TunnelStateTransition<TunnelState> {
        use self::TunnelStateTransition::*;

        macro_rules! handle_event {
            ( $($state:ident),* $(,)* ) => {
                match self {
                    $(
                        TunnelState::$state(state) => {
                            match state.handle_event(requests, shared_values) {
                                NewState(tunnel_state) => NewState(tunnel_state),
                                SameState(state) => SameState(TunnelState::$state(state)),
                                NoEvents(state) => NoEvents(TunnelState::$state(state)),
                            }
                        }
                    )*
                }
            }
        }

        handle_event! {
            Disconnected,
            Connecting,
            Connected,
            Disconnecting,
            Reconnecting,
        }
    }
}

/// Internal handle to request tunnel to be closed.
struct CloseHandle {
    tunnel_close_handle: tunnel::CloseHandle,
    tunnel_close_event: Shared<oneshot::Receiver<()>>,
}

impl CloseHandle {
    fn new(
        tunnel_close_handle: tunnel::CloseHandle,
        tunnel_close_event: Shared<oneshot::Receiver<()>>,
    ) -> Self {
        CloseHandle {
            tunnel_close_handle,
            tunnel_close_event,
        }
    }

    fn close(self) -> Shared<oneshot::Receiver<()>> {
        let close_result = self
            .tunnel_close_handle
            .close()
            .chain_err(|| "Failed to request tunnel monitor to close the tunnel");

        if let Err(error) = close_result {
            error!("{}", error.display_chain());
        }

        self.tunnel_close_event
    }
}

/// No tunnel is running.
struct DisconnectedState;

impl DisconnectedState {
    fn new(shared_values: &mut SharedTunnelStateValues) -> TunnelState {
        Self::reset_security_policy(shared_values);

        TunnelState::from(DisconnectedState)
    }

    fn reset_security_policy(shared_values: &mut SharedTunnelStateValues) {
        debug!("Reset security policy");
        if let Err(error) = shared_values.firewall.reset_policy() {
            let chained_error = Error::with_chain(error, "Failed to reset security policy");
            error!("{}", chained_error.display_chain());
        }
    }
}

impl TunnelStateProgress for DisconnectedState {
    fn handle_event(
        self,
        requests: &mut mpsc::UnboundedReceiver<TunnelRequest>,
        shared_values: &mut SharedTunnelStateValues,
    ) -> TunnelStateTransition<Self> {
        use self::TunnelStateTransition::*;

        match try_handle_event!(self, requests.poll()) {
            Ok(TunnelRequest::Connect(parameters)) => {
                NewState(ConnectingState::start(shared_values, parameters))
            }
            _ => SameState(self),
        }
    }
}

/// The tunnel has been started, but it is not established/functional.
struct ConnectingState {
    tunnel_events: mpsc::UnboundedReceiver<TunnelEvent>,
    tunnel_endpoint: TunnelEndpoint,
    tunnel_parameters: TunnelParameters,
    tunnel_close_event: Shared<oneshot::Receiver<()>>,
    close_handle: CloseHandle,
}

impl ConnectingState {
    fn start(
        shared_values: &mut SharedTunnelStateValues,
        parameters: TunnelParameters,
    ) -> TunnelState {
        match Self::new(shared_values, parameters) {
            Ok(connecting) => TunnelState::from(connecting),
            Err(error) => {
                let chained_error = error.chain_err(|| "Failed to start a new tunnel");
                error!("{}", chained_error);
                DisconnectedState::new(shared_values)
            }
        }
    }

    fn restart(
        shared_values: &mut SharedTunnelStateValues,
        parameters: TunnelParameters,
    ) -> TunnelState {
        info!("Tunnel closed. Reconnecting.");
        Self::start(shared_values, parameters)
    }

    fn new(
        shared_values: &mut SharedTunnelStateValues,
        parameters: TunnelParameters,
    ) -> Result<Self> {
        Self::set_security_policy(shared_values, parameters.endpoint, parameters.allow_lan)?;

        let tunnel_endpoint = parameters.endpoint;
        let (tunnel_events, tunnel_close_event, close_handle) = Self::start_tunnel(&parameters)?;

        Ok(ConnectingState {
            tunnel_events,
            tunnel_endpoint,
            tunnel_parameters: parameters,
            tunnel_close_event,
            close_handle,
        })
    }

    fn set_security_policy(
        shared_values: &mut SharedTunnelStateValues,
        endpoint: TunnelEndpoint,
        allow_lan: bool,
    ) -> Result<()> {
        let policy = SecurityPolicy::Connecting {
            relay_endpoint: endpoint.to_endpoint(),
            allow_lan,
        };

        debug!("Set security policy: {:?}", policy);
        shared_values
            .firewall
            .apply_policy(policy)
            .chain_err(|| "Failed to apply security policy for connecting state")
    }

    fn start_tunnel(
        parameters: &TunnelParameters,
    ) -> Result<(
        mpsc::UnboundedReceiver<TunnelEvent>,
        Shared<oneshot::Receiver<()>>,
        CloseHandle,
    )> {
        let (event_tx, event_rx) = mpsc::unbounded();
        let monitor = Self::spawn_tunnel_monitor(&parameters, event_tx.wait())?;
        let tunnel_close_handle = monitor.close_handle();
        let tunnel_close_event = Self::spawn_tunnel_monitor_wait_thread(monitor).shared();
        let close_handle = CloseHandle::new(tunnel_close_handle, tunnel_close_event.clone());

        Ok((event_rx, tunnel_close_event, close_handle))
    }

    fn spawn_tunnel_monitor(
        parameters: &TunnelParameters,
        events: Wait<mpsc::UnboundedSender<TunnelEvent>>,
    ) -> Result<TunnelMonitor> {
        let event_tx = Mutex::new(events);
        let on_tunnel_event = move |event| {
            let send_result = event_tx
                .lock()
                .expect("A thread panicked while sending a tunnel event")
                .send(event);

            if send_result.is_err() {
                warn!("Tunnel state machine stopped before tunnel event was received");
            }
        };
        let log_file = Self::prepare_tunnel_log_file(&parameters)?;

        TunnelMonitor::new(
            parameters.endpoint,
            &parameters.options,
            TUNNEL_INTERFACE_ALIAS.to_owned().map(OsString::from),
            &parameters.account_token,
            log_file.as_ref().map(PathBuf::as_path),
            &parameters.resource_dir,
            on_tunnel_event,
        ).chain_err(|| "Unable to start tunnel monitor")
    }

    fn prepare_tunnel_log_file(parameters: &TunnelParameters) -> Result<Option<PathBuf>> {
        if let Some(ref log_dir) = parameters.log_dir {
            let filename = match parameters.endpoint.tunnel {
                TunnelEndpointData::OpenVpn(_) => OPENVPN_LOG_FILENAME,
                TunnelEndpointData::Wireguard(_) => WIREGUARD_LOG_FILENAME,
            };
            let tunnel_log = log_dir.join(filename);
            logging::rotate_log(&tunnel_log).chain_err(|| "Unable to rotate tunnel log")?;
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
                    warn!("{}", chained_error);
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

    fn handle_requests(
        mut self,
        requests: &mut mpsc::UnboundedReceiver<TunnelRequest>,
        shared_values: &mut SharedTunnelStateValues,
    ) -> TunnelStateTransition<Self> {
        use self::TunnelStateTransition::*;

        match try_handle_event!(self, requests.poll()) {
            Ok(TunnelRequest::Connect(parameters)) => {
                if parameters != self.tunnel_parameters {
                    NewState(ReconnectingState::wait_for(self.close_handle, parameters))
                } else {
                    SameState(self)
                }
            }
            Ok(TunnelRequest::Reconnect(parameters)) => {
                NewState(ReconnectingState::wait_for(self.close_handle, parameters))
            }
            Ok(TunnelRequest::Disconnect) | Err(_) => {
                NewState(DisconnectingState::wait_for(self.close_handle))
            }
            Ok(TunnelRequest::AllowLan(allow_lan)) => {
                self.tunnel_parameters.allow_lan = allow_lan;
                match Self::set_security_policy(shared_values, self.tunnel_endpoint, allow_lan) {
                    Ok(()) => SameState(self),
                    Err(error) => {
                        error!("{}", error.chain_err(|| "Failed to update security policy"));
                        NewState(DisconnectingState::wait_for(self.close_handle))
                    }
                }
            }
        }
    }

    fn handle_tunnel_events(
        mut self,
        shared_values: &mut SharedTunnelStateValues,
    ) -> TunnelStateTransition<Self> {
        use self::TunnelStateTransition::*;

        match try_handle_event!(self, self.tunnel_events.poll()) {
            Ok(TunnelEvent::Up(metadata)) => NewState(ConnectedState::new(
                shared_values,
                metadata,
                self.tunnel_events,
                self.tunnel_endpoint,
                self.tunnel_parameters,
                self.tunnel_close_event,
                self.close_handle,
            )),
            Ok(_) => SameState(self),
            Err(_) => NewState(ReconnectingState::wait_for(
                self.close_handle,
                self.tunnel_parameters,
            )),
        }
    }

    fn handle_tunnel_close_event(
        mut self,
        shared_values: &mut SharedTunnelStateValues,
    ) -> TunnelStateTransition<Self> {
        use self::TunnelStateTransition::*;

        match self.tunnel_close_event.poll() {
            Ok(Async::Ready(_)) => {}
            Ok(Async::NotReady) => return NoEvents(self),
            Err(_cancelled) => warn!("Tunnel monitor thread has stopped unexpectedly"),
        }

        NewState(ConnectingState::restart(
            shared_values,
            self.tunnel_parameters,
        ))
    }
}

impl TunnelStateProgress for ConnectingState {
    fn handle_event(
        self,
        requests: &mut mpsc::UnboundedReceiver<TunnelRequest>,
        shared_values: &mut SharedTunnelStateValues,
    ) -> TunnelStateTransition<Self> {
        self.handle_requests(requests, shared_values)
            .or_else(Self::handle_tunnel_events, shared_values)
            .or_else(Self::handle_tunnel_close_event, shared_values)
    }
}

/// The tunnel is up and working.
struct ConnectedState {
    tunnel_events: mpsc::UnboundedReceiver<TunnelEvent>,
    tunnel_endpoint: TunnelEndpoint,
    metadata: TunnelMetadata,
    tunnel_parameters: TunnelParameters,
    tunnel_close_event: Shared<oneshot::Receiver<()>>,
    close_handle: CloseHandle,
}

impl ConnectedState {
    fn new(
        shared_values: &mut SharedTunnelStateValues,
        metadata: TunnelMetadata,
        tunnel_events: mpsc::UnboundedReceiver<TunnelEvent>,
        tunnel_endpoint: TunnelEndpoint,
        tunnel_parameters: TunnelParameters,
        tunnel_close_event: Shared<oneshot::Receiver<()>>,
        close_handle: CloseHandle,
    ) -> TunnelState {
        let allow_lan = tunnel_parameters.allow_lan;

        match Self::set_security_policy(shared_values, tunnel_endpoint, metadata.clone(), allow_lan)
        {
            Ok(()) => ConnectedState {
                tunnel_events,
                tunnel_endpoint,
                metadata,
                tunnel_parameters,
                tunnel_close_event,
                close_handle,
            }.into(),
            Err(error) => {
                error!("{}", error);
                DisconnectingState::wait_for(close_handle)
            }
        }
    }

    fn set_security_policy(
        shared_values: &mut SharedTunnelStateValues,
        endpoint: TunnelEndpoint,
        metadata: TunnelMetadata,
        allow_lan: bool,
    ) -> Result<()> {
        let policy = SecurityPolicy::Connected {
            relay_endpoint: endpoint.to_endpoint(),
            tunnel: metadata,
            allow_lan,
        };

        debug!("Set security policy: {:?}", policy);
        shared_values
            .firewall
            .apply_policy(policy)
            .chain_err(|| "Failed to apply security policy for connected state")
    }

    fn handle_requests(
        mut self,
        requests: &mut mpsc::UnboundedReceiver<TunnelRequest>,
        shared_values: &mut SharedTunnelStateValues,
    ) -> TunnelStateTransition<Self> {
        use self::TunnelStateTransition::*;

        match try_handle_event!(self, requests.poll()) {
            Ok(TunnelRequest::Connect(parameters)) => {
                if parameters != self.tunnel_parameters {
                    NewState(ReconnectingState::wait_for(self.close_handle, parameters))
                } else {
                    SameState(self)
                }
            }
            Ok(TunnelRequest::Reconnect(parameters)) => {
                NewState(ReconnectingState::wait_for(self.close_handle, parameters))
            }
            Ok(TunnelRequest::Disconnect) | Err(_) => {
                NewState(DisconnectingState::wait_for(self.close_handle))
            }
            Ok(TunnelRequest::AllowLan(allow_lan)) => {
                self.tunnel_parameters.allow_lan = allow_lan;

                let set_security_policy_result = Self::set_security_policy(
                    shared_values,
                    self.tunnel_endpoint,
                    self.metadata.clone(),
                    allow_lan,
                );

                match set_security_policy_result {
                    Ok(()) => SameState(self),
                    Err(error) => {
                        error!("{}", error.chain_err(|| "Failed to update security policy"));
                        NewState(DisconnectingState::wait_for(self.close_handle))
                    }
                }
            }
        }
    }

    fn handle_tunnel_events(
        mut self,
        _shared_values: &mut SharedTunnelStateValues,
    ) -> TunnelStateTransition<Self> {
        use self::TunnelStateTransition::*;

        match try_handle_event!(self, self.tunnel_events.poll()) {
            Ok(TunnelEvent::Down) | Err(_) => NewState(ReconnectingState::wait_for(
                self.close_handle,
                self.tunnel_parameters,
            )),
            Ok(_) => SameState(self),
        }
    }

    fn handle_tunnel_close_event(
        mut self,
        shared_values: &mut SharedTunnelStateValues,
    ) -> TunnelStateTransition<Self> {
        use self::TunnelStateTransition::*;

        match self.tunnel_close_event.poll() {
            Ok(Async::Ready(_)) => {}
            Ok(Async::NotReady) => return NoEvents(self),
            Err(_cancelled) => warn!("Tunnel monitor thread has stopped unexpectedly"),
        }

        NewState(ConnectingState::restart(
            shared_values,
            self.tunnel_parameters,
        ))
    }
}

impl TunnelStateProgress for ConnectedState {
    fn handle_event(
        self,
        requests: &mut mpsc::UnboundedReceiver<TunnelRequest>,
        shared_values: &mut SharedTunnelStateValues,
    ) -> TunnelStateTransition<Self> {
        self.handle_requests(requests, shared_values)
            .or_else(Self::handle_tunnel_events, shared_values)
            .or_else(Self::handle_tunnel_close_event, shared_values)
    }
}

/// This state is active from when we manually trigger a tunnel kill until the tunnel wait
/// operation (TunnelExit) returned.
struct DisconnectingState {
    exited: Shared<oneshot::Receiver<()>>,
}

impl DisconnectingState {
    fn new(exited: Shared<oneshot::Receiver<()>>) -> TunnelState {
        DisconnectingState { exited }.into()
    }

    fn wait_for(close_handle: CloseHandle) -> TunnelState {
        Self::new(close_handle.close())
    }

    fn handle_requests(
        self,
        requests: &mut mpsc::UnboundedReceiver<TunnelRequest>,
    ) -> TunnelStateTransition<Self> {
        use self::TunnelStateTransition::*;

        match try_handle_event!(self, requests.poll()) {
            Ok(TunnelRequest::Connect(parameters)) => {
                NewState(ReconnectingState::new(self.exited, parameters))
            }
            _ => SameState(self),
        }
    }

    fn handle_exit_event(
        mut self,
        shared_values: &mut SharedTunnelStateValues,
    ) -> TunnelStateTransition<Self> {
        use self::TunnelStateTransition::*;

        match self.exited.poll() {
            Ok(Async::NotReady) => NoEvents(self),
            Ok(Async::Ready(_)) | Err(_) => NewState(DisconnectedState::new(shared_values)),
        }
    }
}

impl TunnelStateProgress for DisconnectingState {
    fn handle_event(
        self,
        requests: &mut mpsc::UnboundedReceiver<TunnelRequest>,
        shared_values: &mut SharedTunnelStateValues,
    ) -> TunnelStateTransition<Self> {
        self.handle_requests(requests)
            .or_else(Self::handle_exit_event, shared_values)
    }
}

/// This state is active when the tunnel is being closed but will be reopened shortly afterwards.
struct ReconnectingState {
    exited: Shared<oneshot::Receiver<()>>,
    parameters: TunnelParameters,
}

impl ReconnectingState {
    fn new(exited: Shared<oneshot::Receiver<()>>, parameters: TunnelParameters) -> TunnelState {
        ReconnectingState { exited, parameters }.into()
    }

    fn wait_for(close_handle: CloseHandle, parameters: TunnelParameters) -> TunnelState {
        Self::new(close_handle.close(), parameters)
    }

    fn handle_requests(
        mut self,
        requests: &mut mpsc::UnboundedReceiver<TunnelRequest>,
    ) -> TunnelStateTransition<Self> {
        use self::TunnelStateTransition::*;

        match try_handle_event!(self, requests.poll()) {
            Ok(TunnelRequest::Connect(parameters)) | Ok(TunnelRequest::Reconnect(parameters)) => {
                self.parameters = parameters;
                SameState(self)
            }
            Ok(TunnelRequest::Disconnect) | Err(_) => {
                NewState(DisconnectingState::new(self.exited))
            }
            Ok(TunnelRequest::AllowLan(allow_lan)) => {
                self.parameters.allow_lan = allow_lan;
                SameState(self)
            }
        }
    }

    fn handle_exit_event(
        mut self,
        shared_values: &mut SharedTunnelStateValues,
    ) -> TunnelStateTransition<Self> {
        use self::TunnelStateTransition::*;

        match self.exited.poll() {
            Ok(Async::NotReady) => NoEvents(self),
            Ok(Async::Ready(_)) | Err(_) => {
                NewState(ConnectingState::start(shared_values, self.parameters))
            }
        }
    }
}

impl TunnelStateProgress for ReconnectingState {
    fn handle_event(
        self,
        requests: &mut mpsc::UnboundedReceiver<TunnelRequest>,
        shared_values: &mut SharedTunnelStateValues,
    ) -> TunnelStateTransition<Self> {
        self.handle_requests(requests)
            .or_else(Self::handle_exit_event, shared_values)
    }
}

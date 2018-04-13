use std::ffi::OsString;
use std::io;
use std::path::PathBuf;
use std::sync::Mutex;
use std::thread;
use std::time::{Duration, Instant};

use error_chain::ChainedError;
use futures::sink::Wait;
use futures::sync::{mpsc, oneshot};
use futures::{Async, Future, Poll, Sink, Stream};
use tokio_core::reactor::Core;

use mullvad_types::account::AccountToken;
use talpid_core::tunnel::{self, TunnelEvent, TunnelMonitor};
use talpid_types::net::{TunnelEndpoint, TunnelEndpointData, TunnelOptions};

use super::{OPENVPN_LOG_FILENAME, WIREGUARD_LOG_FILENAME};
use logging;

error_chain!{}

const MIN_TUNNEL_ALIVE_TIME: Duration = Duration::from_millis(1000);

#[cfg(windows)]
const TUNNEL_INTERFACE_ALIAS: Option<&str> = Some("Mullvad");
#[cfg(not(windows))]
const TUNNEL_INTERFACE_ALIAS: Option<&str> = None;

/// Spawn the tunnel state machine thread, returning a channel for sending tunnel requests.
pub fn spawn() -> mpsc::UnboundedSender<TunnelRequest> {
    let (request_tx, request_rx) = mpsc::unbounded();

    thread::spawn(move || {
        if let Err(error) = event_loop(request_rx) {
            error!("{}", error.display_chain());
        }
    });

    request_tx
}

fn event_loop(requests: mpsc::UnboundedReceiver<TunnelRequest>) -> Result<()> {
    let mut reactor =
        Core::new().chain_err(|| "Failed to initialize tunnel state machine event loop")?;

    let state_machine = TunnelStateMachine::new(requests);

    reactor
        .run(state_machine)
        .chain_err(|| "Tunnel state machine finished with an error")
}

/// Representation of external requests for the tunnel state machine.
pub enum TunnelRequest {
    /// Request a tunnel to be opened.
    Connect(TunnelParameters),
    /// Request a tunnel to be closed.
    Disconnect,
}

/// Information necessary to open a tunnel.
pub struct TunnelParameters {
    pub endpoint: TunnelEndpoint,
    pub options: TunnelOptions,
    pub log_dir: Option<PathBuf>,
    pub resource_dir: PathBuf,
    pub account_token: AccountToken,
}

/// Asynchronous handling of the tunnel state machine.
///
/// This type implements `Future`, and attempts to advance the state machine based on the events
/// received on the requests stream and possibly on events that specific states are also listening
/// to.
struct TunnelStateMachine {
    current_state: Option<TunnelState>,
    requests: mpsc::UnboundedReceiver<TunnelRequest>,
}

impl TunnelStateMachine {
    fn new(requests: mpsc::UnboundedReceiver<TunnelRequest>) -> Self {
        TunnelStateMachine {
            current_state: Some(TunnelState::from(DisconnectedState)),
            requests,
        }
    }
}

impl Future for TunnelStateMachine {
    type Item = ();
    type Error = Error;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        let mut state = self
            .current_state
            .take()
            .ok_or_else(|| Error::from("State machine lost track of its state!"))?;
        let mut event_was_received = true;

        while event_was_received {
            let transition = state.handle_event(&mut self.requests);

            event_was_received = transition.is_because_of_an_event();
            state = transition.into_tunnel_state();
        }

        self.current_state = Some(state);

        Ok(Async::NotReady)
    }
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
    /// Checks if this transition happened after an event was received.
    pub fn is_because_of_an_event(&self) -> bool {
        use self::TunnelStateTransition::*;

        match self {
            NewState(_) | SameState(_) => true,
            NoEvents(_) => false,
        }
    }

    /// Helper method to chain handling multiple different event types.
    ///
    /// The `handle_event` is only called if no events were handled so far.
    pub fn or_else<F>(self, handle_event: F) -> Self
    where
        F: FnOnce(T) -> Self,
    {
        use self::TunnelStateTransition::*;

        match self {
            NewState(state) => NewState(state),
            SameState(state) => SameState(state),
            NoEvents(state) => handle_event(state),
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

macro_rules! impl_from_for_tunnel_state {
    ($($state_type:ident -> $state_variant:ident),* $(,)*) => {
        $(
            impl From<$state_type> for TunnelState {
                fn from(state: $state_type) -> Self {
                    TunnelState::$state_variant(state)
                }
            }
        )*
    };
}

impl_from_for_tunnel_state! {
    DisconnectedState -> Disconnected,
    ConnectingState -> Connecting,
    ConnectedState -> Connected,
    DisconnectingState -> Disconnecting,
    ReconnectingState -> Reconnecting,
}

impl TunnelStateProgress for TunnelState {
    /// Main state function.
    ///
    /// This is the state entry point. It consumes itself and returns the next state to advance to
    /// when it has completed, or `None` if the requests channel has closed. The requests channel
    /// contains `TunnelRequest` events that are handled by the state to advance the state machine.
    fn handle_event(
        self,
        requests: &mut mpsc::UnboundedReceiver<TunnelRequest>,
    ) -> TunnelStateTransition<TunnelState> {
        use self::TunnelStateTransition::*;

        macro_rules! handle_event {
            ( $($state:ident),* $(,)* ) => {
                match self {
                    $(
                        TunnelState::$state(state) => match state.handle_event(requests) {
                            NewState(tunnel_state) => NewState(tunnel_state),
                            SameState(state) => SameState(TunnelState::$state(state)),
                            NoEvents(state) => NoEvents(TunnelState::$state(state)),
                        },
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
}

impl CloseHandle {
    fn new(tunnel_monitor: &TunnelMonitor) -> Self {
        CloseHandle {
            tunnel_close_handle: tunnel_monitor.close_handle(),
        }
    }

    fn close(self) -> oneshot::Receiver<io::Result<()>> {
        let (close_tx, close_rx) = oneshot::channel();

        thread::spawn(move || {
            let _ = close_tx.send(self.tunnel_close_handle.close());
            trace!("Tunnel kill thread exit");
        });

        close_rx
    }
}

/// No tunnel is running.
struct DisconnectedState;

impl TunnelStateProgress for DisconnectedState {
    fn handle_event(
        self,
        requests: &mut mpsc::UnboundedReceiver<TunnelRequest>,
    ) -> TunnelStateTransition<Self> {
        use self::TunnelStateTransition::*;

        match try_handle_event!(self, requests.poll()) {
            Ok(TunnelRequest::Connect(parameters)) => NewState(ConnectingState::start(parameters)),
            Ok(TunnelRequest::Disconnect) | Err(_) => SameState(self),
        }
    }
}

/// The tunnel has been started, but it is not established/functional.
struct ConnectingState {
    close_handle: CloseHandle,
    tunnel_events: mpsc::UnboundedReceiver<TunnelEvent>,
}

impl ConnectingState {
    fn start(parameters: TunnelParameters) -> TunnelState {
        match Self::new(parameters) {
            Ok(connecting) => TunnelState::from(connecting),
            Err(error) => {
                let chained_error = error.chain_err(|| "Failed to start a new tunnel");
                error!("{}", chained_error);
                DisconnectedState.into()
            }
        }
    }

    fn new(parameters: TunnelParameters) -> Result<Self> {
        let (event_tx, event_rx) = mpsc::unbounded();
        let monitor = Self::spawn_tunnel_monitor(parameters, event_tx.wait())?;
        let close_handle = CloseHandle::new(&monitor);

        Self::spawn_tunnel_monitor_wait_thread(monitor);

        Ok(ConnectingState {
            close_handle,
            tunnel_events: event_rx,
        })
    }

    fn spawn_tunnel_monitor(
        parameters: TunnelParameters,
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

    fn spawn_tunnel_monitor_wait_thread(tunnel_monitor: TunnelMonitor) {
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

            trace!("Tunnel monitor thread exit");
        });
    }

    fn handle_requests(
        self,
        requests: &mut mpsc::UnboundedReceiver<TunnelRequest>,
    ) -> TunnelStateTransition<Self> {
        use self::TunnelStateTransition::*;

        match try_handle_event!(self, requests.poll()) {
            Ok(TunnelRequest::Connect(_)) => SameState(self),
            Ok(TunnelRequest::Disconnect) | Err(_) => {
                NewState(DisconnectingState::wait_for(self.close_handle))
            }
        }
    }

    fn handle_tunnel_events(mut self) -> TunnelStateTransition<Self> {
        use self::TunnelStateTransition::*;

        match try_handle_event!(self, self.tunnel_events.poll()) {
            Ok(TunnelEvent::Up(_)) => {
                NewState(ConnectedState::new(self.tunnel_events, self.close_handle))
            }
            Ok(_) => SameState(self),
            Err(_) => NewState(DisconnectingState::wait_for(self.close_handle)),
        }
    }
}

impl TunnelStateProgress for ConnectingState {
    fn handle_event(
        self,
        requests: &mut mpsc::UnboundedReceiver<TunnelRequest>,
    ) -> TunnelStateTransition<Self> {
        self.handle_requests(requests)
            .or_else(Self::handle_tunnel_events)
    }
}

/// The tunnel is up and working.
struct ConnectedState {
    close_handle: CloseHandle,
    tunnel_events: mpsc::UnboundedReceiver<TunnelEvent>,
}

impl ConnectedState {
    fn new(
        tunnel_events: mpsc::UnboundedReceiver<TunnelEvent>,
        close_handle: CloseHandle,
    ) -> TunnelState {
        ConnectedState {
            close_handle,
            tunnel_events,
        }.into()
    }

    fn handle_requests(
        self,
        requests: &mut mpsc::UnboundedReceiver<TunnelRequest>,
    ) -> TunnelStateTransition<Self> {
        use self::TunnelStateTransition::*;

        match try_handle_event!(self, requests.poll()) {
            Ok(TunnelRequest::Connect(_)) => SameState(self),
            Ok(TunnelRequest::Disconnect) | Err(_) => {
                NewState(DisconnectingState::wait_for(self.close_handle))
            }
        }
    }

    fn handle_tunnel_events(mut self) -> TunnelStateTransition<Self> {
        use self::TunnelStateTransition::*;

        match try_handle_event!(self, self.tunnel_events.poll()) {
            Ok(TunnelEvent::Down) | Err(_) => {
                NewState(DisconnectingState::wait_for(self.close_handle))
            }
            Ok(_) => SameState(self),
        }
    }
}

impl TunnelStateProgress for ConnectedState {
    fn handle_event(
        self,
        requests: &mut mpsc::UnboundedReceiver<TunnelRequest>,
    ) -> TunnelStateTransition<Self> {
        self.handle_requests(requests)
            .or_else(Self::handle_tunnel_events)
    }
}

/// This state is active from when we manually trigger a tunnel kill until the tunnel wait
/// operation (TunnelExit) returned.
struct DisconnectingState {
    exited: oneshot::Receiver<io::Result<()>>,
}

impl DisconnectingState {
    fn new(exited: oneshot::Receiver<io::Result<()>>) -> TunnelState {
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
            Ok(TunnelRequest::Disconnect) | Err(_) => SameState(self),
        }
    }

    fn handle_exit_event(mut self) -> TunnelStateTransition<Self> {
        use self::TunnelStateTransition::*;

        match self.exited.poll() {
            Ok(Async::NotReady) => NoEvents(self),
            Ok(Async::Ready(_)) | Err(_) => NewState(DisconnectedState.into()),
        }
    }
}

impl TunnelStateProgress for DisconnectingState {
    fn handle_event(
        self,
        requests: &mut mpsc::UnboundedReceiver<TunnelRequest>,
    ) -> TunnelStateTransition<Self> {
        self.handle_requests(requests)
            .or_else(Self::handle_exit_event)
    }
}

/// This state is active when the tunnel is being closed but will be reopened shortly afterwards.
struct ReconnectingState {
    exited: oneshot::Receiver<io::Result<()>>,
    parameters: TunnelParameters,
}

impl ReconnectingState {
    fn new(exited: oneshot::Receiver<io::Result<()>>, parameters: TunnelParameters) -> TunnelState {
        ReconnectingState { exited, parameters }.into()
    }

    fn wait_for(close_handle: CloseHandle, parameters: TunnelParameters) -> TunnelState {
        Self::new(close_handle.close(), parameters)
    }

    fn handle_requests(
        self,
        requests: &mut mpsc::UnboundedReceiver<TunnelRequest>,
    ) -> TunnelStateTransition<Self> {
        use self::TunnelStateTransition::*;

        match try_handle_event!(self, requests.poll()) {
            Ok(TunnelRequest::Connect(_)) => SameState(self),
            Ok(TunnelRequest::Disconnect) | Err(_) => {
                NewState(DisconnectingState::new(self.exited))
            }
        }
    }

    fn handle_exit_event(mut self) -> TunnelStateTransition<Self> {
        use self::TunnelStateTransition::*;

        match self.exited.poll() {
            Ok(Async::NotReady) => NoEvents(self),
            Ok(Async::Ready(_)) | Err(_) => NewState(ConnectingState::start(self.parameters)),
        }
    }
}

impl TunnelStateProgress for ReconnectingState {
    fn handle_event(
        self,
        requests: &mut mpsc::UnboundedReceiver<TunnelRequest>,
    ) -> TunnelStateTransition<Self> {
        self.handle_requests(requests)
            .or_else(Self::handle_exit_event)
    }
}

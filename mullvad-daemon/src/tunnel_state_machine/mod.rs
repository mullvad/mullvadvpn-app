#[macro_use]
mod macros;

mod connected_state;
mod connecting_state;
mod disconnected_state;
mod disconnecting_state;

use std::fmt::{Debug, Formatter, Result as FmtResult};
use std::io;
use std::path::PathBuf;
use std::thread;

use error_chain::ChainedError;
use futures::sync::{mpsc, oneshot};
use futures::{Async, Poll, Stream};
use tokio_core::reactor::Core;

use mullvad_types::account::AccountToken;
use talpid_core::mpsc::IntoSender;
use talpid_core::tunnel::{self, TunnelMetadata, TunnelMonitor};
use talpid_types::net::{TunnelEndpoint, TunnelOptions};

use self::connected_state::{ConnectedState, ConnectedStateBootstrap};
use self::connecting_state::ConnectingState;
use self::disconnected_state::DisconnectedState;
use self::disconnecting_state::{AfterDisconnect, DisconnectingState};
use super::{OPENVPN_LOG_FILENAME, WIREGUARD_LOG_FILENAME};

error_chain!{}

/// Spawn the tunnel state machine thread, returning a channel for sending tunnel commands.
pub fn spawn<T>(
    state_change_listener: IntoSender<TunnelStateTransition, T>,
) -> mpsc::UnboundedSender<TunnelCommand>
where
    T: From<TunnelStateTransition> + Send + 'static,
{
    let (command_tx, command_rx) = mpsc::unbounded();

    thread::spawn(move || {
        if let Err(error) = event_loop(command_rx, state_change_listener) {
            error!("{}", error.display_chain());
        }
    });

    command_tx
}

fn event_loop<T>(
    commands: mpsc::UnboundedReceiver<TunnelCommand>,
    state_change_listener: IntoSender<TunnelStateTransition, T>,
) -> Result<()>
where
    T: From<TunnelStateTransition> + Send + 'static,
{
    let mut reactor =
        Core::new().chain_err(|| "Failed to initialize tunnel state machine event loop")?;

    let state_machine = TunnelStateMachine::new(commands);

    reactor
        .run(state_machine.for_each(|state_change_event| {
            state_change_listener
                .send(state_change_event)
                .chain_err(|| "Failed to send state change event to listener")
        })).chain_err(|| "Tunnel state machine finished with an error")
}

/// Representation of external commands for the tunnel state machine.
pub enum TunnelCommand {
    /// Open tunnel connection.
    Connect(TunnelParameters),
    /// Close tunnel connection.
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
}

/// Event resulting from a transition to a new tunnel state.
#[derive(Clone, Debug, PartialEq)]
pub enum TunnelStateTransition {
    Disconnected,
    Connecting(TunnelEndpoint),
    Connected(TunnelEndpoint, TunnelMetadata),
    Disconnecting,
}

/// Asynchronous handling of the tunnel state machine.
///
/// This type implements `Stream`, and attempts to advance the state machine based on the events
/// received on the commands stream and possibly on events that specific states are also listening
/// to. Every time it successfully advances the state machine a `TunnelStateTransition` is emitted
/// by the stream.
struct TunnelStateMachine {
    current_state: Option<TunnelStateWrapper>,
    commands: mpsc::UnboundedReceiver<TunnelCommand>,
}

impl TunnelStateMachine {
    fn new(commands: mpsc::UnboundedReceiver<TunnelCommand>) -> Self {
        let initial_state =
            TunnelStateWrapper::enter(()).expect("Failed to create initial tunnel state");

        TunnelStateMachine {
            current_state: Some(initial_state),
            commands,
        }
    }
}

impl Stream for TunnelStateMachine {
    type Item = TunnelStateTransition;
    type Error = Error;

    fn poll(&mut self) -> Poll<Option<Self::Item>, Self::Error> {
        let mut state = match self.current_state.take() {
            Some(state) => state,
            None => {
                // State machine has halted
                return Ok(Async::Ready(None));
            }
        };

        loop {
            let event_consequence = state.handle_event(&mut self.commands);
            let action = TunnelStateMachineAction::from(event_consequence);

            match action {
                TunnelStateMachineAction::Repeat(returned_state) => {
                    state = returned_state;
                }
                TunnelStateMachineAction::Notify(state, result) => {
                    self.current_state = state;
                    return result;
                }
            }
        }
    }
}

/// Action the state machine should take, which is discovered base on an event consequence.
///
/// The action can be to execute another iteration or to notify that something happened. Executing
/// another iteration happens when an event is received and ignored, which causes the tunnel state
/// machine to stay in the same state. The state machine can notify its caller that a state
/// transition has occurred, that it has finished, or that it has paused to wait for new events.
enum TunnelStateMachineAction {
    Repeat(TunnelStateWrapper),
    Notify(
        Option<TunnelStateWrapper>,
        Poll<Option<TunnelStateTransition>, Error>,
    ),
}

impl From<EventConsequence<TunnelStateWrapper>> for TunnelStateMachineAction {
    fn from(event_consequence: EventConsequence<TunnelStateWrapper>) -> Self {
        use self::EventConsequence::*;
        use self::TunnelStateMachineAction::*;

        match event_consequence {
            NewState(Ok(state)) | NewState(Err((_, state))) => {
                let transition = state.info();

                Notify(Some(state), Ok(Async::Ready(Some(transition))))
            }
            SameState(state) => Repeat(state),
            NoEvents(state) => Notify(Some(state), Ok(Async::NotReady)),
        }
    }
}

/// Asynchronous result of an attempt to progress a state.
enum EventConsequence<T: TunnelState> {
    /// Transition to a new state.
    NewState(StateEntryResult),
    /// An event was received, but it was ignored by the state so no transition is performed.
    SameState(T),
    /// No events were received, the event loop should block until one becomes available.
    NoEvents(T),
}

impl<T> EventConsequence<T>
where
    T: TunnelState,
{
    /// Helper method to chain handling multiple different event types.
    ///
    /// The `handle_event` is only called if no events were handled so far.
    pub fn or_else<F>(self, handle_event: F) -> Self
    where
        F: FnOnce(T) -> Self,
    {
        use self::EventConsequence::*;

        match self {
            NoEvents(state) => handle_event(state),
            consequence => consequence,
        }
    }
}

/// Result of entering a `T: TunnelState`.
///
/// It is either the state itself when successful, or an error paired with a fallback state.
type StateEntryResult = ::std::result::Result<TunnelStateWrapper, (Error, TunnelStateWrapper)>;

/// Trait that contains the method all states should implement to handle an event and advance the
/// state machine.
trait TunnelState: Into<TunnelStateWrapper> + Sized {
    /// Type representing extra information required for entering the state.
    type Bootstrap;

    /// Constructor function.
    ///
    /// This is the state entry point. It attempts to enter the state, and may fail by entering an
    /// error or fallback state instead.
    fn enter(bootstrap: Self::Bootstrap) -> StateEntryResult;

    /// Main state function.
    ///
    /// This is state exit point. It consumes itself and returns the next state to advance to when
    /// it has completed, or itself if it wants to ignore a received event or if no events were
    /// ready to be received. See [`EventConsequence`] for more details.
    ///
    /// An implementation can handle events from many sources, but it should also handle command
    /// events received through the provided `commands` stream.
    ///
    /// [`EventConsequence`]: enum.EventConsequence.html
    fn handle_event(
        self,
        commands: &mut mpsc::UnboundedReceiver<TunnelCommand>,
    ) -> EventConsequence<Self>;
}

/// Valid states of the tunnel.
///
/// All implementations must implement `TunnelState` so that they can handle events and
/// commands in order to advance the state machine.
enum TunnelStateWrapper {
    Disconnected(DisconnectedState),
    Connecting(ConnectingState),
    Connected(ConnectedState),
    Disconnecting(DisconnectingState),
}

impl TunnelStateWrapper {
    /// Returns information describing the state.
    fn info(&self) -> TunnelStateTransition {
        match *self {
            TunnelStateWrapper::Disconnected(_) => TunnelStateTransition::Disconnected,
            TunnelStateWrapper::Connecting(ref state) => state.info(),
            TunnelStateWrapper::Connected(ref state) => state.info(),
            TunnelStateWrapper::Disconnecting(_) => TunnelStateTransition::Disconnecting,
        }
    }
}

macro_rules! impl_from_for_tunnel_state {
    ($state_variant:ident($state_type:ident)) => {
        impl From<$state_type> for TunnelStateWrapper {
            fn from(state: $state_type) -> Self {
                TunnelStateWrapper::$state_variant(state)
            }
        }
    };
}

impl_from_for_tunnel_state!(Disconnected(DisconnectedState));
impl_from_for_tunnel_state!(Connecting(ConnectingState));
impl_from_for_tunnel_state!(Connected(ConnectedState));
impl_from_for_tunnel_state!(Disconnecting(DisconnectingState));

impl TunnelState for TunnelStateWrapper {
    type Bootstrap = <DisconnectedState as TunnelState>::Bootstrap;

    fn enter(bootstrap: Self::Bootstrap) -> StateEntryResult {
        DisconnectedState::enter(bootstrap)
    }

    fn handle_event(
        self,
        commands: &mut mpsc::UnboundedReceiver<TunnelCommand>,
    ) -> EventConsequence<TunnelStateWrapper> {
        use self::EventConsequence::*;

        macro_rules! handle_event {
            ( $($state:ident),* $(,)* ) => {
                match self {
                    $(
                        TunnelStateWrapper::$state(state) => match state.handle_event(commands) {
                            NewState(tunnel_state) => NewState(tunnel_state),
                            SameState(state) => SameState(TunnelStateWrapper::$state(state)),
                            NoEvents(state) => NoEvents(TunnelStateWrapper::$state(state)),
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
        }
    }
}

impl Debug for TunnelStateWrapper {
    fn fmt(&self, formatter: &mut Formatter) -> FmtResult {
        use self::TunnelStateWrapper::*;

        match *self {
            Disconnected(_) => write!(formatter, "TunnelStateWrapper::Disconnected(_)"),
            Connecting(_) => write!(formatter, "TunnelStateWrapper::Connecting(_)"),
            Connected(_) => write!(formatter, "TunnelStateWrapper::Connected(_)"),
            Disconnecting(_) => write!(formatter, "TunnelStateWrapper::Disconnecting(_)"),
        }
    }
}

/// Internal handle to request tunnel to be closed.
pub struct CloseHandle {
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

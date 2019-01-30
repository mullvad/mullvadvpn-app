#[macro_use]
mod macros;

mod blocked_state;
mod connected_state;
mod connecting_state;
mod disconnected_state;
mod disconnecting_state;

use std::{
    path::{Path, PathBuf},
    sync::mpsc as sync_mpsc,
    thread,
};

use error_chain::ChainedError;
use futures::{sync::mpsc, Async, Future, Poll, Stream};
use tokio_core::reactor::Core;

use talpid_types::{
    net::TunnelParameters,
    tunnel::{BlockReason, TunnelStateTransition},
};

use self::{
    blocked_state::BlockedState,
    connected_state::{ConnectedState, ConnectedStateBootstrap},
    connecting_state::ConnectingState,
    disconnected_state::DisconnectedState,
    disconnecting_state::{AfterDisconnect, DisconnectingState},
};
use crate::{
    mpsc::IntoSender,
    offline,
    security::{DnsMonitor, NetworkSecurity},
};

error_chain! {
    errors {
        /// An error occurred while setting up the network security.
        NetworkSecurityError {
            description("Network security error")
        }
        /// Unable to start the DNS settings monitor and enforcer.
        DnsMonitorError {
            description("Unable to start the DNS settings enforcer and monitor")
        }
        /// An error occurred while attempting to set up the event loop for the tunnel state
        /// machine.
        ReactorError {
            description("Failed to initialize tunnel state machine event loop executor")
        }
    }
}

/// Spawn the tunnel state machine thread, returning a channel for sending tunnel commands.
pub fn spawn<P, T>(
    allow_lan: bool,
    block_when_disconnected: bool,
    tunnel_parameters_generator: impl TunnelParametersGenerator,
    log_dir: Option<PathBuf>,
    resource_dir: PathBuf,
    cache_dir: P,
    state_change_listener: IntoSender<TunnelStateTransition, T>,
) -> Result<mpsc::UnboundedSender<TunnelCommand>>
where
    P: AsRef<Path> + Send + 'static,
    T: From<TunnelStateTransition> + Send + 'static,
{
    let (command_tx, command_rx) = mpsc::unbounded();
    let offline_monitor = offline::spawn_monitor(command_tx.clone())
        .chain_err(|| "Unable to spawn offline state monitor")?;
    let is_offline = offline::is_offline();

    let (startup_result_tx, startup_result_rx) = sync_mpsc::channel();
    thread::spawn(move || {
        match create_event_loop(
            allow_lan,
            block_when_disconnected,
            is_offline,
            tunnel_parameters_generator,
            log_dir,
            resource_dir,
            cache_dir,
            command_rx,
            state_change_listener,
        ) {
            Ok((mut reactor, event_loop)) => {
                startup_result_tx.send(Ok(())).expect(
                    "Tunnel state machine won't be started because the owner thread crashed",
                );

                if let Err(error) = reactor.run(event_loop) {
                    let chained_error =
                        Error::with_chain(error, "Tunnel state machine exited with an error");
                    log::error!("{}", chained_error.display_chain());
                }
            }
            Err(startup_error) => {
                startup_result_tx
                    .send(Err(startup_error))
                    .expect("Failed to send startup error");
            }
        }
        std::mem::drop(offline_monitor);
    });

    startup_result_rx
        .recv()
        .expect("Failed to start tunnel state machine thread")
        .map(|_| command_tx)
}

fn create_event_loop<T>(
    allow_lan: bool,
    block_when_disconnected: bool,
    is_offline: bool,
    tunnel_parameters_generator: impl TunnelParametersGenerator,
    log_dir: Option<PathBuf>,
    resource_dir: PathBuf,
    cache_dir: impl AsRef<Path>,
    commands: mpsc::UnboundedReceiver<TunnelCommand>,
    state_change_listener: IntoSender<TunnelStateTransition, T>,
) -> Result<(Core, impl Future<Item = (), Error = Error>)>
where
    T: From<TunnelStateTransition> + Send + 'static,
{
    let reactor = Core::new().chain_err(|| ErrorKind::ReactorError)?;
    let state_machine = TunnelStateMachine::new(
        allow_lan,
        block_when_disconnected,
        is_offline,
        tunnel_parameters_generator,
        log_dir,
        resource_dir,
        cache_dir,
        commands,
    )?;

    let future = state_machine.for_each(move |state_change_event| {
        state_change_listener
            .send(state_change_event)
            .chain_err(|| "Failed to send state change event to listener")
    });

    Ok((reactor, future))
}

/// Representation of external commands for the tunnel state machine.
pub enum TunnelCommand {
    /// Enable or disable LAN access in the firewall.
    AllowLan(bool),
    /// Enable or disable the block_when_disconnected feature.
    BlockWhenDisconnected(bool),
    /// Notify the state machine of the connectivity of the device.
    IsOffline(bool),
    /// Open tunnel connection.
    Connect,
    /// Close tunnel connection.
    Disconnect,
    /// Disconnect any open tunnel and block all network access
    Block(BlockReason),
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
    shared_values: SharedTunnelStateValues,
}

impl TunnelStateMachine {
    fn new(
        allow_lan: bool,
        block_when_disconnected: bool,
        is_offline: bool,
        tunnel_parameters_generator: impl TunnelParametersGenerator,
        log_dir: Option<PathBuf>,
        resource_dir: PathBuf,
        cache_dir: impl AsRef<Path>,
        commands: mpsc::UnboundedReceiver<TunnelCommand>,
    ) -> Result<Self> {
        let security = NetworkSecurity::new().chain_err(|| ErrorKind::NetworkSecurityError)?;
        let dns_monitor = DnsMonitor::new(cache_dir).chain_err(|| ErrorKind::DnsMonitorError)?;
        let mut shared_values = SharedTunnelStateValues {
            security,
            dns_monitor,
            allow_lan,
            block_when_disconnected,
            is_offline,
            tunnel_parameters_generator: Box::new(tunnel_parameters_generator),
            log_dir,
            resource_dir,
        };

        let (initial_state, _) = DisconnectedState::enter(&mut shared_values, ());
        Ok(TunnelStateMachine {
            current_state: Some(initial_state),
            commands,
            shared_values,
        })
    }
}

impl Stream for TunnelStateMachine {
    type Item = TunnelStateTransition;
    type Error = Error;

    fn poll(&mut self) -> Poll<Option<Self::Item>, Self::Error> {
        while let Some(state_wrapper) = self.current_state.take() {
            match state_wrapper.handle_event(&mut self.commands, &mut self.shared_values) {
                TunnelStateMachineAction::Repeat(repeat_state_wrapper) => {
                    self.current_state = Some(repeat_state_wrapper);
                }
                TunnelStateMachineAction::Notify(state_wrapper, result) => {
                    self.current_state = state_wrapper;
                    return result;
                }
            }
        }
        Ok(Async::Ready(None))
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

impl<T: TunnelState> From<EventConsequence<T>> for TunnelStateMachineAction {
    fn from(event_consequence: EventConsequence<T>) -> Self {
        use self::{EventConsequence::*, TunnelStateMachineAction::*};

        match event_consequence {
            NewState((state_wrapper, transition)) => {
                Notify(Some(state_wrapper), Ok(Async::Ready(Some(transition))))
            }
            SameState(state) => Repeat(state.into()),
            NoEvents(state) => Notify(Some(state.into()), Ok(Async::NotReady)),
            Finished => Notify(None, Ok(Async::Ready(None))),
        }
    }
}

/// Trait for any type that can provide a stream of `TunnelParameters` to the `TunnelStateMachine`.
pub trait TunnelParametersGenerator: Send + 'static {
    /// Given the number of consecutive failed retry attempts, it should yield a `TunnelParameters`
    /// to establish a tunnel with.
    /// If this returns `None` then the state machine goes into the `Blocked` state.
    fn generate(&mut self, retry_attempt: u32) -> Option<TunnelParameters>;
}

/// Values that are common to all tunnel states.
struct SharedTunnelStateValues {
    security: NetworkSecurity,
    dns_monitor: DnsMonitor,
    /// Should LAN access be allowed outside the tunnel.
    allow_lan: bool,
    /// Should network access be allowed when in the disconnected state.
    block_when_disconnected: bool,
    /// True when the computer is known to be offline.
    is_offline: bool,
    /// The generator of new `TunnelParameter`s
    tunnel_parameters_generator: Box<dyn TunnelParametersGenerator>,
    /// Directory to store tunnel log file.
    log_dir: Option<PathBuf>,
    /// Resource directory path.
    resource_dir: PathBuf,
}

/// Asynchronous result of an attempt to progress a state.
enum EventConsequence<T: TunnelState> {
    /// Transition to a new state.
    NewState((TunnelStateWrapper, TunnelStateTransition)),
    /// An event was received, but it was ignored by the state so no transition is performed.
    SameState(T),
    /// No events were received, the event loop should block until one becomes available.
    NoEvents(T),
    /// The state machine has finished its execution.
    Finished,
}

impl<T> EventConsequence<T>
where
    T: TunnelState,
{
    /// Helper method to chain handling multiple different event types.
    ///
    /// The `handle_event` is only called if no events were handled so far.
    pub fn or_else<F>(self, handle_event: F, shared_values: &mut SharedTunnelStateValues) -> Self
    where
        F: FnOnce(T, &mut SharedTunnelStateValues) -> Self,
    {
        use self::EventConsequence::*;

        match self {
            NoEvents(state) => handle_event(state, shared_values),
            consequence => consequence,
        }
    }
}

/// Trait that contains the method all states should implement to handle an event and advance the
/// state machine.
trait TunnelState: Into<TunnelStateWrapper> + Sized {
    /// Type representing extra information required for entering the state.
    type Bootstrap;

    /// Constructor function.
    ///
    /// This is the state entry point. It attempts to enter the state, and may fail by entering an
    /// error or fallback state instead.
    fn enter(
        shared_values: &mut SharedTunnelStateValues,
        bootstrap: Self::Bootstrap,
    ) -> (TunnelStateWrapper, TunnelStateTransition);

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
        shared_values: &mut SharedTunnelStateValues,
    ) -> EventConsequence<Self>;
}

macro_rules! state_wrapper {
    (enum $wrapper_name:ident { $($state_variant:ident($state_type:ident)),* $(,)* }) => {
        /// Valid states of the tunnel.
        ///
        /// All implementations must implement `TunnelState` so that they can handle events and
        /// commands in order to advance the state machine.
        enum $wrapper_name {
            $($state_variant($state_type),)*
        }

        $(impl From<$state_type> for $wrapper_name {
            fn from(state: $state_type) -> Self {
                $wrapper_name::$state_variant(state)
            }
        })*

        impl $wrapper_name {
            fn handle_event(
                self,
                commands: &mut mpsc::UnboundedReceiver<TunnelCommand>,
                shared_values: &mut SharedTunnelStateValues,
            ) -> TunnelStateMachineAction {
                match self {
                    $($wrapper_name::$state_variant(state) => {
                        let event_consequence = state.handle_event(commands, shared_values);
                        TunnelStateMachineAction::from(event_consequence)
                    })*
                }
            }
        }
    }
}

state_wrapper! {
    enum TunnelStateWrapper {
        Disconnected(DisconnectedState),
        Connecting(ConnectingState),
        Connected(ConnectedState),
        Disconnecting(DisconnectingState),
        Blocked(BlockedState),
    }
}

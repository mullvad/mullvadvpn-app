#[macro_use]
mod macros;

mod connected_state;
mod connecting_state;
mod disconnected_state;
mod disconnecting_state;
mod error_state;

use self::{
    connected_state::{ConnectedState, ConnectedStateBootstrap},
    connecting_state::ConnectingState,
    disconnected_state::DisconnectedState,
    disconnecting_state::{AfterDisconnect, DisconnectingState},
    error_state::ErrorState,
};
use crate::{
    dns::DnsMonitor,
    firewall::{Firewall, FirewallArguments},
    mpsc::Sender,
    offline,
    routing::RouteManager,
    tunnel::tun_provider::TunProvider,
};

use futures::{
    channel::{mpsc, oneshot},
    StreamExt,
};
use futures01::{sync::mpsc as old_mpsc, Async, Poll, Stream};
use std::{
    collections::HashSet,
    io,
    path::{Path, PathBuf},
    sync::{mpsc as sync_mpsc, Arc},
};
#[cfg(target_os = "android")]
use talpid_types::{android::AndroidContext, ErrorExt};
use talpid_types::{
    net::TunnelParameters,
    tunnel::{ErrorStateCause, ParameterGenerationError, TunnelStateTransition},
};

/// Errors that can happen when setting up or using the state machine.
#[derive(err_derive::Error, Debug)]
pub enum Error {
    /// Unable to spawn offline state monitor
    #[error(display = "Unable to spawn offline state monitor")]
    OfflineMonitorError(#[error(source)] crate::offline::Error),

    /// Unable to set up split tunneling
    #[cfg(target_os = "linux")]
    #[error(display = "Failed to initialize split tunneling")]
    InitSplitTunneling(#[error(source)] crate::split_tunnel::Error),

    /// Failed to initialize the system firewall integration.
    #[error(display = "Failed to initialize the system firewall integration")]
    InitFirewallError(#[error(source)] crate::firewall::Error),

    /// Failed to initialize the system DNS manager and monitor.
    #[error(display = "Failed to initialize the system DNS manager and monitor")]
    InitDnsMonitorError(#[error(source)] crate::dns::Error),

    /// Failed to initialize the route manager.
    #[error(display = "Failed to initialize the route manager")]
    InitRouteManagerError(#[error(source)] crate::routing::Error),

    /// Failed to initialize tunnel state machine event loop executor
    #[error(display = "Failed to initialize tunnel state machine event loop executor")]
    ReactorError(#[error(source)] io::Error),

    /// Failed to send state change event to listener
    #[error(display = "Failed to send state change event to listener")]
    SendStateChange,
}

/// Spawn the tunnel state machine thread, returning a channel for sending tunnel commands.
pub async fn spawn(
    allow_lan: bool,
    block_when_disconnected: bool,
    tunnel_parameters_generator: impl TunnelParametersGenerator,
    log_dir: Option<PathBuf>,
    resource_dir: PathBuf,
    cache_dir: impl AsRef<Path> + Send + 'static,
    state_change_listener: impl Sender<TunnelStateTransition> + Send + 'static,
    shutdown_tx: oneshot::Sender<()>,
    reset_firewall: bool,
    #[cfg(target_os = "android")] android_context: AndroidContext,
) -> Result<Arc<mpsc::UnboundedSender<TunnelCommand>>, Error> {
    let (command_tx, mut command_rx) = mpsc::unbounded();
    let command_tx = Arc::new(command_tx);
    let mut offline_monitor = offline::spawn_monitor(
        Arc::downgrade(&command_tx),
        #[cfg(target_os = "android")]
        android_context.clone(),
    )
    .await
    .map_err(Error::OfflineMonitorError)?;
    let is_offline = offline_monitor.is_offline().await;

    let tun_provider = TunProvider::new(
        #[cfg(target_os = "android")]
        android_context,
        #[cfg(target_os = "android")]
        allow_lan,
    );

    // Hide internal 0.1 futures from the client
    let (command_adapter_tx, command_adapter_rx) = old_mpsc::unbounded();
    tokio::spawn(async move {
        while let Some(command) = command_rx.next().await {
            if command_adapter_tx.unbounded_send(command).is_err() {
                log::error!("Failed to forward daemon command");
            }
        }
    });

    let (startup_result_tx, startup_result_rx) = sync_mpsc::channel();
    std::thread::spawn(move || {
        let state_machine = TunnelStateMachine::new(
            allow_lan,
            block_when_disconnected,
            is_offline,
            tunnel_parameters_generator,
            tun_provider,
            log_dir,
            resource_dir,
            cache_dir,
            command_adapter_rx,
            reset_firewall,
        );
        let state_machine = match state_machine {
            Ok(state_machine) => {
                startup_result_tx.send(Ok(())).unwrap();
                state_machine
            }
            Err(error) => {
                startup_result_tx.send(Err(error)).unwrap();
                return;
            }
        };

        let mut iter = state_machine.wait();
        while let Some(Ok(change_event)) = iter.next() {
            if let Err(error) = state_change_listener
                .send(change_event)
                .map_err(|_| Error::SendStateChange)
            {
                log::error!("{}", error);
                break;
            }
        }
        if shutdown_tx.send(()).is_err() {
            log::error!("Can't send shutdown completion to daemon");
        }

        std::mem::drop(offline_monitor);
    });

    startup_result_rx
        .recv()
        .expect("Failed to start tunnel state machine thread")?;
    Ok(command_tx)
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
    Block(ErrorStateCause),
}

/// Asynchronous handling of the tunnel state machine.
///
/// This type implements `Stream`, and attempts to advance the state machine based on the events
/// received on the commands stream and possibly on events that specific states are also listening
/// to. Every time it successfully advances the state machine a `TunnelStateTransition` is emitted
/// by the stream.
struct TunnelStateMachine {
    current_state: Option<TunnelStateWrapper>,
    commands: old_mpsc::UnboundedReceiver<TunnelCommand>,
    shared_values: SharedTunnelStateValues,
}

impl TunnelStateMachine {
    fn new(
        allow_lan: bool,
        block_when_disconnected: bool,
        is_offline: bool,
        tunnel_parameters_generator: impl TunnelParametersGenerator,
        tun_provider: TunProvider,
        log_dir: Option<PathBuf>,
        resource_dir: PathBuf,
        cache_dir: impl AsRef<Path>,
        commands: old_mpsc::UnboundedReceiver<TunnelCommand>,
        reset_firewall: bool,
    ) -> Result<Self, Error> {
        let args = FirewallArguments {
            initialize_blocked: block_when_disconnected || !reset_firewall,
            allow_lan,
        };

        let firewall = Firewall::new(args).map_err(Error::InitFirewallError)?;
        let dns_monitor = DnsMonitor::new(cache_dir).map_err(Error::InitDnsMonitorError)?;
        let route_manager =
            RouteManager::new(HashSet::new()).map_err(Error::InitRouteManagerError)?;
        let mut shared_values = SharedTunnelStateValues {
            firewall,
            dns_monitor,
            route_manager,
            allow_lan,
            block_when_disconnected,
            is_offline,
            tunnel_parameters_generator: Box::new(tunnel_parameters_generator),
            tun_provider,
            log_dir,
            resource_dir,
        };

        let (initial_state, _) = DisconnectedState::enter(&mut shared_values, reset_firewall);

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
    /// If this returns `None` then the state machine goes into the `Error` state.
    fn generate(
        &mut self,
        retry_attempt: u32,
    ) -> Result<TunnelParameters, ParameterGenerationError>;
}

/// Values that are common to all tunnel states.
struct SharedTunnelStateValues {
    firewall: Firewall,
    dns_monitor: DnsMonitor,
    route_manager: RouteManager,
    /// Should LAN access be allowed outside the tunnel.
    allow_lan: bool,
    /// Should network access be allowed when in the disconnected state.
    block_when_disconnected: bool,
    /// True when the computer is known to be offline.
    is_offline: bool,
    /// The generator of new `TunnelParameter`s
    tunnel_parameters_generator: Box<dyn TunnelParametersGenerator>,
    /// The provider of tunnel devices.
    tun_provider: TunProvider,
    /// Directory to store tunnel log file.
    log_dir: Option<PathBuf>,
    /// Resource directory path.
    resource_dir: PathBuf,
}

impl SharedTunnelStateValues {
    pub fn set_allow_lan(&mut self, allow_lan: bool) -> Result<(), ErrorStateCause> {
        if self.allow_lan != allow_lan {
            self.allow_lan = allow_lan;

            #[cfg(target_os = "android")]
            {
                if let Err(error) = self.tun_provider.set_allow_lan(allow_lan) {
                    log::error!(
                        "{}",
                        error.display_chain_with_msg(&format!(
                            "Failed to restart tunnel after {} LAN connections",
                            if allow_lan { "allowing" } else { "blocking" }
                        ))
                    );
                    return Err(ErrorStateCause::StartTunnelError);
                }
            }
        }

        Ok(())
    }
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
        commands: &mut old_mpsc::UnboundedReceiver<TunnelCommand>,
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
                commands: &mut old_mpsc::UnboundedReceiver<TunnelCommand>,
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
        Error(ErrorState),
    }
}

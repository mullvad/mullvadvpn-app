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
#[cfg(windows)]
use crate::split_tunnel;
use crate::{
    dns::DnsMonitor,
    firewall::{Firewall, FirewallArguments},
    mpsc::Sender,
    offline,
    routing::RouteManager,
    tunnel::{tun_provider::TunProvider, TunnelEvent},
};
#[cfg(windows)]
use std::ffi::OsString;

use futures::{
    channel::{mpsc, oneshot},
    stream, StreamExt,
};
#[cfg(target_os = "android")]
use std::os::unix::io::RawFd;
use std::{
    collections::HashSet,
    io,
    net::IpAddr,
    path::{Path, PathBuf},
    sync::{mpsc as sync_mpsc, Arc},
};
#[cfg(target_os = "android")]
use talpid_types::{android::AndroidContext, ErrorExt};
use talpid_types::{
    net::{Endpoint, TunnelParameters},
    tunnel::{ErrorStateCause, ParameterGenerationError, TunnelStateTransition},
};

/// Errors that can happen when setting up or using the state machine.
#[derive(err_derive::Error, Debug)]
pub enum Error {
    /// Unable to spawn offline state monitor
    #[error(display = "Unable to spawn offline state monitor")]
    OfflineMonitorError(#[error(source)] crate::offline::Error),

    /// Unable to set up split tunneling
    #[cfg(target_os = "windows")]
    #[error(display = "Failed to initialize split tunneling")]
    InitSplitTunneling(#[error(source)] split_tunnel::Error),

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

/// Settings used to initialize the tunnel state machine.
pub struct InitialTunnelState {
    /// Whether to allow LAN traffic when not in the (non-blocking) disconnected state.
    pub allow_lan: bool,
    /// Block traffic unless connected to the VPN.
    pub block_when_disconnected: bool,
    /// DNS servers to use. If `None`, the tunnel gateway is used.
    pub dns_servers: Option<Vec<IpAddr>>,
    /// A single endpoint that is allowed to communicate outside the tunnel, i.e.
    /// in any of the blocking states.
    pub allowed_endpoint: Endpoint,
    /// Whether to reset any existing firewall rules when initializing the disconnected state.
    pub reset_firewall: bool,
    /// Programs to exclude from the tunnel using the split tunnel driver.
    #[cfg(windows)]
    pub exclude_paths: Vec<OsString>,
}

/// Spawn the tunnel state machine thread, returning a channel for sending tunnel commands.
pub async fn spawn(
    runtime: tokio::runtime::Handle,
    initial_settings: InitialTunnelState,
    tunnel_parameters_generator: impl TunnelParametersGenerator,
    log_dir: Option<PathBuf>,
    resource_dir: PathBuf,
    cache_dir: impl AsRef<Path> + Send + 'static,
    state_change_listener: impl Sender<TunnelStateTransition> + Send + 'static,
    offline_state_listener: mpsc::UnboundedSender<bool>,
    shutdown_tx: oneshot::Sender<()>,
    #[cfg(target_os = "android")] android_context: AndroidContext,
) -> Result<Arc<mpsc::UnboundedSender<TunnelCommand>>, Error> {
    let (command_tx, command_rx) = mpsc::unbounded();
    let command_tx = Arc::new(command_tx);

    let tun_provider = TunProvider::new(
        #[cfg(target_os = "android")]
        android_context.clone(),
        #[cfg(target_os = "android")]
        initial_settings.allow_lan,
        #[cfg(target_os = "android")]
        initial_settings.allowed_endpoint.address.ip(),
        #[cfg(target_os = "android")]
        initial_settings.dns_servers.clone(),
    );

    let (startup_result_tx, startup_result_rx) = sync_mpsc::channel();
    let weak_command_tx = Arc::downgrade(&command_tx);
    std::thread::spawn(move || {
        let state_machine = runtime.block_on(TunnelStateMachine::new(
            runtime.clone(),
            initial_settings,
            weak_command_tx,
            offline_state_listener,
            tunnel_parameters_generator,
            tun_provider,
            log_dir,
            resource_dir,
            cache_dir,
            command_rx,
            #[cfg(target_os = "android")]
            android_context,
        ));
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

        state_machine.run(state_change_listener);

        if shutdown_tx.send(()).is_err() {
            log::error!("Can't send shutdown completion to daemon");
        }
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
    /// Endpoint that should never be blocked.
    /// If an error occurs, the sender is dropped.
    AllowEndpoint(Endpoint, oneshot::Sender<()>),
    /// Set DNS servers to use.
    Dns(Option<Vec<IpAddr>>),
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
    /// Bypass a socket, allowing traffic to flow through outside the tunnel.
    #[cfg(target_os = "android")]
    BypassSocket(RawFd, oneshot::Sender<()>),
    /// Set applications that are allowed to send and receive traffic outside of the tunnel.
    #[cfg(windows)]
    SetExcludedApps(
        oneshot::Sender<Result<(), split_tunnel::Error>>,
        Vec<OsString>,
    ),
}

type TunnelCommandReceiver = stream::Fuse<mpsc::UnboundedReceiver<TunnelCommand>>;

enum EventResult {
    Command(Option<TunnelCommand>),
    Event(Option<(TunnelEvent, oneshot::Sender<()>)>),
    Close(Result<Option<ErrorStateCause>, oneshot::Canceled>),
}

/// Asynchronous handling of the tunnel state machine.
///
/// This type implements `Stream`, and attempts to advance the state machine based on the events
/// received on the commands stream and possibly on events that specific states are also listening
/// to. Every time it successfully advances the state machine a `TunnelStateTransition` is emitted
/// by the stream.
struct TunnelStateMachine {
    current_state: Option<TunnelStateWrapper>,
    commands: TunnelCommandReceiver,
    shared_values: SharedTunnelStateValues,
}

impl TunnelStateMachine {
    async fn new(
        runtime: tokio::runtime::Handle,
        settings: InitialTunnelState,
        command_tx: std::sync::Weak<mpsc::UnboundedSender<TunnelCommand>>,
        offline_state_tx: mpsc::UnboundedSender<bool>,
        tunnel_parameters_generator: impl TunnelParametersGenerator,
        tun_provider: TunProvider,
        log_dir: Option<PathBuf>,
        resource_dir: PathBuf,
        cache_dir: impl AsRef<Path>,
        commands_rx: mpsc::UnboundedReceiver<TunnelCommand>,
        #[cfg(target_os = "android")] android_context: AndroidContext,
    ) -> Result<Self, Error> {
        #[cfg(windows)]
        let split_tunnel = split_tunnel::SplitTunnel::new(runtime.clone(), command_tx.clone())
            .map_err(Error::InitSplitTunneling)?;

        let args = FirewallArguments {
            initialize_blocked: settings.block_when_disconnected || !settings.reset_firewall,
            allow_lan: settings.allow_lan,
            allowed_endpoint: Some(settings.allowed_endpoint),
        };

        let firewall = Firewall::new(args).map_err(Error::InitFirewallError)?;
        let route_manager = RouteManager::new(runtime.clone(), HashSet::new())
            .await
            .map_err(Error::InitRouteManagerError)?;
        let dns_monitor = DnsMonitor::new(
            runtime.clone(),
            cache_dir,
            #[cfg(target_os = "linux")]
            route_manager
                .handle()
                .map_err(Error::InitRouteManagerError)?,
        )
        .map_err(Error::InitDnsMonitorError)?;

        let (offline_tx, mut offline_rx) = mpsc::unbounded();
        let initial_offline_state_tx = offline_state_tx.clone();
        tokio::spawn(async move {
            while let Some(offline) = offline_rx.next().await {
                if let Some(tx) = command_tx.upgrade() {
                    let _ = tx.unbounded_send(TunnelCommand::IsOffline(offline));
                } else {
                    break;
                }
                let _ = offline_state_tx.unbounded_send(offline);
            }
        });
        let mut offline_monitor = offline::spawn_monitor(
            offline_tx,
            #[cfg(target_os = "linux")]
            route_manager
                .handle()
                .map_err(Error::InitRouteManagerError)?,
            #[cfg(target_os = "android")]
            android_context,
        )
        .await
        .map_err(Error::OfflineMonitorError)?;
        let is_offline = offline_monitor.is_offline().await;
        let _ = initial_offline_state_tx.unbounded_send(is_offline);

        #[cfg(windows)]
        split_tunnel
            .set_paths_sync(&settings.exclude_paths)
            .map_err(Error::InitSplitTunneling)?;

        let mut shared_values = SharedTunnelStateValues {
            #[cfg(windows)]
            split_tunnel,
            runtime,
            firewall,
            dns_monitor,
            route_manager,
            _offline_monitor: offline_monitor,
            allow_lan: settings.allow_lan,
            block_when_disconnected: settings.block_when_disconnected,
            is_offline,
            dns_servers: settings.dns_servers,
            allowed_endpoint: settings.allowed_endpoint,
            tunnel_parameters_generator: Box::new(tunnel_parameters_generator),
            tun_provider,
            log_dir,
            resource_dir,
            #[cfg(target_os = "linux")]
            connectivity_check_was_enabled: None,
        };

        let (initial_state, _) =
            DisconnectedState::enter(&mut shared_values, settings.reset_firewall);

        Ok(TunnelStateMachine {
            current_state: Some(initial_state),
            commands: commands_rx.fuse(),
            shared_values,
        })
    }

    fn run(mut self, change_listener: impl Sender<TunnelStateTransition> + Send + 'static) {
        use EventConsequence::*;

        let runtime = self.shared_values.runtime.clone();

        while let Some(state_wrapper) = self.current_state.take() {
            match state_wrapper.handle_event(&runtime, &mut self.commands, &mut self.shared_values)
            {
                NewState((state, transition)) => {
                    self.current_state = Some(state);

                    if let Err(error) = change_listener
                        .send(transition)
                        .map_err(|_| Error::SendStateChange)
                    {
                        log::error!("{}", error);
                        break;
                    }
                }
                SameState(state) => {
                    self.current_state = Some(state);
                }
                Finished => (),
            }
        }

        log::debug!("Exiting tunnel state machine loop");
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
    /// Management of excluded apps.
    /// This object should be dropped before deinitializing WinFw (dropping the `Firewall`
    /// instance), since the driver may add filters to the same sublayer.
    #[cfg(windows)]
    split_tunnel: split_tunnel::SplitTunnel,
    runtime: tokio::runtime::Handle,
    firewall: Firewall,
    dns_monitor: DnsMonitor,
    route_manager: RouteManager,
    _offline_monitor: offline::MonitorHandle,
    /// Should LAN access be allowed outside the tunnel.
    allow_lan: bool,
    /// Should network access be allowed when in the disconnected state.
    block_when_disconnected: bool,
    /// True when the computer is known to be offline.
    is_offline: bool,
    /// DNS servers to use (overriding default).
    dns_servers: Option<Vec<IpAddr>>,
    /// Endpoint that should not be blocked by the firewall.
    allowed_endpoint: Endpoint,
    /// The generator of new `TunnelParameter`s
    tunnel_parameters_generator: Box<dyn TunnelParametersGenerator>,
    /// The provider of tunnel devices.
    tun_provider: TunProvider,
    /// Directory to store tunnel log file.
    log_dir: Option<PathBuf>,
    /// Resource directory path.
    resource_dir: PathBuf,

    /// NetworkManager's connecitivity check state.
    #[cfg(target_os = "linux")]
    connectivity_check_was_enabled: Option<bool>,
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

    pub fn set_allowed_endpoint(&mut self, endpoint: Endpoint) -> bool {
        if self.allowed_endpoint != endpoint {
            self.allowed_endpoint = endpoint;

            #[cfg(target_os = "android")]
            self.tun_provider
                .set_allowed_endpoint(endpoint.address.ip());

            true
        } else {
            false
        }
    }

    pub fn set_dns_servers(
        &mut self,
        dns_servers: Option<Vec<IpAddr>>,
    ) -> Result<bool, ErrorStateCause> {
        if self.dns_servers != dns_servers {
            self.dns_servers = dns_servers.clone();

            #[cfg(target_os = "android")]
            {
                if let Err(error) = self.tun_provider.set_dns_servers(dns_servers) {
                    log::error!(
                        "{}",
                        error.display_chain_with_msg(
                            "Failed to restart tunnel after changing DNS servers",
                        )
                    );
                    return Err(ErrorStateCause::StartTunnelError);
                }
            }

            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// NetworkManager's connectivity check can get hung when DNS requests fail, thus the TSM
    /// should always disable it before applying firewall rules. The connectivity check should be
    /// reset whenever the firewall is cleared.
    #[cfg(target_os = "linux")]
    pub fn disable_connectivity_check(&mut self) {
        if self.connectivity_check_was_enabled.is_none() {
            if let Ok(nm) = talpid_dbus::network_manager::NetworkManager::new() {
                self.connectivity_check_was_enabled = nm.disable_connectivity_check();
            }
        } else {
            log::trace!("Daemon already disabled connectivity check");
        }
    }

    /// Reset NetworkManager's connectivity check if it was disabled.
    #[cfg(target_os = "linux")]
    pub fn reset_connectivity_check(&mut self) {
        if self.connectivity_check_was_enabled.take() == Some(true) {
            if let Ok(nm) = talpid_dbus::network_manager::NetworkManager::new() {
                nm.enable_connectivity_check();
            }
        } else {
            log::trace!("Connectivity check wasn't disabled by the daemon");
        }
    }

    #[cfg(target_os = "android")]
    pub fn bypass_socket(&mut self, fd: RawFd, tx: oneshot::Sender<()>) {
        if let Err(err) = self.tun_provider.bypass(fd) {
            log::error!("Failed to bypass socket {}", err);
        }
        let _ = tx.send(());
    }
}

/// Asynchronous result of an attempt to progress a state.
enum EventConsequence {
    /// Transition to a new state.
    NewState((TunnelStateWrapper, TunnelStateTransition)),
    /// An event was received, but it was ignored by the state so no transition is performed.
    SameState(TunnelStateWrapper),
    /// The state machine has finished its execution.
    Finished,
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
        runtime: &tokio::runtime::Handle,
        commands: &mut TunnelCommandReceiver,
        shared_values: &mut SharedTunnelStateValues,
    ) -> EventConsequence;
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
                runtime: &tokio::runtime::Handle,
                commands: &mut TunnelCommandReceiver,
                shared_values: &mut SharedTunnelStateValues,
            ) -> EventConsequence {
                match self {
                    $($wrapper_name::$state_variant(state) => {
                        state.handle_event(runtime, commands, shared_values)
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

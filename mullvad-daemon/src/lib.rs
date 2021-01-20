#![deny(rust_2018_idioms)]
#![recursion_limit = "256"]

#[macro_use]
extern crate serde;


pub mod account_history;
pub mod exception_logging;
mod geoip;
pub mod logging;
#[cfg(not(target_os = "android"))]
pub mod management_interface;
mod relays;
#[cfg(not(target_os = "android"))]
pub mod rpc_uniqueness_check;
pub mod runtime;
mod settings;
pub mod version;
mod version_check;

use futures::{
    channel::{mpsc, oneshot},
    future::{abortable, AbortHandle, Future},
    SinkExt, StreamExt,
};
use log::{debug, error, info, warn};
use mullvad_rpc::AccountsProxy;
use mullvad_types::{
    account::{AccountData, AccountToken, VoucherSubmission},
    endpoint::MullvadEndpoint,
    location::GeoIpLocation,
    relay_constraints::{
        BridgeSettings, BridgeState, Constraint, InternalBridgeConstraints, RelaySettings,
        RelaySettingsUpdate,
    },
    relay_list::{Relay, RelayList},
    settings::{DnsOptions, Settings},
    states::{TargetState, TunnelState},
    version::{AppVersion, AppVersionInfo},
    wireguard::KeygenEvent,
};
use settings::SettingsPersister;
#[cfg(not(target_os = "android"))]
use std::path::Path;
use std::{
    fs::{self, File},
    io,
    marker::PhantomData,
    mem,
    net::IpAddr,
    path::PathBuf,
    sync::{mpsc as sync_mpsc, Arc, Weak},
    time::Duration,
};
#[cfg(target_os = "linux")]
use talpid_core::split_tunnel;
use talpid_core::{
    mpsc::Sender,
    tunnel_state_machine::{self, TunnelCommand, TunnelParametersGenerator},
};
#[cfg(target_os = "android")]
use talpid_types::android::AndroidContext;
use talpid_types::{
    net::{openvpn, Endpoint, TransportProtocol, TunnelEndpoint, TunnelParameters, TunnelType},
    tunnel::{ErrorStateCause, ParameterGenerationError, TunnelStateTransition},
    ErrorExt,
};

#[path = "wireguard.rs"]
mod wireguard;

const TARGET_START_STATE_FILE: &str = "target-start-state.json";

const TUNNEL_STATE_MACHINE_SHUTDOWN_TIMEOUT: Duration = Duration::from_secs(5);

/// Timeout for first WireGuard key pushing
const FIRST_KEY_PUSH_TIMEOUT: Duration = Duration::from_secs(5);

/// Delay between generating a new WireGuard key and reconnecting
const WG_RECONNECT_DELAY: Duration = Duration::from_secs(30);

#[derive(err_derive::Error, Debug)]
#[error(no_from)]
pub enum Error {
    #[error(display = "Failed to send command to daemon because it is not running")]
    DaemonUnavailable,

    #[error(display = "Unable to initialize network event loop")]
    InitIoEventLoop(#[error(source)] io::Error),

    #[error(display = "Unable to create RPC client")]
    InitRpcFactory(#[error(source)] mullvad_rpc::Error),

    #[error(display = "Unable to load account history with wireguard key cache")]
    LoadAccountHistory(#[error(source)] account_history::Error),

    #[cfg(target_os = "linux")]
    #[error(display = "Unable to initialize split tunneling")]
    InitSplitTunneling(#[error(source)] split_tunnel::Error),

    #[error(display = "No wireguard private key available")]
    NoKeyAvailable,

    #[error(display = "No bridge available")]
    NoBridgeAvailable,

    #[error(display = "Account history problems")]
    AccountHistory(#[error(source)] account_history::Error),

    #[error(display = "Tunnel state machine error")]
    TunnelError(#[error(source)] tunnel_state_machine::Error),

    #[error(display = "Failed to remove directory {}", _0)]
    RemoveDirError(String, #[error(source)] io::Error),

    #[error(display = "Failed to create directory {}", _0)]
    CreateDirError(String, #[error(source)] io::Error),

    #[error(display = "Failed to get path")]
    PathError(#[error(source)] mullvad_paths::Error),

    #[cfg(target_os = "windows")]
    #[error(display = "Failed to get file type info")]
    FileTypeError(#[error(source)] io::Error),

    #[cfg(target_os = "windows")]
    #[error(display = "Failed to get dir entry")]
    FileEntryError(#[error(source)] io::Error),

    #[cfg(target_os = "windows")]
    #[error(display = "Failed to read dir entries")]
    ReadDirError(#[error(source)] io::Error),

    #[error(display = "Failed to read cached target tunnel state")]
    ReadCachedTargetState(#[error(source)] serde_json::Error),

    #[error(display = "Failed to open cached target tunnel state")]
    OpenCachedTargetState(#[error(source)] io::Error),
}

/// Enum representing commands that can be sent to the daemon.
pub enum DaemonCommand {
    /// Set target state. Does nothing if the daemon already has the state that is being set.
    SetTargetState(oneshot::Sender<bool>, TargetState),
    /// Reconnect the tunnel, if one is connecting/connected.
    Reconnect(oneshot::Sender<bool>),
    /// Request the current state.
    GetState(oneshot::Sender<TunnelState>),
    /// Get the current geographical location.
    GetCurrentLocation(oneshot::Sender<Option<GeoIpLocation>>),
    CreateNewAccount(oneshot::Sender<Result<String, mullvad_rpc::rest::Error>>),
    /// Request the metadata for an account.
    GetAccountData(
        oneshot::Sender<Result<AccountData, mullvad_rpc::rest::Error>>,
        AccountToken,
    ),
    /// Request www auth token for an account
    GetWwwAuthToken(oneshot::Sender<Result<String, mullvad_rpc::rest::Error>>),
    /// Submit voucher to add time to the current account. Returns time added in seconds
    SubmitVoucher(
        oneshot::Sender<Result<VoucherSubmission, mullvad_rpc::rest::Error>>,
        String,
    ),
    /// Request account history
    GetAccountHistory(oneshot::Sender<Vec<AccountToken>>),
    /// Request account history
    RemoveAccountFromHistory(oneshot::Sender<()>, AccountToken),
    /// Clear account history
    ClearAccountHistory(oneshot::Sender<()>),
    /// Get the list of countries and cities where there are relays.
    GetRelayLocations(oneshot::Sender<RelayList>),
    /// Trigger an asynchronous relay list update. This returns before the relay list is actually
    /// updated.
    UpdateRelayLocations,
    /// Set which account token to use for subsequent connection attempts.
    SetAccount(oneshot::Sender<()>, Option<AccountToken>),
    /// Place constraints on the type of tunnel and relay
    UpdateRelaySettings(oneshot::Sender<()>, RelaySettingsUpdate),
    /// Set the allow LAN setting.
    SetAllowLan(oneshot::Sender<()>, bool),
    /// Set the beta program setting.
    SetShowBetaReleases(oneshot::Sender<()>, bool),
    /// Set the block_when_disconnected setting.
    SetBlockWhenDisconnected(oneshot::Sender<()>, bool),
    /// Set the auto-connect setting.
    SetAutoConnect(oneshot::Sender<()>, bool),
    /// Set the mssfix argument for OpenVPN
    SetOpenVpnMssfix(oneshot::Sender<()>, Option<u16>),
    /// Set proxy details for OpenVPN
    SetBridgeSettings(oneshot::Sender<Result<(), settings::Error>>, BridgeSettings),
    /// Set proxy state
    SetBridgeState(oneshot::Sender<Result<(), settings::Error>>, BridgeState),
    /// Set if IPv6 should be enabled in the tunnel
    SetEnableIpv6(oneshot::Sender<()>, bool),
    /// Set custom DNS servers to use instead of passing requests to the gateway
    SetDnsOptions(oneshot::Sender<()>, DnsOptions),
    /// Set MTU for wireguard tunnels
    SetWireguardMtu(oneshot::Sender<()>, Option<u16>),
    /// Set automatic key rotation interval for wireguard tunnels
    SetWireguardRotationInterval(oneshot::Sender<()>, Option<u32>),
    /// Get the daemon settings
    GetSettings(oneshot::Sender<Settings>),
    /// Generate new wireguard key
    GenerateWireguardKey(oneshot::Sender<wireguard::KeygenEvent>),
    /// Return a public key of the currently set wireguard private key, if there is one
    GetWireguardKey(oneshot::Sender<Option<wireguard::PublicKey>>),
    /// Verify if the currently set wireguard key is valid.
    VerifyWireguardKey(oneshot::Sender<bool>),
    /// Get information about the currently running and latest app versions
    GetVersionInfo(oneshot::Sender<Option<AppVersionInfo>>),
    /// Get current version of the app
    GetCurrentVersion(oneshot::Sender<AppVersion>),
    /// Remove settings and clear the cache
    #[cfg(not(target_os = "android"))]
    FactoryReset(oneshot::Sender<()>),
    /// Request list of processes excluded from the tunnel
    #[cfg(target_os = "linux")]
    GetSplitTunnelProcesses(oneshot::Sender<Vec<i32>>),
    /// Exclude traffic of a process (PID) from the tunnel
    #[cfg(target_os = "linux")]
    AddSplitTunnelProcess(oneshot::Sender<()>, i32),
    /// Remove process (PID) from list of processes excluded from the tunnel
    #[cfg(target_os = "linux")]
    RemoveSplitTunnelProcess(oneshot::Sender<()>, i32),
    /// Clear list of processes excluded from the tunnel
    #[cfg(target_os = "linux")]
    ClearSplitTunnelProcesses(oneshot::Sender<()>),
    /// Makes the daemon exit the main loop and quit.
    Shutdown,
    /// Saves the target tunnel state and enters a blocking state. The state is restored
    /// upon restart.
    PrepareRestart,
}

/// All events that can happen in the daemon. Sent from various threads and exposed interfaces.
pub(crate) enum InternalDaemonEvent {
    /// Tunnel has changed state.
    TunnelStateTransition(TunnelStateTransition),
    /// Request from the `MullvadTunnelParametersGenerator` to obtain a new relay.
    GenerateTunnelParameters(
        sync_mpsc::Sender<Result<TunnelParameters, ParameterGenerationError>>,
        u32,
    ),
    /// A command sent to the daemon.
    Command(DaemonCommand),
    /// Daemon shutdown triggered by a signal, ctrl-c or similar.
    TriggerShutdown,
    /// Wireguard key generation event
    WgKeyEvent(
        (
            AccountToken,
            Result<mullvad_types::wireguard::WireguardData, wireguard::Error>,
        ),
    ),
    /// New Account created
    NewAccountEvent(
        AccountToken,
        oneshot::Sender<Result<String, mullvad_rpc::rest::Error>>,
    ),
    /// The background job fetching new `AppVersionInfo`s got a new info object.
    NewAppVersionInfo(AppVersionInfo),
}

impl From<TunnelStateTransition> for InternalDaemonEvent {
    fn from(tunnel_state_transition: TunnelStateTransition) -> Self {
        InternalDaemonEvent::TunnelStateTransition(tunnel_state_transition)
    }
}

impl From<DaemonCommand> for InternalDaemonEvent {
    fn from(command: DaemonCommand) -> Self {
        InternalDaemonEvent::Command(command)
    }
}

impl From<AppVersionInfo> for InternalDaemonEvent {
    fn from(command: AppVersionInfo) -> Self {
        InternalDaemonEvent::NewAppVersionInfo(command)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum DaemonExecutionState {
    Running,
    Exiting,
    Finished,
}

impl DaemonExecutionState {
    pub fn shutdown(&mut self, tunnel_state: &TunnelState) {
        use self::DaemonExecutionState::*;

        match self {
            Running => {
                match tunnel_state {
                    TunnelState::Disconnected => mem::replace(self, Finished),
                    _ => mem::replace(self, Exiting),
                };
            }
            Exiting | Finished => {}
        };
    }

    pub fn disconnected(&mut self) {
        use self::DaemonExecutionState::*;

        match self {
            Exiting => {
                let _ = mem::replace(self, Finished);
            }
            Running | Finished => {}
        };
    }

    pub fn is_running(&self) -> bool {
        use self::DaemonExecutionState::*;

        match self {
            Running => true,
            Exiting | Finished => false,
        }
    }
}

pub struct DaemonCommandChannel {
    sender: DaemonCommandSender,
    receiver: mpsc::UnboundedReceiver<InternalDaemonEvent>,
}

impl DaemonCommandChannel {
    pub fn new() -> Self {
        let (untracked_sender, receiver) = mpsc::unbounded();
        let sender = DaemonCommandSender(Arc::new(untracked_sender));

        Self { sender, receiver }
    }

    pub fn sender(&self) -> DaemonCommandSender {
        self.sender.clone()
    }

    fn destructure(
        self,
    ) -> (
        DaemonEventSender,
        mpsc::UnboundedReceiver<InternalDaemonEvent>,
    ) {
        let event_sender = DaemonEventSender::new(Arc::downgrade(&self.sender.0));

        (event_sender, self.receiver)
    }
}

#[derive(Clone)]
pub struct DaemonCommandSender(Arc<mpsc::UnboundedSender<InternalDaemonEvent>>);

impl DaemonCommandSender {
    pub fn send(&self, command: DaemonCommand) -> Result<(), Error> {
        self.0
            .unbounded_send(InternalDaemonEvent::Command(command))
            .map_err(|_| Error::DaemonUnavailable)
    }
}

pub(crate) struct DaemonEventSender<E = InternalDaemonEvent> {
    sender: Weak<mpsc::UnboundedSender<InternalDaemonEvent>>,
    _event: PhantomData<E>,
}

impl<E> Clone for DaemonEventSender<E>
where
    InternalDaemonEvent: From<E>,
{
    fn clone(&self) -> Self {
        DaemonEventSender {
            sender: self.sender.clone(),
            _event: PhantomData,
        }
    }
}

impl DaemonEventSender {
    pub fn new(sender: Weak<mpsc::UnboundedSender<InternalDaemonEvent>>) -> Self {
        DaemonEventSender {
            sender,
            _event: PhantomData,
        }
    }

    pub fn to_specialized_sender<E>(&self) -> DaemonEventSender<E>
    where
        InternalDaemonEvent: From<E>,
    {
        DaemonEventSender {
            sender: self.sender.clone(),
            _event: PhantomData,
        }
    }
}

impl<E> DaemonEventSender<E>
where
    InternalDaemonEvent: From<E>,
{
    pub fn is_closed(&self) -> bool {
        self.sender
            .upgrade()
            .map(|sender| sender.is_closed())
            .unwrap_or(true)
    }
}

impl<E> Sender<E> for DaemonEventSender<E>
where
    InternalDaemonEvent: From<E>,
{
    fn send(&self, event: E) -> Result<(), ()> {
        if let Some(sender) = self.sender.upgrade() {
            sender
                .unbounded_send(InternalDaemonEvent::from(event))
                .map_err(|_| ())
        } else {
            Err(())
        }
    }
}

/// Trait representing something that can broadcast daemon events.
pub trait EventListener {
    /// Notify that the tunnel state changed.
    fn notify_new_state(&self, new_state: TunnelState);

    /// Notify that the settings changed.
    fn notify_settings(&self, settings: Settings);

    /// Notify that the relay list changed.
    fn notify_relay_list(&self, relay_list: RelayList);

    /// Notify that info about the latest available app version changed.
    /// Or some flag about the currently running version is changed.
    fn notify_app_version(&self, app_version_info: AppVersionInfo);

    /// Notify clients of a key generation event.
    fn notify_key_event(&self, key_event: KeygenEvent);
}

pub struct Daemon<L: EventListener> {
    tunnel_command_tx: Arc<mpsc::UnboundedSender<TunnelCommand>>,
    tunnel_state: TunnelState,
    target_state: TargetState,
    clean_up_target_cache: bool,
    state: DaemonExecutionState,
    #[cfg(target_os = "linux")]
    exclude_pids: split_tunnel::PidManager,
    rx: mpsc::UnboundedReceiver<InternalDaemonEvent>,
    tx: DaemonEventSender,
    reconnection_job: Option<AbortHandle>,
    event_listener: L,
    settings: SettingsPersister,
    account_history: account_history::AccountHistory,
    accounts_proxy: AccountsProxy,
    rpc_runtime: mullvad_rpc::MullvadRpcRuntime,
    rpc_handle: mullvad_rpc::rest::MullvadRestHandle,
    wireguard_key_manager: wireguard::KeyManager,
    version_updater_handle: version_check::VersionUpdaterHandle,
    relay_selector: relays::RelaySelector,
    last_generated_relay: Option<Relay>,
    last_generated_bridge_relay: Option<Relay>,
    app_version_info: Option<AppVersionInfo>,
    shutdown_callbacks: Vec<Box<dyn FnOnce()>>,
    /// oneshot channel that completes once the tunnel state machine has been shut down
    tunnel_state_machine_shutdown_signal: oneshot::Receiver<()>,
    cache_dir: PathBuf,
}

impl<L> Daemon<L>
where
    L: EventListener + Clone + Send + 'static,
{
    pub async fn start(
        log_dir: Option<PathBuf>,
        resource_dir: PathBuf,
        settings_dir: PathBuf,
        cache_dir: PathBuf,
        event_listener: L,
        command_channel: DaemonCommandChannel,
        #[cfg(target_os = "android")] android_context: AndroidContext,
    ) -> Result<Self, Error> {
        let (tunnel_state_machine_shutdown_tx, tunnel_state_machine_shutdown_signal) =
            oneshot::channel();

        let (internal_event_tx, internal_event_rx) = command_channel.destructure();
        let (address_change_tx, mut address_change_rx) = mpsc::channel(0);
        let address_change_tx = std::sync::Mutex::new(address_change_tx);
        let address_change_runtime = tokio::runtime::Handle::current();

        let mut rpc_runtime = mullvad_rpc::MullvadRpcRuntime::with_cache(
            tokio::runtime::Handle::current(),
            Some(&resource_dir),
            &cache_dir,
            true,
            move |address| {
                let (result_tx, result_rx) = oneshot::channel();

                let mut tx = address_change_tx.lock().unwrap().clone();
                address_change_runtime.block_on(async move {
                    let tunnel_command = TunnelCommand::AllowEndpoint(
                        Endpoint::from_socket_address(address, TransportProtocol::Tcp),
                        result_tx,
                    );
                    let _ = tx.send(tunnel_command).await;
                    result_rx.await.map_err(|_| ())
                })
            },
        )
        .await
        .map_err(Error::InitRpcFactory)?;
        let rpc_handle = rpc_runtime.mullvad_rest_handle();

        let relay_list_listener = event_listener.clone();
        let on_relay_list_update = move |relay_list: &RelayList| {
            relay_list_listener.notify_relay_list(relay_list.clone());
        };
        let mut relay_selector = relays::RelaySelector::new(
            rpc_handle.clone(),
            on_relay_list_update,
            &resource_dir,
            &cache_dir,
        );


        let mut settings = SettingsPersister::load(&settings_dir);

        if version::is_beta_version() {
            let _ = settings.set_show_beta_releases(true);
        }

        let app_version_info = version_check::load_cache(&cache_dir);
        let (version_updater, version_updater_handle) = version_check::VersionUpdater::new(
            rpc_handle.clone(),
            cache_dir.clone(),
            internal_event_tx.to_specialized_sender(),
            app_version_info.clone(),
            settings.show_beta_releases,
        );
        tokio::spawn(version_updater.run());
        let account_history =
            account_history::AccountHistory::new(&cache_dir, &settings_dir, rpc_handle.clone())
                .await
                .map_err(Error::LoadAccountHistory)?;

        // Restore the tunnel to a previous state
        let target_cache = cache_dir.join(TARGET_START_STATE_FILE);
        let cached_target_state: Option<TargetState> = match File::open(&target_cache) {
            Ok(handle) => serde_json::from_reader(io::BufReader::new(handle))
                .map(Some)
                .map_err(Error::ReadCachedTargetState),
            Err(e) => {
                if e.kind() == io::ErrorKind::NotFound {
                    debug!("No cached target state to load");
                    Ok(None)
                } else {
                    Err(Error::OpenCachedTargetState(e))
                }
            }
        }
        .unwrap_or_else(|error| {
            error!("{}", error.display_chain());
            Some(TargetState::Secured)
        });
        if let Some(cached_target_state) = &cached_target_state {
            info!(
                "Loaded cached target state \"{}\" from {}",
                cached_target_state,
                target_cache.display()
            );
            let _ = fs::remove_file(target_cache).map_err(|e| {
                error!("Cannot delete target tunnel state cache: {}", e);
            });
        }

        let tunnel_parameters_generator = MullvadTunnelParametersGenerator {
            tx: internal_event_tx.clone(),
        };


        let initial_target_state = if settings.get_account_token().is_some() {
            if settings.auto_connect {
                // Note: Auto-connect overrides the cached target state
                info!("Automatically connecting since auto-connect is turned on");
                TargetState::Secured
            } else {
                cached_target_state.unwrap_or(TargetState::Unsecured)
            }
        } else {
            TargetState::Unsecured
        };

        let initial_api_endpoint = Endpoint::from_socket_address(
            rpc_runtime.address_cache.peek_address(),
            TransportProtocol::Tcp,
        );

        let tunnel_command_tx = tunnel_state_machine::spawn(
            settings.allow_lan,
            settings.block_when_disconnected,
            Self::get_custom_resolvers(&settings.tunnel_options.dns_options),
            initial_api_endpoint,
            tunnel_parameters_generator,
            log_dir,
            resource_dir,
            cache_dir.clone(),
            internal_event_tx.to_specialized_sender(),
            tunnel_state_machine_shutdown_tx,
            initial_target_state != TargetState::Secured,
            #[cfg(target_os = "android")]
            android_context,
        )
        .await
        .map_err(Error::TunnelError)?;

        let tsm_api_address_change_tx = Arc::downgrade(&tunnel_command_tx);
        tokio::spawn(async move {
            while let Some(address_change) = address_change_rx.next().await {
                if let Some(tx) = tsm_api_address_change_tx.upgrade() {
                    let _ = tx.unbounded_send(address_change);
                } else {
                    return;
                }
            }
        });

        let wireguard_key_manager =
            wireguard::KeyManager::new(internal_event_tx.clone(), rpc_handle.clone());

        // Attempt to download a fresh relay list
        relay_selector.update().await;

        let mut daemon = Daemon {
            tunnel_command_tx,
            tunnel_state: TunnelState::Disconnected,
            target_state: initial_target_state,
            clean_up_target_cache: true,
            state: DaemonExecutionState::Running,
            #[cfg(target_os = "linux")]
            exclude_pids: split_tunnel::PidManager::new().map_err(Error::InitSplitTunneling)?,
            rx: internal_event_rx,
            tx: internal_event_tx,
            reconnection_job: None,
            event_listener,
            settings,
            account_history,
            rpc_runtime,
            accounts_proxy: AccountsProxy::new(rpc_handle.clone()),
            rpc_handle,
            wireguard_key_manager,
            version_updater_handle,
            relay_selector,
            last_generated_relay: None,
            last_generated_bridge_relay: None,
            app_version_info,
            shutdown_callbacks: vec![],
            tunnel_state_machine_shutdown_signal,
            cache_dir,
        };

        daemon.ensure_wireguard_keys_for_current_account().await;

        Ok(daemon)
    }

    fn get_custom_resolvers(dns_options: &DnsOptions) -> Option<Vec<IpAddr>> {
        if dns_options.custom {
            Some(dns_options.addresses.clone())
        } else {
            None
        }
    }

    /// Consume the `Daemon` and run the main event loop. Blocks until an error happens or a
    /// shutdown event is received.
    pub async fn run(mut self) -> Result<(), Error> {
        if self.target_state == TargetState::Secured {
            self.connect_tunnel();
        }

        while let Some(event) = self.rx.next().await {
            self.handle_event(event).await;
            if self.state == DaemonExecutionState::Finished {
                break;
            }
        }

        // If auto-connect is enabled, block all traffic before shutting down to ensure
        // that no traffic can leak during boot.
        #[cfg(windows)]
        if self.settings.auto_connect {
            self.send_tunnel_command(TunnelCommand::BlockWhenDisconnected(true));
        }

        self.finalize().await;
        Ok(())
    }

    async fn finalize(self) {
        let (
            event_listener,
            shutdown_callbacks,
            rpc_runtime,
            tunnel_state_machine_shutdown_signal,
            cache_dir,
            clean_up_target_cache,
        ) = self.shutdown();
        for cb in shutdown_callbacks {
            cb();
        }

        let shutdown_signal = tokio::time::timeout(
            TUNNEL_STATE_MACHINE_SHUTDOWN_TIMEOUT,
            tunnel_state_machine_shutdown_signal,
        );
        match shutdown_signal.await {
            Ok(_) => log::info!("Tunnel state machine shut down"),
            Err(_) => log::error!("Tunnel state machine did not shut down gracefully"),
        }

        mem::drop(event_listener);
        mem::drop(rpc_runtime);

        if clean_up_target_cache {
            let target_cache = cache_dir.join(TARGET_START_STATE_FILE);
            let _ = fs::remove_file(target_cache).map_err(|e| {
                error!("Cannot delete target tunnel state cache: {}", e);
            });
        }
    }

    /// Shuts down the daemon without shutting down the underlying event listener and the shutdown
    /// callbacks
    fn shutdown(
        self,
    ) -> (
        L,
        Vec<Box<dyn FnOnce()>>,
        mullvad_rpc::MullvadRpcRuntime,
        oneshot::Receiver<()>,
        PathBuf,
        bool,
    ) {
        let Daemon {
            event_listener,
            shutdown_callbacks,
            rpc_runtime,
            tunnel_state_machine_shutdown_signal,
            cache_dir,
            clean_up_target_cache,
            ..
        } = self;
        (
            event_listener,
            shutdown_callbacks,
            rpc_runtime,
            tunnel_state_machine_shutdown_signal,
            cache_dir,
            clean_up_target_cache,
        )
    }


    async fn handle_event(&mut self, event: InternalDaemonEvent) {
        use self::InternalDaemonEvent::*;
        match event {
            TunnelStateTransition(transition) => {
                self.handle_tunnel_state_transition(transition).await
            }
            GenerateTunnelParameters(tunnel_parameters_tx, retry_attempt) => {
                self.handle_generate_tunnel_parameters(&tunnel_parameters_tx, retry_attempt)
                    .await
            }
            Command(command) => self.handle_command(command).await,
            TriggerShutdown => self.trigger_shutdown_event(),
            WgKeyEvent(key_event) => self.handle_wireguard_key_event(key_event).await,
            NewAccountEvent(account_token, tx) => {
                self.handle_new_account_event(account_token, tx).await
            }
            NewAppVersionInfo(app_version_info) => {
                self.handle_new_app_version_info(app_version_info)
            }
        }
    }

    async fn handle_tunnel_state_transition(
        &mut self,
        tunnel_state_transition: TunnelStateTransition,
    ) {
        self.reset_rpc_sockets_on_tunnel_state_transition(&tunnel_state_transition)
            .await;
        let tunnel_state = match tunnel_state_transition {
            TunnelStateTransition::Disconnected => TunnelState::Disconnected,
            TunnelStateTransition::Connecting(endpoint) => TunnelState::Connecting {
                endpoint,
                location: self.build_location_from_relay(),
            },
            TunnelStateTransition::Connected(endpoint) => TunnelState::Connected {
                endpoint,
                location: self.build_location_from_relay(),
            },
            TunnelStateTransition::Disconnecting(after_disconnect) => {
                TunnelState::Disconnecting(after_disconnect)
            }
            TunnelStateTransition::Error(error_state) => TunnelState::Error(error_state),
        };


        self.unschedule_reconnect();

        debug!("New tunnel state: {:?}", tunnel_state);
        match tunnel_state {
            TunnelState::Disconnected => self.state.disconnected(),
            TunnelState::Error(ref error_state) => {
                if error_state.is_blocking() {
                    info!(
                        "Blocking all network connections, reason: {}",
                        error_state.cause()
                    );
                } else {
                    error!(
                        "FAILED TO BLOCK NETWORK CONNECTIONS, ENTERED ERROR STATE BECAUSE: {}",
                        error_state.cause()
                    );
                }

                if let ErrorStateCause::AuthFailed(_) = error_state.cause() {
                    self.schedule_reconnect(Duration::from_secs(60)).await
                }
            }
            _ => {}
        }

        self.tunnel_state = tunnel_state.clone();
        self.event_listener.notify_new_state(tunnel_state);
    }

    async fn reset_rpc_sockets_on_tunnel_state_transition(
        &mut self,
        tunnel_state_transition: &TunnelStateTransition,
    ) {
        match (&self.tunnel_state, &tunnel_state_transition) {
            // only reset the API sockets if when connected or leaving the connected state
            (&TunnelState::Connected { .. }, _) | (_, &TunnelStateTransition::Connected(_)) => {
                self.rpc_handle.service().reset().await;
            }
            _ => (),
        };
    }

    async fn handle_generate_tunnel_parameters(
        &mut self,
        tunnel_parameters_tx: &sync_mpsc::Sender<
            Result<TunnelParameters, ParameterGenerationError>,
        >,
        retry_attempt: u32,
    ) {
        if let Some(account_token) = self.settings.get_account_token() {
            let result = match self.settings.get_relay_settings() {
                RelaySettings::CustomTunnelEndpoint(custom_relay) => {
                    self.last_generated_relay = None;
                    custom_relay
                        // TODO(emilsp): generate proxy settings for custom tunnels
                        .to_tunnel_parameters(self.settings.tunnel_options.clone(), None)
                        .map_err(|e| {
                            log::error!("Failed to resolve hostname for custom tunnel config: {}", e);
                            ParameterGenerationError::CustomTunnelHostResultionError
                        })
                }
                RelaySettings::Normal(constraints) => {
                    let endpoint = self
                        .relay_selector
                        .get_tunnel_endpoint(
                            &constraints,
                            self.settings.get_bridge_state(),
                            retry_attempt,
                            self.account_history
                                .get(&account_token)
                                .await
                                .unwrap_or(None)
                                .and_then(|entry| entry.wireguard)
                                .is_some(),
                        )
                        .ok();
                    if let Some((relay, endpoint)) = endpoint {
                        let result = self
                            .create_tunnel_parameters(
                                &relay,
                                endpoint,
                                account_token,
                                retry_attempt,
                            )
                            .await;
                        self.last_generated_relay = Some(relay);
                        match result {
                            Ok(result) => Ok(result),
                            Err(Error::NoKeyAvailable) => {
                                Err(ParameterGenerationError::NoWireguardKey)
                            }
                            Err(Error::NoBridgeAvailable) => {
                                Err(ParameterGenerationError::NoMatchingBridgeRelay)
                            }
                            Err(err) => {
                                log::error!(
                                    "{}",
                                    err.display_chain_with_msg(
                                        "Failed to generate tunnel parameters"
                                    )
                                );
                                Err(ParameterGenerationError::NoMatchingRelay)
                            }
                        }
                    } else {
                        Err(ParameterGenerationError::NoMatchingRelay)
                    }
                }
            };
            if tunnel_parameters_tx.send(result).is_err() {
                log::error!("Failed to send tunnel parameters");
            }
        } else {
            error!("No account token configured");
        }
    }

    async fn create_tunnel_parameters(
        &mut self,
        relay: &Relay,
        endpoint: MullvadEndpoint,
        account_token: String,
        retry_attempt: u32,
    ) -> Result<TunnelParameters, Error> {
        let tunnel_options = self.settings.tunnel_options.clone();
        let location = relay.location.as_ref().expect("Relay has no location set");
        self.last_generated_bridge_relay = None;
        match endpoint {
            MullvadEndpoint::OpenVpn(endpoint) => {
                let proxy_settings = match &self.settings.bridge_settings {
                    BridgeSettings::Normal(settings) => {
                        let bridge_constraints = InternalBridgeConstraints {
                            location: settings.location.clone(),
                            provider: settings.provider.clone(),
                            // FIXME: This is temporary while talpid-core only supports TCP proxies
                            transport_protocol: Constraint::Only(TransportProtocol::Tcp),
                        };
                        match self.settings.get_bridge_state() {
                            BridgeState::On => {
                                let (bridge_settings, bridge_relay) = self
                                    .relay_selector
                                    .get_proxy_settings(&bridge_constraints, location)
                                    .ok_or(Error::NoBridgeAvailable)?;
                                self.last_generated_bridge_relay = Some(bridge_relay);
                                Some(bridge_settings)
                            }
                            BridgeState::Auto => {
                                if let Some((bridge_settings, bridge_relay)) =
                                    self.relay_selector.get_auto_proxy_settings(
                                        &bridge_constraints,
                                        location,
                                        retry_attempt,
                                    )
                                {
                                    self.last_generated_bridge_relay = Some(bridge_relay);
                                    Some(bridge_settings)
                                } else {
                                    None
                                }
                            }
                            BridgeState::Off => None,
                        }
                    }
                    BridgeSettings::Custom(proxy_settings) => {
                        match self.settings.get_bridge_state() {
                            BridgeState::On => Some(proxy_settings.clone()),
                            BridgeState::Auto => {
                                if self.relay_selector.should_use_bridge(retry_attempt) {
                                    Some(proxy_settings.clone())
                                } else {
                                    None
                                }
                            }
                            BridgeState::Off => None,
                        }
                    }
                };

                Ok(openvpn::TunnelParameters {
                    config: openvpn::ConnectionConfig::new(
                        endpoint,
                        account_token,
                        "-".to_string(),
                    ),
                    options: tunnel_options.openvpn,
                    generic_options: tunnel_options.generic,
                    proxy: proxy_settings,
                }
                .into())
            }
            MullvadEndpoint::Wireguard {
                peer,
                ipv4_gateway,
                ipv6_gateway,
            } => {
                let wg_data = self
                    .account_history
                    .get(&account_token)
                    .await
                    .map_err(Error::AccountHistory)?
                    .and_then(|entry| entry.wireguard)
                    .ok_or(Error::NoKeyAvailable)?;
                let tunnel = wireguard::TunnelConfig {
                    private_key: wg_data.private_key,
                    addresses: vec![
                        wg_data.addresses.ipv4_address.ip().into(),
                        wg_data.addresses.ipv6_address.ip().into(),
                    ],
                };
                Ok(wireguard::TunnelParameters {
                    connection: wireguard::ConnectionConfig {
                        tunnel,
                        peer,
                        ipv4_gateway,
                        ipv6_gateway: Some(ipv6_gateway),
                    },
                    options: tunnel_options.wireguard,
                    generic_options: tunnel_options.generic,
                }
                .into())
            }
        }
    }

    async fn schedule_reconnect(&mut self, delay: Duration) {
        let tunnel_command_tx = self.tx.to_specialized_sender();
        let (future, abort_handle) = abortable(Box::pin(async move {
            tokio::time::delay_for(delay).await;
            log::debug!("Attempting to reconnect");
            let (tx, _) = oneshot::channel();
            let _ = tunnel_command_tx.send(DaemonCommand::Reconnect(tx));
        }));

        tokio::spawn(future);
        self.reconnection_job = Some(abort_handle);
    }

    fn unschedule_reconnect(&mut self) {
        if let Some(job) = self.reconnection_job.take() {
            job.abort();
        }
    }


    async fn handle_command(&mut self, command: DaemonCommand) {
        use self::DaemonCommand::*;
        if !self.state.is_running() {
            log::trace!("Dropping daemon command because the daemon is shutting down",);
            return;
        }
        match command {
            SetTargetState(tx, state) => self.on_set_target_state(tx, state),
            Reconnect(tx) => self.on_reconnect(tx),
            GetState(tx) => self.on_get_state(tx),
            GetCurrentLocation(tx) => self.on_get_current_location(tx).await,
            CreateNewAccount(tx) => self.on_create_new_account(tx).await,
            GetAccountData(tx, account_token) => self.on_get_account_data(tx, account_token).await,
            GetWwwAuthToken(tx) => self.on_get_www_auth_token(tx).await,
            SubmitVoucher(tx, voucher) => self.on_submit_voucher(tx, voucher).await,
            GetRelayLocations(tx) => self.on_get_relay_locations(tx),
            UpdateRelayLocations => self.on_update_relay_locations().await,
            SetAccount(tx, account_token) => self.on_set_account(tx, account_token).await,
            GetAccountHistory(tx) => self.on_get_account_history(tx),
            RemoveAccountFromHistory(tx, account_token) => {
                self.on_remove_account_from_history(tx, account_token).await
            }
            ClearAccountHistory(tx) => self.on_clear_account_history(tx).await,
            UpdateRelaySettings(tx, update) => self.on_update_relay_settings(tx, update),
            SetAllowLan(tx, allow_lan) => self.on_set_allow_lan(tx, allow_lan),
            SetShowBetaReleases(tx, enabled) => self.on_set_show_beta_releases(tx, enabled).await,
            SetBlockWhenDisconnected(tx, block_when_disconnected) => {
                self.on_set_block_when_disconnected(tx, block_when_disconnected)
            }
            SetAutoConnect(tx, auto_connect) => self.on_set_auto_connect(tx, auto_connect),
            SetOpenVpnMssfix(tx, mssfix_arg) => self.on_set_openvpn_mssfix(tx, mssfix_arg),
            SetBridgeSettings(tx, bridge_settings) => {
                self.on_set_bridge_settings(tx, bridge_settings)
            }
            SetBridgeState(tx, bridge_state) => self.on_set_bridge_state(tx, bridge_state),
            SetEnableIpv6(tx, enable_ipv6) => self.on_set_enable_ipv6(tx, enable_ipv6),
            SetDnsOptions(tx, dns_servers) => self.on_set_dns_options(tx, dns_servers),
            SetWireguardMtu(tx, mtu) => self.on_set_wireguard_mtu(tx, mtu),
            SetWireguardRotationInterval(tx, interval) => {
                self.on_set_wireguard_rotation_interval(tx, interval).await
            }
            GetSettings(tx) => self.on_get_settings(tx),
            GenerateWireguardKey(tx) => self.on_generate_wireguard_key(tx).await,
            GetWireguardKey(tx) => self.on_get_wireguard_key(tx).await,
            VerifyWireguardKey(tx) => self.on_verify_wireguard_key(tx).await,
            GetVersionInfo(tx) => self.on_get_version_info(tx).await,
            GetCurrentVersion(tx) => self.on_get_current_version(tx),
            #[cfg(not(target_os = "android"))]
            FactoryReset(tx) => self.on_factory_reset(tx).await,
            #[cfg(target_os = "linux")]
            GetSplitTunnelProcesses(tx) => self.on_get_split_tunnel_processes(tx),
            #[cfg(target_os = "linux")]
            AddSplitTunnelProcess(tx, pid) => self.on_add_split_tunnel_process(tx, pid),
            #[cfg(target_os = "linux")]
            RemoveSplitTunnelProcess(tx, pid) => self.on_remove_split_tunnel_process(tx, pid),
            #[cfg(target_os = "linux")]
            ClearSplitTunnelProcesses(tx) => self.on_clear_split_tunnel_processes(tx),
            Shutdown => self.trigger_shutdown_event(),
            PrepareRestart => self.on_prepare_restart(),
        }
    }

    async fn handle_wireguard_key_event(
        &mut self,
        event: (
            AccountToken,
            Result<mullvad_types::wireguard::WireguardData, wireguard::Error>,
        ),
    ) {
        let (account, result) = event;
        // If the account has been reset whilst a key was being generated, the event should be
        // dropped even if a new key was generated.
        if self
            .settings
            .get_account_token()
            .map(|current_account| current_account != account)
            .unwrap_or(true)
        {
            log::info!("Dropping wireguard key event since account has been changed");
            return;
        }

        match result {
            Ok(data) => {
                let public_key = data.get_public_key();
                let mut account_entry = self
                    .account_history
                    .get(&account)
                    .await
                    .ok()
                    .and_then(|entry| entry)
                    .unwrap_or_else(|| account_history::AccountEntry {
                        account: account.clone(),
                        wireguard: None,
                    });
                // if no key existed before
                let first_key_for_account_on_host = account_entry.wireguard.is_none();
                account_entry.wireguard = Some(data);
                match self.account_history.insert(account_entry).await {
                    Ok(_) => {
                        if let Some(TunnelType::Wireguard) = self.get_connected_tunnel_type() {
                            self.schedule_reconnect(WG_RECONNECT_DELAY).await;
                        }
                        self.event_listener
                            .notify_key_event(KeygenEvent::NewKey(public_key));
                        if first_key_for_account_on_host {
                            self.ensure_key_rotation().await;
                        }
                    }
                    Err(e) => {
                        log::error!(
                            "{}",
                            e.display_chain_with_msg(
                                "Failed to add new wireguard key to account data"
                            )
                        );
                        self.event_listener
                            .notify_key_event(KeygenEvent::GenerationFailure)
                    }
                }
            }
            Err(wireguard::Error::TooManyKeys) => {
                self.event_listener
                    .notify_key_event(KeygenEvent::TooManyKeys);
            }
            Err(e) => {
                log::error!(
                    "{}",
                    e.display_chain_with_msg("Failed to generate wireguard key")
                );
                self.event_listener
                    .notify_key_event(KeygenEvent::GenerationFailure);
            }
        }
    }

    async fn ensure_key_rotation(&mut self) {
        if let Some(token) = self.settings.get_account_token() {
            let rotation_interval = self
                .settings
                .tunnel_options
                .wireguard
                .automatic_rotation
                .map(|hours| Duration::from_secs(60u64 * 60u64 * hours as u64));

            self.wireguard_key_manager
                .set_rotation_interval(&mut self.account_history, token, rotation_interval)
                .await;
        }
    }

    async fn handle_new_account_event(
        &mut self,
        new_token: AccountToken,
        tx: oneshot::Sender<Result<String, mullvad_rpc::rest::Error>>,
    ) {
        match self.set_account(Some(new_token.clone())).await {
            Ok(_) => {
                self.set_target_state(TargetState::Unsecured);
                let _ = tx.send(Ok(new_token));
            }
            Err(err) => {
                log::error!("Failed to save new account - {}", err);
            }
        };
    }

    fn handle_new_app_version_info(&mut self, app_version_info: AppVersionInfo) {
        self.app_version_info = Some(app_version_info.clone());
        self.event_listener.notify_app_version(app_version_info);
    }

    fn on_set_target_state(&mut self, tx: oneshot::Sender<bool>, new_target_state: TargetState) {
        if self.state.is_running() {
            let state_change_initated = self.set_target_state(new_target_state);
            Self::oneshot_send(tx, state_change_initated, "state change initiated");
        } else {
            warn!("Ignoring target state change request due to shutdown");
        }
    }

    fn on_reconnect(&mut self, tx: oneshot::Sender<bool>) {
        if self.target_state == TargetState::Secured || self.tunnel_state.is_in_error_state() {
            self.connect_tunnel();
            Self::oneshot_send(tx, true, "reconnect issued");
        } else {
            debug!("Ignoring reconnect command. Currently not in secured state");
            Self::oneshot_send(tx, false, "reconnect issued");
        }
    }

    fn on_get_state(&self, tx: oneshot::Sender<TunnelState>) {
        Self::oneshot_send(tx, self.tunnel_state.clone(), "current state");
    }

    async fn on_get_current_location(&mut self, tx: oneshot::Sender<Option<GeoIpLocation>>) {
        use self::TunnelState::*;

        match &self.tunnel_state {
            Disconnected => {
                let location = self.get_geo_location();
                tokio::spawn(async {
                    Self::oneshot_send(tx, location.await.ok(), "current location");
                });
            }
            Connecting { location, .. } => {
                Self::oneshot_send(tx, location.clone(), "current location")
            }
            Disconnecting(..) => {
                Self::oneshot_send(tx, self.build_location_from_relay(), "current location")
            }
            Connected { location, .. } => {
                let relay_location = location.clone();
                let location_future = self.get_geo_location();
                tokio::spawn(async {
                    let location = location_future.await;
                    Self::oneshot_send(
                        tx,
                        location.ok().map(|fetched_location| GeoIpLocation {
                            ipv4: fetched_location.ipv4,
                            ipv6: fetched_location.ipv6,
                            ..relay_location.unwrap_or(fetched_location)
                        }),
                        "current location",
                    );
                });
            }
            Error(_) => {
                // We are not online at all at this stage so no location data is available.
                Self::oneshot_send(tx, None, "current location");
            }
        }
    }

    fn get_geo_location(&mut self) -> impl Future<Output = Result<GeoIpLocation, ()>> {
        let https_handle = self.rpc_runtime.rest_handle();

        async {
            geoip::send_location_request(https_handle)
                .await
                .map_err(|e| {
                    warn!("Unable to fetch GeoIP location: {}", e.display_chain());
                })
        }
    }

    fn build_location_from_relay(&self) -> Option<GeoIpLocation> {
        let relay = self.last_generated_relay.as_ref()?;
        let bridge_hostname = self
            .last_generated_bridge_relay
            .as_ref()
            .map(|bridge| bridge.hostname.clone());
        let location = relay.location.as_ref().cloned().unwrap();
        let hostname = relay.hostname.clone();

        Some(GeoIpLocation {
            ipv4: None,
            ipv6: None,
            country: location.country,
            city: Some(location.city),
            latitude: location.latitude,
            longitude: location.longitude,
            mullvad_exit_ip: true,
            hostname: Some(hostname),
            bridge_hostname,
        })
    }

    async fn on_create_new_account(
        &mut self,
        tx: oneshot::Sender<Result<String, mullvad_rpc::rest::Error>>,
    ) {
        let daemon_tx = self.tx.clone();
        let future = self.accounts_proxy.create_account();

        tokio::spawn(async move {
            match future.await {
                Ok(account_token) => {
                    let _ = daemon_tx.send(InternalDaemonEvent::NewAccountEvent(account_token, tx));
                }
                Err(err) => {
                    let _ = tx.send(Err(err));
                }
            }
        });
    }

    async fn on_get_account_data(
        &mut self,
        tx: oneshot::Sender<Result<AccountData, mullvad_rpc::rest::Error>>,
        account_token: AccountToken,
    ) {
        let expiry_fut = self.accounts_proxy.get_expiry(account_token);
        let rpc_call = async {
            let result = expiry_fut.await.map(|expiry| AccountData { expiry });
            Self::oneshot_send(tx, result, "account data");
        };
        tokio::spawn(rpc_call);
    }

    async fn on_get_www_auth_token(
        &mut self,
        tx: oneshot::Sender<Result<String, mullvad_rpc::rest::Error>>,
    ) {
        if let Some(account_token) = self.settings.get_account_token() {
            let future = self.accounts_proxy.get_www_auth_token(account_token);
            let rpc_call = async {
                Self::oneshot_send(tx, future.await, "get_www_auth_token response");
            };
            tokio::spawn(rpc_call);
        }
    }

    async fn on_submit_voucher(
        &mut self,
        tx: oneshot::Sender<Result<VoucherSubmission, mullvad_rpc::rest::Error>>,
        voucher: String,
    ) {
        if let Some(account_token) = self.settings.get_account_token() {
            let future = self.accounts_proxy.submit_voucher(account_token, voucher);
            let rpc_call = async {
                Self::oneshot_send(tx, future.await, "submit_voucher response");
            };
            tokio::spawn(rpc_call);
        }
    }

    fn on_get_relay_locations(&mut self, tx: oneshot::Sender<RelayList>) {
        Self::oneshot_send(tx, self.relay_selector.get_locations(), "relay locations");
    }

    async fn on_update_relay_locations(&mut self) {
        self.relay_selector.update().await;
    }

    async fn on_set_account(&mut self, tx: oneshot::Sender<()>, account_token: Option<String>) {
        match self.set_account(account_token.clone()).await {
            Ok(account_changed) => {
                if account_changed {
                    match account_token {
                        Some(_) => {
                            info!("Initiating tunnel restart because the account token changed");
                            self.reconnect_tunnel();
                        }
                        None => {
                            info!("Disconnecting because account token was cleared");
                            self.set_target_state(TargetState::Unsecured);
                        }
                    };
                }
                Self::oneshot_send(tx, (), "set_account response");
            }
            Err(e) => {
                log::error!("Failed to set account - {}", e);
            }
        }
    }

    async fn set_account(
        &mut self,
        account_token: Option<String>,
    ) -> Result<bool, settings::Error> {
        let account_changed = self.settings.set_account_token(account_token.clone())?;
        if account_changed {
            self.event_listener
                .notify_settings(self.settings.to_settings());

            // Bump account history if a token was set
            if let Some(token) = account_token.clone() {
                if let Err(e) = self.account_history.bump_history(&token).await {
                    log::error!("Failed to bump account history: {}", e);
                }
            }

            self.ensure_wireguard_keys_for_current_account().await;
        }
        Ok(account_changed)
    }

    fn on_get_account_history(&mut self, tx: oneshot::Sender<Vec<AccountToken>>) {
        Self::oneshot_send(
            tx,
            self.account_history.get_account_history(),
            "get_account_history response",
        );
    }

    async fn on_remove_account_from_history(
        &mut self,
        tx: oneshot::Sender<()>,
        account_token: AccountToken,
    ) {
        if self
            .account_history
            .remove_account(&account_token)
            .await
            .is_ok()
        {
            Self::oneshot_send(tx, (), "remove_account_from_history response");
        }
    }

    async fn on_clear_account_history(&mut self, tx: oneshot::Sender<()>) {
        match self.account_history.clear().await {
            Ok(_) => {
                self.set_target_state(TargetState::Unsecured);
                Self::oneshot_send(tx, (), "clear_account_history response");
            }
            Err(err) => log::error!(
                "{}",
                err.display_chain_with_msg("Failed to clear account history")
            ),
        }
    }

    async fn on_get_version_info(&mut self, tx: oneshot::Sender<Option<AppVersionInfo>>) {
        if self.app_version_info.is_none() {
            log::debug!("No version cache found. Fetching new info");
            let mut handle = self.version_updater_handle.clone();
            handle.run_version_check().await;
        }

        Self::oneshot_send(
            tx,
            self.app_version_info.clone(),
            "get_version_info response",
        );
    }

    fn on_get_current_version(&mut self, tx: oneshot::Sender<AppVersion>) {
        Self::oneshot_send(
            tx,
            version::PRODUCT_VERSION.to_owned(),
            "get_current_version response",
        );
    }

    #[cfg(not(target_os = "android"))]
    async fn on_factory_reset(&mut self, tx: oneshot::Sender<()>) {
        let mut failed = false;


        if let Err(e) = self.settings.reset() {
            log::error!("Failed to reset settings - {}", e);
            failed = true;
        }

        if let Err(e) = self.account_history.clear().await {
            log::error!("Failed to clear account history - {}", e);
            failed = true;
        }

        // Shut the daemon down.
        self.trigger_shutdown_event();

        self.shutdown_callbacks.push(Box::new(move || {
            if let Err(e) = Self::clear_cache_directory() {
                log::error!(
                    "{}",
                    e.display_chain_with_msg("Failed to clear cache directory")
                );
                failed = true;
            }

            if let Err(e) = Self::clear_log_directory() {
                log::error!(
                    "{}",
                    e.display_chain_with_msg("Failed to clear log directory")
                );
                failed = true;
            }
            if !failed {
                Self::oneshot_send(tx, (), "factory_reset response");
            }
        }));
    }

    #[cfg(target_os = "linux")]
    fn on_get_split_tunnel_processes(&mut self, tx: oneshot::Sender<Vec<i32>>) {
        match self.exclude_pids.list() {
            Ok(pids) => Self::oneshot_send(tx, pids, "get_split_tunnel_processes response"),
            Err(e) => error!("{}", e.display_chain_with_msg("Unable to obtain PIDs")),
        }
    }

    #[cfg(target_os = "linux")]
    fn on_add_split_tunnel_process(&mut self, tx: oneshot::Sender<()>, pid: i32) {
        match self.exclude_pids.add(pid) {
            Ok(()) => Self::oneshot_send(tx, (), "add_split_tunnel_process response"),
            Err(e) => error!("{}", e.display_chain_with_msg("Unable to add PID")),
        }
    }

    #[cfg(target_os = "linux")]
    fn on_remove_split_tunnel_process(&mut self, tx: oneshot::Sender<()>, pid: i32) {
        match self.exclude_pids.remove(pid) {
            Ok(()) => Self::oneshot_send(tx, (), "remove_split_tunnel_process response"),
            Err(e) => error!("{}", e.display_chain_with_msg("Unable to remove PID")),
        }
    }

    #[cfg(target_os = "linux")]
    fn on_clear_split_tunnel_processes(&mut self, tx: oneshot::Sender<()>) {
        match self.exclude_pids.clear() {
            Ok(()) => Self::oneshot_send(tx, (), "clear_split_tunnel_processes response"),
            Err(e) => error!("{}", e.display_chain_with_msg("Unable to clear PIDs")),
        }
    }

    fn on_update_relay_settings(&mut self, tx: oneshot::Sender<()>, update: RelaySettingsUpdate) {
        let save_result = self.settings.update_relay_settings(update);
        match save_result {
            Ok(settings_changed) => {
                Self::oneshot_send(tx, (), "update_relay_settings response");
                if settings_changed {
                    self.event_listener
                        .notify_settings(self.settings.to_settings());
                    info!("Initiating tunnel restart because the relay settings changed");
                    self.reconnect_tunnel();
                }
            }
            Err(e) => error!("{}", e.display_chain_with_msg("Unable to save settings")),
        }
    }

    fn on_set_allow_lan(&mut self, tx: oneshot::Sender<()>, allow_lan: bool) {
        let save_result = self.settings.set_allow_lan(allow_lan);
        match save_result {
            Ok(settings_changed) => {
                Self::oneshot_send(tx, (), "set_allow_lan response");
                if settings_changed {
                    self.event_listener
                        .notify_settings(self.settings.to_settings());
                    self.send_tunnel_command(TunnelCommand::AllowLan(allow_lan));
                }
            }
            Err(e) => error!("{}", e.display_chain_with_msg("Unable to save settings")),
        }
    }

    async fn on_set_show_beta_releases(&mut self, tx: oneshot::Sender<()>, enabled: bool) {
        let save_result = self.settings.set_show_beta_releases(enabled);
        match save_result {
            Ok(settings_changed) => {
                Self::oneshot_send(tx, (), "set_show_beta_releases response");
                if settings_changed {
                    self.event_listener
                        .notify_settings(self.settings.to_settings());
                    let mut handle = self.version_updater_handle.clone();
                    handle.set_show_beta_releases(enabled).await;
                }
            }
            Err(e) => error!("{}", e.display_chain_with_msg("Unable to save settings")),
        }
    }

    fn on_set_block_when_disconnected(
        &mut self,
        tx: oneshot::Sender<()>,
        block_when_disconnected: bool,
    ) {
        let save_result = self
            .settings
            .set_block_when_disconnected(block_when_disconnected);
        match save_result {
            Ok(settings_changed) => {
                Self::oneshot_send(tx, (), "set_block_when_disconnected response");
                if settings_changed {
                    self.event_listener
                        .notify_settings(self.settings.to_settings());
                    self.send_tunnel_command(TunnelCommand::BlockWhenDisconnected(
                        block_when_disconnected,
                    ));
                }
            }
            Err(e) => error!("{}", e.display_chain_with_msg("Unable to save settings")),
        }
    }

    fn on_set_auto_connect(&mut self, tx: oneshot::Sender<()>, auto_connect: bool) {
        let save_result = self.settings.set_auto_connect(auto_connect);
        match save_result {
            Ok(settings_changed) => {
                Self::oneshot_send(tx, (), "set auto-connect response");
                if settings_changed {
                    self.event_listener
                        .notify_settings(self.settings.to_settings());
                }
            }
            Err(e) => error!("{}", e.display_chain_with_msg("Unable to save settings")),
        }
    }

    fn on_set_openvpn_mssfix(&mut self, tx: oneshot::Sender<()>, mssfix_arg: Option<u16>) {
        let save_result = self.settings.set_openvpn_mssfix(mssfix_arg);
        match save_result {
            Ok(settings_changed) => {
                Self::oneshot_send(tx, (), "set_openvpn_mssfix response");
                if settings_changed {
                    self.event_listener
                        .notify_settings(self.settings.to_settings());
                    if let Some(TunnelType::OpenVpn) = self.get_connected_tunnel_type() {
                        info!(
                            "Initiating tunnel restart because the OpenVPN mssfix setting changed"
                        );
                        self.reconnect_tunnel();
                    }
                }
            }
            Err(e) => error!("{}", e.display_chain_with_msg("Unable to save settings")),
        }
    }

    fn on_set_bridge_settings(
        &mut self,
        tx: oneshot::Sender<Result<(), settings::Error>>,
        new_settings: BridgeSettings,
    ) {
        match self.settings.set_bridge_settings(new_settings) {
            Ok(settings_changes) => {
                if settings_changes {
                    self.event_listener
                        .notify_settings(self.settings.to_settings());
                    self.reconnect_tunnel();
                };
                Self::oneshot_send(tx, Ok(()), "set_bridge_settings");
            }

            Err(e) => {
                log::error!(
                    "{}",
                    e.display_chain_with_msg("Failed to set new bridge settings")
                );
                Self::oneshot_send(tx, Err(e), "set_bridge_settings");
            }
        }
    }

    fn on_set_bridge_state(
        &mut self,
        tx: oneshot::Sender<Result<(), settings::Error>>,
        bridge_state: BridgeState,
    ) {
        let result = match self.settings.set_bridge_state(bridge_state) {
            Ok(settings_changed) => {
                if settings_changed {
                    self.event_listener
                        .notify_settings(self.settings.to_settings());
                    log::info!("Initiating tunnel restart because bridge state changed");
                    self.reconnect_tunnel();
                }
                Ok(())
            }
            Err(error) => {
                log::error!(
                    "{}",
                    error.display_chain_with_msg("Failed to set new bridge state")
                );
                Err(error)
            }
        };
        Self::oneshot_send(tx, result, "on_set_bridge_state response");
    }


    fn on_set_enable_ipv6(&mut self, tx: oneshot::Sender<()>, enable_ipv6: bool) {
        let save_result = self.settings.set_enable_ipv6(enable_ipv6);
        match save_result {
            Ok(settings_changed) => {
                Self::oneshot_send(tx, (), "set_enable_ipv6 response");
                if settings_changed {
                    self.event_listener
                        .notify_settings(self.settings.to_settings());
                    info!("Initiating tunnel restart because the enable IPv6 setting changed");
                    self.reconnect_tunnel();
                }
            }
            Err(e) => error!("{}", e.display_chain_with_msg("Unable to save settings")),
        }
    }

    fn on_set_dns_options(&mut self, tx: oneshot::Sender<()>, dns_options: DnsOptions) {
        let save_result = self.settings.set_dns_options(dns_options.clone());
        match save_result {
            Ok(settings_changed) => {
                Self::oneshot_send(tx, (), "set_dns_options response");
                if settings_changed {
                    let settings = self.settings.to_settings();
                    let resolvers =
                        Self::get_custom_resolvers(&settings.tunnel_options.dns_options);
                    self.event_listener.notify_settings(settings);
                    self.send_tunnel_command(TunnelCommand::CustomDns(resolvers));
                }
            }
            Err(e) => error!("{}", e.display_chain_with_msg("Unable to save settings")),
        }
    }

    fn on_set_wireguard_mtu(&mut self, tx: oneshot::Sender<()>, mtu: Option<u16>) {
        let save_result = self.settings.set_wireguard_mtu(mtu);
        match save_result {
            Ok(settings_changed) => {
                Self::oneshot_send(tx, (), "set_wireguard_mtu response");
                if settings_changed {
                    self.event_listener
                        .notify_settings(self.settings.to_settings());
                    if let Some(TunnelType::Wireguard) = self.get_connected_tunnel_type() {
                        info!(
                            "Initiating tunnel restart because the WireGuard MTU setting changed"
                        );
                        self.reconnect_tunnel();
                    }
                }
            }
            Err(e) => error!("{}", e.display_chain_with_msg("Unable to save settings")),
        }
    }

    async fn on_set_wireguard_rotation_interval(
        &mut self,
        tx: oneshot::Sender<()>,
        interval: Option<u32>,
    ) {
        let save_result = self.settings.set_wireguard_rotation_interval(interval);
        match save_result {
            Ok(settings_changed) => {
                Self::oneshot_send(tx, (), "set_wireguard_rotation_interval response");
                if settings_changed {
                    self.ensure_key_rotation().await;
                    self.event_listener
                        .notify_settings(self.settings.to_settings());
                }
            }
            Err(e) => error!("{}", e.display_chain_with_msg("Unable to save settings")),
        }
    }

    async fn ensure_wireguard_keys_for_current_account(&mut self) {
        if let Some(account) = self.settings.get_account_token() {
            if self
                .account_history
                .get(&account)
                .await
                .map(|entry| entry.map(|e| e.wireguard.is_none()).unwrap_or(true))
                .unwrap_or(true)
            {
                log::info!("Automatically generating new wireguard key for account");
                self.wireguard_key_manager
                    .spawn_key_generation_task(account, Some(FIRST_KEY_PUSH_TIMEOUT))
                    .await;
            } else {
                log::info!("Account already has wireguard key, starting key rotation.");
                self.ensure_key_rotation().await;
            }
        }
    }

    async fn on_generate_wireguard_key(&mut self, tx: oneshot::Sender<KeygenEvent>) {
        match self.on_generate_wireguard_key_inner().await {
            Ok(key_event) => {
                Self::oneshot_send(tx, key_event, "generate_wireguard_key response");
            }
            Err(e) => {
                log::error!("Failed to generate new wireguard key - {}", e);
            }
        }
    }

    async fn on_generate_wireguard_key_inner(&mut self) -> Result<KeygenEvent, String> {
        let account_token = self
            .settings
            .get_account_token()
            .ok_or_else(|| "No account token set".to_owned())?;

        let mut account_entry = self
            .account_history
            .get(&account_token)
            .await
            .map_err(|e| format!("Failed to read account entry from history: {}", e))
            .map(|data| {
                data.unwrap_or_else(|| {
                    log::error!("Account token set in settings but not in account history");
                    account_history::AccountEntry {
                        account: account_token.clone(),
                        wireguard: None,
                    }
                })
            })?;

        let gen_result = match &account_entry.wireguard {
            Some(wireguard_data) => {
                self.wireguard_key_manager
                    .replace_key(account_token.clone(), wireguard_data.get_public_key())
                    .await
            }
            None => {
                self.wireguard_key_manager
                    .generate_key_sync(account_token.clone())
                    .await
            }
        };

        match gen_result {
            Ok(new_data) => {
                let public_key = new_data.get_public_key();
                account_entry.wireguard = Some(new_data);
                self.account_history
                    .insert(account_entry)
                    .await
                    .map_err(|e| {
                        format!("Failed to add new wireguard key to account data: {}", e)
                    })?;
                if let Some(TunnelType::Wireguard) = self.get_connected_tunnel_type() {
                    self.reconnect_tunnel();
                }
                let keygen_event = KeygenEvent::NewKey(public_key);
                self.event_listener.notify_key_event(keygen_event.clone());

                // update automatic rotation
                self.wireguard_key_manager
                    .set_rotation_interval(
                        &mut self.account_history,
                        account_token,
                        self.settings
                            .tunnel_options
                            .wireguard
                            .automatic_rotation
                            .map(|hours| Duration::from_secs(60u64 * 60u64 * hours as u64)),
                    )
                    .await;

                Ok(keygen_event)
            }
            Err(wireguard::Error::TooManyKeys) => Ok(KeygenEvent::TooManyKeys),
            Err(e) => Err(format!(
                "Failed to generate new key - {}",
                e.display_chain_with_msg("Failed to generate new wireguard key:")
            )),
        }
    }

    async fn on_get_wireguard_key(&mut self, tx: oneshot::Sender<Option<wireguard::PublicKey>>) {
        let token = self.settings.get_account_token();
        if let Some(token) = token {
            let entry = self.account_history.get(&token).await;
            if let Ok(Some(entry)) = entry {
                let key = entry.wireguard.map(|wg| wg.get_public_key());
                Self::oneshot_send(tx, key, "get_wireguard_key response");
            }
        }
    }

    async fn on_verify_wireguard_key(&mut self, tx: oneshot::Sender<bool>) {
        let account = match self.settings.get_account_token() {
            Some(account) => account,
            None => {
                Self::oneshot_send(tx, false, "verify_wireguard_key response");
                return;
            }
        };

        let key = self
            .account_history
            .get(&account)
            .await
            .map(|entry| entry.and_then(|e| e.wireguard.map(|wg| wg.private_key.public_key())));

        let public_key = match key {
            Ok(Some(public_key)) => public_key,
            Ok(None) => {
                Self::oneshot_send(tx, false, "verify_wireguard_key response");
                return;
            }
            Err(e) => {
                log::error!("Failed to read key data: {}", e);
                return;
            }
        };

        let verification_rpc = self
            .wireguard_key_manager
            .verify_wireguard_key(account, public_key);

        tokio::spawn(async move {
            match verification_rpc.await {
                Ok(is_valid) => {
                    Self::oneshot_send(tx, is_valid, "verify_wireguard_key response");
                }
                Err(err) => {
                    log::error!("Failed to verify wireguard key - {}", err);
                }
            }
        });
    }

    fn on_get_settings(&self, tx: oneshot::Sender<Settings>) {
        Self::oneshot_send(tx, self.settings.to_settings(), "get_settings response");
    }

    fn oneshot_send<T>(tx: oneshot::Sender<T>, t: T, msg: &'static str) {
        if tx.send(t).is_err() {
            warn!("Unable to send {} to the daemon command sender", msg);
        }
    }

    fn trigger_shutdown_event(&mut self) {
        self.state.shutdown(&self.tunnel_state);
        self.disconnect_tunnel();
    }

    fn on_prepare_restart(&mut self) {
        // TODO: See if this can be made to also shut down the daemon
        //       without causing the service to be restarted.

        if self.target_state == TargetState::Secured {
            self.send_tunnel_command(TunnelCommand::BlockWhenDisconnected(true));
        }

        self.clean_up_target_cache = false;
    }

    /// Set the target state of the client. If it changed trigger the operations needed to
    /// progress towards that state.
    /// Returns a bool representing whether or not a state change was initiated.
    fn set_target_state(&mut self, new_state: TargetState) -> bool {
        if new_state != self.target_state || self.tunnel_state.is_in_error_state() {
            debug!("Target state {:?} => {:?}", self.target_state, new_state);

            if new_state != self.target_state {
                self.target_state = new_state;

                // Cache the current target state
                let cache_file = self.cache_dir.join(TARGET_START_STATE_FILE);
                log::trace!("Saving tunnel target state to {}", cache_file.display());
                match File::create(&cache_file) {
                    Ok(handle) => {
                        if let Err(e) =
                            serde_json::to_writer(io::BufWriter::new(handle), &self.target_state)
                        {
                            log::error!("Failed to cache target state: {}", e);
                        }
                    }
                    Err(e) => {
                        log::error!("Failed to cache target state: {}", e);
                    }
                }
            }

            match self.target_state {
                TargetState::Secured => self.connect_tunnel(),
                TargetState::Unsecured => self.disconnect_tunnel(),
            }
            true
        } else {
            false
        }
    }

    fn connect_tunnel(&mut self) {
        self.send_tunnel_command(TunnelCommand::Connect);
    }

    fn disconnect_tunnel(&mut self) {
        self.send_tunnel_command(TunnelCommand::Disconnect);
    }

    fn reconnect_tunnel(&mut self) {
        if self.target_state == TargetState::Secured {
            self.connect_tunnel();
        }
    }

    fn get_connected_tunnel_type(&self) -> Option<TunnelType> {
        if let TunnelState::Connected {
            endpoint: TunnelEndpoint { tunnel_type, .. },
            ..
        } = self.tunnel_state
        {
            Some(tunnel_type)
        } else {
            None
        }
    }

    fn get_target_tunnel_type(&self) -> Option<TunnelType> {
        match self.tunnel_state {
            TunnelState::Connected {
                endpoint: TunnelEndpoint { tunnel_type, .. },
                ..
            }
            | TunnelState::Connecting {
                endpoint: TunnelEndpoint { tunnel_type, .. },
                ..
            } => Some(tunnel_type),
            _ => None,
        }
    }

    fn send_tunnel_command(&mut self, command: TunnelCommand) {
        self.tunnel_command_tx
            .unbounded_send(command)
            .expect("Tunnel state machine has stopped");
    }

    #[cfg(not(target_os = "android"))]
    fn clear_log_directory() -> Result<(), Error> {
        let log_dir = mullvad_paths::get_log_dir().map_err(Error::PathError)?;
        Self::clear_directory(&log_dir)
    }

    #[cfg(not(target_os = "android"))]
    fn clear_cache_directory() -> Result<(), Error> {
        let cache_dir = mullvad_paths::cache_dir().map_err(Error::PathError)?;
        Self::clear_directory(&cache_dir)
    }

    #[cfg(not(target_os = "android"))]
    fn clear_directory(path: &Path) -> Result<(), Error> {
        #[cfg(not(target_os = "windows"))]
        {
            fs::remove_dir_all(path)
                .map_err(|e| Error::RemoveDirError(path.display().to_string(), e))?;
            fs::create_dir_all(path)
                .map_err(|e| Error::CreateDirError(path.display().to_string(), e))
        }
        #[cfg(target_os = "windows")]
        {
            fs::read_dir(&path)
                .map_err(Error::ReadDirError)
                .and_then(|dir_entries| {
                    dir_entries
                        .into_iter()
                        .map(|entry| {
                            let entry = entry.map_err(Error::FileEntryError)?;
                            let entry_type = entry.file_type().map_err(Error::FileTypeError)?;


                            let removal = if entry_type.is_file() || entry_type.is_symlink() {
                                fs::remove_file(entry.path())
                            } else {
                                fs::remove_dir_all(entry.path())
                            };
                            removal.map_err(|e| {
                                Error::RemoveDirError(entry.path().display().to_string(), e)
                            })
                        })
                        .collect::<Result<(), Error>>()
                })
        }
    }


    pub fn shutdown_handle(&self) -> DaemonShutdownHandle {
        DaemonShutdownHandle {
            tx: self.tx.clone(),
        }
    }
}

pub struct DaemonShutdownHandle {
    tx: DaemonEventSender,
}

impl DaemonShutdownHandle {
    pub fn shutdown(&self) {
        let _ = self.tx.send(InternalDaemonEvent::TriggerShutdown);
    }
}

struct MullvadTunnelParametersGenerator {
    tx: DaemonEventSender,
}

impl TunnelParametersGenerator for MullvadTunnelParametersGenerator {
    fn generate(
        &mut self,
        retry_attempt: u32,
    ) -> Result<TunnelParameters, ParameterGenerationError> {
        let (response_tx, response_rx) = sync_mpsc::channel();
        if self
            .tx
            .send(InternalDaemonEvent::GenerateTunnelParameters(
                response_tx,
                retry_attempt,
            ))
            .is_err()
        {
            log::error!("Failed to send daemon command to generate tunnel parameters!");
            return Err(ParameterGenerationError::NoMatchingRelay);
        }

        match response_rx.recv() {
            Ok(result) => result,
            Err(_) => {
                log::error!("Failed to receive tunnel parameter generation result!");
                Err(ParameterGenerationError::NoMatchingRelay)
            }
        }
    }
}

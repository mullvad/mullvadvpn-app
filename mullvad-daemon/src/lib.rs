#![deny(rust_2018_idioms)]
#![recursion_limit = "512"]

mod access_method;
pub mod account_history;
mod api;
#[cfg(not(target_os = "android"))]
mod cleanup;
mod custom_list;
pub mod device;
mod dns;
pub mod exception_logging;
mod geoip;
pub mod logging;
#[cfg(target_os = "macos")]
mod macos;
#[cfg(not(target_os = "android"))]
pub mod management_interface;
mod migrations;
#[cfg(not(target_os = "android"))]
pub mod rpc_uniqueness_check;
pub mod runtime;
pub mod settings;
pub mod shutdown;
mod target_state;
mod tunnel;
pub mod version;
mod version_check;

use crate::{geoip::get_geo_location, target_state::PersistentTargetState};
use api::AccessMethodEvent;
use device::{AccountEvent, PrivateAccountAndDevice, PrivateDeviceEvent};
use futures::{
    channel::{mpsc, oneshot},
    future::{abortable, AbortHandle, Future, LocalBoxFuture},
    StreamExt,
};
use mullvad_api::proxy::ApiConnectionMode;
use mullvad_relay_selector::{
    updater::{RelayListUpdater, RelayListUpdaterHandle},
    RelaySelector, SelectorConfig,
};
#[cfg(target_os = "android")]
use mullvad_types::account::{PlayPurchase, PlayPurchasePaymentToken};
use mullvad_types::{
    access_method::{AccessMethod, AccessMethodSetting},
    account::{AccountData, AccountToken, VoucherSubmission},
    auth_failed::AuthFailed,
    custom_list::CustomList,
    device::{Device, DeviceEvent, DeviceEventCause, DeviceId, DeviceState, RemoveDeviceEvent},
    location::GeoIpLocation,
    relay_constraints::{
        BridgeSettings, BridgeState, ObfuscationSettings, RelayOverride, RelaySettings,
    },
    relay_list::RelayList,
    settings::{DnsOptions, Settings},
    states::{TargetState, TunnelState},
    version::{AppVersion, AppVersionInfo},
    wireguard::{PublicKey, QuantumResistantState, RotationInterval},
};
use settings::SettingsPersister;
#[cfg(target_os = "android")]
use std::os::unix::io::RawFd;
#[cfg(target_os = "windows")]
use std::{collections::HashSet, ffi::OsString};
use std::{
    marker::PhantomData,
    mem,
    path::PathBuf,
    pin::Pin,
    sync::{Arc, Weak},
    time::Duration,
};
#[cfg(any(target_os = "linux", windows))]
use talpid_core::split_tunnel;
use talpid_core::{
    mpsc::Sender,
    tunnel_state_machine::{self, TunnelCommand, TunnelStateMachineHandle},
};
#[cfg(target_os = "android")]
use talpid_types::android::AndroidContext;
#[cfg(target_os = "windows")]
use talpid_types::split_tunnel::ExcludedProcess;
use talpid_types::{
    net::{TunnelEndpoint, TunnelType},
    tunnel::{ErrorStateCause, TunnelStateTransition},
    ErrorExt,
};
#[cfg(any(target_os = "macos", target_os = "linux"))]
use tokio::fs;
use tokio::io;

/// Delay between generating a new WireGuard key and reconnecting
const WG_RECONNECT_DELAY: Duration = Duration::from_secs(4 * 60);

pub type ResponseTx<T, E> = oneshot::Sender<Result<T, E>>;

#[derive(err_derive::Error, Debug)]
#[error(no_from)]
pub enum Error {
    #[error(display = "Failed to send command to daemon because it is not running")]
    DaemonUnavailable,

    #[error(display = "Unable to initialize network event loop")]
    InitIoEventLoop(#[error(source)] io::Error),

    #[error(display = "Unable to create RPC client")]
    InitRpcFactory(#[error(source)] mullvad_api::Error),

    #[error(display = "REST request failed")]
    RestError(#[error(source)] mullvad_api::rest::Error),

    #[error(display = "API availability check failed")]
    ApiCheckError(#[error(source)] mullvad_api::availability::Error),

    #[error(display = "Unable to load account history")]
    LoadAccountHistory(#[error(source)] account_history::Error),

    #[error(display = "Failed to start account manager")]
    LoadAccountManager(#[error(source)] device::Error),

    #[error(display = "Failed to log in to account")]
    LoginError(#[error(source)] device::Error),

    #[error(display = "Failed to log out of account")]
    LogoutError(#[error(source)] device::Error),

    #[error(display = "Failed to rotate WireGuard key")]
    KeyRotationError(#[error(source)] device::Error),

    #[error(display = "Failed to list devices")]
    ListDevicesError(#[error(source)] device::Error),

    #[error(display = "Failed to remove device")]
    RemoveDeviceError(#[error(source)] device::Error),

    #[error(display = "Failed to update device")]
    UpdateDeviceError(#[error(source)] device::Error),

    #[error(display = "Failed to submit voucher")]
    VoucherSubmission(#[error(source)] device::Error),

    #[cfg(target_os = "linux")]
    #[error(display = "Unable to initialize split tunneling")]
    InitSplitTunneling(#[error(source)] split_tunnel::Error),

    #[cfg(windows)]
    #[error(display = "Split tunneling error")]
    SplitTunnelError(#[error(source)] split_tunnel::Error),

    #[error(display = "An account is already set")]
    AlreadyLoggedIn,

    #[error(display = "No account token is set")]
    NoAccountToken,

    #[error(display = "No account history available for the token")]
    NoAccountTokenHistory,

    #[error(display = "Settings error")]
    SettingsError(#[error(source)] settings::Error),

    #[error(display = "Account history error")]
    AccountHistory(#[error(source)] account_history::Error),

    #[cfg(not(target_os = "android"))]
    #[error(display = "Factory reset partially failed: {}", _0)]
    FactoryResetError(&'static str),

    #[error(display = "Tunnel state machine error")]
    TunnelError(#[error(source)] tunnel_state_machine::Error),

    /// Custom list already exists
    #[error(display = "A list with that name already exists")]
    CustomListExists,

    /// Custom list does not exist
    #[error(display = "A list with that name does not exist")]
    CustomListNotFound,

    #[error(display = "Access method error")]
    AccessMethodError(#[error(source)] access_method::Error),

    #[error(display = "API connection mode error")]
    ApiConnectionModeError(#[error(source)] api::Error),

    #[cfg(target_os = "macos")]
    #[error(display = "Failed to set exclusion group")]
    GroupIdError(#[error(source)] io::Error),

    #[cfg(target_os = "android")]
    #[error(display = "Failed to initialize play purchase")]
    InitPlayPurchase(#[error(source)] device::Error),

    #[cfg(target_os = "android")]
    #[error(display = "Failed to verify play purchase")]
    VerifyPlayPurchase(#[error(source)] device::Error),
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
    CreateNewAccount(ResponseTx<String, Error>),
    /// Request the metadata for an account.
    GetAccountData(
        ResponseTx<AccountData, mullvad_api::rest::Error>,
        AccountToken,
    ),
    /// Request www auth token for an account
    GetWwwAuthToken(ResponseTx<String, Error>),
    /// Submit voucher to add time to the current account. Returns time added in seconds
    SubmitVoucher(ResponseTx<VoucherSubmission, Error>, String),
    /// Request account history
    GetAccountHistory(oneshot::Sender<Option<AccountToken>>),
    /// Remove the last used account, if there is one
    ClearAccountHistory(ResponseTx<(), Error>),
    /// Get the list of countries and cities where there are relays.
    GetRelayLocations(oneshot::Sender<RelayList>),
    /// Trigger an asynchronous relay list update. This returns before the relay list is actually
    /// updated.
    UpdateRelayLocations,
    /// Log in with a given account and create a new device.
    LoginAccount(ResponseTx<(), Error>, AccountToken),
    /// Log out of the current account and remove the device, if they exist.
    LogoutAccount(ResponseTx<(), Error>),
    /// Return the current device configuration.
    GetDevice(ResponseTx<DeviceState, Error>),
    /// Update/check the current device, if there is one.
    UpdateDevice(ResponseTx<(), Error>),
    /// Return all the devices for a given account token.
    ListDevices(ResponseTx<Vec<Device>, Error>, AccountToken),
    /// Remove device from a given account.
    RemoveDevice(ResponseTx<(), Error>, AccountToken, DeviceId),
    /// Place constraints on the type of tunnel and relay
    SetRelaySettings(ResponseTx<(), settings::Error>, RelaySettings),
    /// Set the allow LAN setting.
    SetAllowLan(ResponseTx<(), settings::Error>, bool),
    /// Set the beta program setting.
    SetShowBetaReleases(ResponseTx<(), settings::Error>, bool),
    /// Set the block_when_disconnected setting.
    SetBlockWhenDisconnected(ResponseTx<(), settings::Error>, bool),
    /// Set the auto-connect setting.
    SetAutoConnect(ResponseTx<(), settings::Error>, bool),
    /// Set the mssfix argument for OpenVPN
    SetOpenVpnMssfix(ResponseTx<(), settings::Error>, Option<u16>),
    /// Set proxy details for OpenVPN
    SetBridgeSettings(ResponseTx<(), settings::Error>, BridgeSettings),
    /// Set proxy state
    SetBridgeState(ResponseTx<(), settings::Error>, BridgeState),
    /// Set if IPv6 should be enabled in the tunnel
    SetEnableIpv6(ResponseTx<(), settings::Error>, bool),
    /// Set whether to enable PQ PSK exchange in the tunnel
    SetQuantumResistantTunnel(ResponseTx<(), settings::Error>, QuantumResistantState),
    /// Set DNS options or servers to use
    SetDnsOptions(ResponseTx<(), settings::Error>, DnsOptions),
    /// Set override options to use for a given relay
    SetRelayOverride(ResponseTx<(), settings::Error>, RelayOverride),
    /// Remove all relay override options
    ClearAllRelayOverrides(ResponseTx<(), settings::Error>),
    /// Toggle macOS network check leak
    /// Set MTU for wireguard tunnels
    SetWireguardMtu(ResponseTx<(), settings::Error>, Option<u16>),
    /// Set automatic key rotation interval for wireguard tunnels
    SetWireguardRotationInterval(ResponseTx<(), settings::Error>, Option<RotationInterval>),
    /// Get the daemon settings
    GetSettings(oneshot::Sender<Settings>),
    /// Generate new wireguard key
    RotateWireguardKey(ResponseTx<(), Error>),
    /// Return a public key of the currently set wireguard private key, if there is one
    GetWireguardKey(ResponseTx<Option<PublicKey>, Error>),
    /// Create custom list
    CreateCustomList(ResponseTx<mullvad_types::custom_list::Id, Error>, String),
    /// Delete custom list
    DeleteCustomList(ResponseTx<(), Error>, mullvad_types::custom_list::Id),
    /// Update a custom list with a given id
    UpdateCustomList(ResponseTx<(), Error>, CustomList),
    /// Get API access methods
    GetApiAccessMethods(ResponseTx<Vec<AccessMethodSetting>, Error>),
    /// Add API access methods
    AddApiAccessMethod(
        ResponseTx<mullvad_types::access_method::Id, Error>,
        String,
        bool,
        AccessMethod,
    ),
    /// Remove an API access method
    RemoveApiAccessMethod(ResponseTx<(), Error>, mullvad_types::access_method::Id),
    /// Set the API access method to use
    SetApiAccessMethod(ResponseTx<(), Error>, mullvad_types::access_method::Id),
    /// Edit an API access method
    UpdateApiAccessMethod(ResponseTx<(), Error>, AccessMethodSetting),
    /// Get the currently used API access method
    GetCurrentAccessMethod(ResponseTx<AccessMethodSetting, Error>),
    /// Test an API access method
    TestApiAccessMethod(ResponseTx<bool, Error>, mullvad_types::access_method::Id),
    /// Get information about the currently running and latest app versions
    GetVersionInfo(oneshot::Sender<Option<AppVersionInfo>>),
    /// Return whether the daemon is performing post-upgrade tasks
    IsPerformingPostUpgrade(oneshot::Sender<bool>),
    /// Get current version of the app
    GetCurrentVersion(oneshot::Sender<AppVersion>),
    /// Remove settings and clear the cache
    #[cfg(not(target_os = "android"))]
    FactoryReset(ResponseTx<(), Error>),
    /// Request list of processes excluded from the tunnel
    #[cfg(target_os = "linux")]
    GetSplitTunnelProcesses(ResponseTx<Vec<i32>, split_tunnel::Error>),
    /// Exclude traffic of a process (PID) from the tunnel
    #[cfg(target_os = "linux")]
    AddSplitTunnelProcess(ResponseTx<(), split_tunnel::Error>, i32),
    /// Remove process (PID) from list of processes excluded from the tunnel
    #[cfg(target_os = "linux")]
    RemoveSplitTunnelProcess(ResponseTx<(), split_tunnel::Error>, i32),
    /// Clear list of processes excluded from the tunnel
    #[cfg(target_os = "linux")]
    ClearSplitTunnelProcesses(ResponseTx<(), split_tunnel::Error>),
    /// Exclude traffic of an application from the tunnel
    #[cfg(windows)]
    AddSplitTunnelApp(ResponseTx<(), Error>, PathBuf),
    /// Remove application from list of apps to exclude from the tunnel
    #[cfg(windows)]
    RemoveSplitTunnelApp(ResponseTx<(), Error>, PathBuf),
    /// Clear list of apps to exclude from the tunnel
    #[cfg(windows)]
    ClearSplitTunnelApps(ResponseTx<(), Error>),
    /// Enable or disable split tunneling
    #[cfg(windows)]
    SetSplitTunnelState(ResponseTx<(), Error>, bool),
    /// Returns all processes currently being excluded from the tunnel
    #[cfg(windows)]
    GetSplitTunnelProcesses(ResponseTx<Vec<ExcludedProcess>, split_tunnel::Error>),
    /// Notify the split tunnel monitor that a volume was mounted or dismounted
    #[cfg(target_os = "windows")]
    CheckVolumes(ResponseTx<(), Error>),
    /// Register settings for WireGuard obfuscator
    SetObfuscationSettings(ResponseTx<(), settings::Error>, ObfuscationSettings),
    /// Saves the target tunnel state and enters a blocking state. The state is restored
    /// upon restart.
    PrepareRestart,
    /// Causes a socket to bypass the tunnel. This has no effect when connected. It is only used
    /// to bypass the tunnel in blocking states.
    #[cfg(target_os = "android")]
    BypassSocket(RawFd, oneshot::Sender<()>),
    /// Initialize a google play purchase through the API.
    #[cfg(target_os = "android")]
    InitPlayPurchase(ResponseTx<PlayPurchasePaymentToken, Error>),
    /// Verify that a google play payment was successful through the API.
    #[cfg(target_os = "android")]
    VerifyPlayPurchase(ResponseTx<(), Error>, PlayPurchase),
    /// Patch the settings using a blob of JSON settings
    ApplyJsonSettings(ResponseTx<(), settings::patch::Error>, String),
}

/// All events that can happen in the daemon. Sent from various threads and exposed interfaces.
pub(crate) enum InternalDaemonEvent {
    /// Tunnel has changed state.
    TunnelStateTransition(TunnelStateTransition),
    /// A command sent to the daemon.
    Command(DaemonCommand),
    /// Daemon shutdown triggered by a signal, ctrl-c or similar.
    /// The boolean should indicate whether the shutdown was user-initiated.
    TriggerShutdown(bool),
    /// The background job fetching new `AppVersionInfo`s got a new info object.
    NewAppVersionInfo(AppVersionInfo),
    /// Sent when a device is updated in any way (key rotation, login, logout, etc.).
    DeviceEvent(AccountEvent),
    /// Sent when access methods are changed in any way (new active access method).
    AccessMethodEvent(AccessMethodEvent),
    /// Handles updates from versions without devices.
    DeviceMigrationEvent(Result<PrivateAccountAndDevice, device::Error>),
    /// The split tunnel paths or state were updated.
    #[cfg(target_os = "windows")]
    ExcludedPathsEvent(ExcludedPathsUpdate, oneshot::Sender<Result<(), Error>>),
}

#[cfg(target_os = "windows")]
pub(crate) enum ExcludedPathsUpdate {
    SetState(bool),
    SetPaths(HashSet<PathBuf>),
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

impl From<AccountEvent> for InternalDaemonEvent {
    fn from(event: AccountEvent) -> Self {
        InternalDaemonEvent::DeviceEvent(event)
    }
}

impl From<AccessMethodEvent> for InternalDaemonEvent {
    fn from(event: AccessMethodEvent) -> Self {
        InternalDaemonEvent::AccessMethodEvent(event)
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

impl Default for DaemonCommandChannel {
    fn default() -> Self {
        Self::new()
    }
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

    /// Shuts down the daemon. This triggers the shutdown as though the user would shut it down
    /// because blocking traffic on Android relies on the daemon process being alive and keeping a
    /// tunnel device open.
    #[cfg(target_os = "android")]
    pub fn shutdown(&self) -> Result<(), Error> {
        self.0
            .unbounded_send(InternalDaemonEvent::TriggerShutdown(true))
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
    fn send(&self, event: E) -> Result<(), talpid_core::mpsc::Error> {
        if let Some(sender) = self.sender.upgrade() {
            sender
                .unbounded_send(InternalDaemonEvent::from(event))
                .map_err(|_| talpid_core::mpsc::Error::ChannelClosed)
        } else {
            Err(talpid_core::mpsc::Error::ChannelClosed)
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

    /// Notify that device changed (login, logout, or key rotation).
    fn notify_device_event(&self, event: DeviceEvent);

    /// Notify that a device was revoked using `RemoveDevice`.
    fn notify_remove_device_event(&self, event: RemoveDeviceEvent);

    /// Notify that the api access method changed.
    fn notify_new_access_method(&self, new_access_method: AccessMethodSetting);
}

pub struct Daemon<L: EventListener> {
    tunnel_state: TunnelState,
    target_state: PersistentTargetState,
    state: DaemonExecutionState,
    #[cfg(target_os = "linux")]
    exclude_pids: split_tunnel::PidManager,
    rx: mpsc::UnboundedReceiver<InternalDaemonEvent>,
    tx: DaemonEventSender,
    reconnection_job: Option<AbortHandle>,
    event_listener: L,
    migration_complete: migrations::MigrationComplete,
    settings: SettingsPersister,
    account_history: account_history::AccountHistory,
    device_checker: device::TunnelStateChangeHandler,
    account_manager: device::AccountManagerHandle,
    connection_modes_handler: api::AccessModeSelectorHandle,
    api_runtime: mullvad_api::Runtime,
    api_handle: mullvad_api::rest::MullvadRestHandle,
    version_updater_handle: version_check::VersionUpdaterHandle,
    relay_selector: RelaySelector,
    relay_list_updater: RelayListUpdaterHandle,
    parameters_generator: tunnel::ParametersGenerator,
    app_version_info: Option<AppVersionInfo>,
    shutdown_tasks: Vec<Pin<Box<dyn Future<Output = ()>>>>,
    tunnel_state_machine_handle: TunnelStateMachineHandle,
    #[cfg(target_os = "windows")]
    volume_update_tx: mpsc::UnboundedSender<()>,
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
        #[cfg(target_os = "macos")]
        macos::bump_filehandle_limit();

        mullvad_api::proxy::ApiConnectionMode::try_delete_cache(&cache_dir).await;

        let (internal_event_tx, internal_event_rx) = command_channel.destructure();

        let api_runtime = mullvad_api::Runtime::with_cache(
            &cache_dir,
            true,
            #[cfg(target_os = "android")]
            api::create_bypass_tx(&internal_event_tx),
        )
        .await
        .map_err(Error::InitRpcFactory)?;

        let api_availability = api_runtime.availability_handle();
        api_availability.suspend();

        let migration_data = migrations::migrate_all(&cache_dir, &settings_dir)
            .await
            .unwrap_or_else(|error| {
                log::error!(
                    "{}",
                    error.display_chain_with_msg("Failed to migrate settings or cache")
                );
                None
            });

        let settings_event_listener = event_listener.clone();
        let mut settings = SettingsPersister::load(&settings_dir).await;
        settings.register_change_listener(move |settings| {
            // Notify management interface server of changes to the settings
            settings_event_listener.notify_settings(settings.to_owned());
        });

        let app_version_info = version_check::load_cache(&cache_dir).await;

        let initial_selector_config = new_selector_config(&settings);
        let relay_selector = RelaySelector::new(initial_selector_config, &resource_dir, &cache_dir);

        let settings_relay_selector = relay_selector.clone();
        settings.register_change_listener(move |settings| {
            // Notify relay selector of changes to the settings/selector config
            settings_relay_selector
                .clone()
                .set_config(new_selector_config(settings));
        });

        let connection_modes = settings.api_access_methods.collect_enabled();

        let connection_modes_handler = api::AccessModeSelector::spawn(
            cache_dir.clone(),
            relay_selector.clone(),
            connection_modes,
            internal_event_tx.to_specialized_sender(),
        );

        let api_handle = api_runtime
            .mullvad_rest_handle(Box::pin(connection_modes_handler.clone().into_stream()))
            .await;

        let migration_complete = if let Some(migration_data) = migration_data {
            migrations::migrate_device(
                migration_data,
                api_handle.clone(),
                internal_event_tx.clone(),
            )
        } else {
            migrations::MigrationComplete::new(true)
        };

        let (account_manager, data) = device::AccountManager::spawn(
            api_handle.clone(),
            &settings_dir,
            settings
                .tunnel_options
                .wireguard
                .rotation_interval
                .unwrap_or_default(),
            internal_event_tx.to_specialized_sender(),
        )
        .await
        .map_err(Error::LoadAccountManager)?;

        let account_history = account_history::AccountHistory::new(
            &settings_dir,
            data.device().map(|device| device.account_token.clone()),
        )
        .await
        .map_err(Error::LoadAccountHistory)?;

        let target_state = if settings.auto_connect {
            log::info!("Automatically connecting since auto-connect is turned on");
            PersistentTargetState::force(&cache_dir, TargetState::Secured).await
        } else {
            PersistentTargetState::new(&cache_dir).await
        };

        #[cfg(windows)]
        let exclude_paths = if settings.split_tunnel.enable_exclusions {
            settings
                .split_tunnel
                .apps
                .iter()
                .map(OsString::from)
                .collect()
        } else {
            vec![]
        };

        // TODO: Can this be made cleaner? For example, don't tell the API
        // runtime the initial API endpoint, since it already knows how to
        // request an allowed endpoint.
        let initial_api_endpoint =
            api::get_allowed_endpoint(talpid_types::net::Endpoint::from_socket_address(
                api_runtime.address_cache.get_address().await,
                talpid_types::net::TransportProtocol::Tcp,
            ));
        let parameters_generator = tunnel::ParametersGenerator::new(
            account_manager.clone(),
            relay_selector.clone(),
            settings.tunnel_options.clone(),
        );

        let param_gen = parameters_generator.clone();
        let (param_gen_tx, mut param_gen_rx) = mpsc::unbounded();
        tokio::spawn(async move {
            while let Some(tunnel_options) = param_gen_rx.next().await {
                param_gen.set_tunnel_options(&tunnel_options).await;
            }
        });
        settings.register_change_listener(move |settings| {
            let _ = param_gen_tx.unbounded_send(settings.tunnel_options.to_owned());
        });

        let (offline_state_tx, offline_state_rx) = mpsc::unbounded();
        #[cfg(target_os = "windows")]
        let (volume_update_tx, volume_update_rx) = mpsc::unbounded();
        let tunnel_state_machine_handle = tunnel_state_machine::spawn(
            tunnel_state_machine::InitialTunnelState {
                allow_lan: settings.allow_lan,
                block_when_disconnected: settings.block_when_disconnected,
                dns_servers: dns::addresses_from_options(&settings.tunnel_options.dns_options),
                allowed_endpoint: initial_api_endpoint,
                reset_firewall: *target_state != TargetState::Secured,
                #[cfg(windows)]
                exclude_paths,
            },
            parameters_generator.clone(),
            log_dir,
            resource_dir.clone(),
            internal_event_tx.to_specialized_sender(),
            offline_state_tx,
            #[cfg(target_os = "windows")]
            volume_update_rx,
            #[cfg(target_os = "android")]
            android_context,
            #[cfg(target_os = "linux")]
            tunnel_state_machine::LinuxNetworkingIdentifiers {
                fwmark: mullvad_types::TUNNEL_FWMARK,
                table_id: mullvad_types::TUNNEL_TABLE_ID,
            },
        )
        .await
        .map_err(Error::TunnelError)?;

        api::forward_offline_state(api_availability.clone(), offline_state_rx);

        let relay_list_listener = event_listener.clone();
        let on_relay_list_update = move |relay_list: &RelayList| {
            relay_list_listener.notify_relay_list(relay_list.clone());
        };

        let mut relay_list_updater = RelayListUpdater::spawn(
            relay_selector.clone(),
            api_handle.clone(),
            &cache_dir,
            on_relay_list_update,
        );

        let (version_updater, version_updater_handle) = version_check::VersionUpdater::new(
            api_handle.clone(),
            api_availability.clone(),
            cache_dir.clone(),
            internal_event_tx.to_specialized_sender(),
            app_version_info.clone(),
            settings.show_beta_releases,
        );
        tokio::spawn(version_updater.run());

        // Attempt to download a fresh relay list
        relay_list_updater.update().await;

        let daemon = Daemon {
            tunnel_state: TunnelState::Disconnected,
            target_state,
            state: DaemonExecutionState::Running,
            #[cfg(target_os = "linux")]
            exclude_pids: split_tunnel::PidManager::new().map_err(Error::InitSplitTunneling)?,
            rx: internal_event_rx,
            tx: internal_event_tx,
            reconnection_job: None,
            event_listener,
            migration_complete,
            settings,
            account_history,
            device_checker: device::TunnelStateChangeHandler::new(account_manager.clone()),
            account_manager,
            connection_modes_handler,
            api_runtime,
            api_handle,
            version_updater_handle,
            relay_selector,
            relay_list_updater,
            parameters_generator,
            app_version_info,
            shutdown_tasks: vec![],
            tunnel_state_machine_handle,
            #[cfg(target_os = "windows")]
            volume_update_tx,
        };

        api_availability.unsuspend();

        Ok(daemon)
    }

    /// Consume the `Daemon` and run the main event loop. Blocks until an error happens or a
    /// shutdown event is received.
    pub async fn run(mut self) -> Result<(), Error> {
        if *self.target_state == TargetState::Secured {
            self.connect_tunnel();
        }

        while let Some(event) = self.rx.next().await {
            self.handle_event(event).await;
            if self.state == DaemonExecutionState::Finished {
                break;
            }
        }

        self.finalize().await;
        Ok(())
    }

    async fn finalize(self) {
        let (event_listener, shutdown_tasks, api_runtime, tunnel_state_machine_handle) =
            self.shutdown();
        for future in shutdown_tasks {
            future.await;
        }

        tunnel_state_machine_handle.try_join().await;

        drop(event_listener);
        drop(api_runtime);

        #[cfg(any(target_os = "macos", target_os = "linux"))]
        if let Err(err) = fs::remove_file(mullvad_paths::get_rpc_socket_path()).await {
            if err.kind() != std::io::ErrorKind::NotFound {
                log::error!("Failed to remove old RPC socket: {}", err);
            }
        }
    }

    /// Shuts down the daemon without shutting down the underlying event listener and the shutdown
    /// callbacks
    fn shutdown<'a>(
        self,
    ) -> (
        L,
        Vec<LocalBoxFuture<'a, ()>>,
        mullvad_api::Runtime,
        TunnelStateMachineHandle,
    ) {
        let Daemon {
            event_listener,
            mut shutdown_tasks,
            api_runtime,
            tunnel_state_machine_handle,
            target_state,
            account_manager,
            ..
        } = self;

        shutdown_tasks.push(Box::pin(target_state.finalize()));
        shutdown_tasks.push(Box::pin(account_manager.shutdown()));

        (
            event_listener,
            shutdown_tasks,
            api_runtime,
            tunnel_state_machine_handle,
        )
    }

    async fn handle_event(&mut self, event: InternalDaemonEvent) {
        use self::InternalDaemonEvent::*;
        match event {
            TunnelStateTransition(transition) => {
                self.handle_tunnel_state_transition(transition).await
            }
            Command(command) => self.handle_command(command).await,
            TriggerShutdown(user_init_shutdown) => self.trigger_shutdown_event(user_init_shutdown),
            NewAppVersionInfo(app_version_info) => {
                self.handle_new_app_version_info(app_version_info);
            }
            DeviceEvent(event) => self.handle_device_event(event).await,
            AccessMethodEvent(event) => self.handle_access_method_event(event).await,
            DeviceMigrationEvent(event) => self.handle_device_migration_event(event),
            #[cfg(windows)]
            ExcludedPathsEvent(update, tx) => self.handle_new_excluded_paths(update, tx).await,
        }
    }

    async fn handle_tunnel_state_transition(
        &mut self,
        tunnel_state_transition: TunnelStateTransition,
    ) {
        self.reset_rpc_sockets_on_tunnel_state_transition(&tunnel_state_transition);
        self.device_checker
            .handle_state_transition(&tunnel_state_transition);

        let tunnel_state = match tunnel_state_transition {
            TunnelStateTransition::Disconnected => TunnelState::Disconnected,
            TunnelStateTransition::Connecting(endpoint) => TunnelState::Connecting {
                endpoint,
                location: self.parameters_generator.get_last_location().await,
            },
            TunnelStateTransition::Connected(endpoint) => TunnelState::Connected {
                endpoint,
                location: self.parameters_generator.get_last_location().await,
            },
            TunnelStateTransition::Disconnecting(after_disconnect) => {
                TunnelState::Disconnecting(after_disconnect)
            }
            TunnelStateTransition::Error(error_state) => TunnelState::Error(error_state),
        };

        if !tunnel_state.is_connected() {
            // Cancel reconnects except when entering the connected state.
            // Exempt the latter because a reconnect scheduled while connecting should not be
            // aborted.
            self.unschedule_reconnect();
        }

        log::debug!("New tunnel state: {:?}", tunnel_state);

        match tunnel_state {
            TunnelState::Disconnected => {
                self.api_handle.availability.reset_inactivity_timer();
            }
            _ => {
                self.api_handle.availability.stop_inactivity_timer();
            }
        }

        match &tunnel_state {
            TunnelState::Disconnected => self.state.disconnected(),
            TunnelState::Connecting { .. } => {
                log::debug!("Settings: {}", self.settings.summary());
            }
            TunnelState::Error(error_state) => {
                if error_state.is_blocking() {
                    log::info!(
                        "Blocking all network connections, reason: {}",
                        error_state.cause()
                    );
                } else {
                    log::error!(
                        "FAILED TO BLOCK NETWORK CONNECTIONS, ENTERED ERROR STATE BECAUSE: {}",
                        error_state.cause()
                    );
                }

                if let ErrorStateCause::AuthFailed(_) = error_state.cause() {
                    // If time is added outside of the app, no notifications
                    // are received. So we must continually try to reconnect.
                    self.schedule_reconnect(Duration::from_secs(60))
                }
            }
            _ => {}
        }

        self.tunnel_state = tunnel_state.clone();
        self.event_listener.notify_new_state(tunnel_state);
    }

    fn reset_rpc_sockets_on_tunnel_state_transition(
        &mut self,
        tunnel_state_transition: &TunnelStateTransition,
    ) {
        match (&self.tunnel_state, &tunnel_state_transition) {
            // Only reset the API sockets when entering or leaving the connected state
            (&TunnelState::Connected { .. }, _) | (_, &TunnelStateTransition::Connected(_)) => {
                self.api_handle.service().reset();
            }
            _ => (),
        };
    }

    fn schedule_reconnect(&mut self, delay: Duration) {
        self.unschedule_reconnect();

        let daemon_command_tx = self.tx.to_specialized_sender();
        let (future, abort_handle) = abortable(Box::pin(async move {
            tokio::time::sleep(delay).await;
            log::debug!("Attempting to reconnect");
            let (tx, rx) = oneshot::channel();
            let _ = daemon_command_tx.send(DaemonCommand::Reconnect(tx));
            // suppress "unable to send" warning:
            let _ = rx.await;
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

        if self.tunnel_state.is_disconnected() {
            self.api_handle.availability.reset_inactivity_timer();
        }

        match command {
            SetTargetState(tx, state) => self.on_set_target_state(tx, state).await,
            Reconnect(tx) => self.on_reconnect(tx),
            GetState(tx) => self.on_get_state(tx),
            GetCurrentLocation(tx) => self.on_get_current_location(tx).await,
            CreateNewAccount(tx) => self.on_create_new_account(tx),
            GetAccountData(tx, account_token) => self.on_get_account_data(tx, account_token),
            GetWwwAuthToken(tx) => self.on_get_www_auth_token(tx).await,
            SubmitVoucher(tx, voucher) => self.on_submit_voucher(tx, voucher),
            GetRelayLocations(tx) => self.on_get_relay_locations(tx),
            UpdateRelayLocations => self.on_update_relay_locations().await,
            LoginAccount(tx, account_token) => self.on_login_account(tx, account_token),
            LogoutAccount(tx) => self.on_logout_account(tx),
            GetDevice(tx) => self.on_get_device(tx),
            UpdateDevice(tx) => self.on_update_device(tx),
            ListDevices(tx, account_token) => self.on_list_devices(tx, account_token),
            RemoveDevice(tx, account_token, device_id) => {
                self.on_remove_device(tx, account_token, device_id)
            }
            GetAccountHistory(tx) => self.on_get_account_history(tx),
            ClearAccountHistory(tx) => self.on_clear_account_history(tx).await,
            SetRelaySettings(tx, update) => self.on_set_relay_settings(tx, update).await,
            SetAllowLan(tx, allow_lan) => self.on_set_allow_lan(tx, allow_lan).await,
            SetShowBetaReleases(tx, enabled) => self.on_set_show_beta_releases(tx, enabled).await,
            SetBlockWhenDisconnected(tx, block_when_disconnected) => {
                self.on_set_block_when_disconnected(tx, block_when_disconnected)
                    .await
            }
            SetAutoConnect(tx, auto_connect) => self.on_set_auto_connect(tx, auto_connect).await,
            SetOpenVpnMssfix(tx, mssfix_arg) => self.on_set_openvpn_mssfix(tx, mssfix_arg).await,
            SetBridgeSettings(tx, bridge_settings) => {
                self.on_set_bridge_settings(tx, bridge_settings).await
            }
            SetBridgeState(tx, bridge_state) => self.on_set_bridge_state(tx, bridge_state).await,
            SetEnableIpv6(tx, enable_ipv6) => self.on_set_enable_ipv6(tx, enable_ipv6).await,
            SetQuantumResistantTunnel(tx, quantum_resistant_state) => {
                self.on_set_quantum_resistant_tunnel(tx, quantum_resistant_state)
                    .await
            }
            SetDnsOptions(tx, dns_servers) => self.on_set_dns_options(tx, dns_servers).await,
            SetRelayOverride(tx, relay_override) => {
                self.on_set_relay_override(tx, relay_override).await
            }
            ClearAllRelayOverrides(tx) => self.on_clear_all_relay_overrides(tx).await,
            SetWireguardMtu(tx, mtu) => self.on_set_wireguard_mtu(tx, mtu).await,
            SetWireguardRotationInterval(tx, interval) => {
                self.on_set_wireguard_rotation_interval(tx, interval).await
            }
            GetSettings(tx) => self.on_get_settings(tx),
            RotateWireguardKey(tx) => self.on_rotate_wireguard_key(tx),
            GetWireguardKey(tx) => self.on_get_wireguard_key(tx).await,
            CreateCustomList(tx, name) => self.on_create_custom_list(tx, name).await,
            DeleteCustomList(tx, id) => self.on_delete_custom_list(tx, id).await,
            UpdateCustomList(tx, update) => self.on_update_custom_list(tx, update).await,
            GetVersionInfo(tx) => self.on_get_version_info(tx),
            GetApiAccessMethods(tx) => self.on_get_api_access_methods(tx),
            AddApiAccessMethod(tx, name, enabled, access_method) => {
                self.on_add_access_method(tx, name, enabled, access_method)
                    .await
            }
            RemoveApiAccessMethod(tx, method) => self.on_remove_api_access_method(tx, method).await,
            UpdateApiAccessMethod(tx, method) => self.on_update_api_access_method(tx, method).await,
            GetCurrentAccessMethod(tx) => self.on_get_current_api_access_method(tx),
            SetApiAccessMethod(tx, method) => self.on_set_api_access_method(tx, method).await,
            TestApiAccessMethod(tx, method) => self.on_test_api_access_method(tx, method),
            IsPerformingPostUpgrade(tx) => self.on_is_performing_post_upgrade(tx),
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
            #[cfg(windows)]
            AddSplitTunnelApp(tx, path) => self.on_add_split_tunnel_app(tx, path),
            #[cfg(windows)]
            RemoveSplitTunnelApp(tx, path) => self.on_remove_split_tunnel_app(tx, path),
            #[cfg(windows)]
            ClearSplitTunnelApps(tx) => self.on_clear_split_tunnel_apps(tx),
            #[cfg(windows)]
            SetSplitTunnelState(tx, enabled) => self.on_set_split_tunnel_state(tx, enabled),
            #[cfg(windows)]
            GetSplitTunnelProcesses(tx) => self.on_get_split_tunnel_processes(tx),
            #[cfg(target_os = "windows")]
            CheckVolumes(tx) => self.on_check_volumes(tx),
            SetObfuscationSettings(tx, settings) => {
                self.on_set_obfuscation_settings(tx, settings).await
            }
            PrepareRestart => self.on_prepare_restart(),
            #[cfg(target_os = "android")]
            BypassSocket(fd, tx) => self.on_bypass_socket(fd, tx),
            #[cfg(target_os = "android")]
            InitPlayPurchase(tx) => self.on_init_play_purchase(tx),
            #[cfg(target_os = "android")]
            VerifyPlayPurchase(tx, play_purchase) => {
                self.on_verify_play_purchase(tx, play_purchase)
            }
            ApplyJsonSettings(tx, blob) => self.on_apply_json_settings(tx, blob).await,
        }
    }

    fn handle_new_app_version_info(&mut self, app_version_info: AppVersionInfo) {
        self.app_version_info = Some(app_version_info.clone());
        self.event_listener.notify_app_version(app_version_info);
    }

    async fn handle_device_event(&mut self, event: AccountEvent) {
        match &event {
            AccountEvent::Device(PrivateDeviceEvent::Login(device)) => {
                if let Err(error) = self.account_history.set(device.account_token.clone()).await {
                    log::error!(
                        "{}",
                        error.display_chain_with_msg("Failed to update account history")
                    );
                }
                if *self.target_state == TargetState::Secured {
                    log::debug!("Initiating tunnel restart because the account token changed");
                    self.reconnect_tunnel();
                }
            }
            AccountEvent::Device(PrivateDeviceEvent::Logout) => {
                log::info!("Disconnecting because account token was cleared");
                self.set_target_state(TargetState::Unsecured).await;
            }
            AccountEvent::Device(PrivateDeviceEvent::Revoked) => {
                // If we're currently in a secured state, reconnect to make sure we immediately
                // enter the error state.
                if *self.target_state == TargetState::Secured {
                    self.connect_tunnel();
                }
            }
            AccountEvent::Device(PrivateDeviceEvent::RotatedKey(_)) => {
                if self.get_target_tunnel_type() == Some(TunnelType::Wireguard) {
                    self.schedule_reconnect(WG_RECONNECT_DELAY);
                }
            }
            AccountEvent::Expiry(expiry) if *self.target_state == TargetState::Secured => {
                if expiry >= &chrono::Utc::now() {
                    if let TunnelState::Error(ref state) = self.tunnel_state {
                        if matches!(state.cause(), ErrorStateCause::AuthFailed(_)) {
                            log::debug!("Reconnecting since the account has time on it");
                            self.connect_tunnel();
                        }
                    }
                } else if self.get_target_tunnel_type() == Some(TunnelType::Wireguard) {
                    log::debug!("Entering blocking state since the account is out of time");
                    self.send_tunnel_command(TunnelCommand::Block(ErrorStateCause::AuthFailed(
                        Some(AuthFailed::ExpiredAccount.as_str().to_string()),
                    )))
                }
            }
            _ => (),
        }
        if let AccountEvent::Device(event) = event {
            self.event_listener
                .notify_device_event(DeviceEvent::from(event));
        }
    }

    async fn handle_access_method_event(&mut self, event: AccessMethodEvent) {
        match event {
            AccessMethodEvent::Active(access_method) => {
                log::info!("HANDLING INTERNVAL DAEMON EVENT: Setting new active access method");
                // TODO(markus): Update the tunnel state machine to punch an appropriate hole in the firewall
                let connection_mode: ApiConnectionMode = ApiConnectionMode::Direct;
                let allowed_endpoint = talpid_types::net::AllowedEndpoint {
                    clients: connection_mode.allowed_clients(),
                    endpoint: match connection_mode.get_endpoint() {
                        Some(endpoint) => endpoint,
                        None => talpid_types::net::Endpoint::from_socket_address(
                            self.api_runtime.address_cache.get_address().await,
                            talpid_types::net::TransportProtocol::Tcp,
                        ),
                    },
                };
                let (result_tx, result_rx) = oneshot::channel();
                log::warn!(
                    "API endpoint: {endpoint}",
                    endpoint = allowed_endpoint.endpoint
                );
                self.send_tunnel_command(TunnelCommand::AllowEndpoint(allowed_endpoint, result_tx));
                //  Wait for the firewall policy to be updated.
                let _ = result_rx.await;

                self.event_listener.notify_new_access_method(access_method);

                // if let Err(error) = self.force_api_endpoint_rotation().await {
                //     log::error!(
                //         "{}",
                //         error.display_chain_with_msg("Failed to rotate access mehod")
                //     );
                // }
                // if let Err(error) = self.set_active_access_method(access_method).await {
                //     log::error!(
                //         "{}",
                //         error.display_chain_with_msg("Failed to set active access mehod")
                //     );
                // }
            }
        }
    }

    fn handle_device_migration_event(
        &mut self,
        result: Result<PrivateAccountAndDevice, device::Error>,
    ) {
        let account_manager = self.account_manager.clone();
        let event_listener = self.event_listener.clone();
        tokio::spawn(async move {
            if let Ok(Some(_)) = account_manager
                .data_after_login()
                .await
                .map(|s| s.into_device())
            {
                // Discard stale device
                return;
            }

            let result = async { account_manager.set(result?).await }.await;

            if let Err(error) = result {
                log::error!(
                    "{}",
                    error.display_chain_with_msg("Failed to move over account from old settings")
                );
                // Synthesize a logout or revocation if migration fails.
                let event = match error {
                    device::Error::InvalidDevice => DeviceEvent {
                        cause: DeviceEventCause::Revoked,
                        new_state: DeviceState::Revoked,
                    },
                    _ => DeviceEvent {
                        cause: DeviceEventCause::LoggedOut,
                        new_state: DeviceState::LoggedOut,
                    },
                };
                event_listener.notify_device_event(event);
            }
        });
    }

    #[cfg(windows)]
    async fn handle_new_excluded_paths(
        &mut self,
        update: ExcludedPathsUpdate,
        tx: ResponseTx<(), Error>,
    ) {
        let save_result = match update {
            ExcludedPathsUpdate::SetState(state) => self
                .settings
                .update(move |settings| settings.split_tunnel.enable_exclusions = state)
                .await
                .map_err(Error::SettingsError),
            ExcludedPathsUpdate::SetPaths(paths) => self
                .settings
                .update(move |settings| settings.split_tunnel.apps = paths)
                .await
                .map_err(Error::SettingsError),
        };
        let _ = tx.send(save_result.map(|_| ()));
    }

    async fn on_set_target_state(
        &mut self,
        tx: oneshot::Sender<bool>,
        new_target_state: TargetState,
    ) {
        if self.state.is_running() {
            let state_change_initated = self.set_target_state(new_target_state).await;
            Self::oneshot_send(tx, state_change_initated, "state change initiated");
        } else {
            log::warn!("Ignoring target state change request due to shutdown");
        }
    }

    fn on_reconnect(&mut self, tx: oneshot::Sender<bool>) {
        if *self.target_state == TargetState::Secured || self.tunnel_state.is_in_error_state() {
            self.connect_tunnel();
            Self::oneshot_send(tx, true, "reconnect issued");
        } else {
            log::debug!("Ignoring reconnect command. Currently not in secured state");
            Self::oneshot_send(tx, false, "reconnect issued");
        }
    }

    fn on_get_state(&self, tx: oneshot::Sender<TunnelState>) {
        Self::oneshot_send(tx, self.tunnel_state.clone(), "current state");
    }

    fn on_is_performing_post_upgrade(&self, tx: oneshot::Sender<bool>) {
        let performing_post_upgrade = !self.migration_complete.is_complete();
        Self::oneshot_send(tx, performing_post_upgrade, "performing post upgrade");
    }

    async fn on_get_current_location(&mut self, tx: oneshot::Sender<Option<GeoIpLocation>>) {
        use self::TunnelState::*;

        match &self.tunnel_state {
            Disconnected => self.update_and_send_geo_location(tx, None).await,
            Connecting { location, .. } => {
                Self::oneshot_send(tx, location.clone(), "current location")
            }
            Disconnecting(..) => Self::oneshot_send(
                tx,
                self.parameters_generator.get_last_location().await,
                "current location",
            ),
            Connected { location, .. } => {
                self.update_and_send_geo_location(tx, location.clone())
                    .await
            }
            // We are not online at all at this stage so no location data is available.
            Error(_) => Self::oneshot_send(tx, None, "current location"),
        };
    }

    /// Fetch the current `GeoIpLocation` and send it on the return channel,
    /// in a non-blocking fashion. Optionally give a chached previous location.
    async fn update_and_send_geo_location(
        &mut self,
        tx: oneshot::Sender<Option<GeoIpLocation>>,
        current_relay_location: Option<GeoIpLocation>,
    ) {
        let rest_service = self.api_runtime.rest_handle().await;
        let use_ipv6 = self.settings.tunnel_options.generic.enable_ipv6;
        let api_handle = self.api_handle.availability.clone();
        tokio::spawn(async move {
            let new_location = get_geo_location(rest_service, use_ipv6, api_handle).await;
            Self::oneshot_send(
                tx,
                new_location.map(|fetched_location| GeoIpLocation {
                    ipv4: fetched_location.ipv4,
                    ipv6: fetched_location.ipv6,
                    ..current_relay_location.unwrap_or(fetched_location)
                }),
                "current location",
            );
        });
    }

    fn on_create_new_account(&mut self, tx: ResponseTx<String, Error>) {
        let account_manager = self.account_manager.clone();
        tokio::spawn(async move {
            let result = async {
                if let Ok(data) = account_manager.data().await {
                    if data.logged_in() {
                        return Err(Error::AlreadyLoggedIn);
                    }
                }
                let token = account_manager
                    .account_service
                    .create_account()
                    .await
                    .map_err(Error::RestError)?;
                account_manager
                    .login(token.clone())
                    .await
                    .map_err(|error| {
                        log::error!(
                            "{}",
                            error.display_chain_with_msg("Creating new account failed")
                        );
                        Error::LoginError(error)
                    })?;
                Ok(token)
            };
            Self::oneshot_send(tx, result.await, "create new account");
        });
    }

    fn on_get_account_data(
        &mut self,
        tx: ResponseTx<AccountData, mullvad_api::rest::Error>,
        account_token: AccountToken,
    ) {
        let account = self.account_manager.account_service.clone();
        tokio::spawn(async move {
            let result = account.check_expiry(account_token).await;
            Self::oneshot_send(
                tx,
                result.map(|expiry| AccountData { expiry }),
                "account data",
            );
        });
    }

    async fn on_get_www_auth_token(&mut self, tx: ResponseTx<String, Error>) {
        if let Ok(Some(device)) = self.account_manager.data().await.map(|s| s.into_device()) {
            let future = self
                .account_manager
                .account_service
                .get_www_auth_token(device.account_token);
            tokio::spawn(async {
                Self::oneshot_send(
                    tx,
                    future.await.map_err(Error::RestError),
                    "get_www_auth_token response",
                );
            });
        } else {
            Self::oneshot_send(
                tx,
                Err(Error::NoAccountToken),
                "get_www_auth_token response",
            );
        }
    }

    fn on_submit_voucher(&mut self, tx: ResponseTx<VoucherSubmission, Error>, voucher: String) {
        let manager = self.account_manager.clone();
        tokio::spawn(async move {
            Self::oneshot_send(
                tx,
                manager
                    .submit_voucher(voucher)
                    .await
                    .map_err(Error::VoucherSubmission),
                "submit_voucher response",
            );
        });
    }

    fn on_get_relay_locations(&mut self, tx: oneshot::Sender<RelayList>) {
        Self::oneshot_send(tx, self.relay_selector.get_locations(), "relay locations");
    }

    async fn on_update_relay_locations(&mut self) {
        self.relay_list_updater.update().await;
    }

    fn on_login_account(&mut self, tx: ResponseTx<(), Error>, account_token: String) {
        let account_manager = self.account_manager.clone();
        let availability = self.api_runtime.availability_handle();
        tokio::spawn(async move {
            let result = async {
                account_manager
                    .login(account_token)
                    .await
                    .map_err(|error| {
                        log::error!("{}", error.display_chain_with_msg("Login failed"));
                        Error::LoginError(error)
                    })?;

                availability.resume_background();

                Ok(())
            };
            Self::oneshot_send(tx, result.await, "login_account response");
        });
    }

    fn on_logout_account(&mut self, tx: ResponseTx<(), Error>) {
        let account_manager = self.account_manager.clone();
        tokio::spawn(async move {
            let result = async {
                account_manager.logout().await.map_err(|error| {
                    log::error!("{}", error.display_chain_with_msg("Logout failed"));
                    Error::LogoutError(error)
                })
            };
            Self::oneshot_send(tx, result.await, "logout_account response");
        });
    }

    fn on_get_device(&mut self, tx: ResponseTx<DeviceState, Error>) {
        let account_manager = self.account_manager.clone();
        tokio::spawn(async move {
            Self::oneshot_send(
                tx,
                account_manager
                    .data()
                    .await
                    .map_err(|_| Error::NoAccountToken)
                    .map(DeviceState::from),
                "get_device response",
            );
        });
    }

    fn on_update_device(&mut self, tx: ResponseTx<(), Error>) {
        let account_manager = self.account_manager.clone();
        tokio::spawn(async move {
            let result = match account_manager.validate_device().await {
                Ok(_) | Err(device::Error::NoDevice) => Ok(()),
                Err(error) => Err(error),
            };
            Self::oneshot_send(
                tx,
                result.map_err(Error::UpdateDeviceError),
                "update_device response",
            );
        });
    }

    fn on_list_devices(&self, tx: ResponseTx<Vec<Device>, Error>, token: AccountToken) {
        let service = self.account_manager.device_service.clone();
        tokio::spawn(async move {
            Self::oneshot_send(
                tx,
                service
                    .list_devices(token)
                    .await
                    .map_err(Error::ListDevicesError),
                "list_devices response",
            );
        });
    }

    fn on_remove_device(
        &mut self,
        tx: ResponseTx<(), Error>,
        account_token: AccountToken,
        device_id: DeviceId,
    ) {
        let device_service = self.account_manager.device_service.clone();
        let event_listener = self.event_listener.clone();

        tokio::spawn(async move {
            let result = device_service
                .remove_device(account_token.clone(), device_id)
                .await
                .map(move |new_devices| {
                    // FIXME: We should be able to get away with only returning the removed ID,
                    //        and not have to request the list from the API.
                    event_listener.notify_remove_device_event(RemoveDeviceEvent {
                        account_token,
                        new_devices,
                    });
                });
            Self::oneshot_send(
                tx,
                result.map_err(Error::RemoveDeviceError),
                "remove_device response",
            );
        });
    }

    fn on_get_account_history(&mut self, tx: oneshot::Sender<Option<AccountToken>>) {
        Self::oneshot_send(
            tx,
            self.account_history.get(),
            "get_account_history response",
        );
    }

    async fn on_clear_account_history(&mut self, tx: ResponseTx<(), Error>) {
        let result = self
            .account_history
            .clear()
            .await
            .map_err(Error::AccountHistory);
        Self::oneshot_send(tx, result, "clear_account_history response");
    }

    fn on_get_version_info(&mut self, tx: oneshot::Sender<Option<AppVersionInfo>>) {
        if self.app_version_info.is_none() {
            log::debug!("No version cache found. Fetching new info");
            let mut handle = self.version_updater_handle.clone();
            tokio::spawn(async move {
                Self::oneshot_send(
                    tx,
                    handle
                        .run_version_check()
                        .await
                        .map_err(|error| {
                            log::error!(
                                "{}",
                                error.display_chain_with_msg("Error running version check")
                            )
                        })
                        .ok(),
                    "get_version_info response",
                );
            });
        } else {
            Self::oneshot_send(
                tx,
                self.app_version_info.clone(),
                "get_version_info response",
            );
        }
    }

    fn on_get_current_version(&mut self, tx: oneshot::Sender<AppVersion>) {
        Self::oneshot_send(
            tx,
            mullvad_version::VERSION.to_owned(),
            "get_current_version response",
        );
    }

    #[cfg(not(target_os = "android"))]
    async fn on_factory_reset(&mut self, tx: ResponseTx<(), Error>) {
        let mut last_error = Ok(());

        if let Err(error) = self.account_manager.logout().await {
            log::error!(
                "{}",
                error.display_chain_with_msg("Failed to clear device cache")
            );
        }

        if let Err(error) = self.account_history.clear().await {
            log::error!(
                "{}",
                error.display_chain_with_msg("Failed to clear account history")
            );
            last_error = Err(Error::FactoryResetError("Failed to clear account history"));
        }

        if let Err(e) = self.settings.reset().await {
            log::error!("Failed to reset settings: {}", e);
            last_error = Err(Error::FactoryResetError("Failed to reset settings"));
        }

        // Shut the daemon down.
        self.trigger_shutdown_event(false);

        self.shutdown_tasks.push(Box::pin(async move {
            if let Err(e) = cleanup::clear_directories().await {
                log::error!(
                    "{}",
                    e.display_chain_with_msg("Failed to clear cache and log directories")
                );
                last_error = Err(Error::FactoryResetError(
                    "Failed to clear cache and log directories",
                ));
            }
            Self::oneshot_send(tx, last_error, "factory_reset response");
        }));
    }

    #[cfg(target_os = "linux")]
    fn on_get_split_tunnel_processes(&mut self, tx: ResponseTx<Vec<i32>, split_tunnel::Error>) {
        let result = self.exclude_pids.list().map_err(|error| {
            log::error!("{}", error.display_chain_with_msg("Unable to obtain PIDs"));
            error
        });
        Self::oneshot_send(tx, result, "get_split_tunnel_processes response");
    }

    #[cfg(target_os = "linux")]
    fn on_add_split_tunnel_process(&mut self, tx: ResponseTx<(), split_tunnel::Error>, pid: i32) {
        let result = self.exclude_pids.add(pid).map_err(|error| {
            log::error!("{}", error.display_chain_with_msg("Unable to add PID"));
            error
        });
        Self::oneshot_send(tx, result, "add_split_tunnel_process response");
    }

    #[cfg(target_os = "linux")]
    fn on_remove_split_tunnel_process(
        &mut self,
        tx: ResponseTx<(), split_tunnel::Error>,
        pid: i32,
    ) {
        let result = self.exclude_pids.remove(pid).map_err(|error| {
            log::error!("{}", error.display_chain_with_msg("Unable to remove PID"));
            error
        });
        Self::oneshot_send(tx, result, "remove_split_tunnel_process response");
    }

    #[cfg(target_os = "linux")]
    fn on_clear_split_tunnel_processes(&mut self, tx: ResponseTx<(), split_tunnel::Error>) {
        let result = self.exclude_pids.clear().map_err(|error| {
            log::error!("{}", error.display_chain_with_msg("Unable to clear PIDs"));
            error
        });
        Self::oneshot_send(tx, result, "clear_split_tunnel_processes response");
    }

    /// Update the split app paths in both the settings and tunnel
    #[cfg(windows)]
    fn set_split_tunnel_paths(
        &mut self,
        tx: ResponseTx<(), Error>,
        response_msg: &'static str,
        settings: Settings,
        update: ExcludedPathsUpdate,
    ) {
        let new_list = match update {
            ExcludedPathsUpdate::SetPaths(ref paths) => {
                if *paths == settings.split_tunnel.apps {
                    Self::oneshot_send(tx, Ok(()), response_msg);
                    return;
                }
                paths.iter()
            }
            ExcludedPathsUpdate::SetState(_) => settings.split_tunnel.apps.iter(),
        };
        let new_state = match update {
            ExcludedPathsUpdate::SetPaths(_) => settings.split_tunnel.enable_exclusions,
            ExcludedPathsUpdate::SetState(state) => {
                if state == settings.split_tunnel.enable_exclusions {
                    Self::oneshot_send(tx, Ok(()), response_msg);
                    return;
                }
                state
            }
        };

        if new_state || new_state != settings.split_tunnel.enable_exclusions {
            let tunnel_list = if new_state {
                new_list.map(OsString::from).collect()
            } else {
                vec![]
            };

            let (result_tx, result_rx) = oneshot::channel();
            self.send_tunnel_command(TunnelCommand::SetExcludedApps(result_tx, tunnel_list));
            let daemon_tx = self.tx.clone();

            tokio::spawn(async move {
                match result_rx.await {
                    Ok(Ok(_)) => (),
                    Ok(Err(error)) => {
                        log::error!(
                            "{}",
                            error.display_chain_with_msg("Failed to set excluded apps list")
                        );
                        Self::oneshot_send(tx, Err(Error::SplitTunnelError(error)), response_msg);
                        return;
                    }
                    Err(_) => {
                        log::error!("The tunnel failed to return a result");
                        return;
                    }
                }

                let _ = daemon_tx.send(InternalDaemonEvent::ExcludedPathsEvent(update, tx));
            });
        } else {
            let _ = self
                .tx
                .send(InternalDaemonEvent::ExcludedPathsEvent(update, tx));
        }
    }

    #[cfg(windows)]
    fn on_add_split_tunnel_app(&mut self, tx: ResponseTx<(), Error>, path: PathBuf) {
        let settings = self.settings.to_settings();

        let mut new_list = settings.split_tunnel.apps.clone();
        new_list.insert(path);

        self.set_split_tunnel_paths(
            tx,
            "add_split_tunnel_app response",
            settings,
            ExcludedPathsUpdate::SetPaths(new_list),
        );
    }

    #[cfg(windows)]
    fn on_remove_split_tunnel_app(&mut self, tx: ResponseTx<(), Error>, path: PathBuf) {
        let settings = self.settings.to_settings();

        let mut new_list = settings.split_tunnel.apps.clone();
        new_list.remove(&path);

        self.set_split_tunnel_paths(
            tx,
            "remove_split_tunnel_app response",
            settings,
            ExcludedPathsUpdate::SetPaths(new_list),
        );
    }

    #[cfg(windows)]
    fn on_clear_split_tunnel_apps(&mut self, tx: ResponseTx<(), Error>) {
        let settings = self.settings.to_settings();
        let new_list = HashSet::new();
        self.set_split_tunnel_paths(
            tx,
            "clear_split_tunnel_apps response",
            settings,
            ExcludedPathsUpdate::SetPaths(new_list),
        );
    }

    #[cfg(windows)]
    fn on_set_split_tunnel_state(&mut self, tx: ResponseTx<(), Error>, state: bool) {
        let settings = self.settings.to_settings();
        self.set_split_tunnel_paths(
            tx,
            "set_split_tunnel_state response",
            settings,
            ExcludedPathsUpdate::SetState(state),
        );
    }

    #[cfg(windows)]
    fn on_get_split_tunnel_processes(
        &self,
        tx: ResponseTx<Vec<ExcludedProcess>, split_tunnel::Error>,
    ) {
        Self::oneshot_send(
            tx,
            self.tunnel_state_machine_handle
                .split_tunnel()
                .get_processes(),
            "get_split_tunnel_processes response",
        );
    }

    #[cfg(windows)]
    fn on_check_volumes(&mut self, tx: ResponseTx<(), Error>) {
        if self.volume_update_tx.unbounded_send(()).is_ok() {
            let _ = tx.send(Ok(()));
        }
    }

    async fn on_set_relay_settings(
        &mut self,
        tx: ResponseTx<(), settings::Error>,
        update: RelaySettings,
    ) {
        match self
            .settings
            .update(move |settings| settings.set_relay_settings(update))
            .await
        {
            Ok(settings_changed) => {
                Self::oneshot_send(tx, Ok(()), "set_relay_settings response");
                if settings_changed {
                    log::info!("Initiating tunnel restart because the relay settings changed");
                    self.reconnect_tunnel();
                }
            }
            Err(e) => {
                log::error!("{}", e.display_chain_with_msg("Unable to save settings"));
                Self::oneshot_send(tx, Err(e), "set_relay_settings response");
            }
        }
    }

    async fn on_set_allow_lan(&mut self, tx: ResponseTx<(), settings::Error>, allow_lan: bool) {
        match self
            .settings
            .update(move |settings| settings.allow_lan = allow_lan)
            .await
        {
            Ok(settings_changed) => {
                if settings_changed {
                    self.send_tunnel_command(TunnelCommand::AllowLan(
                        allow_lan,
                        oneshot_map(tx, |tx, ()| {
                            Self::oneshot_send(tx, Ok(()), "set_allow_lan response");
                        }),
                    ));
                } else {
                    Self::oneshot_send(tx, Ok(()), "set_allow_lan response");
                }
            }
            Err(e) => {
                log::error!("{}", e.display_chain_with_msg("Unable to save settings"));
                Self::oneshot_send(tx, Err(e), "set_allow_lan response");
            }
        }
    }

    async fn on_set_show_beta_releases(
        &mut self,
        tx: ResponseTx<(), settings::Error>,
        enabled: bool,
    ) {
        match self
            .settings
            .update(move |settings| settings.show_beta_releases = enabled)
            .await
        {
            Ok(settings_changed) => {
                Self::oneshot_send(tx, Ok(()), "set_show_beta_releases response");
                if settings_changed {
                    let mut handle = self.version_updater_handle.clone();
                    handle.set_show_beta_releases(enabled).await;
                }
            }
            Err(e) => {
                log::error!("{}", e.display_chain_with_msg("Unable to save settings"));
                Self::oneshot_send(tx, Err(e), "set_show_beta_releases response");
            }
        }
    }

    async fn on_set_block_when_disconnected(
        &mut self,
        tx: ResponseTx<(), settings::Error>,
        block_when_disconnected: bool,
    ) {
        match self
            .settings
            .update(move |settings| settings.block_when_disconnected = block_when_disconnected)
            .await
        {
            Ok(settings_changed) => {
                if settings_changed {
                    self.send_tunnel_command(TunnelCommand::BlockWhenDisconnected(
                        block_when_disconnected,
                        oneshot_map(tx, |tx, ()| {
                            Self::oneshot_send(tx, Ok(()), "set_block_when_disconnected response");
                        }),
                    ));
                } else {
                    Self::oneshot_send(tx, Ok(()), "set_block_when_disconnected response");
                }
            }
            Err(e) => {
                log::error!("{}", e.display_chain_with_msg("Unable to save settings"));
                Self::oneshot_send(tx, Err(e), "set_block_when_disconnected response");
            }
        }
    }

    async fn on_set_auto_connect(
        &mut self,
        tx: ResponseTx<(), settings::Error>,
        auto_connect: bool,
    ) {
        match self
            .settings
            .update(move |settings| settings.auto_connect = auto_connect)
            .await
        {
            Ok(_settings_changed) => {
                Self::oneshot_send(tx, Ok(()), "set auto-connect response");
            }
            Err(e) => {
                log::error!("{}", e.display_chain_with_msg("Unable to save settings"));
                Self::oneshot_send(tx, Err(e), "set auto-connect response");
            }
        }
    }

    async fn on_set_openvpn_mssfix(
        &mut self,
        tx: ResponseTx<(), settings::Error>,
        mssfix: Option<u16>,
    ) {
        match self
            .settings
            .update(move |settings| settings.tunnel_options.openvpn.mssfix = mssfix)
            .await
        {
            Ok(settings_changed) => {
                Self::oneshot_send(tx, Ok(()), "set_openvpn_mssfix response");
                if settings_changed && self.get_target_tunnel_type() == Some(TunnelType::OpenVpn) {
                    log::info!(
                        "Initiating tunnel restart because the OpenVPN mssfix setting changed"
                    );
                    self.reconnect_tunnel();
                }
            }
            Err(e) => {
                log::error!("{}", e.display_chain_with_msg("Unable to save settings"));
                Self::oneshot_send(tx, Err(e), "set_openvpn_mssfix response");
            }
        }
    }

    async fn on_set_bridge_settings(
        &mut self,
        tx: ResponseTx<(), settings::Error>,
        new_settings: BridgeSettings,
    ) {
        match self
            .settings
            .update(move |settings| settings.bridge_settings = new_settings)
            .await
        {
            Ok(settings_changes) => {
                if settings_changes {
                    if let Err(error) = self.api_handle.service().next_api_endpoint().await {
                        log::error!("Failed to rotate API endpoint: {}", error);
                    }
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

    async fn on_set_obfuscation_settings(
        &mut self,
        tx: ResponseTx<(), settings::Error>,
        new_settings: ObfuscationSettings,
    ) {
        match self
            .settings
            .update(move |settings| settings.obfuscation_settings = new_settings)
            .await
        {
            Ok(settings_changed) => {
                if settings_changed {
                    self.reconnect_tunnel();
                }
                Self::oneshot_send(tx, Ok(()), "set_obfuscation_settings");
            }
            Err(err) => {
                log::error!(
                    "{}",
                    err.display_chain_with_msg("Failed to set obfuscation settings")
                );
                Self::oneshot_send(tx, Err(err), "set_obfuscation_settings");
            }
        }
    }

    async fn on_set_bridge_state(
        &mut self,
        tx: ResponseTx<(), settings::Error>,
        bridge_state: BridgeState,
    ) {
        let result = match self
            .settings
            .update(move |settings| settings.bridge_state = bridge_state)
            .await
        {
            Ok(settings_changed) => {
                if settings_changed {
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

    async fn on_set_enable_ipv6(&mut self, tx: ResponseTx<(), settings::Error>, enable_ipv6: bool) {
        match self
            .settings
            .update(|settings| settings.tunnel_options.generic.enable_ipv6 = enable_ipv6)
            .await
        {
            Ok(settings_changed) => {
                Self::oneshot_send(tx, Ok(()), "set_enable_ipv6 response");
                if settings_changed {
                    log::info!("Initiating tunnel restart because the enable IPv6 setting changed");
                    self.reconnect_tunnel();
                }
            }
            Err(e) => {
                log::error!("{}", e.display_chain_with_msg("Unable to save settings"));
                Self::oneshot_send(tx, Err(e), "set_enable_ipv6 response");
            }
        }
    }

    async fn on_set_quantum_resistant_tunnel(
        &mut self,
        tx: ResponseTx<(), settings::Error>,
        quantum_resistant: QuantumResistantState,
    ) {
        match self
            .settings
            .update(|settings| {
                settings.tunnel_options.wireguard.quantum_resistant = quantum_resistant
            })
            .await
        {
            Ok(settings_changed) => {
                Self::oneshot_send(tx, Ok(()), "set_quantum_resistant_tunnel response");
                if settings_changed && self.get_target_tunnel_type() == Some(TunnelType::Wireguard)
                {
                    log::info!("Reconnecting because the PQ safety setting changed");
                    self.reconnect_tunnel();
                }
            }
            Err(e) => {
                log::error!("{}", e.display_chain_with_msg("Unable to save settings"));
                Self::oneshot_send(tx, Err(e), "set_quantum_resistant_tunnel response");
            }
        }
    }

    async fn on_set_dns_options(
        &mut self,
        tx: ResponseTx<(), settings::Error>,
        dns_options: DnsOptions,
    ) {
        match self
            .settings
            .update(move |settings| settings.tunnel_options.dns_options = dns_options)
            .await
        {
            Ok(settings_changed) => {
                if settings_changed {
                    let settings = self.settings.to_settings();
                    let resolvers =
                        dns::addresses_from_options(&settings.tunnel_options.dns_options);
                    self.send_tunnel_command(TunnelCommand::Dns(
                        resolvers,
                        oneshot_map(tx, |tx, ()| {
                            Self::oneshot_send(tx, Ok(()), "set_dns_options response");
                        }),
                    ));
                } else {
                    Self::oneshot_send(tx, Ok(()), "set_dns_options response");
                }
            }
            Err(e) => {
                log::error!("{}", e.display_chain_with_msg("Unable to save settings"));
                Self::oneshot_send(tx, Err(e), "set_dns_options response");
            }
        }
    }

    async fn on_set_relay_override(
        &mut self,
        tx: ResponseTx<(), settings::Error>,
        relay_override: RelayOverride,
    ) {
        match self
            .settings
            .update(move |settings| settings.set_relay_override(relay_override))
            .await
        {
            Ok(settings_changed) => {
                Self::oneshot_send(tx, Ok(()), "set_relay_override response");
                if settings_changed {
                    self.reconnect_tunnel();
                }
            }
            Err(e) => {
                log::error!("{}", e.display_chain_with_msg("Unable to save settings"));
                Self::oneshot_send(tx, Err(e), "set_relay_override response");
            }
        }
    }

    async fn on_clear_all_relay_overrides(&mut self, tx: ResponseTx<(), settings::Error>) {
        match self
            .settings
            .update(move |settings| settings.relay_overrides.clear())
            .await
        {
            Ok(settings_changed) => {
                Self::oneshot_send(tx, Ok(()), "clear_all_relay_overrides response");
                if settings_changed {
                    self.reconnect_tunnel();
                }
            }
            Err(e) => {
                log::error!("{}", e.display_chain_with_msg("Unable to save settings"));
                Self::oneshot_send(tx, Err(e), "clear_all_relay_overrides response");
            }
        }
    }

    async fn on_set_wireguard_mtu(
        &mut self,
        tx: ResponseTx<(), settings::Error>,
        mtu: Option<u16>,
    ) {
        match self
            .settings
            .update(move |settings| settings.tunnel_options.wireguard.mtu = mtu)
            .await
        {
            Ok(settings_changed) => {
                Self::oneshot_send(tx, Ok(()), "set_wireguard_mtu response");
                if settings_changed {
                    if let Some(TunnelType::Wireguard) = self.get_connected_tunnel_type() {
                        log::info!(
                            "Initiating tunnel restart because the WireGuard MTU setting changed"
                        );
                        self.reconnect_tunnel();
                    }
                }
            }
            Err(e) => {
                log::error!("{}", e.display_chain_with_msg("Unable to save settings"));
                Self::oneshot_send(tx, Err(e), "set_wireguard_mtu response");
            }
        }
    }

    async fn on_set_wireguard_rotation_interval(
        &mut self,
        tx: ResponseTx<(), settings::Error>,
        interval: Option<RotationInterval>,
    ) {
        match self
            .settings
            .update(move |settings| settings.tunnel_options.wireguard.rotation_interval = interval)
            .await
        {
            Ok(settings_changed) => {
                Self::oneshot_send(tx, Ok(()), "set_wireguard_rotation_interval response");
                if settings_changed {
                    if let Err(error) = self
                        .account_manager
                        .set_rotation_interval(interval.unwrap_or_default())
                        .await
                    {
                        log::error!(
                            "{}",
                            error.display_chain_with_msg("Failed to update rotation interval")
                        );
                    }
                }
            }
            Err(e) => {
                log::error!("{}", e.display_chain_with_msg("Unable to save settings"));
                Self::oneshot_send(tx, Err(e), "set_wireguard_rotation_interval response");
            }
        }
    }

    fn on_rotate_wireguard_key(&self, tx: ResponseTx<(), Error>) {
        let manager = self.account_manager.clone();
        tokio::spawn(async move {
            let result = manager
                .rotate_key()
                .await
                .map(|_| ())
                .map_err(Error::KeyRotationError);
            Self::oneshot_send(tx, result, "rotate_wireguard_key response");
        });
    }

    async fn on_get_wireguard_key(&self, tx: ResponseTx<Option<PublicKey>, Error>) {
        let result =
            if let Ok(Some(config)) = self.account_manager.data().await.map(|s| s.into_device()) {
                Ok(Some(config.device.wg_data.get_public_key()))
            } else {
                Err(Error::NoAccountToken)
            };
        Self::oneshot_send(tx, result, "get_wireguard_key response");
    }

    async fn on_create_custom_list(
        &mut self,
        tx: ResponseTx<mullvad_types::custom_list::Id, Error>,
        name: String,
    ) {
        let result = self.create_custom_list(name).await;
        Self::oneshot_send(tx, result, "create_custom_list response");
    }

    async fn on_delete_custom_list(
        &mut self,
        tx: ResponseTx<(), Error>,
        id: mullvad_types::custom_list::Id,
    ) {
        let result = self.delete_custom_list(id).await;
        Self::oneshot_send(tx, result, "delete_custom_list response");
    }

    async fn on_update_custom_list(&mut self, tx: ResponseTx<(), Error>, new_list: CustomList) {
        let result = self.update_custom_list(new_list).await;
        Self::oneshot_send(tx, result, "update_custom_list response");
    }

    fn on_get_api_access_methods(&mut self, tx: ResponseTx<Vec<AccessMethodSetting>, Error>) {
        let result = Ok(self.settings.api_access_methods.cloned());
        Self::oneshot_send(tx, result, "get_api_access_methods response");
    }

    async fn on_add_access_method(
        &mut self,
        tx: ResponseTx<mullvad_types::access_method::Id, Error>,
        name: String,
        enabled: bool,
        access_method: AccessMethod,
    ) {
        let result = self
            .add_access_method(name, enabled, access_method)
            .await
            .map_err(Error::AccessMethodError);
        Self::oneshot_send(tx, result, "add_api_access_method response");
    }

    async fn on_remove_api_access_method(
        &mut self,
        tx: ResponseTx<(), Error>,
        api_access_method: mullvad_types::access_method::Id,
    ) {
        let result = self
            .remove_access_method(api_access_method)
            .await
            .map_err(Error::AccessMethodError);
        Self::oneshot_send(tx, result, "remove_api_access_method response");
    }

    async fn on_set_api_access_method(
        &mut self,
        tx: ResponseTx<(), Error>,
        access_method: mullvad_types::access_method::Id,
    ) {
        let result = self
            .set_api_access_method(access_method)
            .await
            .map_err(Error::AccessMethodError);
        Self::oneshot_send(tx, result, "set_api_access_method response");
    }

    async fn on_update_api_access_method(
        &mut self,
        tx: ResponseTx<(), Error>,
        method: AccessMethodSetting,
    ) {
        let result = self
            .update_access_method(method)
            .await
            .map_err(Error::AccessMethodError);
        Self::oneshot_send(tx, result, "update_api_access_method response");
    }

    fn on_get_current_api_access_method(&mut self, tx: ResponseTx<AccessMethodSetting, Error>) {
        let handle = self.connection_modes_handler.clone();
        tokio::spawn(async move {
            let result = handle
                .get_access_method()
                .await
                .map_err(Error::ApiConnectionModeError);
            Self::oneshot_send(tx, result, "get_current_api_access_method response");
        });
    }

    fn on_test_api_access_method(
        &mut self,
        tx: ResponseTx<bool, Error>,
        access_method: mullvad_types::access_method::Id,
    ) {
        // NOTE: Preferably we would block all new API calls until the test is
        // done and the previous access method is reset. Otherwise we run the
        // risk of errounously triggering a rotation of the currently in-use
        // access method.
        let api_handle = self.api_handle.clone();
        let handle = self.connection_modes_handler.clone();
        let access_method_lookup = self
            .get_api_access_method(access_method)
            .map_err(Error::AccessMethodError);

        match access_method_lookup {
            Ok(access_method) => {
                tokio::spawn(async move {
                    let result =
                        access_method::test_access_method(access_method, handle, api_handle)
                            .await
                            .map_err(Error::AccessMethodError);
                    Self::oneshot_send(tx, result, "on_test_api_access_method response");
                });
            }
            Err(err) => {
                Self::oneshot_send(tx, Err(err), "on_test_api_access_method response");
            }
        }
    }

    fn on_get_settings(&self, tx: oneshot::Sender<Settings>) {
        Self::oneshot_send(tx, self.settings.to_settings(), "get_settings response");
    }

    fn oneshot_send<T>(tx: oneshot::Sender<T>, t: T, msg: &'static str) {
        if tx.send(t).is_err() {
            log::warn!("Unable to send {} to the daemon command sender", msg);
        }
    }

    fn trigger_shutdown_event(&mut self, user_init_shutdown: bool) {
        // Block all traffic before shutting down to ensure that no traffic can leak on boot or
        // shutdown.
        if !user_init_shutdown
            && (*self.target_state == TargetState::Secured || self.settings.auto_connect)
        {
            log::debug!("Blocking firewall during shutdown since system is going down");
            let (tx, _rx) = oneshot::channel();
            self.send_tunnel_command(TunnelCommand::BlockWhenDisconnected(true, tx));
        }

        self.state.shutdown(&self.tunnel_state);
        self.disconnect_tunnel();
    }

    fn on_prepare_restart(&mut self) {
        // TODO: See if this can be made to also shut down the daemon
        //       without causing the service to be restarted.

        if *self.target_state == TargetState::Secured {
            let (tx, _rx) = oneshot::channel();
            self.send_tunnel_command(TunnelCommand::BlockWhenDisconnected(true, tx));
        }
        self.target_state.lock();
    }

    #[cfg(target_os = "android")]
    fn on_bypass_socket(&mut self, fd: RawFd, tx: oneshot::Sender<()>) {
        match self.tunnel_state {
            // When connected, the API connection shouldn't be bypassed.
            TunnelState::Connected { .. } => {
                log::trace!("Not bypassing connection because the tunnel is up");
                let _ = tx.send(());
            }
            _ => {
                self.send_tunnel_command(TunnelCommand::BypassSocket(fd, tx));
            }
        }
    }

    #[cfg(target_os = "android")]
    fn on_init_play_purchase(&mut self, tx: ResponseTx<PlayPurchasePaymentToken, Error>) {
        let manager = self.account_manager.clone();
        tokio::spawn(async move {
            Self::oneshot_send(
                tx,
                manager
                    .init_play_purchase()
                    .await
                    .map_err(Error::InitPlayPurchase),
                "init_play_purchase response",
            );
        });
    }

    #[cfg(target_os = "android")]
    fn on_verify_play_purchase(&mut self, tx: ResponseTx<(), Error>, play_purchase: PlayPurchase) {
        let manager = self.account_manager.clone();
        tokio::spawn(async move {
            Self::oneshot_send(
                tx,
                manager
                    .verify_play_purchase(play_purchase)
                    .await
                    .map_err(Error::VerifyPlayPurchase),
                "verify_play_purchase response",
            );
        });
    }

    async fn on_apply_json_settings(
        &mut self,
        tx: ResponseTx<(), settings::patch::Error>,
        blob: String,
    ) {
        let result = settings::patch::merge_validate_patch(&mut self.settings, &blob).await;
        if result.is_ok() {
            self.reconnect_tunnel();
        }
        Self::oneshot_send(tx, result, "apply_json_settings response");
    }

    /// Set the target state of the client. If it changed trigger the operations needed to
    /// progress towards that state.
    /// Returns a bool representing whether or not a state change was initiated.
    async fn set_target_state(&mut self, new_state: TargetState) -> bool {
        if new_state != *self.target_state || self.tunnel_state.is_in_error_state() {
            log::debug!("Target state {:?} => {:?}", *self.target_state, new_state);

            self.target_state.set(new_state).await;

            match *self.target_state {
                TargetState::Secured => self.connect_tunnel(),
                TargetState::Unsecured => self.disconnect_tunnel(),
            }
            true
        } else {
            false
        }
    }

    fn connect_tunnel(&mut self) {
        self.api_runtime.availability_handle().resume_background();
        self.send_tunnel_command(TunnelCommand::Connect);
    }

    fn disconnect_tunnel(&mut self) {
        self.send_tunnel_command(TunnelCommand::Disconnect);
    }

    fn reconnect_tunnel(&mut self) {
        if *self.target_state == TargetState::Secured {
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

    fn send_tunnel_command(&self, command: TunnelCommand) {
        self.tunnel_state_machine_handle
            .command_tx()
            .unbounded_send(command)
            .expect("Tunnel state machine has stopped");
    }

    pub fn shutdown_handle(&self) -> DaemonShutdownHandle {
        DaemonShutdownHandle {
            tx: self.tx.clone(),
        }
    }
}

#[derive(Clone)]
pub struct DaemonShutdownHandle {
    tx: DaemonEventSender,
}

impl DaemonShutdownHandle {
    pub fn shutdown(&self, user_init_shutdown: bool) {
        let _ = self
            .tx
            .send(InternalDaemonEvent::TriggerShutdown(user_init_shutdown));
    }
}

fn new_selector_config(settings: &Settings) -> SelectorConfig {
    SelectorConfig {
        relay_settings: settings.relay_settings.clone(),
        bridge_state: settings.bridge_state,
        bridge_settings: settings.bridge_settings.clone(),
        obfuscation_settings: settings.obfuscation_settings.clone(),
        custom_lists: settings.custom_lists.clone(),
        relay_overrides: settings.relay_overrides.clone(),
    }
}

/// Consume a oneshot sender of `T1` and return a sender that takes a different type `T2`.
/// `forwarder` should map `T1` back to `T2` and send the result back to the original receiver.
fn oneshot_map<T1: Send + 'static, T2: Send + 'static>(
    tx: oneshot::Sender<T1>,
    forwarder: impl Fn(oneshot::Sender<T1>, T2) + Send + 'static,
) -> oneshot::Sender<T2> {
    let (new_tx, new_rx) = oneshot::channel();
    tokio::spawn(async move {
        match new_rx.await {
            Ok(result) => forwarder(tx, result),
            Err(oneshot::Canceled) => (),
        }
    });
    new_tx
}

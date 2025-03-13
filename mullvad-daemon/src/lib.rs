#![recursion_limit = "512"]
#![allow(rustdoc::private_intra_doc_links)]

mod access_method;
pub mod account_history;
mod android_dns;
mod api;
mod api_address_updater;
#[cfg(not(target_os = "android"))]
mod cleanup;
mod custom_list;
pub mod device;
mod dns;
pub mod exception_logging;
mod geoip;
mod leak_checker;
pub mod logging;
#[cfg(target_os = "macos")]
mod macos;
pub mod management_interface;
mod migrations;
mod relay_list;
#[cfg(not(target_os = "android"))]
pub mod rpc_uniqueness_check;
pub mod runtime;
pub mod settings;
pub mod shutdown;
mod target_state;
mod tunnel;
pub mod version;
mod version_check;

use crate::target_state::PersistentTargetState;
use api::DaemonAccessMethodResolver;
use device::{AccountEvent, PrivateAccountAndDevice, PrivateDeviceEvent};
use futures::{
    channel::{mpsc, oneshot},
    future::{abortable, AbortHandle, Future},
    StreamExt,
};
use geoip::GeoIpHandler;
use leak_checker::{LeakChecker, LeakInfo};
use management_interface::ManagementInterfaceServer;
use mullvad_api::{access_mode::AccessMethodEvent, ApiEndpoint};
use mullvad_encrypted_dns_proxy::state::EncryptedDnsProxyState;
use mullvad_relay_selector::{RelaySelector, SelectorConfig};
#[cfg(target_os = "android")]
use mullvad_types::account::{PlayPurchase, PlayPurchasePaymentToken};
#[cfg(any(windows, target_os = "android", target_os = "macos"))]
use mullvad_types::settings::SplitApp;
#[cfg(daita)]
use mullvad_types::wireguard::DaitaSettings;
use mullvad_types::{
    access_method::{AccessMethod, AccessMethodSetting},
    account::{AccountData, AccountNumber, VoucherSubmission},
    auth_failed::AuthFailed,
    custom_list::CustomList,
    device::{Device, DeviceEvent, DeviceEventCause, DeviceId, DeviceState, RemoveDeviceEvent},
    features::{compute_feature_indicators, FeatureIndicator, FeatureIndicators},
    location::{GeoIpLocation, LocationEventData},
    relay_constraints::{
        BridgeSettings, BridgeState, BridgeType, ObfuscationSettings, RelayOverride, RelaySettings,
    },
    relay_list::RelayList,
    settings::{DnsOptions, Settings},
    states::{Secured, TargetState, TargetStateStrict, TunnelState},
    version::{AppVersion, AppVersionInfo},
    wireguard::{PublicKey, QuantumResistantState, RotationInterval},
};
use relay_list::{RelayListUpdater, RelayListUpdaterHandle, RELAYS_FILENAME};
use settings::SettingsPersister;
#[cfg(any(windows, target_os = "android", target_os = "macos"))]
use std::collections::HashSet;
#[cfg(target_os = "android")]
use std::os::unix::io::RawFd;
use std::{
    marker::PhantomData,
    path::PathBuf,
    pin::Pin,
    sync::{Arc, Weak},
    time::Duration,
};
use talpid_core::{
    mpsc::Sender,
    split_tunnel,
    tunnel_state_machine::{self, TunnelCommand, TunnelStateMachineHandle},
};
use talpid_routing::RouteManagerHandle;
#[cfg(target_os = "android")]
use talpid_types::android::AndroidContext;
#[cfg(target_os = "windows")]
use talpid_types::split_tunnel::ExcludedProcess;
use talpid_types::{
    net::{IpVersion, TunnelType},
    tunnel::{ErrorStateCause, TunnelStateTransition},
    ErrorExt,
};
use tokio::io;

#[cfg(target_os = "android")]
use talpid_core::connectivity_listener::ConnectivityListener;

/// Delay between generating a new WireGuard key and reconnecting
const WG_RECONNECT_DELAY: Duration = Duration::from_secs(4 * 60);

pub type ResponseTx<T, E> = oneshot::Sender<Result<T, E>>;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Failed to send command to daemon because it is not running")]
    DaemonUnavailable,

    #[error("Unable to initialize network event loop")]
    InitIoEventLoop(#[source] io::Error),

    #[error("Unable to create RPC client")]
    InitRpcFactory(#[source] mullvad_api::Error),

    #[error("REST request failed")]
    RestError(#[source] mullvad_api::rest::Error),

    #[error("Management interface error")]
    ManagementInterfaceError(#[source] management_interface::Error),

    #[error("API availability check failed")]
    ApiCheckError(#[source] mullvad_api::availability::Error),

    #[error("Version check failed")]
    VersionCheckError(#[source] version_check::Error),

    #[error("Unable to load account history")]
    LoadAccountHistory(#[source] account_history::Error),

    #[error("Failed to start account manager")]
    LoadAccountManager(#[source] device::Error),

    #[error("Failed to log in to account")]
    LoginError(#[source] device::Error),

    #[error("Failed to log out of account")]
    LogoutError(#[source] device::Error),

    #[error("Failed to rotate WireGuard key")]
    KeyRotationError(#[source] device::Error),

    #[error("Failed to list devices")]
    ListDevicesError(#[source] device::Error),

    #[error("Failed to remove device")]
    RemoveDeviceError(#[source] device::Error),

    #[error("Failed to update device")]
    UpdateDeviceError(#[source] device::Error),

    #[error("Failed to submit voucher")]
    VoucherSubmission(#[source] device::Error),

    #[cfg(target_os = "linux")]
    #[error("Unable to initialize split tunneling")]
    InitSplitTunneling(#[source] split_tunnel::Error),

    #[cfg(any(windows, target_os = "android", target_os = "macos"))]
    #[error("Split tunneling error")]
    SplitTunnelError(#[source] split_tunnel::Error),

    #[error("An account is already set")]
    AlreadyLoggedIn,

    #[error("No account number is set")]
    NoAccountNumber,

    #[error("No account history available for the token")]
    NoAccountNumberHistory,

    #[error("Settings error")]
    SettingsError(#[source] settings::Error),

    #[error("Account history error")]
    AccountHistory(#[source] account_history::Error),

    #[cfg(not(target_os = "android"))]
    #[error("Factory reset partially failed: {0}")]
    FactoryResetError(&'static str),

    #[error("Tunnel state machine error")]
    TunnelError(#[source] tunnel_state_machine::Error),

    /// Errors from [talpid_routing::RouteManagerHandle].
    #[error("Route manager error")]
    RouteManager(#[source] talpid_routing::Error),

    /// Custom list already exists
    #[error("Custom list error: {0}")]
    CustomListError(#[source] mullvad_types::custom_list::Error),

    #[error("Access method error")]
    AccessMethodError(#[source] access_method::Error),

    #[error("API connection mode error")]
    ApiConnectionModeError(#[source] mullvad_api::access_mode::Error),
    #[error("No custom bridge has been specified")]
    NoCustomProxySaved,

    #[cfg(target_os = "macos")]
    #[error("Failed to set exclusion group")]
    GroupIdError(#[source] io::Error),

    #[cfg(target_os = "android")]
    #[error("Failed to initialize play purchase")]
    InitPlayPurchase(#[source] device::Error),

    #[cfg(target_os = "android")]
    #[error("Failed to verify play purchase")]
    VerifyPlayPurchase(#[source] device::Error),
}

/// Enum representing commands that can be sent to the daemon.
pub enum DaemonCommand {
    /// Set target state. Does nothing if the daemon already has the state that is being set.
    SetTargetState(oneshot::Sender<bool>, TargetState),
    /// Reconnect the tunnel, if one is connecting/connected.
    Reconnect(oneshot::Sender<bool>),
    /// Request the current state.
    GetState(oneshot::Sender<TunnelState>),
    CreateNewAccount(ResponseTx<String, Error>),
    /// Request the metadata for an account.
    GetAccountData(
        ResponseTx<AccountData, mullvad_api::rest::Error>,
        AccountNumber,
    ),
    /// Request www auth token for an account
    GetWwwAuthToken(ResponseTx<String, Error>),
    /// Submit voucher to add time to the current account. Returns time added in seconds
    SubmitVoucher(ResponseTx<VoucherSubmission, Error>, String),
    /// Request account history
    GetAccountHistory(oneshot::Sender<Option<AccountNumber>>),
    /// Remove the last used account, if there is one
    ClearAccountHistory(ResponseTx<(), Error>),
    /// Get the list of countries and cities where there are relays.
    GetRelayLocations(oneshot::Sender<RelayList>),
    /// Trigger an asynchronous relay list update. This returns before the relay list is actually
    /// updated.
    UpdateRelayLocations,
    /// Log in with a given account and create a new device.
    LoginAccount(ResponseTx<(), Error>, AccountNumber),
    /// Log out of the current account and remove the device, if they exist.
    LogoutAccount(ResponseTx<(), Error>),
    /// Return the current device configuration.
    GetDevice(ResponseTx<DeviceState, Error>),
    /// Update/check the current device, if there is one.
    UpdateDevice(ResponseTx<(), Error>),
    /// Return all the devices for a given account number.
    ListDevices(ResponseTx<Vec<Device>, Error>, AccountNumber),
    /// Remove device from a given account.
    RemoveDevice(ResponseTx<(), Error>, AccountNumber, DeviceId),
    /// Place constraints on the type of tunnel and relay
    SetRelaySettings(ResponseTx<(), settings::Error>, RelaySettings),
    /// Set the allow LAN setting.
    SetAllowLan(ResponseTx<(), settings::Error>, bool),
    /// Set the beta program setting.
    SetShowBetaReleases(ResponseTx<(), settings::Error>, bool),
    /// Set the block_when_disconnected setting.
    #[cfg(not(target_os = "android"))]
    SetBlockWhenDisconnected(ResponseTx<(), settings::Error>, bool),
    /// Set the auto-connect setting.
    SetAutoConnect(ResponseTx<(), settings::Error>, bool),
    /// Set the mssfix argument for OpenVPN
    SetOpenVpnMssfix(ResponseTx<(), settings::Error>, Option<u16>),
    /// Set proxy details for OpenVPN
    SetBridgeSettings(ResponseTx<(), Error>, BridgeSettings),
    /// Set proxy state
    SetBridgeState(ResponseTx<(), settings::Error>, BridgeState),
    /// Set if IPv6 should be enabled in the tunnel
    SetEnableIpv6(ResponseTx<(), settings::Error>, bool),
    /// Set whether to enable PQ PSK exchange in the tunnel
    SetQuantumResistantTunnel(ResponseTx<(), settings::Error>, QuantumResistantState),
    /// Set DAITA settings for the tunnel
    #[cfg(daita)]
    SetEnableDaita(ResponseTx<(), settings::Error>, bool),
    #[cfg(daita)]
    SetDaitaUseMultihopIfNecessary(ResponseTx<(), settings::Error>, bool),
    #[cfg(daita)]
    SetDaitaSettings(ResponseTx<(), settings::Error>, DaitaSettings),
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
    /// Reset all daemon settings to the defaults
    ResetSettings(ResponseTx<(), settings::Error>),
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
    /// Remove all custom lists
    ClearCustomLists(ResponseTx<(), Error>),
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
    /// Remove all custom API access methods
    ClearCustomApiAccessMethods(ResponseTx<(), Error>),
    /// Get the currently used API access method
    GetCurrentAccessMethod(ResponseTx<AccessMethodSetting, Error>),
    /// Test an API access method
    TestApiAccessMethodById(ResponseTx<bool, Error>, mullvad_types::access_method::Id),
    /// Test a custom API access method
    TestCustomApiAccessMethod(
        ResponseTx<bool, Error>,
        talpid_types::net::proxy::CustomProxy,
    ),
    /// Get information about the currently running and latest app versions
    GetVersionInfo(oneshot::Sender<Result<AppVersionInfo, Error>>),
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
    #[cfg(any(windows, target_os = "android", target_os = "macos"))]
    AddSplitTunnelApp(ResponseTx<(), Error>, SplitApp),
    /// Remove application from list of apps to exclude from the tunnel
    #[cfg(any(windows, target_os = "android", target_os = "macos"))]
    RemoveSplitTunnelApp(ResponseTx<(), Error>, SplitApp),
    /// Clear list of apps to exclude from the tunnel
    #[cfg(any(windows, target_os = "android", target_os = "macos"))]
    ClearSplitTunnelApps(ResponseTx<(), Error>),
    /// Enable or disable split tunneling
    #[cfg(any(windows, target_os = "android", target_os = "macos"))]
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
    PrepareRestart(bool),
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
    /// Patch the settings using a JSON patch
    ApplyJsonSettings(ResponseTx<(), settings::patch::Error>, String),
    /// Return a JSON blob containing all overridable settings, if there are any
    ExportJsonSettings(ResponseTx<String, settings::patch::Error>),
    /// Request the current feature indicators.
    GetFeatureIndicators(oneshot::Sender<FeatureIndicators>),
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
    AccessMethodEvent {
        event: AccessMethodEvent,
        endpoint_active_tx: oneshot::Sender<()>,
    },
    /// Handles updates from versions without devices.
    DeviceMigrationEvent(Result<PrivateAccountAndDevice, device::Error>),
    /// A geographical location has has been received from am.i.mullvad.net
    LocationEvent(LocationEventData),
    /// A generic event for when any settings change.
    SettingsChanged,
    /// The split tunnel paths or state were updated.
    #[cfg(any(windows, target_os = "android", target_os = "macos"))]
    ExcludedPathsEvent(ExcludedPathsUpdate, oneshot::Sender<Result<(), Error>>),
    /// A network leak was detected.
    LeakDetected(LeakInfo),
}

#[cfg(any(windows, target_os = "android", target_os = "macos"))]
pub(crate) enum ExcludedPathsUpdate {
    SetState(bool),
    SetPaths(HashSet<SplitApp>),
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

impl From<(AccessMethodEvent, oneshot::Sender<()>)> for InternalDaemonEvent {
    fn from(event: (AccessMethodEvent, oneshot::Sender<()>)) -> Self {
        InternalDaemonEvent::AccessMethodEvent {
            event: event.0,
            endpoint_active_tx: event.1,
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

#[derive(Debug, Clone)]
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

impl<E> Sender<E> for DaemonEventSender<E>
where
    InternalDaemonEvent: From<E>,
{
    fn send(&self, event: E) -> Result<(), talpid_core::mpsc::Error> {
        match self.sender.upgrade() {
            Some(sender) => sender
                .unbounded_send(InternalDaemonEvent::from(event))
                .map_err(|_| talpid_core::mpsc::Error::ChannelClosed),
            _ => Err(talpid_core::mpsc::Error::ChannelClosed),
        }
    }
}

impl<E> DaemonEventSender<E>
where
    InternalDaemonEvent: From<E>,
{
    pub fn to_unbounded_sender<T>(&self) -> mpsc::UnboundedSender<T>
    where
        InternalDaemonEvent: From<E>,
        T: Send + 'static,
        E: From<T>,
    {
        let (tx, mut rx) = mpsc::unbounded::<T>();
        let sender = self.sender.clone();
        tokio::spawn(async move {
            while let Some(msg) = rx.next().await {
                if let Some(tx) = sender.upgrade() {
                    let e: E = E::from(msg);
                    if tx.send(e.into()).is_err() {
                        return;
                    }
                } else {
                    return;
                };
            }
        });
        tx
    }
}

pub struct Daemon {
    tunnel_state: TunnelState,
    target_state: PersistentTargetState,
    #[cfg(target_os = "linux")]
    exclude_pids: split_tunnel::PidManager,
    rx: mpsc::UnboundedReceiver<InternalDaemonEvent>,
    tx: DaemonEventSender,
    reconnection_job: Option<AbortHandle>,
    management_interface: ManagementInterfaceServer,
    migration_complete: migrations::MigrationComplete,
    settings: SettingsPersister,
    account_history: account_history::AccountHistory,
    device_checker: device::TunnelStateChangeHandler,
    account_manager: device::AccountManagerHandle,
    access_mode_handler: mullvad_api::access_mode::AccessModeSelectorHandle,
    api_runtime: mullvad_api::Runtime,
    api_handle: mullvad_api::rest::MullvadRestHandle,
    version_updater_handle: version_check::VersionUpdaterHandle,
    relay_selector: RelaySelector,
    relay_list_updater: RelayListUpdaterHandle,
    parameters_generator: tunnel::ParametersGenerator,
    shutdown_tasks: Vec<Pin<Box<dyn Future<Output = ()> + Send + Sync>>>,
    tunnel_state_machine_handle: TunnelStateMachineHandle,
    #[cfg(target_os = "windows")]
    volume_update_tx: mpsc::UnboundedSender<()>,
    location_handler: GeoIpHandler,
    leak_checker: LeakChecker,
}
pub struct DaemonConfig {
    pub log_dir: Option<PathBuf>,
    pub resource_dir: PathBuf,
    pub settings_dir: PathBuf,
    pub cache_dir: PathBuf,
    pub rpc_socket_path: PathBuf,
    pub endpoint: ApiEndpoint,
    #[cfg(target_os = "android")]
    pub android_context: AndroidContext,
}

impl Daemon {
    pub async fn start(
        config: DaemonConfig,
        daemon_command_channel: DaemonCommandChannel,
    ) -> Result<Self, Error> {
        #[cfg(target_os = "macos")]
        macos::bump_filehandle_limit();

        let command_sender = daemon_command_channel.sender();
        let management_interface =
            ManagementInterfaceServer::start(command_sender, config.rpc_socket_path)
                .map_err(Error::ManagementInterfaceError)?;

        let (internal_event_tx, internal_event_rx) = daemon_command_channel.destructure();

        #[cfg(target_os = "android")]
        let connectivity_listener = ConnectivityListener::new(config.android_context.clone())
            .inspect_err(|error| {
                log::error!(
                    "{}",
                    error.display_chain_with_msg("Failed to start connectivity listener")
                );
            })
            .map_err(|_| Error::DaemonUnavailable)?;

        mullvad_api::proxy::ApiConnectionMode::try_delete_cache(&config.cache_dir).await;
        let api_runtime = mullvad_api::Runtime::with_cache(
            &config.endpoint,
            &config.cache_dir,
            true,
            #[cfg(target_os = "android")]
            api::create_bypass_tx(&internal_event_tx),
        )
        .await
        .map_err(Error::InitRpcFactory)?;

        let api_availability = api_runtime.availability_handle();
        api_availability.suspend();

        let migration_data = migrations::migrate_all(&config.cache_dir, &config.settings_dir)
            .await
            .unwrap_or_else(|error| {
                log::error!(
                    "{}",
                    error.display_chain_with_msg("Failed to migrate settings or cache")
                );
                None
            });

        let settings_event_listener = management_interface.notifier().clone();
        let mut settings = SettingsPersister::load(&config.settings_dir).await;
        settings.register_change_listener(move |settings| {
            // Notify management interface server of changes to the settings
            settings_event_listener.notify_settings(settings.to_owned());
        });

        let initial_selector_config = SelectorConfig::from_settings(&settings);
        let relay_selector = RelaySelector::new(
            initial_selector_config,
            config.resource_dir.join(RELAYS_FILENAME),
            config.cache_dir.join(RELAYS_FILENAME),
        );

        let settings_relay_selector = relay_selector.clone();
        settings.register_change_listener(move |settings| {
            // Notify relay selector of changes to the settings/selector config
            settings_relay_selector
                .clone()
                .set_config(SelectorConfig::from_settings(settings));
        });

        let encrypted_dns_proxy_cache = EncryptedDnsProxyState::default();
        let bridge_dns_proxy_provider = DaemonAccessMethodResolver::new(
            relay_selector.clone(),
            encrypted_dns_proxy_cache,
            api_runtime.address_cache().clone(),
        );

        let (access_mode_handler, access_mode_provider) =
            mullvad_api::access_mode::AccessModeSelector::spawn(
                config.cache_dir.clone(),
                bridge_dns_proxy_provider,
                settings.api_access_methods.clone(),
                #[cfg(feature = "api-override")]
                config.endpoint.clone(),
                internal_event_tx.to_unbounded_sender(),
            )
            .await
            .map_err(Error::ApiConnectionModeError)?;

        let api_handle = api_runtime.mullvad_rest_handle(access_mode_provider);

        // Continually update the API IP
        tokio::spawn(api_address_updater::run_api_address_fetcher(
            api_runtime.address_cache().clone(),
            api_handle.clone(),
            #[cfg(feature = "api-override")]
            config.endpoint.clone(),
        ));

        let access_method_handle = access_mode_handler.clone();
        settings.register_change_listener(move |settings| {
            let handle = access_method_handle.clone();
            let new_access_methods = settings.api_access_methods.clone();
            tokio::spawn(async move {
                let _ = handle.update_access_methods(new_access_methods).await;
            });
        });

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
            &config.settings_dir,
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
            &config.settings_dir,
            data.device().map(|device| device.account_number.clone()),
        )
        .await
        .map_err(Error::LoadAccountHistory)?;

        let target_state = if settings.auto_connect {
            log::info!("Automatically connecting since auto-connect is turned on");
            PersistentTargetState::new_secured(&config.cache_dir).await
        } else {
            PersistentTargetState::new(&config.cache_dir).await
        };

        #[cfg(any(windows, target_os = "android", target_os = "macos"))]
        let exclude_paths = if settings.split_tunnel.enable_exclusions {
            settings
                .split_tunnel
                .apps
                .iter()
                .cloned()
                .map(SplitApp::to_tunnel_command_repr)
                .collect()
        } else {
            vec![]
        };

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

        // Register a listener for generic settings changes.
        // This is useful for example for updating feature indicators when the settings change.
        let settings_changed_event_sender = internal_event_tx.clone();
        settings.register_change_listener(move |_settings| {
            let _ = settings_changed_event_sender.send(InternalDaemonEvent::SettingsChanged);
        });

        let route_manager = RouteManagerHandle::spawn(
            #[cfg(target_os = "linux")]
            mullvad_types::TUNNEL_FWMARK,
            #[cfg(target_os = "linux")]
            mullvad_types::TUNNEL_TABLE_ID,
            #[cfg(target_os = "android")]
            config.android_context.clone(),
        )
        .await
        .map_err(Error::RouteManager)?;

        let (offline_state_tx, offline_state_rx) = mpsc::unbounded();
        #[cfg(target_os = "windows")]
        let (volume_update_tx, volume_update_rx) = mpsc::unbounded();
        let tunnel_state_machine_handle = tunnel_state_machine::spawn(
            tunnel_state_machine::InitialTunnelState {
                allow_lan: settings.allow_lan,
                #[cfg(not(target_os = "android"))]
                block_when_disconnected: settings.block_when_disconnected,
                dns_config: dns::addresses_from_options(&settings.tunnel_options.dns_options),
                allowed_endpoint: access_mode_handler
                    .get_current()
                    .await
                    .map_err(Error::ApiConnectionModeError)?
                    .endpoint,
                reset_firewall: *target_state != TargetState::Secured,
                #[cfg(any(windows, target_os = "android", target_os = "macos"))]
                exclude_paths,
            },
            parameters_generator.clone(),
            config.log_dir,
            config.resource_dir.clone(),
            internal_event_tx.to_specialized_sender(),
            offline_state_tx,
            route_manager.clone(),
            #[cfg(target_os = "windows")]
            volume_update_rx,
            #[cfg(target_os = "android")]
            config.android_context,
            #[cfg(target_os = "android")]
            connectivity_listener.clone(),
            #[cfg(target_os = "linux")]
            tunnel_state_machine::LinuxNetworkingIdentifiers {
                fwmark: mullvad_types::TUNNEL_FWMARK,
                table_id: mullvad_types::TUNNEL_TABLE_ID,
            },
        )
        .await
        .map_err(Error::TunnelError)?;

        api::forward_offline_state(api_availability.clone(), offline_state_rx);

        let relay_list_listener = management_interface.notifier().clone();
        let on_relay_list_update = move |relay_list: &RelayList| {
            relay_list_listener.notify_relay_list(relay_list.clone());
        };

        let mut relay_list_updater = RelayListUpdater::spawn(
            relay_selector.clone(),
            api_handle.clone(),
            &config.cache_dir,
            on_relay_list_update,
        );

        let version_updater_handle = version_check::VersionUpdater::spawn(
            api_handle.clone(),
            api_availability.clone(),
            config.cache_dir.clone(),
            internal_event_tx.to_specialized_sender(),
            settings.show_beta_releases,
        )
        .await;

        // Attempt to download a fresh relay list
        relay_list_updater.update().await;

        let location_handler = GeoIpHandler::new(
            api_runtime.rest_handle(
                #[cfg(not(target_os = "android"))]
                mullvad_api::DefaultDnsResolver,
                #[cfg(target_os = "android")]
                android_dns::AndroidDnsResolver::new(connectivity_listener),
            ),
            internal_event_tx.clone().to_specialized_sender(),
        );

        let leak_checker = {
            let mut leak_checker = LeakChecker::new(route_manager);
            let internal_event_tx = internal_event_tx.clone();
            leak_checker.add_leak_callback(move |info| {
                internal_event_tx
                    .send(InternalDaemonEvent::LeakDetected(info))
                    .is_ok()
            });
            leak_checker
        };

        let daemon = Daemon {
            tunnel_state: TunnelState::Disconnected {
                location: None,
                #[cfg(not(target_os = "android"))]
                locked_down: settings.block_when_disconnected,
            },
            target_state,
            #[cfg(target_os = "linux")]
            exclude_pids: split_tunnel::PidManager::new().map_err(Error::InitSplitTunneling)?,
            rx: internal_event_rx,
            tx: internal_event_tx,
            reconnection_job: None,
            management_interface,
            migration_complete,
            settings,
            account_history,
            device_checker: device::TunnelStateChangeHandler::new(account_manager.clone()),
            account_manager,
            access_mode_handler,
            api_runtime,
            api_handle,
            version_updater_handle,
            relay_selector,
            relay_list_updater,
            parameters_generator,
            shutdown_tasks: vec![],
            tunnel_state_machine_handle,
            #[cfg(target_os = "windows")]
            volume_update_tx,
            location_handler,
            leak_checker,
        };

        api_availability.unsuspend();

        Ok(daemon)
    }

    /// Consume the `Daemon` and run the main event loop. Blocks until an error happens or a
    /// shutdown event is received.
    pub async fn run(mut self) -> Result<(), Error> {
        self.handle_initial_target_state();
        self.handle_events().await;
        self.disconnect_tunnel_and_wait().await;
        self.finalize().await;
        Ok(())
    }

    fn handle_initial_target_state(&mut self) {
        match self.target_state.to_strict() {
            either::Either::Right(state) => {
                self.send_tunnel_command(Self::secured_state_to_tunnel_command(state));
            }
            either::Either::Left(_) => {
                // Fetching GeoIpLocation is automatically done when connecting.
                // If TargetState is Unsecured we will not connect on lauch and
                // so we have to explicitly fetch this information.
                self.fetch_am_i_mullvad()
            }
        }
    }

    /// Map the secured target state to a tunnel command
    const fn secured_state_to_tunnel_command(_: TargetStateStrict<Secured>) -> TunnelCommand {
        TunnelCommand::Connect
    }

    /// Begin disconnecting and wait for the tunnel state machine to be disconnected
    async fn disconnect_tunnel_and_wait(&mut self) {
        if self.tunnel_state.is_disconnected() {
            return;
        }

        self.disconnect_tunnel();

        while let Some(event) = self.rx.next().await {
            match event {
                InternalDaemonEvent::TunnelStateTransition(transition) => {
                    self.handle_tunnel_state_transition(transition).await;
                }
                _ => {
                    log::trace!("Ignoring event because the daemon is shutting down");
                }
            }

            if self.tunnel_state.is_disconnected() {
                break;
            }
        }
    }

    /// Destroy daemon safely, by dropping all objects in the correct order, waiting for them to
    /// be destroyed, and executing shutdown tasks
    async fn finalize(self) {
        let Daemon {
            management_interface,
            shutdown_tasks,
            api_runtime,
            tunnel_state_machine_handle,
            target_state,
            account_manager,
            ..
        } = self;

        for future in shutdown_tasks {
            future.await;
        }

        target_state.finalize().await;
        account_manager.shutdown().await;

        tunnel_state_machine_handle.try_join().await;
        // Wait for the management interface server to shut down
        management_interface.stop().await;

        drop(api_runtime);
    }

    /// Handle internal daemon events until a shutdown event is received
    async fn handle_events(&mut self) {
        while let Some(event) = self.rx.next().await {
            if self.handle_event(event).await {
                break;
            }
        }
    }

    async fn handle_event(&mut self, event: InternalDaemonEvent) -> bool {
        use self::InternalDaemonEvent::*;
        let mut should_stop = false;
        match event {
            TunnelStateTransition(transition) => {
                self.handle_tunnel_state_transition(transition).await;
            }
            Command(command) => self.handle_command(command).await,
            TriggerShutdown(user_init_shutdown) => {
                self.on_trigger_shutdown(user_init_shutdown);
                should_stop = true;
            }
            NewAppVersionInfo(app_version_info) => {
                self.handle_new_app_version_info(app_version_info);
            }
            DeviceEvent(event) => self.handle_device_event(event).await,
            AccessMethodEvent {
                event,
                endpoint_active_tx,
            } => self.handle_access_method_event(event, endpoint_active_tx),
            DeviceMigrationEvent(event) => self.handle_device_migration_event(event),
            LocationEvent(location_data) => self.handle_location_event(location_data),
            SettingsChanged => {
                self.update_feature_indicators_on_settings_changed();
            }
            #[cfg(any(windows, target_os = "android", target_os = "macos"))]
            ExcludedPathsEvent(update, tx) => self.handle_new_excluded_paths(update, tx).await,
            LeakDetected(leak_info) => {
                log::warn!("Network leak detected! Please contact Mullvad support.");
                log::warn!("{leak_info:?}")
            }
        }
        should_stop
    }

    async fn handle_tunnel_state_transition(
        &mut self,
        tunnel_state_transition: TunnelStateTransition,
    ) {
        self.leak_checker
            .on_tunnel_state_transition(tunnel_state_transition.clone());

        self.reset_rpc_sockets_on_tunnel_state_transition(&tunnel_state_transition);
        self.device_checker
            .handle_state_transition(&tunnel_state_transition);

        let tunnel_state = match tunnel_state_transition {
            #[cfg(not(target_os = "android"))]
            TunnelStateTransition::Disconnected { locked_down } => TunnelState::Disconnected {
                location: None,
                locked_down,
            },
            #[cfg(target_os = "android")]
            TunnelStateTransition::Disconnected => TunnelState::Disconnected { location: None },
            TunnelStateTransition::Connecting(endpoint) => {
                let feature_indicators = compute_feature_indicators(
                    self.settings.settings(),
                    &endpoint,
                    self.parameters_generator.last_relay_was_overridden().await,
                );
                TunnelState::Connecting {
                    endpoint,
                    location: self.parameters_generator.get_last_location().await,
                    feature_indicators,
                }
            }
            TunnelStateTransition::Connected(endpoint) => {
                let feature_indicators = compute_feature_indicators(
                    self.settings.settings(),
                    &endpoint,
                    self.parameters_generator.last_relay_was_overridden().await,
                );
                TunnelState::Connected {
                    endpoint,
                    location: self.parameters_generator.get_last_location().await,
                    feature_indicators,
                }
            }
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

        if self.tunnel_state.is_disconnected() && !tunnel_state.is_disconnected() {
            // Enable background API requests when leaving the disconnected state.
            self.api_handle.availability.resume_background();
        }

        log::debug!("New tunnel state: {:?}", tunnel_state);

        match tunnel_state {
            TunnelState::Disconnected { .. } => {
                self.api_handle.availability.reset_inactivity_timer();
            }
            _ => {
                self.api_handle.availability.stop_inactivity_timer();
            }
        }

        match &tunnel_state {
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
        self.management_interface
            .notifier()
            .notify_new_state(tunnel_state);
        self.fetch_am_i_mullvad();
    }

    /// Get the geographical location from am.i.mullvad.net. When it arrives,
    /// update the "Out IP" field of the front ends by sending a
    /// [`InternalDaemonEvent::LocationEvent`].
    ///
    /// See [`Daemon::handle_location_event()`]
    fn fetch_am_i_mullvad(&mut self) {
        // Always abort any ongoing request when entering a new tunnel state
        self.location_handler.abort_current_request();

        // Whether or not to poll for an IPv6 exit IP
        let use_ipv6 = match &self.tunnel_state {
            // If connected, refer to the tunnel setting
            TunnelState::Connected { .. } => self.settings.tunnel_options.generic.enable_ipv6,
            // If not connected, we have to guess whether the users local connection supports IPv6.
            // The only thing we have to go on is the wireguard setting.
            TunnelState::Disconnected { .. } => {
                match &self.settings.relay_settings {
                    RelaySettings::Normal(relay_constraints) => {
                        // Note that `Constraint::Any` corresponds to just IPv4
                        matches!(
                            relay_constraints.wireguard_constraints.ip_version,
                            mullvad_types::constraints::Constraint::Only(IpVersion::V6)
                        )
                    }
                    _ => false,
                }
            }
            // Fetching IP from am.i.mullvad.net should only be done from a tunnel state where a
            // connection is available. Otherwise we just exist.
            _ => return,
        };

        self.location_handler.send_geo_location_request(use_ipv6);
    }

    /// Receives and handles the geographical exit location received from am.i.mullvad.net, i.e. the
    /// [`InternalDaemonEvent::LocationEvent`] event.
    fn handle_location_event(&mut self, location_data: LocationEventData) {
        let LocationEventData {
            request_id,
            location: fetched_location,
        } = location_data;

        if self.location_handler.request_id != request_id {
            log::debug!("Location from am.i.mullvad.net belongs to an outdated tunnel state");
            return;
        }

        match self.tunnel_state {
            TunnelState::Disconnected {
                ref mut location,
                #[cfg(not(target_os = "android"))]
                    locked_down: _,
            } => *location = Some(fetched_location),
            TunnelState::Connected {
                ref mut location, ..
            } => {
                *location = Some(GeoIpLocation {
                    ipv4: fetched_location.ipv4,
                    ipv6: fetched_location.ipv6,
                    ..location.clone().unwrap_or(fetched_location)
                })
            }
            _ => return,
        };

        self.management_interface
            .notifier()
            .notify_new_state(self.tunnel_state.clone());
    }

    /// Update the set of feature indicators based on the new settings.
    fn update_feature_indicators_on_settings_changed(&mut self) {
        // Updated settings may affect the feature indicators, even if they don't change the tunnel
        // state (e.g. activating lockdown mode). Note that only the connected and connecting states
        // have feature indicators.
        match &mut self.tunnel_state {
            TunnelState::Connecting {
                feature_indicators,
                endpoint,
                ..
            }
            | TunnelState::Connected {
                feature_indicators,
                endpoint,
                ..
            } => {
                // The server IP override feature indicator can only be changed when the tunnels
                // state changes and it is updated in `handle_tunnel_state_transition`. We must rely
                // on this value being up to date as we need the relay to know if
                // the IP override is active.
                let ip_override = feature_indicators
                    .active_features()
                    .any(|f| matches!(&f, FeatureIndicator::ServerIpOverride));
                let new_feature_indicators =
                    compute_feature_indicators(self.settings.settings(), endpoint, ip_override);
                // Update and broadcast the new feature indicators if they have changed
                if *feature_indicators != new_feature_indicators {
                    // Make sure to update the daemon's actual tunnel state. Otherwise, feature
                    // indicator changes won't be persisted.
                    *feature_indicators = new_feature_indicators;

                    self.management_interface
                        .notifier()
                        .notify_new_state(self.tunnel_state.clone());
                }
            }
            _ => {}
        };
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
        if self.tunnel_state.is_disconnected() {
            self.api_handle.availability.reset_inactivity_timer();
        }

        match command {
            SetTargetState(tx, state) => self.on_set_target_state(tx, state).await,
            Reconnect(tx) => self.on_reconnect(tx),
            GetState(tx) => self.on_get_state(tx),
            CreateNewAccount(tx) => self.on_create_new_account(tx),
            GetAccountData(tx, account_number) => self.on_get_account_data(tx, account_number),
            GetWwwAuthToken(tx) => self.on_get_www_auth_token(tx).await,
            SubmitVoucher(tx, voucher) => self.on_submit_voucher(tx, voucher),
            GetRelayLocations(tx) => self.on_get_relay_locations(tx),
            UpdateRelayLocations => self.on_update_relay_locations().await,
            LoginAccount(tx, account_number) => self.on_login_account(tx, account_number),
            LogoutAccount(tx) => self.on_logout_account(tx),
            GetDevice(tx) => self.on_get_device(tx),
            UpdateDevice(tx) => self.on_update_device(tx),
            ListDevices(tx, account_number) => self.on_list_devices(tx, account_number),
            RemoveDevice(tx, account_number, device_id) => {
                self.on_remove_device(tx, account_number, device_id)
            }
            GetAccountHistory(tx) => self.on_get_account_history(tx),
            ClearAccountHistory(tx) => self.on_clear_account_history(tx).await,
            SetRelaySettings(tx, update) => self.on_set_relay_settings(tx, update).await,
            SetAllowLan(tx, allow_lan) => self.on_set_allow_lan(tx, allow_lan).await,
            SetShowBetaReleases(tx, enabled) => self.on_set_show_beta_releases(tx, enabled).await,
            #[cfg(not(target_os = "android"))]
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
            #[cfg(daita)]
            SetEnableDaita(tx, value) => self.on_set_daita_enabled(tx, value).await,
            #[cfg(daita)]
            SetDaitaUseMultihopIfNecessary(tx, value) => {
                self.on_set_daita_use_multihop_if_necessary(tx, value).await
            }
            #[cfg(daita)]
            SetDaitaSettings(tx, daita_settings) => {
                self.on_set_daita_settings(tx, daita_settings).await
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
            ResetSettings(tx) => self.on_reset_settings(tx).await,
            RotateWireguardKey(tx) => self.on_rotate_wireguard_key(tx),
            GetWireguardKey(tx) => self.on_get_wireguard_key(tx).await,
            CreateCustomList(tx, name) => self.on_create_custom_list(tx, name).await,
            DeleteCustomList(tx, id) => self.on_delete_custom_list(tx, id).await,
            UpdateCustomList(tx, update) => self.on_update_custom_list(tx, update).await,
            ClearCustomLists(tx) => self.on_clear_custom_lists(tx).await,
            GetVersionInfo(tx) => self.on_get_version_info(tx),
            AddApiAccessMethod(tx, name, enabled, access_method) => {
                self.on_add_access_method(tx, name, enabled, access_method)
                    .await
            }
            RemoveApiAccessMethod(tx, method) => self.on_remove_api_access_method(tx, method).await,
            UpdateApiAccessMethod(tx, method) => self.on_update_api_access_method(tx, method).await,
            ClearCustomApiAccessMethods(tx) => self.on_clear_custom_api_access_methods(tx).await,
            GetCurrentAccessMethod(tx) => self.on_get_current_api_access_method(tx),
            SetApiAccessMethod(tx, method) => self.on_set_api_access_method(tx, method).await,
            TestApiAccessMethodById(tx, method) => self.on_test_api_access_method(tx, method).await,
            TestCustomApiAccessMethod(tx, proxy) => self.on_test_proxy_as_access_method(tx, proxy),
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
            #[cfg(any(windows, target_os = "android", target_os = "macos"))]
            AddSplitTunnelApp(tx, app) => self.on_add_split_tunnel_app(tx, app),
            #[cfg(any(windows, target_os = "android", target_os = "macos"))]
            RemoveSplitTunnelApp(tx, path) => self.on_remove_split_tunnel_app(tx, path),
            #[cfg(any(windows, target_os = "android", target_os = "macos"))]
            ClearSplitTunnelApps(tx) => self.on_clear_split_tunnel_apps(tx),
            #[cfg(any(windows, target_os = "android", target_os = "macos"))]
            SetSplitTunnelState(tx, enabled) => self.on_set_split_tunnel_state(tx, enabled),
            #[cfg(windows)]
            GetSplitTunnelProcesses(tx) => self.on_get_split_tunnel_processes(tx),
            #[cfg(target_os = "windows")]
            CheckVolumes(tx) => self.on_check_volumes(tx),
            SetObfuscationSettings(tx, settings) => {
                self.on_set_obfuscation_settings(tx, settings).await
            }
            PrepareRestart(shutdown) => self.on_prepare_restart(shutdown),
            #[cfg(target_os = "android")]
            BypassSocket(fd, tx) => self.on_bypass_socket(fd, tx),
            #[cfg(target_os = "android")]
            InitPlayPurchase(tx) => self.on_init_play_purchase(tx),
            #[cfg(target_os = "android")]
            VerifyPlayPurchase(tx, play_purchase) => {
                self.on_verify_play_purchase(tx, play_purchase)
            }
            ApplyJsonSettings(tx, blob) => self.on_apply_json_settings(tx, blob).await,
            ExportJsonSettings(tx) => self.on_export_json_settings(tx),
            GetFeatureIndicators(tx) => self.on_get_feature_indicators(tx),
        }
    }

    fn handle_new_app_version_info(&mut self, app_version_info: AppVersionInfo) {
        self.management_interface
            .notifier()
            .notify_app_version(app_version_info);
    }

    async fn handle_device_event(&mut self, event: AccountEvent) {
        match &event {
            AccountEvent::Device(PrivateDeviceEvent::Login(device)) => {
                if let Err(error) = self
                    .account_history
                    .set(device.account_number.clone())
                    .await
                {
                    log::error!(
                        "{}",
                        error.display_chain_with_msg("Failed to update account history")
                    );
                }
                if *self.target_state == TargetState::Secured {
                    log::debug!("Initiating tunnel restart because the account number changed");
                    self.reconnect_tunnel();
                }
            }
            AccountEvent::Device(PrivateDeviceEvent::Logout) => {
                log::info!("Disconnecting because account number was cleared");
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
            self.management_interface
                .notifier()
                .notify_device_event(DeviceEvent::from(event));
        }
    }

    fn handle_access_method_event(
        &mut self,
        event: AccessMethodEvent,
        endpoint_active_tx: oneshot::Sender<()>,
    ) {
        #[cfg(target_os = "android")]
        match event {
            AccessMethodEvent::New { setting, .. } => {
                // On android mullvad-api invokes protect on a socket to send requests
                // outside the tunnel
                let notifier = self.management_interface.notifier().clone();
                tokio::spawn(async move {
                    // No-op
                    let _ = endpoint_active_tx.send(());
                    // Notify clients about the change if necessary.
                    notifier.notify_new_access_method_event(setting);
                });
            }
        }
        #[cfg(not(target_os = "android"))]
        match event {
            AccessMethodEvent::Allow { endpoint } => {
                let (completion_tx, completion_rx) = oneshot::channel();
                self.send_tunnel_command(TunnelCommand::AllowEndpoint(endpoint, completion_tx));
                tokio::spawn(async move {
                    // Wait for the firewall policy to be updated.
                    let _ = completion_rx.await;
                    // Let the emitter of this event know that the firewall has been updated.
                    let _ = endpoint_active_tx.send(());
                });
            }
            AccessMethodEvent::New { setting, endpoint } => {
                // Update the firewall to exempt a new API endpoint.
                let (completion_tx, completion_rx) = oneshot::channel();
                self.send_tunnel_command(TunnelCommand::AllowEndpoint(endpoint, completion_tx));
                // Announce to all clients listening for updates of the
                // currently active access method. The announcement should be
                // made after the firewall policy has been updated, since the
                // new access method will be useless before then.
                let notifier = self.management_interface.notifier().clone();
                tokio::spawn(async move {
                    // Wait for the firewall policy to be updated.
                    let _ = completion_rx.await;
                    // Let the emitter of this event know that the firewall has been updated.
                    let _ = endpoint_active_tx.send(());
                    // Notify clients about the change if necessary.
                    notifier.notify_new_access_method_event(setting);
                });
            }
        }
    }

    fn handle_device_migration_event(
        &mut self,
        result: Result<PrivateAccountAndDevice, device::Error>,
    ) {
        let account_manager = self.account_manager.clone();
        let notifier = self.management_interface.notifier().clone();
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
                notifier.notify_device_event(event);
            }
        });
    }

    #[cfg(any(windows, target_os = "android", target_os = "macos"))]
    async fn handle_new_excluded_paths(
        &mut self,
        update: ExcludedPathsUpdate,
        tx: ResponseTx<(), Error>,
    ) {
        let save_result = match update {
            ExcludedPathsUpdate::SetState(state) => {
                let split_tunnel_was_enabled =
                    self.settings.settings().split_tunnel.enable_exclusions;
                let save_result = self
                    .settings
                    .update(move |settings| settings.split_tunnel.enable_exclusions = state)
                    .await
                    .map_err(Error::SettingsError);
                // If the user enables split tunneling without also enabling Full Disk Access
                // (FDA), the daemon will enter the error state. This is unlikely, since it should
                // only be possible via the CLI or if the user manages to disable FDA after having
                // successfully enabled split tunneling. In any case, We have observed users
                // getting confused over being blocked in this case, and this we may want to
                // reconnect after disabling split tunneling.
                //
                // Since FDA is an implementation detail of split tunneling, we don't actually have
                // a way of getting this information at this point, so we fallback to issuing a
                // reconnect if the user disables split tunneling while in the error state. This
                // code can be removed if we ever remove our dependency on FDA.
                if cfg!(target_os = "macos") {
                    let split_tunnel_will_be_disabled = !state;
                    if self.tunnel_state.is_in_error_state()
                        && split_tunnel_was_enabled
                        && split_tunnel_will_be_disabled
                    {
                        self.reconnect_tunnel();
                    }
                }
                save_result
            }
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
        let state_change_initated = self.set_target_state(new_target_state).await;
        Self::oneshot_send(tx, state_change_initated, "state change initiated");
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
        account_number: AccountNumber,
    ) {
        let account = self.account_manager.account_service.clone();
        tokio::spawn(async move {
            let result = account.get_data(account_number).await;
            Self::oneshot_send(tx, result, "account data");
        });
    }

    async fn on_get_www_auth_token(&mut self, tx: ResponseTx<String, Error>) {
        match self.account_manager.data().await.map(|s| s.into_device()) {
            Ok(Some(device)) => {
                let future = self
                    .account_manager
                    .account_service
                    .get_www_auth_token(device.account_number);
                tokio::spawn(async {
                    Self::oneshot_send(
                        tx,
                        future.await.map_err(Error::RestError),
                        "get_www_auth_token response",
                    );
                });
            }
            _ => {
                Self::oneshot_send(
                    tx,
                    Err(Error::NoAccountNumber),
                    "get_www_auth_token response",
                );
            }
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
        Self::oneshot_send(tx, self.relay_selector.get_relays(), "relay locations");
    }

    async fn on_update_relay_locations(&mut self) {
        self.relay_list_updater.update().await;
    }

    fn on_login_account(&mut self, tx: ResponseTx<(), Error>, account_number: String) {
        let account_manager = self.account_manager.clone();
        let availability = self.api_runtime.availability_handle();
        tokio::spawn(async move {
            let result = async {
                account_manager
                    .login(account_number)
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
                    .map_err(|_| Error::NoAccountNumber)
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

    fn on_list_devices(&self, tx: ResponseTx<Vec<Device>, Error>, token: AccountNumber) {
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
        account_number: AccountNumber,
        device_id: DeviceId,
    ) {
        let device_service = self.account_manager.device_service.clone();
        let notifier = self.management_interface.notifier().clone();

        tokio::spawn(async move {
            let result = device_service
                .remove_device(account_number.clone(), device_id)
                .await
                .map(move |new_devices| {
                    // FIXME: We should be able to get away with only returning the removed ID,
                    //        and not have to request the list from the API.
                    notifier.notify_remove_device_event(RemoveDeviceEvent {
                        account_number,
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

    fn on_get_account_history(&mut self, tx: oneshot::Sender<Option<AccountNumber>>) {
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

    fn on_get_version_info(&mut self, tx: oneshot::Sender<Result<AppVersionInfo, Error>>) {
        let mut handle = self.version_updater_handle.clone();
        tokio::spawn(async move {
            Self::oneshot_send(
                tx,
                handle
                    .get_version_info()
                    .await
                    .inspect_err(|error| {
                        log::error!(
                            "{}",
                            error.display_chain_with_msg("Error running version check")
                        )
                    })
                    .map_err(Error::VersionCheckError),
                "get_version_info response",
            );
        });
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
        let mut last_error = None;

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
            last_error = Some("Failed to clear account history");
        }

        if let Err(e) = self.settings.reset().await {
            log::error!("Failed to reset settings: {}", e);
            last_error = Some("Failed to reset settings");
        }

        // Shut the daemon down.
        let _ = self.tx.send(InternalDaemonEvent::TriggerShutdown(false));

        self.shutdown_tasks.push(Box::pin(async move {
            if let Err(e) = cleanup::clear_directories().await {
                log::error!(
                    "{}",
                    e.display_chain_with_msg("Failed to clear cache and log directories")
                );
                last_error = Some("Failed to clear cache and log directories");
            }
            let result = last_error
                .map(|error| Err(Error::FactoryResetError(error)))
                .unwrap_or(Ok(()));
            Self::oneshot_send(tx, result, "factory_reset response");
        }));
    }

    #[cfg(target_os = "linux")]
    fn on_get_split_tunnel_processes(&mut self, tx: ResponseTx<Vec<i32>, split_tunnel::Error>) {
        let result = self.exclude_pids.list().inspect_err(|error| {
            log::error!("{}", error.display_chain_with_msg("Unable to obtain PIDs"));
        });
        Self::oneshot_send(tx, result, "get_split_tunnel_processes response");
    }

    #[cfg(target_os = "linux")]
    fn on_add_split_tunnel_process(&mut self, tx: ResponseTx<(), split_tunnel::Error>, pid: i32) {
        let result = self.exclude_pids.add(pid).inspect_err(|error| {
            log::error!("{}", error.display_chain_with_msg("Unable to add PID"));
        });
        Self::oneshot_send(tx, result, "add_split_tunnel_process response");
    }

    #[cfg(target_os = "linux")]
    fn on_remove_split_tunnel_process(
        &mut self,
        tx: ResponseTx<(), split_tunnel::Error>,
        pid: i32,
    ) {
        let result = self.exclude_pids.remove(pid).inspect_err(|error| {
            log::error!("{}", error.display_chain_with_msg("Unable to remove PID"));
        });
        Self::oneshot_send(tx, result, "remove_split_tunnel_process response");
    }

    #[cfg(target_os = "linux")]
    fn on_clear_split_tunnel_processes(&mut self, tx: ResponseTx<(), split_tunnel::Error>) {
        let result = self.exclude_pids.clear().inspect_err(|error| {
            log::error!("{}", error.display_chain_with_msg("Unable to clear PIDs"));
        });
        Self::oneshot_send(tx, result, "clear_split_tunnel_processes response");
    }

    /// Update the split app paths in both the settings and tunnel
    #[cfg(any(windows, target_os = "android"))]
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

        // Update the tunnel state
        if new_state || new_state != settings.split_tunnel.enable_exclusions {
            let tunnel_list = if new_state {
                new_list
                    .cloned()
                    .map(SplitApp::to_tunnel_command_repr)
                    .collect()
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

    /// Update the split app paths in both the settings and tunnel
    #[cfg(target_os = "macos")]
    fn set_split_tunnel_paths(
        &mut self,
        tx: ResponseTx<(), Error>,
        _response_msg: &'static str,
        settings: Settings,
        update: ExcludedPathsUpdate,
    ) {
        let tunnel_list = match update {
            ExcludedPathsUpdate::SetPaths(ref paths) if settings.split_tunnel.enable_exclusions => {
                paths
                    .iter()
                    .cloned()
                    .map(SplitApp::to_tunnel_command_repr)
                    .collect()
            }
            ExcludedPathsUpdate::SetState(true) => settings
                .split_tunnel
                .apps
                .iter()
                .cloned()
                .map(SplitApp::to_tunnel_command_repr)
                .collect(),
            _ => vec![],
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
                    // NOTE: On macOS, we don't care if this fails. The tunnel will prevent us from
                    // connecting if we're in a bad state, and we can reset it by clearing the paths
                }
                Err(_) => {
                    log::error!("The tunnel failed to return a result");
                }
            }
            let _ = daemon_tx.send(InternalDaemonEvent::ExcludedPathsEvent(update, tx));
        });
    }

    #[cfg(any(target_os = "windows", target_os = "macos", target_os = "android"))]
    fn on_add_split_tunnel_app(&mut self, tx: ResponseTx<(), Error>, app: SplitApp) {
        let settings = self.settings.to_settings();

        let excluded_apps = {
            let mut apps = settings.split_tunnel.apps.clone();
            apps.insert(app);
            apps
        };

        self.set_split_tunnel_paths(
            tx,
            "add_split_tunnel_app response",
            settings,
            ExcludedPathsUpdate::SetPaths(excluded_apps),
        );
    }

    #[cfg(any(windows, target_os = "android", target_os = "macos"))]
    fn on_remove_split_tunnel_app(&mut self, tx: ResponseTx<(), Error>, app: impl Into<SplitApp>) {
        let settings = self.settings.to_settings();

        let excluded_apps = {
            let mut apps = settings.split_tunnel.apps.clone();
            apps.remove(&app.into());
            apps
        };

        self.set_split_tunnel_paths(
            tx,
            "remove_split_tunnel_app response",
            settings,
            ExcludedPathsUpdate::SetPaths(excluded_apps),
        );
    }

    #[cfg(any(windows, target_os = "android", target_os = "macos"))]
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

    #[cfg(any(windows, target_os = "android", target_os = "macos"))]
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

    #[cfg(not(target_os = "android"))]
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
        tx: ResponseTx<(), Error>,
        new_settings: BridgeSettings,
    ) {
        if new_settings.custom.is_none() && new_settings.bridge_type == BridgeType::Custom {
            log::info!("Tried to select custom bridge but no custom bridge settings exist");
            Self::oneshot_send(
                tx,
                Err(Error::NoCustomProxySaved),
                "set_bridge_settings response",
            );
            return;
        }

        match self
            .settings
            .update(move |settings| settings.bridge_settings = new_settings)
            .await
        {
            Ok(settings_changes) => {
                if settings_changes {
                    let access_mode_handler = self.access_mode_handler.clone();
                    tokio::spawn(async move {
                        if let Err(error) = access_mode_handler.rotate().await {
                            log::error!("Failed to rotate API endpoint: {error}");
                        }
                    });
                    self.reconnect_tunnel();
                };
                Self::oneshot_send(tx, Ok(()), "set_bridge_settings");
            }

            Err(e) => {
                log::error!(
                    "{}",
                    e.display_chain_with_msg("Failed to set new bridge settings")
                );
                Self::oneshot_send(tx, Err(Error::SettingsError(e)), "set_bridge_settings");
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

    #[cfg(daita)]
    async fn on_set_daita_enabled(&mut self, tx: ResponseTx<(), settings::Error>, value: bool) {
        let result = self
            .settings
            .update(|settings| {
                settings.tunnel_options.wireguard.daita.enabled = value;
            })
            .await;

        match result {
            Ok(settings_changed) => {
                Self::oneshot_send(tx, Ok(()), "set_daita_enabled response");
                let RelaySettings::Normal(constraints) = &self.settings.relay_settings else {
                    return; // DAITA is not supported for custom relays
                };

                let wireguard_enabled = constraints.tunnel_protocol == TunnelType::Wireguard;

                if settings_changed && wireguard_enabled {
                    log::info!("Reconnecting because DAITA settings changed");
                    self.reconnect_tunnel();
                }
            }
            Err(e) => {
                log::error!("{}", e.display_chain_with_msg("Unable to save settings"));
                Self::oneshot_send(tx, Err(e), "set_daita_enabled response");
            }
        }
    }

    #[cfg(daita)]
    async fn on_set_daita_use_multihop_if_necessary(
        &mut self,
        tx: ResponseTx<(), settings::Error>,
        value: bool,
    ) {
        match self
            .settings
            .update(|settings| {
                settings
                    .tunnel_options
                    .wireguard
                    .daita
                    .use_multihop_if_necessary = value
            })
            .await
        {
            Ok(settings_changed) => {
                Self::oneshot_send(tx, Ok(()), "set_daita_use_multihop_if_necessary response");

                let RelaySettings::Normal(constraints) = &self.settings.relay_settings else {
                    return; // DAITA is not supported for custom relays
                };

                let wireguard_enabled = constraints.tunnel_protocol == TunnelType::Wireguard;

                let daita_enabled = self.settings.tunnel_options.wireguard.daita.enabled;

                if settings_changed && wireguard_enabled && daita_enabled {
                    log::info!("Reconnecting because DAITA settings changed");
                    self.reconnect_tunnel();
                }
            }
            Err(e) => {
                log::error!("{}", e.display_chain_with_msg("Unable to save settings"));
                Self::oneshot_send(tx, Err(e), "set_daita_use_multihop_if_necessary response");
            }
        }
    }

    #[cfg(daita)]
    async fn on_set_daita_settings(
        &mut self,
        tx: ResponseTx<(), settings::Error>,
        daita_settings: DaitaSettings,
    ) {
        match self
            .settings
            .update(|settings| settings.tunnel_options.wireguard.daita = daita_settings)
            .await
        {
            Ok(settings_changed) => {
                Self::oneshot_send(tx, Ok(()), "set_daita_settings response");
                if settings_changed && self.get_target_tunnel_type() != Some(TunnelType::OpenVpn) {
                    log::info!("Reconnecting because DAITA settings changed");
                    self.reconnect_tunnel();
                }
            }
            Err(e) => {
                log::error!("{}", e.display_chain_with_msg("Unable to save settings"));
                Self::oneshot_send(tx, Err(e), "set_daita_settings response");
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
                    let settings = self.settings.settings();
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
        let result = match self.account_manager.data().await.map(|s| s.into_device()) {
            Ok(Some(config)) => Ok(Some(config.device.wg_data.get_public_key())),
            _ => Err(Error::NoAccountNumber),
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

    async fn on_clear_custom_lists(&mut self, tx: ResponseTx<(), Error>) {
        let result = self.clear_custom_lists().await;
        Self::oneshot_send(tx, result, "clear_custom_lists response");
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
            .use_api_access_method(access_method)
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

    async fn on_clear_custom_api_access_methods(&mut self, tx: ResponseTx<(), Error>) {
        let result = self
            .clear_custom_api_access_methods()
            .await
            .map_err(Error::AccessMethodError);
        Self::oneshot_send(tx, result, "clear_custom_api_access_methods response");
    }

    fn on_get_current_api_access_method(&mut self, tx: ResponseTx<AccessMethodSetting, Error>) {
        let handle = self.access_mode_handler.clone();
        tokio::spawn(async move {
            let result = handle
                .get_current()
                .await
                .map(|current| current.setting)
                .map_err(Error::ApiConnectionModeError);
            Self::oneshot_send(tx, result, "get_current_api_access_method response");
        });
    }

    fn on_test_proxy_as_access_method(
        &mut self,
        tx: ResponseTx<bool, Error>,
        proxy: talpid_types::net::proxy::CustomProxy,
    ) {
        use mullvad_api::proxy::{ApiConnectionMode, ProxyConfig};
        use talpid_types::net::AllowedEndpoint;

        let connection_mode = ApiConnectionMode::Proxied(ProxyConfig::from(proxy.clone()));
        let api_proxy = self.create_limited_api_proxy(connection_mode.clone());
        let proxy_endpoint = AllowedEndpoint {
            endpoint: proxy.get_remote_endpoint().endpoint,
            clients: api::allowed_clients(&connection_mode),
        };

        let daemon_event_sender = self.tx.to_specialized_sender();
        let access_method_selector = self.access_mode_handler.clone();
        tokio::spawn(async move {
            let result = Self::test_access_method(
                proxy_endpoint,
                access_method_selector,
                daemon_event_sender,
                api_proxy,
            )
            .await
            .map_err(Error::AccessMethodError);

            Self::oneshot_send(tx, result, "on_test_proxy_as_access_method response");
        });
    }

    async fn on_test_api_access_method(
        &mut self,
        tx: ResponseTx<bool, Error>,
        access_method: mullvad_types::access_method::Id,
    ) {
        let reply =
            |response| Self::oneshot_send(tx, response, "on_test_api_access_method response");

        let access_method = match self.get_api_access_method(access_method) {
            Ok(x) => x,
            Err(err) => {
                reply(Err(Error::AccessMethodError(err)));
                return;
            }
        };

        let test_subject = match self
            .access_mode_handler
            .resolve(access_method.clone())
            .await
        {
            Ok(Some(test_subject)) => test_subject,
            Ok(None) => {
                let error =
                    Error::ApiConnectionModeError(mullvad_api::access_mode::Error::Resolve {
                        access_method: access_method.access_method,
                    });
                reply(Err(error));
                return;
            }
            Err(err) => {
                reply(Err(Error::ApiConnectionModeError(err)));
                return;
            }
        };

        let api_proxy = self.create_limited_api_proxy(test_subject.connection_mode);
        let daemon_event_sender = self.tx.to_specialized_sender();
        let access_method_selector = self.access_mode_handler.clone();

        tokio::spawn(async move {
            let result = Self::test_access_method(
                test_subject.endpoint,
                access_method_selector,
                daemon_event_sender,
                api_proxy,
            )
            .await
            .map_err(Error::AccessMethodError);

            log::debug!(
                "API access method {method} {verdict}",
                method = test_subject.setting.name,
                verdict = match result {
                    Ok(true) => "could successfully connect to the Mullvad API",
                    _ => "could not connect to the Mullvad API",
                }
            );

            reply(result);
        });
    }

    fn on_get_settings(&self, tx: oneshot::Sender<Settings>) {
        Self::oneshot_send(tx, self.settings.to_settings(), "get_settings response");
    }

    async fn on_reset_settings(&mut self, tx: ResponseTx<(), settings::Error>) {
        let result = self.settings.reset().await;
        Self::oneshot_send(tx, result, "reset_settings response");

        // TODO: All of the functions below should probably be handled by settings observers
        //       whenever settings are updated. For instance, changing "allow_lan" should probably
        //       cause a tunnel command to be sent.

        #[cfg(any(target_os = "windows", target_os = "macos", target_os = "android"))]
        {
            let (tx, _rx) = oneshot::channel();
            self.send_tunnel_command(TunnelCommand::SetExcludedApps(tx, vec![]));
        }

        #[cfg(not(target_os = "android"))]
        {
            let (tx, _rx) = oneshot::channel();
            self.send_tunnel_command(TunnelCommand::BlockWhenDisconnected(
                self.settings.block_when_disconnected,
                tx,
            ));
        }

        let (tx, _rx) = oneshot::channel();
        self.send_tunnel_command(TunnelCommand::AllowLan(self.settings.allow_lan, tx));

        let (tx, _rx) = oneshot::channel();
        let dns = dns::addresses_from_options(&self.settings.tunnel_options.dns_options);
        self.send_tunnel_command(TunnelCommand::Dns(dns, tx));

        self.version_updater_handle
            .set_show_beta_releases(self.settings.show_beta_releases)
            .await;
        let access_mode_handler = self.access_mode_handler.clone();
        tokio::spawn(async move {
            if let Err(error) = access_mode_handler.rotate().await {
                log::error!("Failed to rotate API endpoint: {error}");
            }
        });

        let interval = self.settings.tunnel_options.wireguard.rotation_interval;
        let account_manager = self.account_manager.clone();
        tokio::spawn(async move {
            if let Err(error) = account_manager
                .set_rotation_interval(interval.unwrap_or_default())
                .await
            {
                log::error!(
                    "{}",
                    error.display_chain_with_msg("Failed to update rotation interval")
                );
            }
        });

        self.reconnect_tunnel();
    }

    fn oneshot_send<T>(tx: oneshot::Sender<T>, t: T, msg: &'static str) {
        if tx.send(t).is_err() {
            log::warn!("Unable to send {} to the daemon command sender", msg);
        }
    }

    #[cfg_attr(target_os = "android", allow(unused_variables))]
    fn on_trigger_shutdown(&mut self, user_init_shutdown: bool) {
        // Block all traffic before shutting down to ensure that no traffic can leak on boot or
        // shutdown.
        #[cfg(not(target_os = "android"))]
        if !user_init_shutdown
            && (*self.target_state == TargetState::Secured || self.settings.auto_connect)
        {
            log::debug!("Blocking firewall during shutdown");
            let (tx, _rx) = oneshot::channel();
            self.send_tunnel_command(TunnelCommand::BlockWhenDisconnected(true, tx));
        }

        self.disconnect_tunnel();
    }

    /// Prepare the daemon for a restart by setting the target state to [`TargetState::Secured`].
    ///
    /// - `shutdown`: If the daemon should shut down itself when after setting the secured target
    ///   state. set to `false` if the intention is to close the daemon process manually.
    fn on_prepare_restart(&mut self, shutdown: bool) {
        // TODO: See if this can be made to also shut down the daemon
        //       without causing the service to be restarted.
        #[cfg(not(target_os = "android"))]
        if *self.target_state == TargetState::Secured {
            let (tx, _rx) = oneshot::channel();
            self.send_tunnel_command(TunnelCommand::BlockWhenDisconnected(true, tx));
        }
        self.target_state.lock();

        if shutdown {
            let _ = self.tx.send(InternalDaemonEvent::TriggerShutdown(false));
        }
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

    fn on_export_json_settings(&mut self, tx: ResponseTx<String, settings::patch::Error>) {
        let result = settings::patch::export_settings(&self.settings);
        Self::oneshot_send(tx, result, "export_json_settings response");
    }

    fn on_get_feature_indicators(&self, tx: oneshot::Sender<FeatureIndicators>) {
        let feature_indicators = match &self.tunnel_state {
            TunnelState::Connecting {
                feature_indicators, ..
            } => feature_indicators.to_owned(),
            TunnelState::Connected {
                feature_indicators, ..
            } => feature_indicators.to_owned(),
            _ => FeatureIndicators::default(),
        };
        Self::oneshot_send(tx, feature_indicators, "get_feature_indicators response");
    }

    /// Set the target state of the client. If it changed trigger the operations needed to
    /// progress towards that state.
    /// Returns a bool representing whether a state change was initiated.
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
        self.send_tunnel_command(TunnelCommand::Connect);
    }

    fn disconnect_tunnel(&self) {
        self.send_tunnel_command(TunnelCommand::Disconnect);
    }

    fn reconnect_tunnel(&mut self) {
        if *self.target_state == TargetState::Secured {
            self.connect_tunnel();
        }
    }

    const fn get_connected_tunnel_type(&self) -> Option<TunnelType> {
        match self.tunnel_state.get_tunnel_type() {
            Some(tunnel_type) if self.tunnel_state.is_connected() => Some(tunnel_type),
            Some(_) | None => None,
        }
    }

    const fn get_target_tunnel_type(&self) -> Option<TunnelType> {
        self.tunnel_state.get_tunnel_type()
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

/// Consume a oneshot sender of `T1` and return a sender that takes a different type `T2`.
/// `forwarder` should map `T1` back to `T2` and send the result back to the original receiver.
fn oneshot_map<T1: Send + 'static, T2: Send + 'static>(
    tx: oneshot::Sender<T1>,
    forwarder: impl Fn(oneshot::Sender<T1>, T2) + Send + 'static,
) -> oneshot::Sender<T2> {
    let (new_tx, new_rx) = oneshot::channel();
    tokio::spawn(async move {
        if let Ok(result) = new_rx.await {
            forwarder(tx, result)
        }
    });
    new_tx
}

/// Remove any old RPC socket (if it exists).
#[cfg(not(windows))]
pub async fn cleanup_old_rpc_socket(rpc_socket_path: impl AsRef<std::path::Path>) {
    if let Err(err) = tokio::fs::remove_file(rpc_socket_path).await {
        if err.kind() != std::io::ErrorKind::NotFound {
            log::error!("Failed to remove old RPC socket: {}", err);
        }
    }
}

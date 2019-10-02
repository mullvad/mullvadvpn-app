use crate::EventListener;
use jsonrpc_core::{
    futures::{
        future,
        sync::{self, oneshot::Sender as OneshotSender},
        Future,
    },
    Error, ErrorCode, MetaIoHandler, Metadata,
};
use jsonrpc_ipc_server;
use jsonrpc_macros::{build_rpc_trait, metadata, pubsub};
use jsonrpc_pubsub::{PubSubHandler, PubSubMetadata, Session, SubscriptionId};
use mullvad_paths;
use mullvad_rpc;
use mullvad_types::{
    account::{AccountData, AccountToken},
    location::GeoIpLocation,
    relay_constraints::{BridgeSettings, BridgeState, RelaySettingsUpdate},
    relay_list::RelayList,
    settings::{self, Settings},
    states::{TargetState, TunnelState},
    version, wireguard, DaemonEvent,
};
use parking_lot::{Mutex, RwLock};
use std::{
    collections::{hash_map::Entry, HashMap},
    sync::Arc,
};
use talpid_core::mpsc::IntoSender;
use talpid_ipc;
use talpid_types::ErrorExt;
use uuid;

/// FIXME(linus): This is here just because the futures crate has deprecated it and jsonrpc_core
/// did not introduce their own yet (https://github.com/paritytech/jsonrpc/pull/196).
/// Remove this and use the one in jsonrpc_core when that is released.
pub type BoxFuture<T, E> = Box<dyn Future<Item = T, Error = E> + Send>;

build_rpc_trait! {
    pub trait ManagementInterfaceApi {
        type Metadata;

        /// Creates and sets a new account
        #[rpc(meta, name = "create_new_account")]
        fn create_new_account(&self, Self::Metadata) -> BoxFuture<String, Error>;

        /// Fetches and returns metadata about an account. Returns an error on non-existing
        /// accounts.
        #[rpc(meta, name = "get_account_data")]
        fn get_account_data(&self, Self::Metadata, AccountToken) -> BoxFuture<AccountData, Error>;

        #[rpc(meta, name = "get_www_auth_token")]
        fn get_www_auth_token(&self, Self::Metadata) -> BoxFuture<String, Error>;

        /// Returns available countries.
        #[rpc(meta, name = "get_relay_locations")]
        fn get_relay_locations(&self, Self::Metadata) -> BoxFuture<RelayList, Error>;

        /// Triggers a relay list update
        #[rpc(meta, name = "update_relay_locations")]
        fn update_relay_locations(&self, Self::Metadata) -> BoxFuture<(), Error>;

        /// Set which account to connect with.
        #[rpc(meta, name = "set_account")]
        fn set_account(&self, Self::Metadata, Option<AccountToken>) -> BoxFuture<(), Error>;

        /// Update constraints put on the type of tunnel connection to use
        #[rpc(meta, name = "update_relay_settings")]
        fn update_relay_settings(
            &self,
            Self::Metadata, RelaySettingsUpdate
            ) -> BoxFuture<(), Error>;

        /// Set if the client should allow communication with the LAN while in secured state.
        #[rpc(meta, name = "set_allow_lan")]
        fn set_allow_lan(&self, Self::Metadata, bool) -> BoxFuture<(), Error>;

        /// Set if the client should allow network communication when in the disconnected state.
        #[rpc(meta, name = "set_block_when_disconnected")]
        fn set_block_when_disconnected(&self, Self::Metadata, bool) -> BoxFuture<(), Error>;

        /// Set if the daemon should automatically establish a tunnel on start or not.
        #[rpc(meta, name = "set_auto_connect")]
        fn set_auto_connect(&self, Self::Metadata, bool) -> BoxFuture<(), Error>;

        /// Try to connect if disconnected, or do nothing if already connecting/connected.
        #[rpc(meta, name = "connect")]
        fn connect(&self, Self::Metadata) -> BoxFuture<(), Error>;

        /// Disconnect the VPN tunnel if it is connecting/connected. Does nothing if already
        /// disconnected.
        #[rpc(meta, name = "disconnect")]
        fn disconnect(&self, Self::Metadata) -> BoxFuture<(), Error>;

        /// Returns the current state of the Mullvad client. Changes to this state will
        /// be announced to subscribers of `new_state`.
        #[rpc(meta, name = "get_state")]
        fn get_state(&self, Self::Metadata) -> BoxFuture<TunnelState, Error>;

        /// Performs a geoIP lookup and returns the current location as perceived by the public
        /// internet.
        #[rpc(meta, name = "get_current_location")]
        fn get_current_location(&self, Self::Metadata) -> BoxFuture<Option<GeoIpLocation>, Error>;

        /// Makes the daemon exit its main loop and quit.
        #[rpc(meta, name = "shutdown")]
        fn shutdown(&self, Self::Metadata) -> BoxFuture<(), Error>;

        /// Get previously used account tokens from the account history
        #[rpc(meta, name = "get_account_history")]
        fn get_account_history(&self, Self::Metadata) -> BoxFuture<Vec<AccountToken>, Error>;

        /// Remove given account token from the account history
        #[rpc(meta, name = "remove_account_from_history")]
        fn remove_account_from_history(&self, Self::Metadata, AccountToken) -> BoxFuture<(), Error>;

        /// Sets openvpn's mssfix parameter
        #[rpc(meta, name = "set_openvpn_mssfix")]
        fn set_openvpn_mssfix(&self, Self::Metadata, Option<u16>) -> BoxFuture<(), Error>;

        /// Sets proxy details for OpenVPN
        #[rpc(meta, name = "set_bridge_settings")]
        fn set_bridge_settings(&self, Self::Metadata, BridgeSettings) -> BoxFuture<(), Error>;

        /// Sets bridge state
        #[rpc(meta, name = "set_bridge_state")]
        fn set_bridge_state(&self, Self::Metadata, BridgeState) -> BoxFuture<(), Error>;

        /// Set if IPv6 is enabled in the tunnel
        #[rpc(meta, name = "set_enable_ipv6")]
        fn set_enable_ipv6(&self, Self::Metadata, bool) -> BoxFuture<(), Error>;

        /// Set MTU for wireguard tunnels
        #[rpc(meta, name = "set_wireguard_mtu")]
        fn set_wireguard_mtu(&self, Self::Metadata, Option<u16>) -> BoxFuture<(), Error>;

        /// Returns the current daemon settings
        #[rpc(meta, name = "get_settings")]
        fn get_settings(&self, Self::Metadata) -> BoxFuture<Settings, Error>;

        /// Generates new wireguard key for current account
        #[rpc(meta, name = "generate_wireguard_key")]
        fn generate_wireguard_key(&self, Self::Metadata) -> BoxFuture<wireguard::KeygenEvent, Error>;

        /// Retrieve a public key for current account if the account has one.
        #[rpc(meta, name = "get_wireguard_key")]
        fn get_wireguard_key(&self, Self::Metadata) -> BoxFuture<Option<wireguard::PublicKey>, Error>;

        /// Verify if current wireguard key is still valid
        #[rpc(meta, name = "verify_wireguard_key")]
        fn verify_wireguard_key(&self, Self::Metadata) -> BoxFuture<bool, Error>;

        /// Retreive version of the app
        #[rpc(meta, name = "get_current_version")]
        fn get_current_version(&self, Self::Metadata) -> BoxFuture<String, Error>;

        /// Retrieve information about the currently running and latest versions of the app
        #[rpc(meta, name = "get_version_info")]
        fn get_version_info(&self, Self::Metadata) -> BoxFuture<version::AppVersionInfo, Error>;

        /// Remove all configuration and cache files
        #[rpc(meta, name = "factory_reset")]
        fn factory_reset(&self, Self::Metadata) -> BoxFuture<(), Error>;

        #[pubsub(name = "daemon_event")] {
            /// Subscribes to events from the daemon.
            #[rpc(name = "daemon_event_subscribe")]
            fn daemon_event_subscribe(
                &self,
                Self::Metadata,
                pubsub::Subscriber<DaemonEvent>
            );

            /// Unsubscribes from the `daemon_event` event notifications.
            #[rpc(name = "daemon_event_unsubscribe")]
            fn daemon_event_unsubscribe(&self, SubscriptionId) -> BoxFuture<(), Error>;
        }
    }
}


/// Enum representing commands coming in on the management interface.
pub enum ManagementCommand {
    /// Change target state.
    SetTargetState(OneshotSender<Result<(), ()>>, TargetState),
    /// Request the current state.
    GetState(OneshotSender<TunnelState>),
    /// Get the current geographical location.
    GetCurrentLocation(OneshotSender<Option<GeoIpLocation>>),
    CreateNewAccount(OneshotSender<std::result::Result<String, mullvad_rpc::Error>>),
    /// Request the metadata for an account.
    GetAccountData(
        OneshotSender<BoxFuture<AccountData, mullvad_rpc::Error>>,
        AccountToken,
    ),
    /// Request www auth token for an account
    GetWwwAuthToken(OneshotSender<BoxFuture<String, mullvad_rpc::Error>>),
    /// Request account history
    GetAccountHistory(OneshotSender<Vec<AccountToken>>),
    /// Request account history
    RemoveAccountFromHistory(OneshotSender<()>, AccountToken),
    /// Get the list of countries and cities where there are relays.
    GetRelayLocations(OneshotSender<RelayList>),
    /// Trigger an asynchronous relay list update. This returns before the relay list is actually
    /// updated.
    UpdateRelayLocations,
    /// Set which account token to use for subsequent connection attempts.
    SetAccount(OneshotSender<()>, Option<AccountToken>),
    /// Place constraints on the type of tunnel and relay
    UpdateRelaySettings(OneshotSender<()>, RelaySettingsUpdate),
    /// Set the allow LAN setting.
    SetAllowLan(OneshotSender<()>, bool),
    /// Set the block_when_disconnected setting.
    SetBlockWhenDisconnected(OneshotSender<()>, bool),
    /// Set the auto-connect setting.
    SetAutoConnect(OneshotSender<()>, bool),
    /// Set the mssfix argument for OpenVPN
    SetOpenVpnMssfix(OneshotSender<()>, Option<u16>),
    /// Set proxy details for OpenVPN
    SetBridgeSettings(OneshotSender<Result<(), settings::Error>>, BridgeSettings),
    /// Set proxy state
    SetBridgeState(OneshotSender<Result<(), settings::Error>>, BridgeState),
    /// Set if IPv6 should be enabled in the tunnel
    SetEnableIpv6(OneshotSender<()>, bool),
    /// Set MTU for wireguard tunnels
    SetWireguardMtu(OneshotSender<()>, Option<u16>),
    /// Get the daemon settings
    GetSettings(OneshotSender<Settings>),
    /// Generate new wireguard key
    GenerateWireguardKey(OneshotSender<wireguard::KeygenEvent>),
    /// Return a public key of the currently set wireguard private key, if there is one
    GetWireguardKey(OneshotSender<Option<wireguard::PublicKey>>),
    /// Verify if the currently set wireguard key is valid.
    VerifyWireguardKey(OneshotSender<bool>),
    /// Get information about the currently running and latest app versions
    GetVersionInfo(OneshotSender<BoxFuture<version::AppVersionInfo, mullvad_rpc::Error>>),
    /// Get current version of the app
    GetCurrentVersion(OneshotSender<version::AppVersion>),
    #[cfg(not(target_os = "android"))]
    /// Remove settings and clear the cache
    FactoryReset(OneshotSender<()>),
    /// Makes the daemon exit the main loop and quit.
    Shutdown,
}

pub struct ManagementInterfaceServer {
    server: talpid_ipc::IpcServer,
    subscriptions: Arc<RwLock<HashMap<SubscriptionId, pubsub::Sink<DaemonEvent>>>>,
}

impl ManagementInterfaceServer {
    pub fn start<T>(tunnel_tx: IntoSender<ManagementCommand, T>) -> Result<Self, talpid_ipc::Error>
    where
        T: From<ManagementCommand> + 'static + Send,
    {
        let rpc = ManagementInterface::new(tunnel_tx);
        let subscriptions = rpc.subscriptions.clone();

        let mut io = PubSubHandler::default();
        io.extend_with(rpc.to_delegate());
        let meta_io: MetaIoHandler<Meta> = io.into();
        let path = mullvad_paths::get_rpc_socket_path();
        let server = talpid_ipc::IpcServer::start_with_metadata(
            meta_io,
            meta_extractor,
            &path.to_string_lossy(),
        )?;
        Ok(ManagementInterfaceServer {
            server,
            subscriptions,
        })
    }

    pub fn socket_path(&self) -> &str {
        self.server.path()
    }

    pub fn event_broadcaster(&self) -> ManagementInterfaceEventBroadcaster {
        ManagementInterfaceEventBroadcaster {
            subscriptions: self.subscriptions.clone(),
            close_handle: Some(self.server.close_handle()),
        }
    }

    /// Consumes the server and waits for it to finish. Returns an error if the server exited
    /// due to an error.
    pub fn wait(self) {
        self.server.wait()
    }
}

/// A handle that allows broadcasting messages to all subscribers of the management interface.
#[derive(Clone)]
pub struct ManagementInterfaceEventBroadcaster {
    subscriptions: Arc<RwLock<HashMap<SubscriptionId, pubsub::Sink<DaemonEvent>>>>,
    close_handle: Option<talpid_ipc::CloseHandle>,
}

impl EventListener for ManagementInterfaceEventBroadcaster {
    /// Sends a new state update to all `new_state` subscribers of the management interface.
    fn notify_new_state(&self, new_state: TunnelState) {
        log::debug!("Broadcasting new state: {:?}", new_state);
        self.notify(DaemonEvent::TunnelState(new_state));
    }

    /// Sends settings to all `settings` subscribers of the management interface.
    fn notify_settings(&self, settings: Settings) {
        log::debug!("Broadcasting new settings");
        self.notify(DaemonEvent::Settings(settings));
    }

    /// Sends settings to all `settings` subscribers of the management interface.
    fn notify_relay_list(&self, relay_list: RelayList) {
        log::debug!("Broadcasting new relay list");
        self.notify(DaemonEvent::RelayList(relay_list));
    }

    fn notify_app_version(&self, app_version_info: version::AppVersionInfo) {
        log::debug!("Broadcasting new app version info");
        self.notify(DaemonEvent::AppVersionInfo(app_version_info));
    }

    fn notify_key_event(&self, key_event: mullvad_types::wireguard::KeygenEvent) {
        log::debug!("Broadcasting new wireguard key event");
        self.notify(DaemonEvent::WireguardKey(key_event));
    }
}

impl ManagementInterfaceEventBroadcaster {
    fn notify(&self, value: DaemonEvent) {
        let subscriptions = self.subscriptions.read();
        for sink in subscriptions.values() {
            let _ = sink.notify(Ok(value.clone())).wait();
        }
    }
}

impl Drop for ManagementInterfaceEventBroadcaster {
    fn drop(&mut self) {
        if let Some(close_handle) = self.close_handle.take() {
            close_handle.close();
        }
    }
}

struct ManagementInterface<T: From<ManagementCommand> + 'static + Send> {
    subscriptions: Arc<RwLock<HashMap<SubscriptionId, pubsub::Sink<DaemonEvent>>>>,
    tx: Mutex<IntoSender<ManagementCommand, T>>,
}

impl<T: From<ManagementCommand> + 'static + Send> ManagementInterface<T> {
    pub fn new(tx: IntoSender<ManagementCommand, T>) -> Self {
        ManagementInterface {
            subscriptions: Default::default(),
            tx: Mutex::new(tx),
        }
    }

    /// Sends a command to the daemon and maps the error to an RPC error.
    fn send_command_to_daemon(
        &self,
        command: ManagementCommand,
    ) -> impl Future<Item = (), Error = Error> {
        future::result(self.tx.lock().send(command)).map_err(|_| Error::internal_error())
    }

    /// Converts the given error to an error that can be given to the caller of the API.
    /// Will let any actual RPC error through as is, any other error is changed to an internal
    /// error.
    fn map_rpc_error(error: &mullvad_rpc::Error) -> Error {
        match error.kind() {
            mullvad_rpc::ErrorKind::JsonRpcError(ref rpc_error) => {
                // We have to manually copy the error since we have different
                // versions of the jsonrpc_core library at the moment.
                Error {
                    code: ErrorCode::from(rpc_error.code.code()),
                    message: rpc_error.message.clone(),
                    data: rpc_error.data.clone(),
                }
            }
            _ => Error::internal_error(),
        }
    }
}

impl<T: From<ManagementCommand> + 'static + Send> ManagementInterfaceApi
    for ManagementInterface<T>
{
    type Metadata = Meta;

    fn create_new_account(&self, _: Self::Metadata) -> BoxFuture<String, Error> {
        let (tx, rx) = sync::oneshot::channel();
        let future = self
            .send_command_to_daemon(ManagementCommand::CreateNewAccount(tx))
            .and_then(|_| rx.map_err(|_| Error::internal_error()))
            .and_then(|result| match result {
                Ok(account_token) => Ok(account_token),
                Err(e) => Err(Self::map_rpc_error(&e)),
            });

        Box::new(future)
    }

    fn get_account_data(
        &self,
        _: Self::Metadata,
        account_token: AccountToken,
    ) -> BoxFuture<AccountData, Error> {
        log::debug!("get_account_data");
        let (tx, rx) = sync::oneshot::channel();
        let future = self
            .send_command_to_daemon(ManagementCommand::GetAccountData(tx, account_token))
            .and_then(|_| rx.map_err(|_| Error::internal_error()))
            .and_then(|rpc_future| {
                rpc_future.map_err(|error: mullvad_rpc::Error| {
                    log::error!(
                        "Unable to get account data from API: {}",
                        error.display_chain()
                    );
                    Self::map_rpc_error(&error)
                })
            });
        Box::new(future)
    }

    fn get_www_auth_token(&self, _: Self::Metadata) -> BoxFuture<String, Error> {
        log::debug!("get_account_data");
        let (tx, rx) = sync::oneshot::channel();
        let future = self
            .send_command_to_daemon(ManagementCommand::GetWwwAuthToken(tx))
            .and_then(|_| rx.map_err(|_| Error::internal_error()))
            .and_then(|rpc_future| {
                rpc_future.map_err(|error: mullvad_rpc::Error| {
                    log::error!(
                        "Unable to get account data from API: {}",
                        error.display_chain()
                    );
                    Self::map_rpc_error(&error)
                })
            });
        Box::new(future)
    }

    fn get_relay_locations(&self, _: Self::Metadata) -> BoxFuture<RelayList, Error> {
        log::debug!("get_relay_locations");
        let (tx, rx) = sync::oneshot::channel();
        let future = self
            .send_command_to_daemon(ManagementCommand::GetRelayLocations(tx))
            .and_then(|_| rx.map_err(|_| Error::internal_error()));
        Box::new(future)
    }

    fn update_relay_locations(&self, _: Self::Metadata) -> BoxFuture<(), Error> {
        log::debug!("update_relay_locations");
        Box::new(self.send_command_to_daemon(ManagementCommand::UpdateRelayLocations))
    }

    fn set_account(
        &self,
        _: Self::Metadata,
        account_token: Option<AccountToken>,
    ) -> BoxFuture<(), Error> {
        log::debug!("set_account");
        let (tx, rx) = sync::oneshot::channel();
        let future = self
            .send_command_to_daemon(ManagementCommand::SetAccount(tx, account_token.clone()))
            .and_then(|_| rx.map_err(|_| Error::internal_error()));
        Box::new(future)
    }

    fn update_relay_settings(
        &self,
        _: Self::Metadata,
        constraints_update: RelaySettingsUpdate,
    ) -> BoxFuture<(), Error> {
        log::debug!("update_relay_settings");
        let (tx, rx) = sync::oneshot::channel();

        let message = ManagementCommand::UpdateRelaySettings(tx, constraints_update);
        let future = self
            .send_command_to_daemon(message)
            .and_then(|_| rx.map_err(|_| Error::internal_error()));
        Box::new(future)
    }

    fn set_allow_lan(&self, _: Self::Metadata, allow_lan: bool) -> BoxFuture<(), Error> {
        log::debug!("set_allow_lan({})", allow_lan);
        let (tx, rx) = sync::oneshot::channel();
        let future = self
            .send_command_to_daemon(ManagementCommand::SetAllowLan(tx, allow_lan))
            .and_then(|_| rx.map_err(|_| Error::internal_error()));
        Box::new(future)
    }

    fn set_block_when_disconnected(
        &self,
        _: Self::Metadata,
        block_when_disconnected: bool,
    ) -> BoxFuture<(), Error> {
        log::debug!("set_block_when_disconnected({})", block_when_disconnected);
        let (tx, rx) = sync::oneshot::channel();
        let future = self
            .send_command_to_daemon(ManagementCommand::SetBlockWhenDisconnected(
                tx,
                block_when_disconnected,
            ))
            .and_then(|_| rx.map_err(|_| Error::internal_error()));
        Box::new(future)
    }

    fn set_auto_connect(&self, _: Self::Metadata, auto_connect: bool) -> BoxFuture<(), Error> {
        log::debug!("set_auto_connect({})", auto_connect);
        let (tx, rx) = sync::oneshot::channel();
        let future = self
            .send_command_to_daemon(ManagementCommand::SetAutoConnect(tx, auto_connect))
            .and_then(|_| rx.map_err(|_| Error::internal_error()));
        Box::new(future)
    }

    fn connect(&self, _: Self::Metadata) -> BoxFuture<(), Error> {
        log::debug!("connect");
        let (tx, rx) = sync::oneshot::channel();
        let future = self
            .send_command_to_daemon(ManagementCommand::SetTargetState(tx, TargetState::Secured))
            .and_then(|_| rx.map_err(|_| Error::internal_error()))
            .and_then(|result| match result {
                Ok(()) => future::ok(()),
                Err(()) => future::err(Error {
                    code: ErrorCode::ServerError(-900),
                    message: "No account token configured".to_owned(),
                    data: None,
                }),
            });
        Box::new(future)
    }

    fn disconnect(&self, _: Self::Metadata) -> BoxFuture<(), Error> {
        log::debug!("disconnect");
        let (tx, _) = sync::oneshot::channel();
        let future = self
            .send_command_to_daemon(ManagementCommand::SetTargetState(
                tx,
                TargetState::Unsecured,
            ))
            .then(|_| future::ok(()));
        Box::new(future)
    }

    fn get_state(&self, _: Self::Metadata) -> BoxFuture<TunnelState, Error> {
        log::debug!("get_state");
        let (state_tx, state_rx) = sync::oneshot::channel();
        let future = self
            .send_command_to_daemon(ManagementCommand::GetState(state_tx))
            .and_then(|_| state_rx.map_err(|_| Error::internal_error()));
        Box::new(future)
    }

    fn get_current_location(&self, _: Self::Metadata) -> BoxFuture<Option<GeoIpLocation>, Error> {
        log::debug!("get_current_location");
        let (tx, rx) = sync::oneshot::channel();
        let future = self
            .send_command_to_daemon(ManagementCommand::GetCurrentLocation(tx))
            .and_then(|_| rx.map_err(|_| Error::internal_error()));
        Box::new(future)
    }

    fn shutdown(&self, _: Self::Metadata) -> BoxFuture<(), Error> {
        log::debug!("shutdown");
        Box::new(self.send_command_to_daemon(ManagementCommand::Shutdown))
    }

    fn get_account_history(&self, _: Self::Metadata) -> BoxFuture<Vec<AccountToken>, Error> {
        log::debug!("get_account_history");
        let (tx, rx) = sync::oneshot::channel();
        let future = self
            .send_command_to_daemon(ManagementCommand::GetAccountHistory(tx))
            .and_then(|_| rx.map_err(|_| Error::internal_error()));
        Box::new(future)
    }

    fn remove_account_from_history(
        &self,
        _: Self::Metadata,
        account_token: AccountToken,
    ) -> BoxFuture<(), Error> {
        log::debug!("remove_account_from_history");
        let (tx, rx) = sync::oneshot::channel();
        let future = self
            .send_command_to_daemon(ManagementCommand::RemoveAccountFromHistory(
                tx,
                account_token,
            ))
            .and_then(|_| rx.map_err(|_| Error::internal_error()));
        Box::new(future)
    }

    fn set_openvpn_mssfix(&self, _: Self::Metadata, mssfix: Option<u16>) -> BoxFuture<(), Error> {
        log::debug!("set_openvpn_mssfix({:?})", mssfix);
        let (tx, rx) = sync::oneshot::channel();
        let future = self
            .send_command_to_daemon(ManagementCommand::SetOpenVpnMssfix(tx, mssfix))
            .and_then(|_| rx.map_err(|_| Error::internal_error()));

        Box::new(future)
    }

    fn set_bridge_settings(
        &self,
        _: Self::Metadata,
        bridge_settings: BridgeSettings,
    ) -> BoxFuture<(), Error> {
        log::debug!("set_bridge_settings({:?})", bridge_settings);
        let (tx, rx) = sync::oneshot::channel();
        let future = self
            .send_command_to_daemon(ManagementCommand::SetBridgeSettings(tx, bridge_settings))
            .and_then(|_| rx.map_err(|_| Error::internal_error()))
            .and_then(|settings_result| {
                settings_result.map_err(|error| match error {
                    settings::Error::InvalidProxyData(reason) => {
                        Error::invalid_params(reason.to_owned())
                    }
                    _ => Error::internal_error(),
                })
            });

        Box::new(future)
    }

    fn set_bridge_state(
        &self,
        _: Self::Metadata,
        bridge_state: BridgeState,
    ) -> BoxFuture<(), Error> {
        log::debug!("set_bridge_state({:?})", bridge_state);
        let (tx, rx) = sync::oneshot::channel();
        let future = self
            .send_command_to_daemon(ManagementCommand::SetBridgeState(tx, bridge_state))
            .and_then(|_| rx.map_err(|_| Error::internal_error()))
            .and_then(|settings_result| settings_result.map_err(|_| Error::internal_error()));

        Box::new(future)
    }

    fn set_enable_ipv6(&self, _: Self::Metadata, enable_ipv6: bool) -> BoxFuture<(), Error> {
        log::debug!("set_enable_ipv6({})", enable_ipv6);
        let (tx, rx) = sync::oneshot::channel();
        let future = self
            .send_command_to_daemon(ManagementCommand::SetEnableIpv6(tx, enable_ipv6))
            .and_then(|_| rx.map_err(|_| Error::internal_error()));

        Box::new(future)
    }

    /// Set MTU for wireguard tunnels
    fn set_wireguard_mtu(&self, _: Self::Metadata, mtu: Option<u16>) -> BoxFuture<(), Error> {
        log::debug!("set_wireguard_mtu({:?})", mtu);
        let (tx, rx) = sync::oneshot::channel();
        let future = self
            .send_command_to_daemon(ManagementCommand::SetWireguardMtu(tx, mtu))
            .and_then(|_| rx.map_err(|_| Error::internal_error()));
        Box::new(future)
    }

    fn get_settings(&self, _: Self::Metadata) -> BoxFuture<Settings, Error> {
        log::debug!("get_settings");
        let (tx, rx) = sync::oneshot::channel();
        let future = self
            .send_command_to_daemon(ManagementCommand::GetSettings(tx))
            .and_then(|_| rx.map_err(|_| Error::internal_error()));
        Box::new(future)
    }

    fn generate_wireguard_key(
        &self,
        _: Self::Metadata,
    ) -> BoxFuture<mullvad_types::wireguard::KeygenEvent, Error> {
        log::debug!("generate_wireguard_key");
        let (tx, rx) = sync::oneshot::channel();
        let future = self
            .send_command_to_daemon(ManagementCommand::GenerateWireguardKey(tx))
            .and_then(|_| rx.map_err(|_| Error::internal_error()));
        Box::new(future)
    }

    fn get_wireguard_key(
        &self,
        _: Self::Metadata,
    ) -> BoxFuture<Option<wireguard::PublicKey>, Error> {
        log::debug!("get_wireguard_key");
        let (tx, rx) = sync::oneshot::channel();
        let future = self
            .send_command_to_daemon(ManagementCommand::GetWireguardKey(tx))
            .and_then(|_| rx.map_err(|_| Error::internal_error()));
        Box::new(future)
    }

    fn verify_wireguard_key(&self, _: Self::Metadata) -> BoxFuture<bool, Error> {
        log::debug!("verify_wireguard_key");
        let (tx, rx) = sync::oneshot::channel();
        let future = self
            .send_command_to_daemon(ManagementCommand::VerifyWireguardKey(tx))
            .and_then(|_| rx.map_err(|_| Error::internal_error()));
        Box::new(future)
    }

    fn get_current_version(&self, _: Self::Metadata) -> BoxFuture<String, Error> {
        log::debug!("get_current_version");
        let (tx, rx) = sync::oneshot::channel();
        let future = self
            .send_command_to_daemon(ManagementCommand::GetCurrentVersion(tx))
            .and_then(|_| rx.map_err(|_| Error::internal_error()));

        Box::new(future)
    }

    fn get_version_info(&self, _: Self::Metadata) -> BoxFuture<version::AppVersionInfo, Error> {
        log::debug!("get_version_info");
        let (tx, rx) = sync::oneshot::channel();
        let future = self
            .send_command_to_daemon(ManagementCommand::GetVersionInfo(tx))
            .and_then(|_| rx.map_err(|_| Error::internal_error()))
            .and_then(|version_future| {
                version_future.map_err(|error| {
                    log::error!(
                        "Unable to get version data from API: {}",
                        error.display_chain()
                    );
                    Self::map_rpc_error(&error)
                })
            });

        Box::new(future)
    }

    fn factory_reset(&self, _: Self::Metadata) -> BoxFuture<(), Error> {
        #[cfg(not(target_os = "android"))]
        {
            log::debug!("factory_reset");
            let (tx, rx) = sync::oneshot::channel();
            let future = self
                .send_command_to_daemon(ManagementCommand::FactoryReset(tx))
                .and_then(|_| rx.map_err(|_| Error::internal_error()));

            Box::new(future)
        }
        #[cfg(target_os = "android")]
        {
            Box::new(future::ok(()))
        }
    }


    fn daemon_event_subscribe(
        &self,
        _: Self::Metadata,
        subscriber: pubsub::Subscriber<DaemonEvent>,
    ) {
        log::debug!("daemon_event_subscribe");
        let mut subscriptions = self.subscriptions.write();
        loop {
            let id = SubscriptionId::String(uuid::Uuid::new_v4().to_string());
            if let Entry::Vacant(entry) = subscriptions.entry(id.clone()) {
                if let Ok(sink) = subscriber.assign_id(id.clone()) {
                    log::debug!("Accepting new subscription with id {:?}", id);
                    entry.insert(sink);
                }
                break;
            }
        }
    }

    fn daemon_event_unsubscribe(&self, id: SubscriptionId) -> BoxFuture<(), Error> {
        log::debug!("daemon_event_unsubscribe");
        let was_removed = self.subscriptions.write().remove(&id).is_some();
        let result = if was_removed {
            log::debug!("Unsubscribing id {:?}", id);
            future::ok(())
        } else {
            future::err(Error {
                code: ErrorCode::InvalidParams,
                message: "Invalid subscription".to_owned(),
                data: None,
            })
        };
        Box::new(result)
    }
}


/// The metadata type. There is one instance associated with each connection. In this pubsub
/// scenario they are created by `meta_extractor` by the server on each new incoming
/// connection.
#[derive(Clone, Debug, Default)]
pub struct Meta {
    session: Option<Arc<Session>>,
}

/// Make the `Meta` type possible to use as jsonrpc metadata type.
impl Metadata for Meta {}

/// Make the `Meta` type possible to use as a pubsub metadata type.
impl PubSubMetadata for Meta {
    fn session(&self) -> Option<Arc<Session>> {
        self.session.clone()
    }
}

/// Metadata extractor function for `Meta`.
fn meta_extractor(context: &jsonrpc_ipc_server::RequestContext<'_>) -> Meta {
    Meta {
        session: Some(Arc::new(Session::new(context.sender.clone()))),
    }
}

use error_chain;

use error_chain::ChainedError;
use jsonrpc_core::futures::sync::oneshot::Sender as OneshotSender;
use jsonrpc_core::futures::{future, sync, Future};
use jsonrpc_core::{Error, ErrorCode, MetaIoHandler, Metadata};
use jsonrpc_ipc_server;
use jsonrpc_macros::pubsub;
use jsonrpc_pubsub::{PubSubHandler, PubSubMetadata, Session, SubscriptionId};
use mullvad_rpc;
use mullvad_types::account::{AccountData, AccountToken};
use mullvad_types::location::GeoIpLocation;

use mullvad_paths;
use mullvad_types::relay_constraints::{RelaySettings, RelaySettingsUpdate};
use mullvad_types::relay_list::RelayList;
use mullvad_types::states::TargetState;
use mullvad_types::version;

use serde;

use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex, RwLock};

use talpid_core::mpsc::IntoSender;
use talpid_ipc;
use talpid_types::net::TunnelOptions;
use talpid_types::tunnel::TunnelStateTransition;
use uuid;

use account_history::{AccountHistory, Error as AccountHistoryError};

/// FIXME(linus): This is here just because the futures crate has deprecated it and jsonrpc_core
/// did not introduce their own yet (https://github.com/paritytech/jsonrpc/pull/196).
/// Remove this and use the one in jsonrpc_core when that is released.
pub type BoxFuture<T, E> = Box<Future<Item = T, Error = E> + Send>;

build_rpc_trait! {
    pub trait ManagementInterfaceApi {
        type Metadata;


        /// Fetches and returns metadata about an account. Returns an error on non-existing
        /// accounts.
        #[rpc(meta, name = "get_account_data")]
        fn get_account_data(&self, Self::Metadata, AccountToken) -> BoxFuture<AccountData, Error>;

        /// Returns available countries.
        #[rpc(meta, name = "get_relay_locations")]
        fn get_relay_locations(&self, Self::Metadata) -> BoxFuture<RelayList, Error>;

        /// Set which account to connect with.
        #[rpc(meta, name = "set_account")]
        fn set_account(&self, Self::Metadata, Option<AccountToken>) -> BoxFuture<(), Error>;

        /// Get which account is configured.
        #[rpc(meta, name = "get_account")]
        fn get_account(&self, Self::Metadata) -> BoxFuture<Option<AccountToken>, Error>;

        /// Update constraints put on the type of tunnel connection to use
        #[rpc(meta, name = "update_relay_settings")]
        fn update_relay_settings(
            &self,
            Self::Metadata, RelaySettingsUpdate
            ) -> BoxFuture<(), Error>;

        /// Update constraints put on the type of tunnel connection to use
        #[rpc(meta, name = "get_relay_settings")]
        fn get_relay_settings(
            &self,
            Self::Metadata
            ) -> BoxFuture<RelaySettings, Error>;

        /// Set if the client should allow communication with the LAN while in secured state.
        #[rpc(meta, name = "set_allow_lan")]
        fn set_allow_lan(&self, Self::Metadata, bool) -> BoxFuture<(), Error>;

        /// Get if the client should allow communication with the LAN while in secured state.
        #[rpc(meta, name = "get_allow_lan")]
        fn get_allow_lan(&self, Self::Metadata) -> BoxFuture<bool, Error>;

        /// Set if the daemon should automatically establish a tunnel on start or not.
        #[rpc(meta, name = "set_auto_connect")]
        fn set_auto_connect(&self, Self::Metadata, bool) -> BoxFuture<(), Error>;

        /// Get if the daemon should automatically establish a tunnel on start or not.
        #[rpc(meta, name = "get_auto_connect")]
        fn get_auto_connect(&self, Self::Metadata) -> BoxFuture<bool, Error>;

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
        fn get_state(&self, Self::Metadata) -> BoxFuture<TunnelStateTransition, Error>;

        /// Performs a geoIP lookup and returns the current location as perceived by the public
        /// internet.
        #[rpc(meta, name = "get_current_location")]
        fn get_current_location(&self, Self::Metadata) -> BoxFuture<GeoIpLocation, Error>;

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

        /// Set if IPv6 is enabled in the tunnel
        #[rpc(meta, name = "set_openvpn_enable_ipv6")]
        fn set_openvpn_enable_ipv6(&self, Self::Metadata, bool) -> BoxFuture<(), Error>;

        /// Gets tunnel specific options
        #[rpc(meta, name = "get_tunnel_options")]
        fn get_tunnel_options(&self, Self::Metadata) -> BoxFuture<TunnelOptions, Error>;

        /// Retreive version of the app
        #[rpc(meta, name = "get_current_version")]
        fn get_current_version(&self, Self::Metadata) -> BoxFuture<String, Error>;

        /// Retrieve information about the currently running and latest versions of the app
        #[rpc(meta, name = "get_version_info")]
        fn get_version_info(&self, Self::Metadata) -> BoxFuture<version::AppVersionInfo, Error>;

        #[pubsub(name = "new_state")] {
            /// Subscribes to the `new_state` event notifications.
            #[rpc(name = "new_state_subscribe")]
            fn new_state_subscribe(
                &self,
                Self::Metadata,
                pubsub::Subscriber<TunnelStateTransition>
            );

            /// Unsubscribes from the `new_state` event notifications.
            #[rpc(name = "new_state_unsubscribe")]
            fn new_state_unsubscribe(&self, SubscriptionId) -> BoxFuture<(), Error>;
        }

        #[pubsub(name = "error")] {
            /// Subscribes to the `error` event notifications.
            #[rpc(name = "error_subscribe")]
            fn error_subscribe(&self, Self::Metadata, pubsub::Subscriber<Vec<String>>);

            /// Unsubscribes from the `error` event notifications.
            #[rpc(name = "error_unsubscribe")]
            fn error_unsubscribe(&self, SubscriptionId) -> BoxFuture<(), Error>;
        }
    }
}


/// Enum representing commands coming in on the management interface.
pub enum ManagementCommand {
    /// Change target state.
    SetTargetState(TargetState),
    /// Request the current state.
    GetState(OneshotSender<TunnelStateTransition>),
    /// Get the current geographical location.
    GetCurrentLocation(OneshotSender<GeoIpLocation>),
    /// Request the metadata for an account.
    GetAccountData(
        OneshotSender<BoxFuture<AccountData, mullvad_rpc::Error>>,
        AccountToken,
    ),
    /// Get the list of countries and cities where there are relays.
    GetRelayLocations(OneshotSender<RelayList>),
    /// Set which account token to use for subsequent connection attempts.
    SetAccount(OneshotSender<()>, Option<AccountToken>),
    /// Request the current account token being used.
    GetAccount(OneshotSender<Option<AccountToken>>),
    /// Place constraints on the type of tunnel and relay
    UpdateRelaySettings(OneshotSender<()>, RelaySettingsUpdate),
    /// Read the constraints put on the tunnel and relay
    GetRelaySettings(OneshotSender<RelaySettings>),
    /// Set the allow LAN setting.
    SetAllowLan(OneshotSender<()>, bool),
    /// Get the current allow LAN setting.
    GetAllowLan(OneshotSender<bool>),
    /// Set the auto-connect setting.
    SetAutoConnect(OneshotSender<()>, bool),
    /// Get the current auto-connect setting.
    GetAutoConnect(OneshotSender<bool>),
    /// Set the mssfix argument for OpenVPN
    SetOpenVpnMssfix(OneshotSender<()>, Option<u16>),
    /// Set if IPv6 should be enabled in the tunnel
    SetOpenVpnEnableIpv6(OneshotSender<()>, bool),
    /// Get the tunnel options
    GetTunnelOptions(OneshotSender<TunnelOptions>),
    /// Get information about the currently running and latest app versions
    GetVersionInfo(OneshotSender<BoxFuture<version::AppVersionInfo, mullvad_rpc::Error>>),
    /// Get current version of the app
    GetCurrentVersion(OneshotSender<version::AppVersion>),
    /// Makes the daemon exit the main loop and quit.
    Shutdown,
}

#[derive(Default)]
struct ActiveSubscriptions {
    new_state_subscriptions: RwLock<HashMap<SubscriptionId, pubsub::Sink<TunnelStateTransition>>>,
    error_subscriptions: RwLock<HashMap<SubscriptionId, pubsub::Sink<Vec<String>>>>,
}

pub struct ManagementInterfaceServer {
    server: talpid_ipc::IpcServer,
    subscriptions: Arc<ActiveSubscriptions>,
}

impl ManagementInterfaceServer {
    pub fn start<T>(
        tunnel_tx: IntoSender<ManagementCommand, T>,
        cache_dir: PathBuf,
    ) -> talpid_ipc::Result<Self>
    where
        T: From<ManagementCommand> + 'static + Send,
    {
        let rpc = ManagementInterface::new(tunnel_tx, cache_dir);
        let subscriptions = rpc.subscriptions.clone();

        let mut io = PubSubHandler::default();
        io.extend_with(rpc.to_delegate());
        let meta_io: MetaIoHandler<Meta> = io.into();
        let path = mullvad_paths::get_rpc_socket_path();
        let server = talpid_ipc::IpcServer::start_with_metadata(
            meta_io,
            meta_extractor,
            path.to_string_lossy().to_string(),
        )?;
        Ok(ManagementInterfaceServer {
            server,
            subscriptions,
        })
    }

    pub fn socket_path(&self) -> &str {
        self.server.path()
    }

    pub fn event_broadcaster(&self) -> EventBroadcaster {
        EventBroadcaster {
            subscriptions: self.subscriptions.clone(),
        }
    }

    /// Consumes the server and waits for it to finish. Returns an error if the server exited
    /// due to an error.
    pub fn wait(self) {
        self.server.wait()
    }
}

/// A handle that allows broadcasting messages to all subscribers of the management interface.
pub struct EventBroadcaster {
    subscriptions: Arc<ActiveSubscriptions>,
}

impl EventBroadcaster {
    /// Sends a new state update to all `new_state` subscribers of the management interface.
    pub fn notify_new_state(&self, new_state: TunnelStateTransition) {
        debug!("Broadcasting new state to listeners: {:?}", new_state);
        self.notify(&self.subscriptions.new_state_subscriptions, new_state);
    }

    /// Sends an error to all `error` subscribers of the management interface.
    pub fn notify_error<E>(&self, error: &E)
    where
        E: error_chain::ChainedError,
    {
        let error_strings = error.iter().map(|e| e.to_string()).collect();
        self.notify(&self.subscriptions.error_subscriptions, error_strings);
    }

    fn notify<T>(
        &self,
        subscriptions_lock: &RwLock<HashMap<SubscriptionId, pubsub::Sink<T>>>,
        value: T,
    ) where
        T: serde::Serialize + Clone,
    {
        let subscriptions = subscriptions_lock.read().unwrap();
        for sink in subscriptions.values() {
            let _ = sink.notify(Ok(value.clone())).wait();
        }
    }
}

struct ManagementInterface<T: From<ManagementCommand> + 'static + Send> {
    subscriptions: Arc<ActiveSubscriptions>,
    tx: Mutex<IntoSender<ManagementCommand, T>>,
    cache_dir: PathBuf,
}

impl<T: From<ManagementCommand> + 'static + Send> ManagementInterface<T> {
    pub fn new(tx: IntoSender<ManagementCommand, T>, cache_dir: PathBuf) -> Self {
        ManagementInterface {
            subscriptions: Default::default(),
            tx: Mutex::new(tx),
            cache_dir,
        }
    }

    fn subscribe<V>(
        subscriber: pubsub::Subscriber<V>,
        subscriptions_lock: &RwLock<HashMap<SubscriptionId, pubsub::Sink<V>>>,
    ) {
        let mut subscriptions = subscriptions_lock.write().unwrap();
        loop {
            let id = SubscriptionId::String(uuid::Uuid::new_v4().to_string());
            if let Entry::Vacant(entry) = subscriptions.entry(id.clone()) {
                if let Ok(sink) = subscriber.assign_id(id.clone()) {
                    debug!("Accepting new subscription with id {:?}", id);
                    entry.insert(sink);
                }
                break;
            }
        }
    }

    fn unsubscribe<V>(
        id: SubscriptionId,
        subscriptions_lock: &RwLock<HashMap<SubscriptionId, pubsub::Sink<V>>>,
    ) -> BoxFuture<(), Error> {
        let was_removed = subscriptions_lock.write().unwrap().remove(&id).is_some();
        let result = if was_removed {
            debug!("Unsubscribing id {:?}", id);
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

    /// Sends a command to the daemon and maps the error to an RPC error.
    fn send_command_to_daemon(&self, command: ManagementCommand) -> BoxFuture<(), Error> {
        Box::new(
            future::result(self.tx.lock().unwrap().send(command))
                .map_err(|_| Error::internal_error()),
        )
    }

    /// Converts the given error to an error that can be given to the caller of the API.
    /// Will let any actual RPC error through as is, any other error is changed to an internal
    /// error.
    fn map_rpc_error(error: mullvad_rpc::Error) -> Error {
        match error.kind() {
            &mullvad_rpc::ErrorKind::JsonRpcError(ref rpc_error) => {
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

    fn load_history(&self) -> Result<AccountHistory, AccountHistoryError> {
        let mut account_history = AccountHistory::new(&self.cache_dir);
        account_history.load()?;
        Ok(account_history)
    }
}

impl<T: From<ManagementCommand> + 'static + Send> ManagementInterfaceApi
    for ManagementInterface<T>
{
    type Metadata = Meta;

    fn get_account_data(
        &self,
        _: Self::Metadata,
        account_token: AccountToken,
    ) -> BoxFuture<AccountData, Error> {
        trace!("get_account_data");
        let (tx, rx) = sync::oneshot::channel();
        let future = self
            .send_command_to_daemon(ManagementCommand::GetAccountData(tx, account_token))
            .and_then(|_| rx.map_err(|_| Error::internal_error()))
            .and_then(|rpc_future| {
                rpc_future.map_err(|error: mullvad_rpc::Error| {
                    error!(
                        "Unable to get account data from API: {}",
                        error.display_chain()
                    );
                    Self::map_rpc_error(error)
                })
            });
        Box::new(future)
    }

    fn get_relay_locations(&self, _: Self::Metadata) -> BoxFuture<RelayList, Error> {
        trace!("get_relay_locations");
        let (tx, rx) = sync::oneshot::channel();
        let future = self
            .send_command_to_daemon(ManagementCommand::GetRelayLocations(tx))
            .and_then(|_| rx.map_err(|_| Error::internal_error()));
        Box::new(future)
    }

    fn set_account(
        &self,
        _: Self::Metadata,
        account_token: Option<AccountToken>,
    ) -> BoxFuture<(), Error> {
        trace!("set_account");
        let (tx, rx) = sync::oneshot::channel();
        let future = self
            .send_command_to_daemon(ManagementCommand::SetAccount(tx, account_token.clone()))
            .and_then(|_| rx.map_err(|_| Error::internal_error()));

        if let Some(new_account_token) = account_token {
            if let Err(e) = self.load_history().and_then(|mut account_history| {
                account_history.add_account_token(new_account_token)
            }) {
                error!(
                    "Unable to add an account into the account history: {}",
                    e.display_chain()
                );
            }
        }

        Box::new(future)
    }

    fn get_account(&self, _: Self::Metadata) -> BoxFuture<Option<AccountToken>, Error> {
        trace!("get_account");
        let (tx, rx) = sync::oneshot::channel();
        let future = self
            .send_command_to_daemon(ManagementCommand::GetAccount(tx))
            .and_then(|_| rx.map_err(|_| Error::internal_error()));
        Box::new(future)
    }

    fn update_relay_settings(
        &self,
        _: Self::Metadata,
        constraints_update: RelaySettingsUpdate,
    ) -> BoxFuture<(), Error> {
        trace!("update_relay_settings");
        let (tx, rx) = sync::oneshot::channel();

        let message = ManagementCommand::UpdateRelaySettings(tx, constraints_update);
        let future = self
            .send_command_to_daemon(message)
            .and_then(|_| rx.map_err(|_| Error::internal_error()));
        Box::new(future)
    }

    fn get_relay_settings(&self, _: Self::Metadata) -> BoxFuture<RelaySettings, Error> {
        trace!("get_relay_settings");
        let (tx, rx) = sync::oneshot::channel();
        let future = self
            .send_command_to_daemon(ManagementCommand::GetRelaySettings(tx))
            .and_then(|_| rx.map_err(|_| Error::internal_error()));
        Box::new(future)
    }

    fn set_allow_lan(&self, _: Self::Metadata, allow_lan: bool) -> BoxFuture<(), Error> {
        trace!("set_allow_lan");
        let (tx, rx) = sync::oneshot::channel();
        let future = self
            .send_command_to_daemon(ManagementCommand::SetAllowLan(tx, allow_lan))
            .and_then(|_| rx.map_err(|_| Error::internal_error()));
        Box::new(future)
    }

    fn get_allow_lan(&self, _: Self::Metadata) -> BoxFuture<bool, Error> {
        trace!("get_allow_lan");
        let (tx, rx) = sync::oneshot::channel();
        let future = self
            .send_command_to_daemon(ManagementCommand::GetAllowLan(tx))
            .and_then(|_| rx.map_err(|_| Error::internal_error()));
        Box::new(future)
    }

    fn set_auto_connect(&self, _: Self::Metadata, auto_connect: bool) -> BoxFuture<(), Error> {
        trace!("set_auto_connect");
        let (tx, rx) = sync::oneshot::channel();
        let future = self
            .send_command_to_daemon(ManagementCommand::SetAutoConnect(tx, auto_connect))
            .and_then(|_| rx.map_err(|_| Error::internal_error()));
        Box::new(future)
    }

    fn get_auto_connect(&self, _: Self::Metadata) -> BoxFuture<bool, Error> {
        trace!("get_auto_connect");
        let (tx, rx) = sync::oneshot::channel();
        let future = self
            .send_command_to_daemon(ManagementCommand::GetAutoConnect(tx))
            .and_then(|_| rx.map_err(|_| Error::internal_error()));
        Box::new(future)
    }

    fn connect(&self, _: Self::Metadata) -> BoxFuture<(), Error> {
        trace!("connect");
        self.send_command_to_daemon(ManagementCommand::SetTargetState(TargetState::Secured))
    }

    fn disconnect(&self, _: Self::Metadata) -> BoxFuture<(), Error> {
        trace!("disconnect");
        self.send_command_to_daemon(ManagementCommand::SetTargetState(TargetState::Unsecured))
    }

    fn get_state(&self, _: Self::Metadata) -> BoxFuture<TunnelStateTransition, Error> {
        trace!("get_state");
        let (state_tx, state_rx) = sync::oneshot::channel();
        let future = self
            .send_command_to_daemon(ManagementCommand::GetState(state_tx))
            .and_then(|_| state_rx.map_err(|_| Error::internal_error()));
        Box::new(future)
    }

    fn get_current_location(&self, _: Self::Metadata) -> BoxFuture<GeoIpLocation, Error> {
        trace!("get_current_location");
        let (tx, rx) = sync::oneshot::channel();
        let future = self
            .send_command_to_daemon(ManagementCommand::GetCurrentLocation(tx))
            .and_then(|_| rx.map_err(|_| Error::internal_error()));
        Box::new(future)
    }

    fn shutdown(&self, _: Self::Metadata) -> BoxFuture<(), Error> {
        trace!("shutdown");
        self.send_command_to_daemon(ManagementCommand::Shutdown)
    }

    fn get_account_history(&self, _: Self::Metadata) -> BoxFuture<Vec<AccountToken>, Error> {
        trace!("get_account_history");
        Box::new(future::result(
            self.load_history()
                .map(|history| history.get_accounts().to_vec())
                .map_err(|error| {
                    error!("Unable to get account history: {}", error.display_chain());
                    Error::internal_error()
                }),
        ))
    }

    fn remove_account_from_history(
        &self,
        _: Self::Metadata,
        account_token: AccountToken,
    ) -> BoxFuture<(), Error> {
        trace!("remove_account_from_history");
        Box::new(future::result(
            self.load_history()
                .and_then(|mut history| history.remove_account_token(account_token))
                .map_err(|error| {
                    error!(
                        "Unable to remove account from history: {}",
                        error.display_chain()
                    );
                    Error::internal_error()
                }),
        ))
    }

    fn set_openvpn_mssfix(&self, _: Self::Metadata, mssfix: Option<u16>) -> BoxFuture<(), Error> {
        trace!("set_openvpn_mssfix");
        let (tx, rx) = sync::oneshot::channel();
        let future = self
            .send_command_to_daemon(ManagementCommand::SetOpenVpnMssfix(tx, mssfix))
            .and_then(|_| rx.map_err(|_| Error::internal_error()));

        Box::new(future)
    }

    fn set_openvpn_enable_ipv6(
        &self,
        _: Self::Metadata,
        enable_ipv6: bool,
    ) -> BoxFuture<(), Error> {
        trace!("set_openvpn_enable_ipv6");
        let (tx, rx) = sync::oneshot::channel();
        let future = self
            .send_command_to_daemon(ManagementCommand::SetOpenVpnEnableIpv6(tx, enable_ipv6))
            .and_then(|_| rx.map_err(|_| Error::internal_error()));

        Box::new(future)
    }

    fn get_tunnel_options(&self, _: Self::Metadata) -> BoxFuture<TunnelOptions, Error> {
        trace!("get_tunnel_options");
        let (tx, rx) = sync::oneshot::channel();
        let future = self
            .send_command_to_daemon(ManagementCommand::GetTunnelOptions(tx))
            .and_then(|_| rx.map_err(|_| Error::internal_error()));
        Box::new(future)
    }

    fn get_current_version(&self, _: Self::Metadata) -> BoxFuture<String, Error> {
        let (tx, rx) = sync::oneshot::channel();
        let future = self
            .send_command_to_daemon(ManagementCommand::GetCurrentVersion(tx))
            .and_then(|_| rx.map_err(|_| Error::internal_error()));

        Box::new(future)
    }

    fn get_version_info(&self, _: Self::Metadata) -> BoxFuture<version::AppVersionInfo, Error> {
        let (tx, rx) = sync::oneshot::channel();
        let future = self
            .send_command_to_daemon(ManagementCommand::GetVersionInfo(tx))
            .and_then(|_| rx.map_err(|_| Error::internal_error()))
            .and_then(|version_future| {
                version_future.map_err(|error| {
                    error!(
                        "Unable to get version data from API: {}",
                        error.display_chain()
                    );
                    Self::map_rpc_error(error)
                })
            });

        Box::new(future)
    }

    fn new_state_subscribe(
        &self,
        _: Self::Metadata,
        subscriber: pubsub::Subscriber<TunnelStateTransition>,
    ) {
        trace!("new_state_subscribe");
        Self::subscribe(subscriber, &self.subscriptions.new_state_subscriptions);
    }

    fn new_state_unsubscribe(&self, id: SubscriptionId) -> BoxFuture<(), Error> {
        trace!("new_state_unsubscribe");
        Self::unsubscribe(id, &self.subscriptions.new_state_subscriptions)
    }

    fn error_subscribe(&self, _: Self::Metadata, subscriber: pubsub::Subscriber<Vec<String>>) {
        trace!("error_subscribe");
        Self::subscribe(subscriber, &self.subscriptions.error_subscriptions);
    }

    fn error_unsubscribe(&self, id: SubscriptionId) -> BoxFuture<(), Error> {
        trace!("error_unsubscribe");
        Self::unsubscribe(id, &self.subscriptions.error_subscriptions)
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
fn meta_extractor(context: &jsonrpc_ipc_server::RequestContext) -> Meta {
    Meta {
        session: Some(Arc::new(Session::new(context.sender.clone()))),
    }
}

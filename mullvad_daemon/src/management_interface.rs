use error_chain;

use jsonrpc_core::{Error, ErrorCode, Metadata};
use jsonrpc_core::futures::{BoxFuture, Future, future, sync};
use jsonrpc_macros::pubsub;
use jsonrpc_pubsub::{PubSubHandler, PubSubMetadata, Session, SubscriptionId};
use jsonrpc_ws_server;

use serde;
use states::{SecurityState, TargetState};

use std::collections::HashMap;
use std::collections::hash_map::Entry;
use std::net::{IpAddr, Ipv4Addr};
use std::sync::{Arc, Mutex, RwLock};

use talpid_core::mpsc::IntoSender;
use talpid_ipc;
use uuid;


pub type AccountToken = String;
pub type CountryCode = String;

#[derive(Serialize)]
pub struct AccountData {
    pub paid_until: String,
}

#[derive(Serialize)]
pub struct Location {
    pub latlong: [f64; 2],
    pub country: String,
    pub city: String,
}


build_rpc_trait! {
    pub trait ManagementInterfaceApi {
        type Metadata;

        /// Fetches and returns metadata about an account. Returns an error on non-existing
        /// accounts.
        #[rpc(name = "get_account_data")]
        fn get_account_data(&self, AccountToken) -> Result<AccountData, Error>;

        /// Returns available countries.
        #[rpc(name = "get_countries")]
        fn get_countries(&self) -> Result<HashMap<CountryCode, String>, Error>;

        /// Set which account to connect with
        #[rpc(name = "set_account")]
        fn set_account(&self, AccountToken) -> Result<(), Error>;

        /// Set which country to connect to
        #[rpc(name = "set_country")]
        fn set_country(&self, CountryCode) -> Result<(), Error>;

        /// Set if the client should automatically establish a tunnel on start or not.
        #[rpc(name = "set_autoconnect")]
        fn set_autoconnect(&self, bool) -> Result<(), Error>;

        /// Try to connect if disconnected, or do nothing if already connecting/connected.
        #[rpc(name = "connect")]
        fn connect(&self) -> Result<(), Error>;

        /// Disconnect the VPN tunnel if it is connecting/connected. Does nothing if already
        /// disconnected.
        #[rpc(name = "disconnect")]
        fn disconnect(&self) -> Result<(), Error>;

        /// Returns the current security state of the Mullvad client. Changes to this state will
        /// be announced to subscribers of `event`.
        #[rpc(async, name = "get_state")]
        fn get_state(&self) -> BoxFuture<SecurityState, Error>;

        /// Returns the current public IP of this computer.
        #[rpc(name = "get_ip")]
        fn get_ip(&self) -> Result<IpAddr, Error>;

        /// Performs a geoIP lookup and returns the current location as perceived by the public
        /// internet.
        #[rpc(name = "get_location")]
        fn get_location(&self) -> Result<Location, Error>;

        #[pubsub(name = "new_state")] {
            /// Subscribes to the `new_state` event notifications.
            #[rpc(name = "new_state_subscribe")]
            fn new_state_subscribe(&self, Self::Metadata, pubsub::Subscriber<SecurityState>);

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
#[derive(Debug)]
pub enum TunnelCommand {
    /// Change target state.
    SetTargetState(TargetState),
    /// Request the current state.
    GetState(sync::oneshot::Sender<SecurityState>),
    /// Set which account token to use for subsequent connection attempts.
    SetAccount(AccountToken),
}

#[derive(Default)]
struct ActiveSubscriptions {
    new_state_subscriptions: RwLock<HashMap<SubscriptionId, pubsub::Sink<SecurityState>>>,
    error_subscriptions: RwLock<HashMap<SubscriptionId, pubsub::Sink<Vec<String>>>>,
}

pub struct ManagementInterfaceServer {
    server: talpid_ipc::IpcServer,
    subscriptions: Arc<ActiveSubscriptions>,
}

impl ManagementInterfaceServer {
    pub fn start<T>(tunnel_tx: IntoSender<TunnelCommand, T>) -> talpid_ipc::Result<Self>
        where T: From<TunnelCommand> + 'static + Send
    {
        let rpc = ManagementInterface::new(tunnel_tx);
        let subscriptions = rpc.subscriptions.clone();

        let mut io = PubSubHandler::default();
        io.extend_with(rpc.to_delegate());
        let server = talpid_ipc::IpcServer::start_with_metadata(io.into(), meta_extractor)?;
        Ok(
            ManagementInterfaceServer {
                server,
                subscriptions,
            },
        )
    }

    pub fn address(&self) -> &str {
        self.server.address()
    }

    pub fn event_broadcaster(&self) -> EventBroadcaster {
        EventBroadcaster { subscriptions: self.subscriptions.clone() }
    }

    /// Consumes the server and waits for it to finish. Returns an error if the server exited
    /// due to an error.
    pub fn wait(self) -> talpid_ipc::Result<()> {
        self.server.wait()
    }
}


/// A handle that allows broadcasting messages to all subscribers of the management interface.
pub struct EventBroadcaster {
    subscriptions: Arc<ActiveSubscriptions>,
}

impl EventBroadcaster {
    /// Sends a new state update to all `new_state` subscribers of the management interface.
    pub fn notify_new_state(&self, new_state: SecurityState) {
        self.notify(&self.subscriptions.new_state_subscriptions, new_state);
    }

    /// Sends an error to all `error` subscribers of the management interface.
    pub fn notify_error<E>(&self, error: &E)
        where E: error_chain::ChainedError
    {
        let error_strings = error.iter().map(|e| e.to_string()).collect();
        self.notify(&self.subscriptions.error_subscriptions, error_strings);
    }

    fn notify<T>(&self,
                 subscriptions_lock: &RwLock<HashMap<SubscriptionId, pubsub::Sink<T>>>,
                 value: T)
        where T: serde::Serialize + Clone
    {
        let subscriptions = subscriptions_lock.read().unwrap();
        for sink in subscriptions.values() {
            let _ = sink.notify(Ok(value.clone())).wait();
        }
    }
}

struct ManagementInterface<T: From<TunnelCommand> + 'static + Send> {
    subscriptions: Arc<ActiveSubscriptions>,
    tx: Mutex<IntoSender<TunnelCommand, T>>,
}

impl<T: From<TunnelCommand> + 'static + Send> ManagementInterface<T> {
    pub fn new(tx: IntoSender<TunnelCommand, T>) -> Self {
        ManagementInterface {
            subscriptions: Default::default(),
            tx: Mutex::new(tx),
        }
    }

    fn subscribe<V>(subscriber: pubsub::Subscriber<V>,
                    subscriptions_lock: &RwLock<HashMap<SubscriptionId, pubsub::Sink<V>>>) {
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

    fn unsubscribe<V>(id: SubscriptionId,
                      subscriptions_lock: &RwLock<HashMap<SubscriptionId, pubsub::Sink<V>>>)
                      -> BoxFuture<(), Error> {
        let was_removed = subscriptions_lock.write().unwrap().remove(&id).is_some();
        let result = if was_removed {
            debug!("Unsubscribing id {:?}", id);
            future::ok(())
        } else {
            future::err(
                Error {
                    code: ErrorCode::InvalidParams,
                    message: "Invalid subscription".to_owned(),
                    data: None,
                },
            )
        };
        result.boxed()
    }
}

impl<T: From<TunnelCommand> + 'static + Send> ManagementInterfaceApi for ManagementInterface<T> {
    type Metadata = Meta;

    fn get_account_data(&self, _account_token: AccountToken) -> Result<AccountData, Error> {
        trace!("get_account_data");
        Ok(AccountData { paid_until: "2018-12-31T16:00:00.000Z".to_owned() },)
    }

    fn get_countries(&self) -> Result<HashMap<CountryCode, String>, Error> {
        trace!("get_countries");
        Ok(HashMap::new())
    }

    fn set_account(&self, account_token: AccountToken) -> Result<(), Error> {
        trace!("set_account");
        self.tx
            .lock()
            .unwrap()
            .send(TunnelCommand::SetAccount(account_token))
            .map_err(|_| Error::internal_error())
    }

    fn set_country(&self, _country_code: CountryCode) -> Result<(), Error> {
        trace!("set_country");
        Ok(())
    }

    fn set_autoconnect(&self, _autoconnect: bool) -> Result<(), Error> {
        trace!("set_autoconnect");
        Ok(())
    }

    fn connect(&self) -> Result<(), Error> {
        trace!("connect");
        self.tx
            .lock()
            .unwrap()
            .send(TunnelCommand::SetTargetState(TargetState::Secured))
            .map_err(|_| Error::internal_error())
    }

    fn disconnect(&self) -> Result<(), Error> {
        trace!("disconnect");
        self.tx
            .lock()
            .unwrap()
            .send(TunnelCommand::SetTargetState(TargetState::Unsecured))
            .map_err(|_| Error::internal_error())
    }

    fn get_state(&self) -> BoxFuture<SecurityState, Error> {
        trace!("get_state");
        let (state_tx, state_rx) = sync::oneshot::channel();
        match self.tx.lock().unwrap().send(TunnelCommand::GetState(state_tx)) {
            Ok(()) => state_rx.map_err(|_| Error::internal_error()).boxed(),
            Err(_) => future::err(Error::internal_error()).boxed(),
        }
    }

    fn get_ip(&self) -> Result<IpAddr, Error> {
        trace!("get_ip");
        Ok(IpAddr::V4(Ipv4Addr::new(1, 2, 3, 4)))
    }

    fn get_location(&self) -> Result<Location, Error> {
        trace!("get_location");
        Ok(
            Location {
                latlong: [1.0, 2.0],
                country: "narnia".to_owned(),
                city: "Le city".to_owned(),
            },
        )
    }

    fn new_state_subscribe(&self,
                           _meta: Self::Metadata,
                           subscriber: pubsub::Subscriber<SecurityState>) {
        trace!("new_state_subscribe");
        Self::subscribe(subscriber, &self.subscriptions.new_state_subscriptions);
    }

    fn new_state_unsubscribe(&self, id: SubscriptionId) -> BoxFuture<(), Error> {
        trace!("new_state_unsubscribe");
        Self::unsubscribe(id, &self.subscriptions.new_state_subscriptions)
    }

    fn error_subscribe(&self, _meta: Self::Metadata, subscriber: pubsub::Subscriber<Vec<String>>) {
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
fn meta_extractor(context: &jsonrpc_ws_server::RequestContext) -> Meta {
    Meta { session: Some(Arc::new(Session::new(context.sender()))) }
}

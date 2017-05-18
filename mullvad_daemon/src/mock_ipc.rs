use ipc_api::*;

use jsonrpc_core::{self, Error, ErrorCode, Metadata};
use jsonrpc_core::futures::{BoxFuture, Future, future};
use jsonrpc_macros::pubsub;
use jsonrpc_pubsub::{PubSubHandler, PubSubMetadata, Session, SubscriptionId};
use jsonrpc_ws_server;

use std::collections::HashMap;
use std::net::{IpAddr, Ipv4Addr};
use std::sync::{Arc, RwLock, atomic};

use talpid_ipc;

type ActiveSubscriptions = Arc<RwLock<HashMap<SubscriptionId, pubsub::Sink<String>>>>;

pub struct IpcServer {
    server: talpid_ipc::IpcServer,
}

impl IpcServer {
    pub fn start() -> talpid_ipc::Result<Self> {
        let active_subscriptions = ActiveSubscriptions::default();
        let rpc = MockIpcApi::new(active_subscriptions.clone());
        let mut io = PubSubHandler::default();
        io.extend_with(rpc.to_delegate());
        let server = talpid_ipc::IpcServer::start_with_metadata(io.into(), meta_extractor)?;

        Self::spawn_broadcast_thread(active_subscriptions);
        Ok(IpcServer { server })
    }

    pub fn address(&self) -> &str {
        &self.server.address()
    }

    pub fn wait(self) -> talpid_ipc::Result<()> {
        self.server.wait()
    }

    // TODO(linus): This thread will never die. But this is just mock anyway so not important.
    fn spawn_broadcast_thread(active_subscriptions: ActiveSubscriptions) {
        ::std::thread::spawn(
            move || loop {
                {
                    let subscribers = active_subscriptions.read().unwrap();
                    for sink in subscribers.values() {
                        let _ = sink.notify(Ok("Hello World!".into())).wait();
                    }
                }
                ::std::thread::sleep(::std::time::Duration::from_secs(1));
            },
        );
    }
}



/// The metadata type. There is one instance associated with each connection. In this pubsub
/// scenario they are created by `From<Sender<String>>::from` by the server on each new incoming
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

/// A mock implementation of the Mullvad frontend API. A very simplified explanation is that for
/// the real implementation `tunnel_is_up` should be replaced with some kind of handle (or proxy to
/// a handle) to the jsonrpc client talking with talpid_core.
pub struct MockIpcApi {
    next_subscription_id: atomic::AtomicUsize,
    active: ActiveSubscriptions,
    country: RwLock<CountryCode>,
    tunnel_is_up: atomic::AtomicBool,
}

impl MockIpcApi {
    pub fn new(active: ActiveSubscriptions) -> Self {
        MockIpcApi {
            next_subscription_id: atomic::AtomicUsize::new(0),
            active: active,
            country: RwLock::new("se".to_owned()),
            tunnel_is_up: atomic::AtomicBool::new(false),
        }
    }
}

impl IpcApi for MockIpcApi {
    type Metadata = Meta;

    fn get_account_data(&self, account_token: AccountToken) -> Result<AccountData, Error> {
        debug!("Login for {}", account_token);

        let paid_until = if account_token.starts_with("1111") {
            // accounts starting with 1111 expire in one month
            Ok("2018-12-31T16:00:00.000Z".to_owned())
        } else if account_token.starts_with("2222") {
            Ok("2012-12-31T16:00:00.000Z".to_owned())
        } else if account_token.starts_with("3333") {
            Ok("2037-12-31T16:00:00.000Z".to_owned())
        } else {
            Err(jsonrpc_core::Error::invalid_params("You are not welcome"))
        }?;
        Ok(AccountData { paid_until: paid_until })
    }

    fn get_countries(&self) -> Result<HashMap<CountryCode, String>, Error> {
        let mut countries = HashMap::new();
        countries.insert("se".to_owned(), "Sweden".to_owned());
        countries.insert("de".to_owned(), "Denmark".to_owned());
        countries.insert("na".to_owned(), "Narnia".to_owned());
        Ok(countries)
    }

    fn set_account(&self, _account_token: AccountToken) -> Result<(), Error> {
        Ok(())
    }

    fn set_country(&self, country_code: CountryCode) -> Result<(), Error> {
        *self.country.write().unwrap() = country_code;
        Ok(())
    }

    fn set_autoconnect(&self, _autoconnect: bool) -> Result<(), Error> {
        Ok(())
    }

    fn connect(&self) -> Result<(), Error> {
        if self.country.read().unwrap().starts_with("se") {
            Err(jsonrpc_core::Error::invalid_params("Invalid server"))
        } else {
            self.tunnel_is_up.store(true, atomic::Ordering::SeqCst);
            Ok(())
        }
    }

    fn disconnect(&self) -> Result<(), Error> {
        self.tunnel_is_up.store(false, atomic::Ordering::SeqCst);
        Ok(())
    }

    fn get_state(&self) -> Result<SecurityState, Error> {
        if self.tunnel_is_up.load(atomic::Ordering::SeqCst) {
            Ok(SecurityState::Secured)
        } else {
            Ok(SecurityState::Unsecured)
        }
    }

    fn get_ip(&self) -> Result<IpAddr, Error> {
        let ip = if self.tunnel_is_up.load(atomic::Ordering::SeqCst) {
                IpAddr::V4(Ipv4Addr::new(1, 2, 3, 4))
            } else {
                IpAddr::V4(Ipv4Addr::new(192, 168, 1, 2))
            }
            .to_owned();
        Ok(ip)
    }

    fn get_location(&self) -> Result<Location, Error> {
        Ok(
            if self.tunnel_is_up.load(atomic::Ordering::SeqCst) {
                Location {
                    latlong: [1.0, 2.0],
                    country: "narnia".to_owned(),
                    city: "Le city".to_owned(),
                }
            } else {
                Location {
                    latlong: [60.0, 61.0],
                    country: "sweden".to_owned(),
                    city: "bollebygd".to_owned(),
                }
            },
        )
    }

    fn subscribe(&self, _meta: Self::Metadata, subscriber: pubsub::Subscriber<String>) {
        let id = self.next_subscription_id.fetch_add(1, atomic::Ordering::SeqCst);
        let sub_id = SubscriptionId::Number(id as u64);
        if let Ok(sink) = subscriber.assign_id(sub_id.clone()) {
            debug!("Accepting new subscription with id {}", id);
            self.active.write().unwrap().insert(sub_id, sink);
        }
    }

    fn unsubscribe(&self, id: SubscriptionId) -> BoxFuture<bool, Error> {
        debug!("Unsubscribing id {:?}", id);
        if self.active.write().unwrap().remove(&id).is_some() {
            future::ok(true).boxed()
        } else {
            future::err(
                Error {
                    code: ErrorCode::InvalidParams,
                    message: "Invalid subscription.".into(),
                    data: None,
                },
            )
                    .boxed()
        }
    }
}

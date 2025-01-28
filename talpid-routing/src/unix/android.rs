use std::sync::Mutex;

use crate::imp::RouteManagerCommand;
use futures::{
    channel::mpsc::{self, UnboundedReceiver, UnboundedSender},
    stream::StreamExt, select_biased,
    future::FutureExt,
};
use ipnetwork::IpNetwork;
use jnix::{
    jni::{objects::JObject, JNIEnv},
    FromJava, JnixEnv,
};
use talpid_types::android::NetworkState;

/// Stub error type for routing errors on Android.
/// Errors that occur while setting up VpnService tunnel.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Failed to send shutdown result")]
    Send,

    #[error("Failed to attach Java VM to tunnel thread")]
    AttachJvmToThread(#[source] jnix::jni::errors::Error),

    #[error("Failed to call Java method TalpidVpnService.{0}")]
    //CallMethod(&'static str, #[source] jnix::jni::errors::Error),
    CallMethod(&'static str),

    #[error("Failed to create Java VM handle clone")]
    CloneJavaVm(#[source] jnix::jni::errors::Error),

    #[error("Failed to find TalpidVpnService.{0} method")]
    FindMethod(&'static str, #[source] jnix::jni::errors::Error),

    #[error("Received an invalid result from TalpidVpnService.{0}: {1}")]
    InvalidMethodResult(&'static str, String),

    #[error("Routes timed out")]
    RoutesTimedOut,

    #[error("Profile for VPN has not been setup")]
    NotPrepared,

    #[error("Another legacy VPN profile is used as always on")]
    OtherLegacyAlwaysOnVpn,
}

/// TODO: Document mee
static ROUTE_UPDATES_TX: Mutex<Option<UnboundedSender<RoutesUpdate>>> = Mutex::new(None);

#[derive(Debug, Clone)]
pub enum RoutesUpdate {
    NewNetworkState(NetworkState),
}

// TODO: This is le actor state
/// Stub route manager for Android
pub struct RouteManagerImpl {
    routes_updates: UnboundedReceiver<RoutesUpdate>,
    listeners: Vec<UnboundedSender<RoutesUpdate>>,
}

pub enum RouteResult {
    CorrectRoutes,
    IncorrectRoutes,
}

impl RouteManagerImpl {
    #[allow(clippy::unused_async)]
    pub async fn new() -> Result<Self, Error> {
        // Create a channel between the kotlin client and route manager
        let (tx, rx) = futures::channel::mpsc::unbounded();
        // TODO: What id `ROUTE_UPDATES_TX` has already been initialized?
        *ROUTE_UPDATES_TX.lock().unwrap() = Some(tx);
        Ok(RouteManagerImpl {
            routes_updates: rx,
            listeners: Default::default(),
        })
    }

    pub(crate) async fn run(
        mut self,
        manage_rx: mpsc::UnboundedReceiver<RouteManagerCommand>,
    ) -> Result<(), Error> {
        let mut manage_rx = manage_rx.fuse();
        loop {
            select_biased! {
                command = manage_rx.next().fuse() => {
                    let Some(command) = command else { break };

                    match command {
                        RouteManagerCommand::NewChangeListener(tx) => {
                            // register a listener for new route updates
                            self.listeners.push(tx);
                        }
                        RouteManagerCommand::Shutdown(tx) => {
                            tx.send(()).map_err(|()| Error::Send)?; // TODO: Surely we can do better than this
                            break;
                        }
                        RouteManagerCommand::AddRoutes(_routes, tx) => {
                            tx.send(Ok(())).map_err(|_x| Error::Send)?;
                        }
                        RouteManagerCommand::ClearRoutes => (),
                    }
                }

                route_update = self.routes_updates.next().fuse() => {
                    let Some(route_update) = route_update else { break };
                    self.notify_change_listeners(route_update);
                }
            }
        }
        Ok(())
    }

    // pub fn wait_for_routes(&mut self, routes: Vec<IpNetwork>) -> impl Stream<Item = bool> { }

    fn notify_change_listeners(&mut self, message: RoutesUpdate) {
        self.listeners
            .retain(|listener| listener.unbounded_send(message.clone()).is_ok());
    }

    fn listen(&mut self) -> UnboundedReceiver<RoutesUpdate> {
        let (tx, rx) = futures::channel::mpsc::unbounded();
        self.listeners.push(tx);
        rx
    }
}

/// Entry point for Android Java code to notify the current default network state.
#[no_mangle]
#[allow(non_snake_case)]
pub extern "system" fn Java_net_mullvad_talpid_ConnectivityListener_notifyDefaultNetworkChange(
    env: JNIEnv<'_>,
    _: JObject<'_>,
    network_state: JObject<'_>, // TODO: Actually get the routes
) {
    let env = JnixEnv::from(env);

    if network_state.is_null() {
        // TODO: We might want to handle this more gracefully
        log::debug!("Received NULL NetworkState");
        return;
    }
    let network_state = NetworkState::from_java(&env, network_state);
    let Some(tx) = &*ROUTE_UPDATES_TX.lock().unwrap() else {
        // No sender has been registered
        log::error!("Received routes notification wíth no channel");
        return;
    };

    log::info!("Received network state {:#?}", network_state);

    if tx
        .unbounded_send(RoutesUpdate::NewNetworkState(network_state))
        .is_err()
    {
        log::warn!("Failed to send offline change event");
    }
}

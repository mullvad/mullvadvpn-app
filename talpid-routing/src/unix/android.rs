use std::collections::{HashSet, VecDeque};
use std::sync::Mutex;
use std::ops::ControlFlow;

use futures::channel::mpsc::{self, UnboundedReceiver, UnboundedSender};
use futures::channel::oneshot;
use futures::future::FutureExt;
use futures::select_biased;
use futures::stream::StreamExt;
use ipnetwork::IpNetwork;
use jnix::jni::{objects::JObject, JNIEnv};
use jnix::{FromJava, JnixEnv};

use crate::{imp::RouteManagerCommand, RequiredRoute};
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

/// The sender used by [Java_net_mullvad_talpid_ConnectivityListener_notifyDefaultNetworkChange]
/// to notify the route manager of changes to the network.
static ROUTE_UPDATES_TX: Mutex<Option<UnboundedSender<NetworkState>>> = Mutex::new(None);

/// Android route manager actor.
#[derive(Debug)]
pub struct RouteManagerImpl {
    network_state_updates: UnboundedReceiver<NetworkState>,

    /// Cached [NetworkState]. If no update events have been received yet, this value will be [None].
    last_state: Option<NetworkState>,

    /// Clients waiting on response to [RouteManagerCommand::AddRoutes].
    waiting_for_route: VecDeque<WaitingForRoutes>,
}

#[derive(Debug)]
struct WaitingForRoutes {
    response_tx: oneshot::Sender<Result<(), Error>>,
    required_routes: HashSet<RequiredRoute>,
}

impl RouteManagerImpl {
    #[allow(clippy::unused_async)]
    pub async fn new() -> Result<Self, Error> {
        // Create a channel between the kotlin client and route manager
        let (tx, rx) = futures::channel::mpsc::unbounded();

        // TODO: What id `ROUTE_UPDATES_TX` has already been initialized?
        *ROUTE_UPDATES_TX.lock().unwrap() = Some(tx);

        Ok(RouteManagerImpl {
            network_state_updates: rx,
            last_state: Default::default(),
            waiting_for_route: Default::default(),
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
                    if self.handle_command(command).is_break() {
                        return Ok(());
                    }
                }

                route_update = self.network_state_updates.next().fuse() => {
                    self.last_state = route_update;
                    // TODO: Handle None (sender closed)
                    // check each waiting client if we have the routes they expect
                    for _ in 0..self.waiting_for_route.len() {
                        // oneshot senders consume themselves, so we need to take them out of the list
                        let Some(client) = self.waiting_for_route.pop_front() else { break };

                        if client.response_tx.is_canceled() {
                            // do nothing, drop the sender
                        } else if has_routes(self.last_state.as_ref(), &client.required_routes) {
                            let _ = client.response_tx.send(Ok(()));
                        } else {
                            self.waiting_for_route.push_back(client);
                        }
                    }
                }
            }
        }
        Ok(())
    }

    fn handle_command(&mut self, command: RouteManagerCommand) -> ControlFlow<()> {
        match command {
            RouteManagerCommand::Shutdown(tx) => {
                let _ = tx.send(());
                return ControlFlow::Break(());
            }
            RouteManagerCommand::AddRoutes(required_routes, response_tx) => {
                log::info!("Current state: {self:#?}");
                log::info!("Looking for deez routes: {required_routes:#?}");
                if has_routes(self.last_state.as_ref(), &required_routes) {
                    let _ = response_tx.send(Ok(()));
                } else {
                    self.waiting_for_route.push_back(WaitingForRoutes {response_tx, required_routes});
                }
            }
            RouteManagerCommand::ClearRoutes => {
                // The VPN tunnel is gone. We can't assume that any (desired) routes are up at this point.
                // TODO: This won't work right away, as we're apparently clearing routes when reconnecting ..
                // self.last_state = None;
                log::debug!("Clearing routes");
            },
        }

        ControlFlow::Continue(())
    }
}

/// Check whether the [NetworkState] contains the provided set of [RequiredRoute]s.
fn has_routes(state: Option<&NetworkState>, routes: &HashSet<RequiredRoute>) -> bool {
    let Some(network_state) = state else {
        return false;
    };
    let Some(route_info) = &network_state.routes else {
        return false;
    };
    // TODO: fugly
    let existing_routes: HashSet<RequiredRoute> = route_info
        .iter()
        .map(|route_info| {
            let network = IpNetwork::new(
                route_info.destination.address,
                route_info.destination.prefix_length as u8,
            )
            .unwrap();
            RequiredRoute::new(network)
        })
        .collect();
    routes.is_subset(&existing_routes)
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

    if tx.unbounded_send(network_state).is_err() {
        log::warn!("Failed to send offline change event");
    }
}

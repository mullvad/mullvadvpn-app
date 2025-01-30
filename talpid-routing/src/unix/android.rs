use std::collections::{HashSet, VecDeque};
use std::ops::ControlFlow;
use std::sync::Mutex;

use futures::channel::mpsc::{self, UnboundedReceiver, UnboundedSender};
use futures::channel::oneshot;
use futures::future::FutureExt;
use futures::select_biased;
use futures::stream::StreamExt;
use jnix::jni::{objects::JObject, JNIEnv};
use jnix::{FromJava, JnixEnv};

use crate::{imp::RouteManagerCommand, RequiredRoute};
use talpid_types::android::NetworkState;

/// Stub error type for routing errors on Android.
/// Errors that occur while setting up VpnService tunnel.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// Timed out when waiting for network routes.
    #[error("Timed out when waiting for network routes")]
    RoutesTimedOut,
}

/// The sender used by [Java_net_mullvad_talpid_ConnectivityListener_notifyDefaultNetworkChange]
/// to notify the route manager of changes to the network.
static ROUTE_UPDATES_TX: Mutex<Option<UnboundedSender<Option<NetworkState>>>> = Mutex::new(None);

/// Android route manager actor.
#[derive(Debug)]
pub struct RouteManagerImpl {
    /// The receiving channel for updates on changes to the network.
    network_state_updates: UnboundedReceiver<Option<NetworkState>>,

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

                network_state_update = self.network_state_updates.next().fuse() => {
                    // None means that the sender was dropped
                    let Some(network_state) = network_state_update else { break };
                    // update the last known NetworkState
                    self.last_state = network_state;
                    // check each waiting client if we have the routes they expect
                    for _ in 0..self.waiting_for_route.len() {
                        // oneshot senders consume themselves, so we need to take them out of the list
                        let Some(client) = self.waiting_for_route.pop_front() else { break };

                        if client.response_tx.is_canceled() {
                            // do nothing, drop the sender
                        } else if has_routes(self.last_state.as_ref(), &client.required_routes) {
                            // notify listener that the required routes (seems to) have been
                            // configured on the Android system, since they are part of the
                            // NetworkState update
                            let _ = client.response_tx.send(Ok(()));
                        } else {
                            // no dice, required routes were not part of this network state change.
                            // patiently wait for the next update
                            self.waiting_for_route.push_back(client);
                        }
                    }
                }
            }
        }

        log::debug!("RouteManager exited");

        Ok(())
    }

    fn handle_command(&mut self, command: RouteManagerCommand) -> ControlFlow<()> {
        match command {
            RouteManagerCommand::Shutdown(tx) => {
                let _ = tx.send(());
                return ControlFlow::Break(());
            }
            RouteManagerCommand::AddRoutes(required_routes, response_tx) => {
                // check if the required routes already have been configured on the Android system.
                // otherwise, register a listener for network state changes. The required routes
                // may come in at any moment in the future.
                if has_routes(self.last_state.as_ref(), &required_routes) {
                    let _ = response_tx.send(Ok(()));
                } else {
                    self.waiting_for_route.push_back(WaitingForRoutes {
                        response_tx,
                        required_routes,
                    });
                }
            }
            RouteManagerCommand::ClearRoutes => {
                // The VPN tunnel is gone. We can't assume that any (desired) routes are up at this point.
                // TODO: This won't work right away, as we're apparently clearing routes when reconnecting ..
                // self.last_state = None;
                log::debug!("Clearing routes");
            }
        }

        ControlFlow::Continue(())
    }
}

/// Check whether the [NetworkState] contains the provided set of [RequiredRoute]s.
fn has_routes(state: Option<&NetworkState>, routes: &HashSet<RequiredRoute>) -> bool {
    let Some(network_state) = state else {
        return false;
    };
    routes.is_subset(&configured_routes(network_state))
}

fn configured_routes(state: &NetworkState) -> HashSet<RequiredRoute> {
    match &state.routes {
        None => Default::default(),
        Some(route_info) => route_info.iter().map(RequiredRoute::from).collect(),
    }
}

/// Entry point for Android Java code to notify the current default network state.
#[no_mangle]
#[allow(non_snake_case)]
pub extern "system" fn Java_net_mullvad_talpid_ConnectivityListener_notifyDefaultNetworkChange(
    env: JNIEnv<'_>,
    _: JObject<'_>,
    network_state: JObject<'_>,
) {
    let env = JnixEnv::from(env);

    let network_state: Option<NetworkState> = FromJava::from_java(&env, network_state);

    let Some(tx) = &*ROUTE_UPDATES_TX.lock().unwrap() else {
        // No sender has been registered
        log::error!("Received routes notification wíth no channel");
        return;
    };

    log::trace!("Received network state update {:#?}", network_state);

    if tx.unbounded_send(network_state).is_err() {
        log::warn!("Failed to send offline change event");
    }
}

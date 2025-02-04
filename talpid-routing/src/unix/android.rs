use std::collections::HashSet;
use std::ops::{ControlFlow, Not};
use std::sync::Mutex;

use futures::channel::mpsc::{self, UnboundedReceiver, UnboundedSender};
use futures::channel::oneshot;
use futures::future::FutureExt;
use futures::select_biased;
use futures::stream::StreamExt;
use jnix::jni::{objects::JObject, JNIEnv};
use jnix::{FromJava, JnixEnv};

use talpid_types::android::NetworkState;

use crate::{imp::RouteManagerCommand, Route};

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

    /// Clients waiting on response to [RouteManagerCommand::WaitForRoutes].
    waiting_for_routes: Vec<oneshot::Sender<()>>,
}

impl RouteManagerImpl {
    #[allow(clippy::unused_async)]
    pub async fn new() -> Result<Self, Error> {
        // Create a channel between the kotlin client and route manager
        let (tx, rx) = futures::channel::mpsc::unbounded();

        *ROUTE_UPDATES_TX.lock().unwrap() = Some(tx);

        let route_manager = RouteManagerImpl {
            network_state_updates: rx,
            last_state: Default::default(),
            waiting_for_routes: Default::default(),
        };

        Ok(route_manager)
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

                    if has_routes(self.last_state.as_ref()) {
                        // notify waiting clients that routes exist
                        for client in self.waiting_for_routes.drain(..) {
                            let _ = client.send(());
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
            RouteManagerCommand::WaitForRoutes(response_tx) => {
                // check if routes have already been configured on the Android system.
                // otherwise, register a listener for network state changes.
                // routes may come in at any moment in the future.
                if has_routes(self.last_state.as_ref()) {
                    let _ = response_tx.send(());
                } else {
                    self.waiting_for_routes.push(response_tx);
                }
            }
        }

        ControlFlow::Continue(())
    }
}

/// Check whether the [NetworkState] contains any routes.
///
/// Since we are the ones telling Android what routes to set, we make the assumption that:
/// If any routes exist whatsoever, they are the the routes we specified.
fn has_routes(state: Option<&NetworkState>) -> bool {
    let Some(network_state) = state else {
        return false;
    };
    configured_routes(network_state).is_empty().not()
}

fn configured_routes(state: &NetworkState) -> HashSet<Route> {
    match &state.routes {
        None => Default::default(),
        Some(route_info) => route_info.iter().map(Route::from).collect(),
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

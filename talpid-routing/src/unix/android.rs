use std::collections::HashSet;
use std::ops::{ControlFlow};
use std::sync::Mutex;

use futures::channel::mpsc::{self, UnboundedReceiver, UnboundedSender};
use futures::channel::oneshot;
use futures::future::FutureExt;
use futures::select_biased;
use futures::stream::StreamExt;
use jnix::jni::objects::JValue;
use jnix::jni::{objects::JObject, JNIEnv};
use jnix::{FromJava, JnixEnv};

use talpid_types::android::{AndroidContext, NetworkState};

use crate::{imp::RouteManagerCommand, Route};

/// Stub error type for routing errors on Android.
/// Errors that occur while setting up VpnService tunnel.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// Timed out when waiting for network routes.
    #[error("Timed out when waiting for network routes")]
    RoutesTimedOut,
}

/// Internal errors that may only happen during the initial poll for [NetworkState].
#[derive(Debug, thiserror::Error)]
enum JvmError {
    #[error("Failed to attach Java VM to tunnel thread")]
    AttachJvmToThread(#[source] jnix::jni::errors::Error),
    #[error("Failed to call Java method {0}")]
    CallMethod(&'static str, #[source] jnix::jni::errors::Error),
    #[error("Failed to create global reference to Java object")]
    CreateGlobalRef(#[source] jnix::jni::errors::Error),
    #[error("Received an invalid result from {0}.{1}: {2}")]
    InvalidMethodResult(&'static str, &'static str, String),
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
    waiting_for_routes: Vec<(oneshot::Sender<()>, Vec<Route>)>,
}

impl RouteManagerImpl {
    #[allow(clippy::unused_async)]
    pub async fn new(android_context: AndroidContext) -> Result<Self, Error> {
        // Create a channel between the kotlin client and route manager
        let (tx, rx) = futures::channel::mpsc::unbounded();

        *ROUTE_UPDATES_TX.lock().unwrap() = Some(tx);

        // Try to poll for the current network state at startup.
        // This will most likely be null, but it covers the edge case where a NetworkState
        // update has been emitted before anyone starts to listen for route updates some
        // time in the future (when connecting).
        let last_state = match current_network_state(android_context) {
            Ok(initial_state) => initial_state,
            Err(err) => {
                log::error!("Failed while polling for initial NetworkState");
                log::error!("{err}");
                None
            }
        };

        let route_manager = RouteManagerImpl {
            network_state_updates: rx,
            last_state,
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
                        break;
                    }
                }

                network_state_update = self.network_state_updates.next().fuse() => {
                    // None means that the sender was dropped
                    let Some(network_state) = network_state_update else { break };
                    // update the last known NetworkState
                    self.last_state = network_state;

                    // notify waiting clients that routes exist
                    let mut unused_routes: Vec<(oneshot::Sender<()>, Vec<Route>)> = Vec::new();
                    let ret = for (client, expected_routes) in self.waiting_for_routes.drain(..) {
                        if has_routes(self.last_state.as_ref(), expected_routes.clone()) {
                            let _ = client.send(());
                        } else {
                            unused_routes.push((client, expected_routes));
                        }
                    };
                    self.waiting_for_routes = unused_routes;
                    ret
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
            RouteManagerCommand::WaitForRoutes(response_tx, expected_routes) => {
                // check if routes have already been configured on the Android system.
                // otherwise, register a listener for network state changes.
                // routes may come in at any moment in the future.
                if has_routes(self.last_state.as_ref(), expected_routes.clone()) {
                    let _ = response_tx.send(());
                } else {
                    self.waiting_for_routes.push((response_tx, expected_routes));
                }
            }
            RouteManagerCommand::ClearRoutes(tx) => {
                self.clear_routes();
                let _ = tx.send(());
            }
        }

        ControlFlow::Continue(())
    }

    pub fn clear_routes(&mut self) {
        self.last_state = None;
    }
}

/// Check whether the [NetworkState] contains expected routes.
///
/// Matches the routes reported from Android and checks if all the routes we expect to be there is
/// present.
fn has_routes(state: Option<&NetworkState>, expected_routes: Vec<Route>) -> bool {
    let Some(network_state) = state else {
        return false;
    };

    let routes = configured_routes(network_state);
    if routes.is_empty() {
        return false;
    }
    routes.is_superset(&HashSet::from_iter(expected_routes))
}

fn configured_routes(state: &NetworkState) -> HashSet<Route> {
    match &state.routes {
        None => Default::default(),
        Some(route_info) => route_info.iter().map(Route::from).collect(),
    }
}

/// Entry point for Android Java code to notify the current default network state.
#[unsafe(no_mangle)]
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

/// Return the current NetworkState according to Android
fn current_network_state(
    android_context: AndroidContext,
) -> Result<Option<NetworkState>, JvmError> {
    let env = JnixEnv::from(
        android_context
            .jvm
            .attach_current_thread_as_daemon()
            .map_err(JvmError::AttachJvmToThread)?,
    );

    let result = env
        .call_method(
            android_context.vpn_service.as_obj(),
            "getConnectivityListener",
            "()Lnet/mullvad/talpid/ConnectivityListener;",
            &[],
        )
        .map_err(|cause| JvmError::CallMethod("getConnectivityListener", cause))?;

    let connectivity_listener = match result {
        JValue::Object(object) => env
            .new_global_ref(object)
            .map_err(JvmError::CreateGlobalRef)?,
        value => {
            return Err(JvmError::InvalidMethodResult(
                "MullvadVpnService",
                "getConnectivityListener",
                format!("{:?}", value),
            ))
        }
    };

    let network_state = env
        .call_method(
            connectivity_listener.as_obj(),
            "getCurrentDefaultNetworkState",
            "()Lnet/mullvad/talpid/model/NetworkState;",
            &[],
        )
        .map_err(|cause| JvmError::CallMethod("getCurrentDefaultNetworkState", cause))?;

    let network_state: Option<NetworkState> = FromJava::from_java(&env, network_state);
    Ok(network_state)
}

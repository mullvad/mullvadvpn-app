use crate::{debounce::BurstGuard, NetNode, Node, RequiredRoute, Route};

use futures::{
    channel::mpsc::{self, UnboundedReceiver},
    future::FutureExt,
    stream::{FusedStream, StreamExt},
};
use ipnetwork::IpNetwork;
use std::{
    collections::{BTreeMap, HashSet},
    pin::Pin,
    sync::Weak,
    time::Duration,
};
use talpid_types::ErrorExt;
use watch::RoutingTable;

use super::{DefaultRouteEvent, RouteManagerCommand};
use data::{Destination, RouteDestination, RouteMessage, RouteSocketMessage};

mod data;
mod interface;
mod routing_socket;
mod watch;

pub type Result<T> = std::result::Result<T, Error>;

const BURST_BUFFER_PERIOD: Duration = Duration::from_millis(200);
const BURST_LONGEST_BUFFER_PERIOD: Duration = Duration::from_secs(2);

/// Errors that can happen in the macOS routing integration.
#[derive(err_derive::Error, Debug)]
#[error(no_from)]
pub enum Error {
    /// Encountered an error when interacting with the routing socket
    #[error(display = "Error occurred when interfacing with the routing table")]
    RoutingTable(#[error(source)] watch::Error),

    /// Failed to remvoe route
    #[error(display = "Error occurred when deleting a route")]
    DeleteRoute(#[error(source)] watch::Error),

    /// Failed to add route
    #[error(display = "Error occurred when adding a route")]
    AddRoute(#[error(source)] watch::Error),

    /// Failed to fetch link addresses
    #[error(display = "Failed to fetch link addresses")]
    FetchLinkAddresses(#[error(source)] std::io::Error),

    /// Received message isn't valid
    #[error(display = "Invalid data")]
    InvalidData(data::Error),
}

/// Convenience macro to get the current default route. Macro because I don't want to borrow `self`
/// mutably.
macro_rules! get_current_best_default_route {
    ($self:expr, $family:expr) => {{
        match $family {
            interface::Family::V4 => &mut $self.v4_default_route,
            interface::Family::V6 => &mut $self.v6_default_route,
        }
    }};
}

/// Route manager can be in 1 of 4 states -
///  - waiting for a route to be added or removed from the route table
///  - obtaining default routes
///  - applying changes to the route table
///  - shutting down
///
/// Only the _shutting down_ state can be reached from all other states, but during normal
/// operation, the route manager will add all the required routes during startup and will start
/// waiting for changes to the route table.  If any change is detected, it will stop listening for
/// new changes, obtain new default routes and reapply routes that should be routed through the
/// default nodes. Once the routes are reapplied, the route table changes are monitored again.
pub struct RouteManagerImpl {
    routing_table: RoutingTable,
    // Routes that use the default non-tunnel interface
    non_tunnel_routes: HashSet<IpNetwork>,
    v4_tunnel_default_route: Option<data::RouteMessage>,
    v6_tunnel_default_route: Option<data::RouteMessage>,
    applied_routes: BTreeMap<RouteDestination, RouteMessage>,
    v4_default_route: Option<data::RouteMessage>,
    v6_default_route: Option<data::RouteMessage>,
    update_trigger: BurstGuard,
    default_route_listeners: Vec<mpsc::UnboundedSender<DefaultRouteEvent>>,
    check_default_routes_restored: Pin<Box<dyn FusedStream<Item = ()> + Send>>,
    unhandled_default_route_changes: bool,
    primary_interface_monitor: interface::PrimaryInterfaceMonitor,
    interface_change_rx: UnboundedReceiver<interface::InterfaceEvent>,
}

impl RouteManagerImpl {
    /// Create new route manager
    #[allow(clippy::unused_async)]
    pub(crate) async fn new(
        manage_tx: Weak<mpsc::UnboundedSender<RouteManagerCommand>>,
    ) -> Result<Self> {
        let (primary_interface_monitor, interface_change_rx) =
            interface::PrimaryInterfaceMonitor::new();
        let routing_table = RoutingTable::new().map_err(Error::RoutingTable)?;

        let update_trigger = BurstGuard::new(
            BURST_BUFFER_PERIOD,
            BURST_LONGEST_BUFFER_PERIOD,
            move || {
                let Some(manage_tx) = manage_tx.upgrade() else {
                    return;
                };
                let _ = manage_tx.unbounded_send(RouteManagerCommand::RefreshRoutes);
            },
        );

        Ok(Self {
            routing_table,
            non_tunnel_routes: HashSet::new(),
            v4_tunnel_default_route: None,
            v6_tunnel_default_route: None,
            applied_routes: BTreeMap::new(),
            v4_default_route: None,
            v6_default_route: None,
            update_trigger,
            default_route_listeners: vec![],
            check_default_routes_restored: Box::pin(futures::stream::pending()),
            unhandled_default_route_changes: false,
            primary_interface_monitor,
            interface_change_rx,
        })
    }

    pub(crate) async fn run(mut self, manage_rx: mpsc::UnboundedReceiver<RouteManagerCommand>) {
        let mut manage_rx = manage_rx.fuse();

        // Initialize default routes
        // NOTE: This isn't race-free, as we're not listening for route changes before initializing
        self.update_best_default_route(interface::Family::V4)
            .unwrap_or_else(|error| {
                log::error!(
                    "{}",
                    error.display_chain_with_msg("Failed to get initial default v4 route")
                );
                false
            });
        self.update_best_default_route(interface::Family::V6)
            .unwrap_or_else(|error| {
                log::error!(
                    "{}",
                    error.display_chain_with_msg("Failed to get initial default v6 route")
                );
                false
            });

        self.debug_offline();

        let mut completion_tx = None;

        loop {
            futures::select_biased! {
                route_message = self.routing_table.next_message().fuse() => {
                    self.handle_route_message(route_message);
                }

                _ = self.check_default_routes_restored.next() => {
                    if self.check_default_routes_restored.is_terminated() {
                        continue;
                    }
                    if self.try_restore_default_routes().await {
                        log::debug!("Unscoped routes were already restored");
                        self.check_default_routes_restored = Box::pin(futures::stream::pending());
                    }
                }

                _event = self.interface_change_rx.next() => {
                    self.update_trigger.trigger();
                }

                command = manage_rx.next() => {
                    match command {
                        Some(RouteManagerCommand::Shutdown(tx)) => {
                            completion_tx = Some(tx);
                            break;
                        },

                        Some(RouteManagerCommand::NewDefaultRouteListener(tx)) => {
                            let (events_tx, events_rx) = mpsc::unbounded();
                            self.default_route_listeners.push(events_tx);
                            let _ = tx.send(events_rx);
                        }
                        Some(RouteManagerCommand::GetDefaultRoutes(tx)) => {
                            // NOTE: The device name isn't really relevant here,
                            // as we only care about routes with a gateway IP.
                            let v4_route = self.v4_default_route.as_ref().map(|route| {
                                Route {
                                    node: Node {
                                        device: None,
                                        ip: route.gateway_ip(),
                                    },
                                    prefix: interface::Family::V4.default_network(),
                                    metric: None,
                                }
                            });
                            let v6_route = self.v6_default_route.as_ref().map(|route| {
                                Route {
                                    node: Node {
                                        device: None,
                                        ip: route.gateway_ip(),
                                    },
                                    prefix: interface::Family::V6.default_network(),
                                    metric: None,
                                }
                            });

                            let _ = tx.send((v4_route, v6_route));
                        }

                        Some(RouteManagerCommand::AddRoutes(routes, tx)) => {
                            if !self.check_default_routes_restored.is_terminated() {
                                log::debug!("Cancelling restoration of default routes");
                                self.check_default_routes_restored = Box::pin(futures::stream::pending());
                            }
                            log::debug!("Adding routes: {routes:?}");
                            let _ = tx.send(self.add_required_routes(routes).await);
                        }
                        Some(RouteManagerCommand::ClearRoutes) => {
                            if let Err(err) = self.cleanup_routes().await {
                                log::error!("Failed to clean up rotues: {err}");
                            }
                        },
                        Some(RouteManagerCommand::RefreshRoutes) => {
                            if let Err(error) = self.refresh_routes().await {
                                log::error!("Failed to refresh routes: {error}");
                            }
                        },
                        None => {
                            break;
                        }
                    }
                },
            };
        }

        if let Err(err) = self.cleanup_routes().await {
            log::error!("Failed to clean up routing table when shutting down: {err}");
        }

        self.update_trigger.stop_nonblocking();

        if let Some(tx) = completion_tx {
            let _ = tx.send(());
        }
    }

    async fn add_required_routes(&mut self, required_routes: HashSet<RequiredRoute>) -> Result<()> {
        let mut routes_to_apply = vec![];

        for route in required_routes {
            match route.node {
                NetNode::DefaultNode => {
                    self.non_tunnel_routes.insert(route.prefix);
                }

                NetNode::RealNode(node) => routes_to_apply.push(Route::new(node, route.prefix)),
            }
        }

        // Map all interfaces to their link addresses
        let interface_link_addrs =
            interface::get_interface_link_addresses().map_err(Error::FetchLinkAddresses)?;

        // Add routes not using the default interface
        for route in routes_to_apply {
            let message = if let Some(ref device) = route.node.device {
                // If we specify route by interface name, use the link address of the given
                // interface
                match interface_link_addrs.get(device) {
                    Some(link_addr) => RouteMessage::new_route(Destination::from(route.prefix))
                        .set_gateway_sockaddr(*link_addr),
                    None => {
                        log::error!("Route with unknown device: {route:?}, {device}");
                        continue;
                    }
                }
            } else {
                log::error!("Specifying gateway by IP rather than device is unimplemented");
                continue;
            };

            // Default routes are a special case: We must apply it after replacing the current
            // default route with an ifscope route.
            if route.prefix.prefix() == 0 {
                if route.prefix.is_ipv4() {
                    self.v4_tunnel_default_route = Some(message);
                } else {
                    self.v6_tunnel_default_route = Some(message);
                }
                continue;
            }

            // Add route
            self.add_route_with_record(message).await?;
        }

        self.apply_tunnel_default_route().await?;

        // Add routes that use the default interface
        if let Err(error) = self.apply_non_tunnel_routes().await {
            self.non_tunnel_routes.clear();
            return Err(error);
        }

        Ok(())
    }

    fn handle_route_message(
        &mut self,
        message: std::result::Result<RouteSocketMessage, watch::Error>,
    ) {
        match message {
            Ok(RouteSocketMessage::DeleteRoute(route)) => {
                // Forget about applied route, if relevant
                match RouteDestination::try_from(&route).map_err(Error::InvalidData) {
                    Ok(destination) => {
                        self.applied_routes.remove(&destination);
                    }
                    Err(err) => {
                        log::error!("Failed to process deleted route: {err}");
                    }
                }
                if route.errno() != 0 {
                    return;
                }
                if route.is_default().unwrap_or(true) {
                    self.unhandled_default_route_changes = true;
                }
                self.update_trigger.trigger();
            }
            Ok(RouteSocketMessage::AddRoute(route))
            | Ok(RouteSocketMessage::ChangeRoute(route)) => {
                if route.errno() != 0 {
                    return;
                }
                if route.is_default().unwrap_or(true) {
                    self.unhandled_default_route_changes = true;
                }
                self.update_trigger.trigger();
            }
            Ok(RouteSocketMessage::AddAddress(_) | RouteSocketMessage::DeleteAddress(_)) => {
                self.update_trigger.trigger();
            }
            // ignore all other message types
            Ok(_) => {}
            Err(err) => {
                log::error!("Failed to receive a message from the routing table: {err}");
            }
        }
    }

    /// Handle changes to the routing table:
    /// * Replace the unscoped default route with a default route for the tunnel interface (i.e.,
    ///   one whose gateway is set to the link address of the tunnel interface).
    /// * At the same time, update the route used by non-tunnel interfaces to reach the relay/VPN
    ///   server. The gateway of the relay route is set to the first interface in the network
    ///   service order that has a working ifscoped default route.
    async fn refresh_routes(&mut self) -> Result<()> {
        self.update_best_default_route(interface::Family::V4)?;
        self.update_best_default_route(interface::Family::V6)?;

        self.debug_offline();

        if !self.unhandled_default_route_changes {
            return Ok(());
        }

        // Remove any existing ifscope route that we've added
        self.remove_applied_routes(|route| {
            route.is_ifscope() && route.is_default().unwrap_or(false)
        })
        .await;

        // Substitute route with a tunnel route
        self.apply_tunnel_default_route().await?;

        // Update routes using default interface
        self.apply_non_tunnel_routes().await?;

        self.unhandled_default_route_changes = false;

        Ok(())
    }

    fn debug_offline(&self) {
        if self.v4_default_route.is_none() && self.v6_default_route.is_none() {
            self.primary_interface_monitor.debug();
        }
    }

    /// Figure out what the best default routes to use are, and send updates to default route change
    /// subscribers. The "best routes" are used by the tunnel device to send packets to the VPN
    /// relay.
    ///
    /// The "best route" is determined by the first interface in the network service order that has
    /// a valid IP address and gateway.
    ///
    /// On success, the function returns whether the previously known best default changed.
    fn update_best_default_route(&mut self, family: interface::Family) -> Result<bool> {
        let best_route = self.primary_interface_monitor.get_route(family);
        let current_route = get_current_best_default_route!(self, family);

        log::trace!("Best route ({family:?}): {best_route:?}");
        if best_route == *current_route {
            return Ok(false);
        }

        self.unhandled_default_route_changes = true;

        let old_pair = current_route
            .as_ref()
            .map(|r| (r.interface_index(), r.gateway_ip()));
        let new_pair = best_route
            .as_ref()
            .map(|r| (r.interface_index(), r.gateway_ip()));
        log::debug!("Best default route ({family}) changed from {old_pair:?} to {new_pair:?}");
        let _ = std::mem::replace(current_route, best_route);

        let changed = current_route.is_some();
        self.notify_default_route_listeners(family, changed);
        Ok(true)
    }

    fn notify_default_route_listeners(&mut self, family: interface::Family, changed: bool) {
        // Notify default route listeners
        let event = match (family, changed) {
            (interface::Family::V4, true) => DefaultRouteEvent::AddedOrChangedV4,
            (interface::Family::V6, true) => DefaultRouteEvent::AddedOrChangedV6,
            (interface::Family::V4, false) => DefaultRouteEvent::RemovedV4,
            (interface::Family::V6, false) => DefaultRouteEvent::RemovedV6,
        };
        self.default_route_listeners
            .retain(|tx| tx.unbounded_send(event).is_ok());
    }

    /// Replace the default routes with an ifscope route, and
    /// add a new default tunnel route.
    async fn apply_tunnel_default_route(&mut self) -> Result<()> {
        // As long as the relay route has a way of reaching the internet, we'll want to add a tunnel
        // route for both IPv4 and IPv6.
        // NOTE: This is incorrect. We're assuming that any "default destination" is used for
        // tunneling.
        let (v4_conn, v6_conn) = self
            .non_tunnel_routes
            .iter()
            .fold((false, false), |(v4, v6), route| {
                (v4 || route.is_ipv4(), v6 || route.is_ipv6())
            });
        let relay_route_is_valid = (v4_conn && self.v4_default_route.is_some())
            || (v6_conn && self.v6_default_route.is_some());

        if !relay_route_is_valid {
            return Ok(());
        }

        for tunnel_route in [
            self.v4_tunnel_default_route.clone(),
            self.v6_tunnel_default_route.clone(),
        ] {
            let tunnel_route = match tunnel_route {
                Some(route) => route,
                None => continue,
            };
            let family = if tunnel_route.is_ipv4() {
                interface::Family::V4
            } else {
                interface::Family::V6
            };

            // Replace the default route with an ifscope route
            self.replace_with_scoped_route(family).await?;

            // Make sure there is really no other unscoped default route
            let mut msg = RouteMessage::new_route(family.default_network().into());
            msg = msg.set_gateway_route(true);
            let old_route = self.routing_table.get_route(&msg).await;
            if let Ok(Some(old_route)) = old_route {
                let tun_gateway_link_addr =
                    tunnel_route.gateway().and_then(|addr| addr.as_link_addr());
                let current_link_addr = old_route.gateway().and_then(|addr| addr.as_link_addr());
                if current_link_addr
                    .map(|addr| Some(addr) != tun_gateway_link_addr)
                    .unwrap_or(true)
                {
                    log::trace!("Removing existing unscoped default route");
                    let _ = self.routing_table.delete_route(&msg).await;
                } else if !old_route.is_ifscope() {
                    // NOTE: Skipping route
                    continue;
                }
            }

            log::debug!("Adding default route for tunnel");
            self.add_route_with_record(tunnel_route).await?;
        }

        Ok(())
    }

    /// Update/add routes that use the default non-tunnel interface. If some applied destination is
    /// a default route, this function replaces the non-tunnel default route with an ifscope route.
    async fn apply_non_tunnel_routes(&mut self) -> Result<()> {
        let v4_gateway = self
            .v4_default_route
            .as_ref()
            .and_then(|route| route.gateway())
            .cloned();
        let v6_gateway = self
            .v6_default_route
            .as_ref()
            .and_then(|route| route.gateway())
            .cloned();

        // Reapply routes that use the default (non-tunnel) node
        for dest in self.non_tunnel_routes.clone() {
            let gateway = if dest.is_ipv4() {
                v4_gateway
            } else {
                v6_gateway
            };
            let gateway = match gateway {
                Some(gateway) => gateway,
                None => continue,
            };
            let route =
                RouteMessage::new_route(Destination::Network(dest)).set_gateway_sockaddr(gateway);

            if let Some(dest) = self
                .applied_routes
                .keys()
                .find(|applied_dest| applied_dest.network == dest)
                .cloned()
            {
                let _ = self.routing_table.delete_route(&route).await;
                self.applied_routes.remove(&dest);
            }

            self.add_route_with_record(route).await?;
        }

        Ok(())
    }

    /// Replace a known default route with an ifscope route.
    async fn replace_with_scoped_route(&mut self, family: interface::Family) -> Result<()> {
        let Some(default_route) = get_current_best_default_route!(self, family) else {
            return Ok(());
        };

        let interface_index = default_route.interface_index();
        let new_route = default_route.clone().set_ifscope(interface_index);

        log::trace!("Setting ifscope: {new_route:?}");

        self.add_route_with_record(new_route).await
    }

    async fn add_route_with_record(&mut self, route: RouteMessage) -> Result<()> {
        let destination = RouteDestination::try_from(&route).map_err(Error::InvalidData)?;

        self.routing_table
            .add_route(&route)
            .await
            .map_err(Error::AddRoute)?;

        self.applied_routes.insert(destination, route);
        Ok(())
    }

    async fn cleanup_routes(&mut self) -> Result<()> {
        self.remove_applied_routes(|_| true).await;

        // We have already removed the applied default routes
        self.v4_tunnel_default_route = None;
        self.v6_tunnel_default_route = None;

        self.try_restore_default_routes().await;

        self.check_default_routes_restored = Self::create_default_route_check_timer();

        self.non_tunnel_routes.clear();

        Ok(())
    }

    /// Remove all applied routes for which `filter` returns true
    async fn remove_applied_routes(&mut self, filter: impl Fn(&RouteMessage) -> bool) {
        let mut deleted_routes = vec![];

        self.applied_routes.retain(|_dest, route| {
            if filter(route) {
                deleted_routes.push(route.clone());
                return false;
            }
            true
        });

        for route in deleted_routes {
            log::trace!("Removing route: {route:?}");
            match self.routing_table.delete_route(&route).await {
                Ok(_) | Err(watch::Error::RouteNotFound) | Err(watch::Error::Unreachable) => (),
                Err(err) => {
                    log::error!("Failed to remove relay route: {err:?}");
                }
            }
        }
    }

    /// FIXME: Hack. Restoring the default routes during cleanup sometimes fails, so repeatedly try
    /// until we have restored unscoped default routes. This function produces a timer for
    /// exponential backoff.
    fn create_default_route_check_timer() -> Pin<Box<dyn FusedStream<Item = ()> + Send>> {
        const RESTORE_HACK_INITIAL_INTERVAL: Duration = Duration::from_millis(500);
        const RESTORE_HACK_INTERVAL_MULTIPLIER: u32 = 5;
        const RESTORE_HACK_MAX_ATTEMPTS: u32 = 3;

        Box::pin(futures::stream::unfold(0, |attempt| async move {
            if attempt >= RESTORE_HACK_MAX_ATTEMPTS {
                return None;
            }

            let next_interval = RESTORE_HACK_INITIAL_INTERVAL
                * RESTORE_HACK_INTERVAL_MULTIPLIER.saturating_pow(attempt);
            tokio::time::sleep(next_interval).await;

            Some(((), attempt + 1))
        }))
    }

    /// Add back unscoped default routes, if they are still missing. This function returns
    /// true when no routes had to be added.
    async fn try_restore_default_routes(&mut self) -> bool {
        self.restore_default_route(interface::Family::V4).await
            && self.restore_default_route(interface::Family::V6).await
    }

    /// Add back unscoped default route for the given `family`, if it is still missing. This
    /// function returns true when no route had to be added.
    async fn restore_default_route(&mut self, family: interface::Family) -> bool {
        let Some(desired_default_route) = self.primary_interface_monitor.get_route(family) else {
            return true;
        };

        let current_default_route = RouteMessage::new_route(family.default_network().into());
        if let Ok(Some(current_default)) =
            self.routing_table.get_route(&current_default_route).await
        {
            // We're done if the route we're looking for is already here
            if route_matches_interface(&current_default, &desired_default_route) {
                return true;
            }
            let _ = self
                .routing_table
                .delete_route(&current_default_route)
                .await;
        };

        if let Err(error) = self.routing_table.add_route(&desired_default_route).await {
            log::trace!("Failed to add unscoped default {family} route: {error}");
        }

        self.update_trigger.trigger();

        false
    }
}

fn route_matches_interface(default_route: &RouteMessage, interface_route: &RouteMessage) -> bool {
    default_route.gateway_ip() == interface_route.gateway_ip()
        && default_route.interface_index() == interface_route.interface_index()
}

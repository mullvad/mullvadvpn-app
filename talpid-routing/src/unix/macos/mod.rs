use crate::{NetNode, Node, RequiredRoute, Route};

use futures::{
    channel::mpsc,
    future::FutureExt,
    stream::{FusedStream, StreamExt},
};
use ipnetwork::IpNetwork;
use nix::sys::socket::{AddressFamily, SockaddrLike, SockaddrStorage};
use std::pin::Pin;
use std::{
    collections::{BTreeMap, HashSet},
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
    FetchLinkAddresses(nix::Error),

    /// Received message isn't valid
    #[error(display = "Invalid data")]
    InvalidData(data::Error),

    /// Restoring unscoped default routes
    #[error(display = "Restoring unscoped default routes")]
    RestoringUnscopedRoutes,
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
    default_route_listeners: Vec<mpsc::UnboundedSender<DefaultRouteEvent>>,
    check_default_routes_restored: Pin<Box<dyn FusedStream<Item = ()> + Send>>,
}

impl RouteManagerImpl {
    /// Create new route manager
    #[allow(clippy::unused_async)]
    pub async fn new() -> Result<Self> {
        let routing_table = RoutingTable::new().map_err(Error::RoutingTable)?;
        Ok(Self {
            routing_table,
            non_tunnel_routes: HashSet::new(),
            v4_tunnel_default_route: None,
            v6_tunnel_default_route: None,
            applied_routes: BTreeMap::new(),
            v4_default_route: None,
            v6_default_route: None,
            default_route_listeners: vec![],
            check_default_routes_restored: Box::pin(futures::stream::pending()),
        })
    }

    pub(crate) async fn run(mut self, manage_rx: mpsc::UnboundedReceiver<RouteManagerCommand>) {
        let mut manage_rx = manage_rx.fuse();

        // Initialize default routes
        // NOTE: This isn't race-free, as we're not listening for route changes before initializing
        self.update_best_default_route(interface::Family::V4)
            .await
            .unwrap_or_else(|error| {
                log::error!(
                    "{}",
                    error.display_chain_with_msg("Failed to get initial default v4 route")
                );
            });
        self.update_best_default_route(interface::Family::V6)
            .await
            .unwrap_or_else(|error| {
                log::error!(
                    "{}",
                    error.display_chain_with_msg("Failed to get initial default v6 route")
                );
            });

        loop {
            futures::select_biased! {
                route_message = self.routing_table.next_message().fuse() => {
                    self.handle_route_message(route_message).await;
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

                command = manage_rx.next() => {
                    match command {
                        Some(RouteManagerCommand::Shutdown(tx)) => {
                            if let Err(err) = self.cleanup_routes().await {
                                log::error!("Failed to clean up routes: {err}");
                            }
                            let _ = tx.send(());
                            return;
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
                                    prefix: IpNetwork::from(interface::Family::V4),
                                    metric: None,
                                }
                            });
                            let v6_route = self.v6_default_route.as_ref().map(|route| {
                                Route {
                                    node: Node {
                                        device: None,
                                        ip: route.gateway_ip(),
                                    },
                                    prefix: IpNetwork::from(interface::Family::V6),
                                    metric: None,
                                }
                            });

                            let _ = tx.send((v4_route, v6_route));
                        }

                        Some(RouteManagerCommand::AddRoutes(routes, tx)) => {
                            if !self.check_default_routes_restored.is_terminated() {
                                // Give it some time to recover, but not too much
                                if !self.try_restore_default_routes().await {
                                    let _ = tokio::time::timeout(
                                        Duration::from_millis(500),
                                        self.check_default_routes_restored.next(),
                                    ).await;

                                    if !self.try_restore_default_routes().await {
                                        log::warn!("Unscoped routes were not restored");
                                        let _ = tx.send(Err(Error::RestoringUnscopedRoutes));
                                        continue;
                                    }
                                }
                                self.check_default_routes_restored = Box::pin(futures::stream::pending());
                                log::debug!("Unscoped routes were already restored");
                            }

                            log::debug!("Adding routes: {routes:?}");
                            let _ = tx.send(self.add_required_routes(routes).await);
                        }
                        Some(RouteManagerCommand::ClearRoutes) => {
                            if let Err(err) = self.cleanup_routes().await {
                                log::error!("Failed to clean up rotues: {err}");
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
        let interface_link_addrs = get_interface_link_addresses()?;

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

    async fn handle_route_message(
        &mut self,
        message: std::result::Result<RouteSocketMessage, watch::Error>,
    ) {
        match message {
            Ok(RouteSocketMessage::DeleteRoute(route)) => {
                // Forget about applied route, if relevant. This is simply prevent ourselves from
                // deleting it later.
                match RouteDestination::try_from(&route).map_err(Error::InvalidData) {
                    Ok(destination) => {
                        self.applied_routes.remove(&destination);
                    }
                    Err(err) => {
                        log::error!("Failed to process deleted route: {err}");
                    }
                }

                if let Err(error) = self.handle_route_socket_message().await {
                    log::error!("Failed to process route change: {error}");
                }
            }
            Ok(RouteSocketMessage::AddRoute(_))
            | Ok(RouteSocketMessage::ChangeRoute(_))
            | Ok(RouteSocketMessage::AddAddress(_) | RouteSocketMessage::DeleteAddress(_)) => {
                if let Err(error) = self.handle_route_socket_message().await {
                    log::error!("Failed to process route/address change: {error}");
                }
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
    async fn handle_route_socket_message(&mut self) -> Result<()> {
        self.update_best_default_route(interface::Family::V4)
            .await?;
        self.update_best_default_route(interface::Family::V6)
            .await?;

        // Substitute route with a tunnel route
        self.apply_tunnel_default_route().await?;

        // Update routes using default interface
        self.apply_non_tunnel_routes().await
    }

    /// Figure out what the best default routes to use are, and send updates to default route change
    /// subscribers. The "best routes" are used by the tunnel device to send packets to the VPN
    /// relay.
    ///
    /// If there is a tunnel device, the "best route" is the first ifscope default route found,
    /// ordered after network service order (after filtering out interfaces without valid IP
    /// addresses).
    ///
    /// If there is no tunnel device, the "best route" is the unscoped default route, whatever it
    /// is.
    async fn update_best_default_route(&mut self, family: interface::Family) -> Result<()> {
        let use_scoped_route = (family == interface::Family::V4
            && self.v4_tunnel_default_route.is_some())
            || (family == interface::Family::V6 && self.v6_tunnel_default_route.is_some());

        let best_route = if use_scoped_route {
            interface::get_best_default_route(&mut self.routing_table, family).await
        } else {
            interface::get_unscoped_default_route(&mut self.routing_table, family).await
        };
        log::trace!("Best route ({family:?}): {best_route:?}");

        let default_route = match family {
            interface::Family::V4 => &mut self.v4_default_route,
            interface::Family::V6 => &mut self.v6_default_route,
        };

        if default_route == &best_route {
            log::trace!("Default route ({family:?}) is unchanged");
            return Ok(());
        }

        let old_route = std::mem::replace(default_route, best_route);

        log::debug!(
            "Default route change ({family:?}): interface {} -> {}",
            old_route.map(|r| r.interface_index()).unwrap_or(0),
            default_route
                .as_ref()
                .map(|r| r.interface_index())
                .unwrap_or(0),
        );

        let changed = default_route.is_some();
        self.notify_default_route_listeners(family, changed);

        Ok(())
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

            // Replace the default route with an ifscope route
            self.set_default_route_ifscope(tunnel_route.is_ipv4(), true)
                .await?;

            // Make sure there is really no other unscoped default route
            let mut msg = RouteMessage::new_route(
                if tunnel_route.is_ipv4() {
                    IpNetwork::from(interface::Family::V4)
                } else {
                    IpNetwork::from(interface::Family::V6)
                }
                .into(),
            );
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

    /// Replace a known default route with an ifscope route, if should_be_ifscoped is true.
    /// If should_be_ifscoped is false, the route is replaced with a non-ifscoped default route
    /// instead.
    async fn set_default_route_ifscope(
        &mut self,
        ipv4: bool,
        should_be_ifscoped: bool,
    ) -> Result<()> {
        let default_route = match (ipv4, &mut self.v4_default_route, &mut self.v6_default_route) {
            (true, Some(default_route), _) | (false, _, Some(default_route)) => default_route,
            _ => {
                return Ok(());
            }
        };

        if default_route.is_ifscope() == should_be_ifscoped {
            return Ok(());
        }

        log::trace!("Setting non-ifscope: {default_route:?}");

        let interface_index = if should_be_ifscoped {
            let interface_index = default_route.interface_index();
            if interface_index == 0 {
                log::error!("Cannot find interface index of default interface");
            }
            interface_index
        } else {
            0
        };
        let new_route = default_route.clone().set_ifscope(interface_index);
        let old_route = std::mem::replace(default_route, new_route);

        self.routing_table
            .delete_route(&old_route)
            .await
            .map_err(Error::DeleteRoute)?;

        self.routing_table
            .add_route(default_route)
            .await
            .map_err(Error::AddRoute)
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
        // Remove all applied routes. This includes default destination routes
        let old_routes = std::mem::take(&mut self.applied_routes);
        for (_dest, route) in old_routes.into_iter() {
            log::trace!("Removing route: {route:?}");
            match self.routing_table.delete_route(&route).await {
                Ok(_) | Err(watch::Error::RouteNotFound) | Err(watch::Error::Unreachable) => (),
                Err(err) => {
                    log::error!("Failed to remove relay route: {err:?}");
                }
            }
        }

        // We have already removed the applied default routes
        self.v4_tunnel_default_route = None;
        self.v6_tunnel_default_route = None;

        self.try_restore_default_routes().await;

        self.check_default_routes_restored = Self::create_default_route_check_timer();

        self.non_tunnel_routes.clear();

        Ok(())
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
        let mut done = true;
        for family in [interface::Family::V4, interface::Family::V6] {
            let current_route = match family {
                interface::Family::V4 => &mut self.v4_default_route,
                interface::Family::V6 => &mut self.v6_default_route,
            };
            let message = RouteMessage::new_route(IpNetwork::from(family).into());
            done &= if matches!(self.routing_table.get_route(&message).await, Ok(Some(_))) {
                true
            } else {
                let new_route =
                    interface::get_best_default_route(&mut self.routing_table, family).await;
                let old_route = std::mem::replace(current_route, new_route);
                let notify = &old_route != current_route;

                let done = if let Some(route) = current_route {
                    let _ = std::mem::replace(route, route.clone().set_ifscope(0));
                    if let Err(error) = self.routing_table.add_route(route).await {
                        log::trace!("Failed to add non-ifscope {family} route: {error}");
                    }
                    false
                } else {
                    true
                };

                if notify {
                    let changed = current_route.is_some();
                    self.notify_default_route_listeners(family, changed);
                }

                done
            };
        }
        done
    }
}

/// Return a map from interface name to link addresses (AF_LINK)
fn get_interface_link_addresses() -> Result<BTreeMap<String, SockaddrStorage>> {
    let mut gateway_link_addrs = BTreeMap::new();
    let addrs = nix::ifaddrs::getifaddrs().map_err(Error::FetchLinkAddresses)?;
    for addr in addrs.into_iter() {
        if addr.address.and_then(|addr| addr.family()) != Some(AddressFamily::Link) {
            continue;
        }
        gateway_link_addrs.insert(addr.interface_name, addr.address.unwrap());
    }
    Ok(gateway_link_addrs)
}

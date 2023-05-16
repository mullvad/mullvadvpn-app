use crate::{
    imp::{imp::watch::data::RouteSocketMessage, RouteManagerCommand},
    NetNode, Node, RequiredRoute, Route,
};

use self::watch::{
    data::{Destination, RouteDestination, RouteMessage},
    RoutingTable,
};
use futures::{channel::mpsc, future::FutureExt, stream::StreamExt};
use ipnetwork::IpNetwork;
use nix::sys::socket::{AddressFamily, SockaddrLike, SockaddrStorage};
use std::{
    collections::{BTreeMap, HashSet},
    net::{Ipv4Addr, Ipv6Addr},
};
use talpid_types::ErrorExt;

use super::DefaultRouteEvent;

mod interface;
pub mod watch;

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
    InvalidData(watch::data::Error),
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
    default_destinations: HashSet<IpNetwork>,
    v4_tunnel_default_route: Option<watch::data::RouteMessage>,
    v6_tunnel_default_route: Option<watch::data::RouteMessage>,
    applied_routes: BTreeMap<RouteDestination, RouteMessage>,
    v4_default_route: Option<watch::data::RouteMessage>,
    v6_default_route: Option<watch::data::RouteMessage>,
    default_route_listeners: Vec<mpsc::UnboundedSender<DefaultRouteEvent>>,
}

impl RouteManagerImpl {
    /// create new route manager
    pub async fn new() -> Result<Self> {
        let routing_table = RoutingTable::new().map_err(Error::RoutingTable)?;
        Ok(Self {
            routing_table,
            default_destinations: HashSet::new(),
            v4_tunnel_default_route: None,
            v6_tunnel_default_route: None,
            applied_routes: BTreeMap::new(),
            v4_default_route: None,
            v6_default_route: None,
            default_route_listeners: vec![],
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
                                    prefix: v4_default(),
                                    metric: None,
                                }
                            });
                            let v6_route = self.v6_default_route.as_ref().map(|route| {
                                Route {
                                    node: Node {
                                        device: None,
                                        ip: route.gateway_ip(),
                                    },
                                    prefix: v6_default(),
                                    metric: None,
                                }
                            });

                            let _ = tx.send((v4_route, v6_route));
                        }

                        Some(RouteManagerCommand::AddRoutes(routes, tx)) => {
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
                    self.default_destinations.insert(route.prefix);
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
                        .set_gateway_sockaddr(link_addr.clone()),
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
        if let Err(error) = self.apply_default_destinations().await {
            self.default_destinations.clear();
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

                // We're ignoring default route removals. We instead add back the tunnel route
                // when a default route is added.
                match route.is_default().map_err(Error::InvalidData) {
                    Ok(true) => {
                        log::trace!("A default route was removed: {route:?}");

                        // TODO: may need to add back tunnel route if it does not exist

                        if let Err(error) = self.handle_route_change(route).await {
                            log::error!("Failed to process route change: {error}");
                        }
                    }
                    Ok(false) => (),
                    Err(error) => {
                        log::error!("Failed to check whether route is default route: {error}");
                    }
                }
            }

            Ok(RouteSocketMessage::AddRoute(route))
            | Ok(RouteSocketMessage::ChangeRoute(route)) => {
                // Refresh routes that are using the default interface
                if let Err(error) = self.handle_route_change(route).await {
                    log::error!("Failed to process route change: {error}");
                }
            }
            // ignore all other message types
            Ok(_) => {}
            Err(err) => {
                log::error!("Failed to receive a message from the routing table: {err}");
            }
        }
    }

    /// Update routes that use the non-tunnel default interface
    async fn handle_route_change(&mut self, route: watch::data::RouteMessage) -> Result<()> {
        // Ignore routes that aren't default routes
        if !route.is_default().map_err(Error::InvalidData)? {
            return Ok(());
        }

        let new_gateway_link_addr = route.gateway().and_then(|addr| addr.as_link_addr());

        // Ignore the new route if it is our tunnel route, lest we create a loop
        for tunnel_default_route in [&self.v4_tunnel_default_route, &self.v6_tunnel_default_route] {
            if let Some(tunnel_route) = tunnel_default_route.clone() {
                let tun_gateway_link_addr =
                    tunnel_route.gateway().and_then(|addr| addr.as_link_addr());

                if new_gateway_link_addr == tun_gateway_link_addr {
                    return Ok(());
                }
            }
        }

        self.update_best_default_route(if route.is_ipv4() {
            interface::Family::V4
        } else {
            interface::Family::V6
        })
        .await
    }

    async fn update_best_default_route(&mut self, family: interface::Family) -> Result<()> {
        let best_route = interface::get_best_default_route(&mut self.routing_table, family).await;
        // FIXME: remove logging
        log::debug!("Best route: {best_route:?}");

        let default_route = match family {
            interface::Family::V4 => &mut self.v4_default_route,
            interface::Family::V6 => &mut self.v6_default_route,
        };

        if default_route == &best_route {
            log::trace!("Default route is unchanged");
            return Ok(());
        }

        let old_route = std::mem::replace(default_route, best_route);

        log::debug!("New default route: {old_route:?} -> {default_route:?}");

        // Notify default route listeners
        let event = match (family, default_route.is_some()) {
            (interface::Family::V4, true) => DefaultRouteEvent::AddedOrChangedV4,
            (interface::Family::V6, true) => DefaultRouteEvent::AddedOrChangedV6,
            (interface::Family::V4, false) => DefaultRouteEvent::RemovedV4,
            (interface::Family::V6, false) => DefaultRouteEvent::RemovedV6,
        };
        self.default_route_listeners
            .retain(|tx| tx.unbounded_send(event).is_ok());

        // Substitute route with a tunnel route
        self.apply_tunnel_default_route().await?;

        // Update routes using default interface
        self.apply_default_destinations().await?;

        Ok(())
    }

    /// Replace the default routes with an ifscope route, and
    /// add a new default tunnel route.
    async fn apply_tunnel_default_route(&mut self) -> Result<()> {
        // As long as the relay route has a way of reaching the internet, we'll want to add a tunnel
        // route for both IPv4 and IPv6.
        // NOTE: This is incorrect. We're assuming that any "default destination" is used for
        // tunneling.
        let (v4_conn, v6_conn) = self
            .default_destinations
            .iter()
            .fold((false, false), |(v4, v6), route| {
                (v4 || route.is_ipv4(), v6 || route.is_ipv6())
            });
        let relay_route_is_valid = (v4_conn && self.v4_default_route.is_some())
            || (v6_conn && self.v6_default_route.is_some());

        for tunnel_route in [
            self.v4_tunnel_default_route.clone(),
            self.v6_tunnel_default_route.clone(),
        ] {
            let tunnel_route = match tunnel_route {
                Some(route) => route,
                None => return Ok(()),
            };

            let ((true, default_route, _) | (false, _, default_route)) = (
                tunnel_route.is_ipv4(),
                &mut self.v4_default_route,
                &mut self.v6_default_route,
            );
            match default_route {
                Some(_route) => (),
                None => {
                    if !relay_route_is_valid {
                        return Ok(());
                    }
                }
            }

            log::debug!("Adding default route for tunnel");

            // Replace the default route with an ifscope route
            self.set_default_route_ifscope(tunnel_route.is_ipv4(), true)
                .await?;
            self.add_route_with_record(tunnel_route).await?;
        }

        Ok(())
    }

    /// Update/add routes that use the default non-tunnel interface. If some applied destination is
    /// a default route, this function replaces the non-tunnel default route with an ifscope route.
    async fn apply_default_destinations(&mut self) -> Result<()> {
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
        for dest in self.default_destinations.clone() {
            let gateway = if dest.is_ipv4() {
                v4_gateway.clone()
            } else {
                v6_gateway.clone()
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

        // FIXME: remove logging
        log::debug!("Setting non-ifscope: {default_route:?}");

        let ifscope_index = if should_be_ifscoped {
            let n = default_route.interface_index();
            if n == 0 {
                log::error!("Cannot find interface index of default interface");
            }
            n
        } else {
            0
        };
        let new_route = default_route.clone().set_ifscope(ifscope_index);
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
        self.routing_table
            .add_route(&route)
            .await
            .map_err(Error::AddRoute)?;

        let destination = RouteDestination::try_from(&route).map_err(Error::InvalidData)?;

        self.applied_routes.insert(destination, route);
        Ok(())
    }

    async fn cleanup_routes(&mut self) -> Result<()> {
        // Remove all applied routes. This includes default destination routes
        let old_routes = std::mem::take(&mut self.applied_routes);
        for (_dest, route) in old_routes
            .into_iter()
        {
            // FIXME: remove logging
            log::debug!("Removing route: {route:?}");
            match self.routing_table.delete_route(&route).await {
                Ok(_) | Err(watch::Error::RouteNotFound) | Err(watch::Error::Unreachable) => (),
                Err(err) => {
                    log::error!("Failed to remove relay route: {err:?}");
                }
            }
        }

        // Reset default route
        if let Err(error) = self
            .set_default_route_ifscope(true, false)
            .await
            .and(self.set_default_route_ifscope(false, false).await)
        {
            log::error!("Failed to restore default routes: {error}");
        }

        // We have already removed the applied default routes
        self.v4_tunnel_default_route = None;
        self.v6_tunnel_default_route = None;

        self.default_destinations.clear();

        Ok(())
    }
}

fn v4_default() -> IpNetwork {
    IpNetwork::new(Ipv4Addr::UNSPECIFIED.into(), 0).unwrap()
}

fn v6_default() -> IpNetwork {
    IpNetwork::new(Ipv6Addr::UNSPECIFIED.into(), 0).unwrap()
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

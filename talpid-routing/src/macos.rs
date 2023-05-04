use crate::{
    imp::{imp::watch::data::{RouteSocketMessage}, RouteManagerCommand},
    NetNode, RequiredRoute, Route,
};

use futures::{
    channel::mpsc,
    future::{self, FutureExt},
    stream::{Stream, StreamExt, TryStreamExt},
};
use ipnetwork::IpNetwork;
use nix::sys::socket::{SockaddrLike, AddressFamily, SockaddrStorage};
use talpid_types::ErrorExt;
use std::{
    collections::{BTreeMap, HashSet},
    io,
    net::{Ipv4Addr, Ipv6Addr},
    process::Stdio,
};

use tokio::{io::AsyncBufReadExt, process::Command};
use tokio_stream::wrappers::LinesStream;

use self::{
    watch::{
        data::{Destination, RouteDestination, RouteMessage},
        RoutingTable,
    },
};

use super::DefaultRouteEvent;

mod ip6addr_ext;

pub mod watch;

pub type Result<T> = std::result::Result<T, Error>;

/// Errors that can happen in the macOS routing integration.
#[derive(err_derive::Error, Debug)]
#[error(no_from)]
pub enum Error {
    /// Failed to add route.
    #[error(display = "Failed to add route")]
    FailedToAddRoute(#[error(source)] watch::Error),

    /// Failed to add route via 'route' subcommand.
    #[error(display = "Failed to add route via subcommand")]
    FailedToAddRouteExec(#[error(source)] io::Error),

    /// Failed to remove route.
    #[error(display = "Failed to remove route")]
    FailedToRemoveRoute(#[error(source)] io::Error),

    /// Error while running "ip route".
    #[error(display = "Error while running \"route get\"")]
    FailedToRunRoute(#[error(source)] io::Error),

    /// Error while monitoring routes with `route -nv monitor`
    #[error(display = "Error while running \"route -nv monitor\"")]
    FailedToMonitorRoutes(#[error(source)] io::Error),

    /// Unexpected output from netstat
    #[error(display = "Unexpected output from netstat")]
    BadOutputFromNetstat,

    /// Encountered an error when interacting with the routing socket
    #[error(display = "Error occurred when interfacing with the routing table")]
    RoutingTable(#[error(source)] watch::Error),

    /// Unknown interface
    #[error(display = "Unknown interface: {}", _0)]
    UnknownInterface(String),

    /// Failed to remvoe route
    #[error(display = "Error occurred when deleting a route")]
    DeleteRoute(#[error(source)] watch::Error),

    /// Failed to change route
    #[error(display = "Failed to change route")]
    ChangeRoute(#[error(source)] watch::Error),

    /// Failed to add route
    #[error(display = "Error occurred when adding a route")]
    AddRoute(#[error(source)] watch::Error),

    /// Failed to fetch link addresses
    #[error(display = "Failed to fetch link addresses")]
    FetchLinkAddresses(nix::Error),

    /// Received message isn't valid
    #[error(display = "Invalid data")]
    InvalidData(watch::data::Error),

    /// Failed to resolve tunnel interface name to an interface index
    #[error(display = "Failed to find tunnel interface by name")]
    NoTunnelInterface,

    /// Gateway route has no IP
    #[error(display = "Gateway route has no gateway address")]
    NoGatewayAddress,

    /// Invalid gateway route
    #[error(display = "Received gateway route is invalid")]
    InvalidGatewayRoute(watch::data::RouteMessage),

    /// Failed to obtain interface indices
    #[error(display = "Failed to obtain list of interface names and indices")]
    GetInterfaceNames(nix::Error),

    /// Failed to find interface name
    #[error(display = "Failed to find name for interface")]
    GetInterfaceName,

    /// Failed to create route destination from route message
    #[error(display = "Failed to derive destination from route message")]
    RouteDestination(watch::data::Error),
}

#[derive(Clone, PartialEq)]
struct AppliedRoute {
    destination: watch::data::RouteDestination,
    route: RouteMessage,
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
    tunnel_default_route: Option<watch::data::RouteMessage>,
    applied_routes: BTreeMap<RouteDestination, AppliedRoute>,
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
            tunnel_default_route: None,
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
        self.v4_default_route = self.routing_table
            .get_route(v4_default())
            .await
            .unwrap_or_else(|error| {
                log::error!("{}", error.display_chain_with_msg("Failed to get initial default v4 route"));
                None
            });
        self.v6_default_route = self.routing_table
            .get_route(v6_default())
            .await
            .unwrap_or_else(|error| {
                log::error!("{}", error.display_chain_with_msg("Failed to get initial default v6 route"));
                None
            });

        loop {
            futures::select_biased! {
                route_message = self.routing_table.next_message().fuse() => {
                    // TODO: forward default route changes
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

                        Some(RouteManagerCommand::AddRoutes(routes, tx)) => {
                            log::debug!("FIXME: Adding routes: {routes:?}");
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
        // TODO: roll back changes if not all succeed?

        // FIXME: all routes need link addr? probably not

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
            // FIXME: handle non-link addrs

            if route.node.device.is_none() {
                log::debug!("FIXME: skipping route with no device: {route:?}");
                continue;
            }

            // If we specify route by interface name, use the link address of the given interface
            let device = route.node.device.clone().unwrap();
            let link_addr = interface_link_addrs.get(&device);

            if link_addr.is_none() {
                log::debug!("FIXME: route with unknown device: {route:?}, {device}");
                // FIXME: should probably fail
                continue;
            }

            let message = RouteMessage::new_route(Destination::from(route.prefix))
                .set_gateway_sockaddr(link_addr.cloned().unwrap());

            // Default routes are a special case: We must apply it after replacing the current
            // default route with an ifscope route.
            if route.prefix.prefix() == 0 {
                // TODO: simplify by just ifscoping existing route here?

                // FIXME: ignoring v6
                if route.prefix.is_ipv6() {
                    continue;
                }
                log::debug!("FIXME: Default tunnel route: {message:?}");

                //FIXME: remove: let cstr_device = CString::new(device).expect("FIXME: handle better");
                //FIXME: remove: let idx = unsafe { if_nametoindex(cstr_device.as_ptr()) };

                //FIXME: remove: log::debug!("FIXME: tun index: {idx}");

                self.tunnel_default_route = Some(message);
                continue;
            }

            // Add route
            self.add_route_with_record(message).await?;
        }

        if let Err(error) = self.apply_tunnel_default_route().await {
            log::debug!("FIXME: applied tunnel def FAILEd");
            return Err(error);
        }

        // Add routes that use the default interface
        if let Err(error) = self.apply_default_destinations().await {
            self.default_destinations.clear();
            log::debug!("FIXME: applied dests FAILEd");
            return Err(error);
        }

        log::debug!("FIXME: applied dests");

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

                // If the default route is deleted, just forget about it. We don't have to do anything else. FIXME: verify
                // FIXME: wrong? we need to add back tunnel route? maybe we can just do that when route is added
                let is_default_ip = route.is_default().map_err(Error::InvalidData);
                //let is_default_link = route.is_default_link().map_err(Error::InvalidData);
                let is_default_link = Ok(false);
                match is_default_ip.and_then(|default_ip| Ok(default_ip || is_default_link?)) {
                    Ok(true) => {
                        log::debug!("A default route was removed: {route:?}");

                        /*
                        // FIXME: if af_link, remove both ip routes

                        if let Some(gateway) = route.gateway() {
                            if let Some(link_addr) = gateway.as_link_addr() {
                                log::debug!("FIXME: AS LINK ADDR!");
                                
                                if let Some(tunnel_route) = self.tunnel_default_route {
                                    tunnel_route.gateway()
                                }
                            }
                        }

                        let ((true, default_route, _) | (false, _, default_route)) = (route.is_ipv4(), &mut self.v4_default_route, &mut self.v6_default_route);

                        if let Some(default) = default_route {
                            if route.is_ifscope() {
                                if route.gateway().is_none() || default.gateway() != route.gateway() {
                                    return;
                                }
                            } else {
                                // FIXME
                                return;
                            }

                            //std::mem::take(default_route);

                            // Notify default route listeners
                            //self.default_route_listeners.retain(|tx| tx.unbounded_send(DefaultRouteEvent::Removed).is_ok());

                            /*log::debug!("HERE IS THE CURRENT DEFAULT_ROUTE: {default:?}");
                            if route.is_ifscope() != default.is_ifscope() {
                                return;
                            }

                            if route.gateway().is_some() && default.gateway() == route.gateway() {
                                log::debug!("FIXME: CURRENT DEFAULT ROUTE WAS REMOVED");

                                // FIXME: is this handled properly?

                                // NOTE: Not making any changes to existing routes

                                std::mem::take(default_route);

                                // Notify default route listeners
                                self.default_route_listeners.retain(|tx| tx.unbounded_send(DefaultRouteEvent::Removed).is_ok());
                            }*/
                        }

                        */

                        if route.is_ifscope() {
                            // Actually kind of incorrect, since this matches against the tunnel interface
                            // I'm using tunnel interface removal as a proxy for no internet in that case
                            return;
                        }

                        let ((true, default_route, _) | (false, _, default_route)) = (route.is_ipv4(), &mut self.v4_default_route, &mut self.v6_default_route);
                        if std::mem::take(default_route).is_some() {
                            // Notify default route listeners
                            self.default_route_listeners.retain(|tx| tx.unbounded_send(DefaultRouteEvent::Removed).is_ok());
                        }
                    }
                    Ok(false) => {
                        //log::debug!("A non-default route was removed: {route:?}");
                    }
                    Err(error) => {
                        log::error!("Failed to check whether route is default route: {error}");
                    }
                }

                // prev note on "delete":
                // handle deletion of a route - only interested default route removals
                // or routes that were applied for our tunnel interface
            }

            Ok(RouteSocketMessage::AddRoute(route))
            | Ok(RouteSocketMessage::ChangeRoute(route)) => {
                // Refresh routes that are using the default interface
                // FIXME: detect default route changes
                log::debug!("A route was added/changed: {route:?}");

                if let Err(error) = self.handle_route_change(route).await {
                    log::error!("Failed to process route change: {error}");
                }

                // prev note on "add":
                // handle new route - if it's a default route, current best default
                // route should be updated. if it's a default route whilst engaged,
                // remove it, route the tunne traffic through it, and apply
            }
            // ignore all other message types
            Ok(_) => {}
            Err(err) => {
                log::error!("Failed to receive a message from the routing table: {err}");
            }
        }
    }

    /// Update routes that use the non-tunnel default interface
    async fn handle_route_change(
        &mut self,
        mut route: watch::data::RouteMessage,
    ) -> Result<()> {
        // Ignore routes that aren't default routes
        if !route.is_default().map_err(Error::InvalidData)? {
            log::debug!("FIXME: ignoring non-default route");
            return Ok(());
        }
    
        let new_gateway_link_addr = route.gateway().and_then(|addr| addr.as_link_addr());

        // Ignore the new route if it is our tunnel route, lest we create a loop
        if let Some(tunnel_route) = self.tunnel_default_route.clone() {
            let tun_gateway_link_addr = tunnel_route.gateway().and_then(|addr| addr.as_link_addr());

            if new_gateway_link_addr == tun_gateway_link_addr {
                log::debug!("FIXME: ignoring tunnel default route");
                return Ok(());
            }
        }

        let ((true, default_route, _) | (false, _, default_route)) = (route.is_ipv4(), &mut self.v4_default_route, &mut self.v6_default_route);

        // Ignore changes to ifscoped routes unless they affect the current default interface.
        if route.is_ifscope() {
            if let Some(default_route) = default_route {
                let default_gateway_link_addr = default_route.gateway().and_then(|addr| addr.as_link_addr());

                if new_gateway_link_addr != default_gateway_link_addr {
                    log::debug!("FIXME: ignoring non-default ifscoped route change");
                    return Ok(());
                }

                // We only care about the gateway IP changing
                // ignore other changes
            }

            log::debug!("FIXME: 'tis an ifscoped change");
            return Ok(()); // FIXME: don't ignore
        } else {
            // Get complete route
            // TODO: Is this overkill? We don't get the interface index without this
            let default_dest = if route.is_ipv4() {
                v4_default()
            } else {
                v6_default()
            };
            match self.routing_table.get_route(default_dest).await {
                Ok(Some(new_route)) => route = new_route,
                Ok(None) => {
                    log::warn!("Expected to find default route");
                    return Ok(());
                }
                Err(error) => {
                    log::error!("Failed to get default route");
                    return Err(Error::RoutingTable(error));
                }
            }
        }

        let new_route = Some(route);

        if &new_route != default_route {
            let old_route = std::mem::replace(default_route, new_route);
            log::debug!("New default route: {old_route:?} -> {default_route:?}");

            // Notify default route listeners
            self.default_route_listeners.retain(|tx| tx.unbounded_send(DefaultRouteEvent::AddedOrChanged).is_ok());

            // Substitute route with a tunnel route
            self.apply_tunnel_default_route().await?;

            // Update routes using default interface
            log::debug!("FIXME: Reapplying default destination routes");
            self.apply_default_destinations().await?;
        } else {
            log::debug!("FIXME: ignoring irrelevant default route: {new_route:?}");
        }

        Ok(())
    }

    /// Replace the default route with an ifscope route, and
    /// add a new default tunnel route.
    async fn apply_tunnel_default_route(&mut self) -> Result<()> {
        let tunnel_route = match self.tunnel_default_route.clone() {
            Some(route) => route,
            None => return Ok(()),
        };

        // Do nothing if the default route is already ifscoped or non-existent
        let ((true, default_route, _) | (false, _, default_route)) = (tunnel_route.is_ipv4(), &mut self.v4_default_route, &mut self.v6_default_route);
        match default_route {
            Some(route) if route.is_ifscope() => return Ok(()),
            None => return Ok(()),
            Some(_) => (),
        }

        log::debug!("Adding default route for tunnel");

        // Replace the default route with an ifscope route
        self.set_default_route_ifscope(tunnel_route.is_ipv4(), true).await?;
        let _ = self.routing_table.delete_route(&tunnel_route).await;
        self.add_route_with_record(tunnel_route).await?;

        log::debug!("FIXME: added replacement");

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
                None => {
                    log::debug!(
                        "FIXME: not adding default destination because there's no gateway addr"
                    );
                    continue;
                }
            };
            let route =
                RouteMessage::new_route(Destination::Network(dest)).set_gateway_sockaddr(gateway);

            // FIXME: don't needlessly delete. it might not even exist
            // TODO: only succeed if it fails because no exist? add_route_with_record could replace route on its own (if there's a record)

            // TODO: can we do better than linearly searching?
            if let Some(dest) = self.applied_routes.iter().find(|(applied_dest, _route)| applied_dest.network == dest).map(|(dest, _)| dest.clone()) {
                let _ = self.routing_table.delete_route(&route).await;

                self.applied_routes.remove(&dest);
            }

            log::debug!("FIXME: ADDING ROUTE: {dest:?} {route:?}");
            // FIXME: return
            let result = self.add_route_with_record(route).await;

            if result.is_err() {
                log::debug!("FIXME: addr error: {:?}", result);
            }

            result?;
        }

        Ok(())
    }

    /// Replace a known default route with an ifscope route, if should_be_ifscoped is true.
    /// If should_be_ifscoped is false, the route is replaced with a non-ifscoped default route
    /// instead.
    async fn set_default_route_ifscope(&mut self, ipv4: bool, should_be_ifscoped: bool) -> Result<()> {
        let default_route = match (ipv4, &mut self.v4_default_route, &mut self.v6_default_route) {
            ((true, Some(default_route), _) | (false, _, Some(default_route))) => default_route,
            _ => {
                log::debug!("FIXME: THERE IS NO DEFAULT ROUTE. IGNORING");
                return Ok(());
            }
        };

        if default_route.is_ifscope() == should_be_ifscoped {
            log::debug!("FIXME: PREF NON-TUNNEL DEFAULT ROUTE IS ALREADY SET TO DESIRED IFSCOPED STATE. IGNORING");
            return Ok(());
        }
        
        //FIXME
        let ifscope_index = if should_be_ifscoped {
            let n = default_route.interface_index();
            if n == 0 {
                log::warn!("Cannot find interface index of default interface");
            }
            n
        } else {
            0
        };
        /*let ifscope_index = if should_be_ifscoped {
            default_route.interface_sockaddr_index().unwrap_or_else(|| {
                log::warn!("Cannot find interface index of default interface");
                0
            })
        } else {
            0
        };*/
        let new_route = default_route.clone().set_ifscope(ifscope_index);
        let old_route = std::mem::replace(default_route, new_route);

        log::debug!("FIXME: Replacing default route with ifscope route (or vice versa): {old_route:?} -> {default_route:?}");

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
        let result = self
            .routing_table
            .add_route(&route)
            .await
            .map_err(Error::AddRoute);

        // FIXME: just erturn
        if result.is_err() {
            log::debug!("FIXME: failed to add route: {result:?}");
        }

        let destination = RouteDestination::try_from(&route).map_err(Error::InvalidData)?;

        self.applied_routes
            .insert(destination.clone(), AppliedRoute { destination, route });
        Ok(())
    }

    async fn cleanup_routes(&mut self) -> Result<()> {
        // Remove all applied routes. This includes default destination routes
        let old_routes = std::mem::take(&mut self.applied_routes);
        for (dest, route) in old_routes.into_iter().map(|(dest, route)| (dest, route.route)) {
            log::debug!("FIXME: deleting route: {route:?}");
            log::debug!("FIXME: deleting DEST: {:?} {:?}, {:?}", dest.gateway, dest.interface, dest.network);

            match self.routing_table.delete_route(&route).await {
                Ok(_) | Err(watch::Error::RouteNotFound) | Err(watch::Error::Unreachable) => (),
                Err(err) => {
                    log::error!("Failed to remove relay route: {err:?}");
                }
            }
        }

        // Reset default route
        if let Err(error) = self.set_default_route_ifscope(true, false).await.and(self.set_default_route_ifscope(false, false).await) {
            log::error!("Failed to restore default routes: {error}");
        }

        // We have already removed the applied default route
        self.tunnel_default_route = None;

        self.default_destinations.clear();

        //self.v4_default_route = None;
        //self.v6_default_route = None;

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

        log::debug!("FIXME: enum addrs, {}", addr.interface_name);

        gateway_link_addrs.insert(addr.interface_name, addr.address.unwrap());
    }
    Ok(gateway_link_addrs)
}

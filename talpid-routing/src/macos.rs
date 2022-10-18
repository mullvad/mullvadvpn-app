use crate::{
    imp::{imp::watch::data::RouteSocketMessage, RouteManagerCommand},
    NetNode, Node, RequiredRoute, Route,
};

use futures::{
    channel::mpsc,
    future::{self, FutureExt},
    stream::{FusedStream, Stream, StreamExt, TryStreamExt},
};
use ipnetwork::{IpNetwork, Ipv4Network, Ipv6Network};
use nix::ifaddrs;
use std::{
    collections::{BTreeMap, BTreeSet, HashSet},
    io,
    net::{IpAddr, Ipv4Addr, Ipv6Addr},
    process::{ExitStatus, Stdio},
    time::Duration,
};
use system_configuration::{
    core_foundation::string::CFString,
    network_configuration::{SCNetworkService, SCNetworkSet},
    preferences::SCPreferences,
};

use talpid_time::Instant;
use talpid_types::net::IpVersion;
use tokio::{io::AsyncBufReadExt, process::Command};
use tokio_stream::wrappers::LinesStream;

use self::{
    interfaces::{RouteValidity, BestRoute},
    watch::{
        data::{self, AddressMessage, Destination, RouteDestination, RouteMessage},
        RoutingTable,
    },
};

mod ip6addr_ext;
use ip6addr_ext::IpAddrExt;

use super::{TunnelRoutesV4, TunnelRoutesV6};

mod interfaces;
mod route_watch;
pub mod watch;
use interfaces::Interfaces;

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

    /// Failed to run scutil
    #[error(display = "Failed to run scutil command")]
    ScUtilCommand,

    /// Encountered unexpected output from scutil
    #[error(display = "Unexpected scutil output")]
    ScUtilUnexpectedOutput,

    /// Encountered an error when interacting with the routing socket
    #[error(display = "Error occured when interfaceing with the routing table")]
    RoutingTable(watch::Error),

    /// Unknown interface
    #[error(display = "Unknown interface: {}", _0)]
    UnkownInterface(String),

    /// Failed to remvoe route
    #[error(display = "Error occured when deleting a route")]
    DeleteRoute(watch::Error),

    /// Failed to change route
    #[error(display = "Failed to change route")]
    ChangeRoute(watch::Error),

    /// Failed to add route
    #[error(display = "Error occured when adding a route")]
    AddRoute(watch::Error),

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

pub async fn get_default_routes() -> std::result::Result<bool, watch::Error> {
    let mut routing_table = RoutingTable::new()?;
    Ok(routing_table
        .get_route(v4_default())
        .await?
        .or(routing_table.get_route(v6_default()).await?)
        .is_some())
}

impl Error {
    fn is_delete_err(&self) -> bool {
        matches!(&self, Error::DeleteRoute(_))
    }

    fn is_add_err(&self) -> bool {
        matches!(&self, Error::AddRoute(_))
    }
}

#[derive(Clone, PartialEq)]
struct AppliedRoute {
    destination: watch::data::RouteDestination,
    route: RouteMessage,
}

impl AppliedRoute {
    fn uses(&self, route: &watch::data::RouteMessage) -> bool {
        // // unimplemented!()
        // self.route.ga == route.gateway_ip()
        //     && self
        //         .route.interface_index()
        //         .and_then(|iface| RouteManagerImpl::get_interface_index(iface))
        //         == route.interface_index().unwrap_or(Some(0))
        false
    }
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
    v4_gateway: Option<watch::data::RouteMessage>,
    v6_gateway: Option<watch::data::RouteMessage>,
    interface_map: BTreeMap<u16, ifaddrs::InterfaceAddress>,
    applied_interface: Option<AppliedInterface>,
    changed_default_routes: BTreeSet<ifaddrs::InterfaceAddress>,
    // required_routes: BTreeMap<RouteDestination, >
    applied_routes: BTreeMap<RouteDestination, AppliedRoute>,
    interfaces: Interfaces,
    unsatisifed_routes: BTreeSet<IpNetwork>,
    v4_default_route_check: Option<tokio::time::Sleep>,
    v6_default_route_check: Option<tokio::time::Sleep>,
}

struct GatewayInterface {
    route_msg: watch::data::RouteMessage,
    interface_address: ifaddrs::InterfaceAddress,
}

struct AppliedInterface {
    index: u16,
    tunnel_routes_v4: TunnelRoutesV4,
    tunnel_routes_v6: Option<TunnelRoutesV6>,
    relay_address: IpAddr,
}

struct PrimaryIfaceBackupV4 {
    interface: String,
    gateway: Ipv4Addr,
    address: Ipv4Addr,
}

struct PrimaryIfaceBackupV6 {
    interface: String,
    gateway: Ipv6Addr,
    address: Ipv6Addr,
}

struct BestV4Interface {
    idx: Option<usize>,
    gateway: Option<Ipv4Addr>,
}

struct BestV6Interface {
    idx: Option<usize>,
    gateway: Option<Ipv6Addr>,
}

impl RouteManagerImpl {
    /// create new route manager
    pub async fn new(_required_routes: HashSet<RequiredRoute>) -> Result<Self> {
        let mut routing_table = RoutingTable::new().map_err(Error::RoutingTable)?;

        let v4_gateway = routing_table
            .get_route(v4_default())
            .await
            .map_err(Error::RoutingTable)?;

        let v6_gateway = routing_table
            .get_route(v6_default())
            .await
            .map_err(Error::RoutingTable)?;

        let manager = Self {
            unsatisifed_routes: BTreeSet::new(),
            routing_table,
            applied_routes: BTreeMap::new(),
            applied_interface: None,
            interface_map: Self::collect_interfaces()?,
            changed_default_routes: BTreeSet::new(),
            v4_gateway,
            v4_default_route_check: None,
            v6_gateway,
            v6_default_route_check: None,
            interfaces: Interfaces::new(),
        };

        Ok(manager)
    }

    fn collect_interfaces() -> Result<BTreeMap<u16, ifaddrs::InterfaceAddress>> {
        Ok(nix::ifaddrs::getifaddrs()
            .map_err(Error::FetchLinkAddresses)?
            .filter(|iface| iface.interface_name != "lo0")
            .filter_map(|iface: ifaddrs::InterfaceAddress| {
                // forcing the interface index to be a 'usize' is an incredibly questionable
                // design choice made in the `nix`
                let ifindex = Self::get_interface_index(&iface)?;
                Some((ifindex, iface))
            })
            .collect::<BTreeMap<_, _>>())
    }

    pub(crate) async fn run(mut self, manage_rx: mpsc::UnboundedReceiver<RouteManagerCommand>) {
        let mut manage_rx = manage_rx.fuse();

        loop {
            futures::select_biased! {
                route_message = self.routing_table.next_message().fuse() => {
                    self.handle_route_mesage(route_message).await;
                }

                command = manage_rx.next() => {
                    match command {
                        Some(RouteManagerCommand::SetupTunnelRoutes {
                            tunnel_interface,
                            relay_address,
                            tunnel_routes_v4,
                            tunnel_routes_v6,
                            response_tx
                        }) => {
                            let result = self.setup_tunnel_routes(tunnel_interface,
                                                                  relay_address,
                                                                  tunnel_routes_v4,
                                                                  tunnel_routes_v6,
                                                                  ).await;
                            if result.is_err() {
                                if let Err(err) = self.cleanup_routes().await {
                                    log::error!("Failed to restore routes {err}");
                                }
                            }
                            let _ = response_tx.send(result);
                        },
                        Some(RouteManagerCommand::Shutdown(tx)) => {
                            if let Err(err) = self.cleanup_routes().await {
                                log::error!("Failed to clean up routes: {err}");
                            }
                            let _ = tx.send(());
                            return;
                        },

                        Some(RouteManagerCommand::AddRoutes(routes, tx)) => {
                            let _ = tx.send(Ok(()));
                        }
                        Some(RouteManagerCommand::ClearRoutes) => {
                            if let Err(err) = self.cleanup_routes().await {
                                log::error!("Failed to clean up rotues: {err}");
                            }
                            self.applied_interface = None;
                        },
                        None => {
                            break;
                        }
                    }
                },
            };
        }

        if let Err(err) = self.cleanup_routes().await {
            log::error!("Failed to clean up routing table when shutitng down: {err}");
        }
    }

    async fn handle_route_mesage(
        &mut self,
        message: std::result::Result<RouteSocketMessage, watch::Error>,
    ) {
        match message {
            Ok(RouteSocketMessage::Interface(interface)) => {
                // handle changes in interfaces, possibly recollect all interfaces
                self.handle_interface_change(interface).await;
            }

            Ok(RouteSocketMessage::AddAddress(address)) => {
                self.handle_add_address(address).await;
            }
            Ok(RouteSocketMessage::DeleteAddress(address)) => {
                self.handle_delete_address(address).await;
            }
            Ok(RouteSocketMessage::DeleteRoute(route)) => {
                self.handle_deleted_route(route).await;
                // handle deletion of a route - only interested default route removals
                // or routes that were applied for our tunnel interface
            }

            Ok(RouteSocketMessage::AddRoute(route)) => {
                self.handle_added_route(route).await;
                // handle new route - if it's a default route, current best default
                // route should be updated. if it's a default route whilst engaged,
                // remove it, route the tunne traffic through it, and apply
            }

            Ok(RouteSocketMessage::ChangeRoute(route)) => {
                self.handle_changed_route(route).await;
            }
            // ignore all other message types
            Ok(_) => {}
            Err(err) => {
                log::error!("Failed to receive a message from the routing table: {err}");
            }
        }
    }

    async fn handle_deleted_route(&mut self, route: watch::data::RouteMessage) {
        match RouteDestination::try_from(&route).map_err(Error::InvalidData) {
            Ok(destination) => {
                self.applied_routes.remove(&destination);
            }
            Err(err) => {
                log::error!("Failed to process deleted route: {}", err);
            }
        }
    }

    async fn handle_added_route(&mut self, route: watch::data::RouteMessage) {
        if let Err(err) = self.handle_added_route_inner(route).await {
            log::error!("Failed to process an added route: {}", err);
        }
    }

    async fn handle_added_route_inner(&mut self, route: watch::data::RouteMessage) -> Result<()> {
        let updated_interface = self.interfaces.handle_add_route(&route)?;
        if let Some(tunnel) = &self.applied_interface {
            if updated_interface {
                match self.try_update_best_interface(tunnel.relay_address).await {
                    Ok(true) => return Ok(()),
                    Ok(false) => {
                        // TODO: enter offline state
                    }
                    Err(_err) => {
                        // TODO: consider removing routes here
                    }
                }
            }
        }

        Ok(())
    }

    fn get_interface_index(iface: &ifaddrs::InterfaceAddress) -> Option<u16> {
        Some(iface.address?.as_link_addr()?.ifindex().try_into().unwrap())
    }

    async fn update_tracked_routes(
        &mut self,
        old_route: watch::data::RouteMessage,
        new_route: &watch::data::RouteMessage,
    ) -> Result<()> {
        let old_interface = self.get_interface_for_route(&old_route);
        let old_gateway = old_route.gateway_ip();
        let interface = self.get_interface_for_route(new_route);
        let gateway = new_route.gateway_ip();

        for (destination, applied_route) in &mut self.applied_routes {
            if applied_route.uses(&old_route) {}
        }
        Ok(())
    }

    async fn drain_unsatisifed_routes(
        &mut self,
        new_route: &watch::data::RouteMessage,
    ) -> Result<()> {
        let new_route_is_ipv4 = new_route
            .destination_ip()
            .map_err(Error::InvalidData)?
            .is_ipv4();

        let gateway = new_route.gateway_ip();
        let interface = self.get_interface_for_route(&new_route);

        let satisfieable_destinations = self
            .unsatisifed_routes
            .iter()
            .filter(|destination| new_route_is_ipv4 == destination.is_ipv4())
            .cloned()
            .collect::<Vec<_>>();

        for destination in satisfieable_destinations {
            let mut route = RouteMessage::new_route(destination.into());
            if let Some(gateway) = gateway {
                route = route.set_gateway_addr(gateway);
            }

            if let Some(interface) = &interface {
                route = route.set_interface_addr(interface);
            }

            self.add_route_with_record(destination, route).await?;
            self.unsatisifed_routes.remove(&destination);
        }

        Ok(())
    }

    async fn handle_changed_route(&mut self, route: watch::data::RouteMessage) {
        if let Err(err) = self.handle_changed_route_inner(route).await {
            log::error!("Failed to process route change: {err}");
        }
    }

    async fn handle_changed_route_inner(&mut self, route: watch::data::RouteMessage) -> Result<()> {
        if self.interfaces.handle_changed_route(&route)? {
            self.refresh_routes().await?;
        }

        Ok(())
    }

    /// Used to refresh routes when routes should be tracked.
    async fn refresh_routes(&mut self) -> Result<()> {
        if let Some(applied_routes) = &self.applied_interface {}
        Ok(())
    }

    async fn add_route_through_default_interface(&mut self, route: RequiredRoute) -> Result<()> {
        let gateway = match (route.prefix, &self.v4_gateway, &self.v6_gateway) {
            (IpNetwork::V4(_), Some(gateway_route), _)
            | (IpNetwork::V6(_), _, Some(gateway_route)) => gateway_route.gateway(),
            _ => {
                log::error!("UNSATISFIABLE ROUTE");
                self.unsatisifed_routes.insert(route.prefix);
                return Ok(());
            }
        };

        match gateway {
            Some(gateway_addr) => {
                let new_route = RouteMessage::new_route(route.prefix.into())
                    .set_gateway_sockaddr(gateway_addr.clone());

                self.add_route_with_record(route.prefix, new_route).await
            }
            None => {
                log::debug!("Gateway route has no gateway IP address");
                self.unsatisifed_routes.insert(route.prefix);
                Ok(())
            }
        }
    }

    async fn add_route_with_record(
        &mut self,
        destination: IpNetwork,
        route: RouteMessage,
    ) -> Result<()> {
        let _ = self
            .routing_table
            .add_route(&route)
            .await
            .map_err(Error::RoutingTable)?;

        let destination = RouteDestination::try_from(&route).map_err(Error::InvalidData)?;

        self.applied_routes
            .insert(destination.clone(), AppliedRoute { destination, route });
        Ok(())
    }

    async fn add_faux_default_routes_v4(
        &mut self,
        tunnel_routes: super::TunnelRoutesV4,
    ) -> Result<()> {
        for half in v4_faux_destinations() {
            let route = RouteMessage::new_route(half.into())
                .set_gateway_addr(tunnel_routes.tunnel_gateway.into());
            self.add_route_with_record(half, route).await?;
        }

        Ok(())
    }

    async fn setup_v4_default_route(
        &mut self,
        v4_routes: &super::TunnelRoutesV4,
        _interface: &ifaddrs::InterfaceAddress,
    ) -> Result<()> {
        if let Some(v4_route) = self.v4_gateway.clone() {
            if !v4_route.is_ifscope() {
                if let Err(route_err) = self.ifscope_route(&v4_route).await {
                    if route_err.is_add_err() {
                        if let Err(err) = self.restore_default_v4().await {
                            log::error!("Failed to restore v4 routes {err}");
                        }
                    }
                    return Err(route_err);
                }
            }
        }

        let default_route = RouteMessage::new_route(Destination::default_v4())
            .set_gateway_addr(v4_routes.tunnel_gateway.into());

        self.routing_table
            .add_route(&default_route)
            .await
            .map_err(Error::AddRoute)?;

        Ok(())
    }

    async fn setup_v6_default_route(
        &mut self,
        _tunnel_interface: &ifaddrs::InterfaceAddress,
        gateway: Ipv6Addr,
    ) -> Result<()> {
        if let Some(v6_route) = self.v6_gateway.clone() {
            if let Err(route_err) = self.ifscope_route(&v6_route).await {
                if route_err.is_add_err() {
                    if let Err(err) = self.restore_default_v6().await {
                        log::error!("Failed to restore v6 routes {err}");
                    }
                }
                return Err(route_err);
            }
        }

        let default_route =
            RouteMessage::new_route(Destination::default_v4()).set_gateway_addr(gateway.into());

        let _ = self
            .routing_table
            .add_route(&default_route)
            .await
            .map_err(Error::AddRoute)?;
        Ok(())
    }

    async fn add_faux_default_routes_v6(
        &mut self,
        tunnel_routes: super::TunnelRoutesV6,
    ) -> Result<()> {
        for half in v6_faux_destinations() {
            let route = RouteMessage::new_route(half.into())
                .set_gateway_addr(tunnel_routes.tunnel_gateway.into());
            self.add_route_with_record(half, route).await?
        }

        Ok(())
    }

    //     async fn setup_v4_default_route(
    //         &mut self,
    //         tunnel_interface: &InterfaceAddress,
    //         gateway: Ipv4Addr,
    //     ) -> Result<()> {
    //         let v4_default_destination: IpNetwork =
    //             IpNetwork::V4(Ipv4Network::new(Ipv4Addr::UNSPECIFIED, 0).unwrap());
    //         if let Some(v4_gateway) = &self.v4_gateway {
    //             let real_interface = self.get_interface_for_route(&v4_gateway);
    //             let gateway_addr = v4_gateway.gateway_v4();
    //             let _ = self
    //                 .routing_table
    //                 .delete_route(
    //                     v4_default_destination,
    //                     real_interface.as_ref(),
    //                     v4_gateway.is_ifscoped().map_err(Error::InvalidData)?,
    //                 )
    //                 .await
    //                 .map_err(Error::RoutingTable)?;
    //             let _ = self
    //                 .routing_table
    //                 .add_route(
    //                     v4_default_destination,
    //                     gateway_addr.map(Into::into),
    //                     real_interface.as_ref(),
    //                     true,
    //                 )
    //                 .await
    //                 .map_err(Error::RoutingTable)?;
    //         }

    //         let _ = self
    //             .routing_table
    //             .add_route(
    //                 v4_default_destination,
    //                 Some(gateway.into()),
    //                 Some(tunnel_interface),
    //                 false,
    //             )
    //             .await
    //             .map_err(Error::RoutingTable)?;
    //         Ok(())
    //     }

    // async fn setup_v6_default_route(
    //     &mut self,
    //     tunnel_interface: &InterfaceAddress,
    //     gateway: Ipv6Addr,
    // // ) -> Result<()> {
    //     let v6_default_destination: IpNetwork =
    //         IpNetwork::V6(Ipv6Network::new(Ipv6Addr::UNSPECIFIED, 0).unwrap());
    //     if let Some(v6_gateway) = &self.v6_gateway {
    //         let real_interface = self.get_interface_for_route(&v6_gateway);
    //         let gateway_addr = v6_gateway.gateway_v4();
    //         let _ = self
    //             .routing_table
    //             .delete_route(
    //                 v6_default_destination,
    //                 real_interface.as_ref(),
    //                 v6_gateway.is_ifscoped().map_err(Error::InvalidData)?,
    //             )
    //             .await
    //             .map_err(Error::RoutingTable)?;
    //         let _ = self
    //             .routing_table
    //             .add_route(
    //                 v6_default_destination,
    //                 gateway_addr.map(Into::into),
    //                 real_interface.as_ref(),
    //                 true,
    //             )
    //             .await
    //             .map_err(Error::RoutingTable)?;
    //     }

    //     let _ = self
    //         .routing_table
    //         .add_route(
    //             v6_default_destination,
    //             Some(gateway.into()),
    //             Some(tunnel_interface),
    //             false,
    //         )
    //         .await
    //         .map_err(Error::RoutingTable)?;
    //     Ok(())
    // }

    async fn update_v6_relay_route(
        &mut self,
        new_default_route: watch::data::RouteMessage,
    ) -> Result<()> {
        if let Some(relay_addr) = self.get_v6_relay_addr() {
            let gateway = new_default_route.gateway_v6();
            let interface_addrs = self.get_interface_for_route(&new_default_route);
            // self.routing_table
            //     .change_route(
            //         IpAddr::from(relay_addr).into(),
            //         gateway.map(Into::into),
            //         interface_addrs.as_ref(),
            //         new_default_route
            //             .is_ifscoped()
            //             .map_err(Error::InvalidData)?,
            //     )
            //     .await
            //     .map_err(Error::RoutingTable)?;
        }
        Ok(())
    }

    async fn update_v4_relay_route(
        &mut self,
        new_default_route: watch::data::RouteMessage,
    ) -> Result<()> {
        if let Some(relay_addr) = self.get_v4_relay_addr() {
            // let gateway = new_default_route.gateway_v4();
            // let interface_addrs = self.get_interface_for_route(&new_default_route);
            // self.routing_table
            //     .change_route(
            //         IpAddr::from(relay_addr).into(),
            //         gateway.map(Into::into),
            //         interface_addrs.as_ref(),
            //         new_default_route
            //             .is_ifscoped()
            //             .map_err(Error::InvalidData)?,
            //     )
            //     .await
            //     .map_err(Error::RoutingTable)?;
        }
        Ok(())
    }

    fn get_v4_relay_addr(&self) -> Option<Ipv4Addr> {
        if let Some(tunnel) = &self.applied_interface {
            if let IpAddr::V4(addr) = tunnel.relay_address {
                return Some(addr);
            }
        }
        None
    }

    fn get_v6_relay_addr(&self) -> Option<Ipv6Addr> {
        if let Some(tunnel) = &self.applied_interface {
            if let IpAddr::V6(addr) = tunnel.relay_address {
                return Some(addr);
            }
        }
        None
    }

    fn route_change_relevant(&self, route: &watch::data::RouteMessage) -> Result<bool> {
        // If route is non-default, it should be disregarded.
        if !route.is_default().map_err(Error::InvalidData)? {
            return Ok(false);
        }
        // If the default route is changed on our interface, it doesn't matter - if it was removed,
        // the correct route will be re-applied when
        // TODO: consider adding a timer to check if a default route was added later
        if Some(route.interface_index()) == self.applied_interface.as_ref().map(|iface| iface.index)
        {
            return Ok(false);
        }

        Ok(true)
    }

    async fn handle_interface_change(&mut self, interface: data::Interface) {
        let interfaces_changed = match self.interfaces.handle_iface_msg(interface) {
            Ok(interface_changed) => interface_changed,
            Err(err) => {
                log::error!("Failed to handle interface change: {err:?}");
                return;
            }
        };

        if interfaces_changed {}

        // TODO: recalculate default route here, if necessary
    }

    async fn restore_default_v4(&mut self) -> Result<()> {
        if self.applied_interface.is_some() {
            if let Some(route) = self.v4_gateway.clone() {
                self.restore_gateway_routes(&route).await?;
            }
        }

        Ok(())
    }

    async fn restore_gateway_routes(&mut self, gateway_route: &RouteMessage) -> Result<()> {
        if !gateway_route.is_ifscope() {
            let ifscoped_route = gateway_route
                .clone()
                .set_ifscope(gateway_route.interface_index());
            if let Err(err) = self
                .routing_table
                .delete_route(&ifscoped_route)
                .await
                .map_err(Error::DeleteRoute)
            {
                log::error!("Failed to remove ifscoped route: {err}");
            }

            let old_route = gateway_route.clone().set_ifscope(0);
            self.routing_table
                .add_route(&old_route)
                .await
                .map_err(Error::AddRoute)?;
        }
        Ok(())
    }

    async fn restore_default_v6(&mut self) -> Result<()> {
        if self.applied_interface.is_some() {
            if let Some(route) = &self.v6_gateway.clone() {
                self.restore_gateway_routes(&route).await?;
            }
        }
        Ok(())
    }

    /// Setup routes specifically for a tunnel
    async fn setup_tunnel_routes(
        &mut self,
        tunnel_interface: String,
        relay_address: IpAddr,
        tunnel_routes_v4: super::TunnelRoutesV4,
        tunnel_routes_v6: Option<super::TunnelRoutesV6>,
    ) -> Result<()> {
        let (index, interface) = self
            .resolve_interface_name(&tunnel_interface)
            .ok_or(Error::NoTunnelInterface)?;
        self.setup_v4_default_route(&tunnel_routes_v4, &interface)
            .await?;
        self.add_faux_default_routes_v4(tunnel_routes_v4).await?;

        if let Some(v6) = tunnel_routes_v6 {
            self.setup_v6_default_route(&interface, v6.tunnel_gateway)
                .await?;
            self.add_faux_default_routes_v6(v6).await?;
        }

        self.applied_interface = Some(AppliedInterface {
            index,
            relay_address,
            tunnel_routes_v4,
            tunnel_routes_v6,
        });

        Ok(())
    }

    /// Removes a route and adds the same route, but ifscoped. Maybe this can be done by just
    /// changing the route - haven't tested but I don't believe so, since the ifscope flag is used
    /// to identify a route.
    async fn ifscope_route(&mut self, original_route: &watch::data::RouteMessage) -> Result<()> {
        let interface_index = original_route.interface_index();
        log::error!("iface index {interface_index} original route {original_route:?}");
        let ifscoped_route = original_route.clone().set_ifscope(interface_index);

        self.routing_table
            .delete_route(original_route)
            .await
            .map_err(Error::DeleteRoute)?;

        self.routing_table
            .add_route(&ifscoped_route)
            .await
            .map_err(Error::AddRoute)?;

        Ok(())
    }

    fn get_interface_for_route(
        &self,
        route: &watch::data::RouteMessage,
    ) -> Option<ifaddrs::InterfaceAddress> {
        let idx = route.interface_index();
        self.interface_map.get(&idx).cloned()
    }

    fn resolve_interface_name(&self, name: &str) -> Option<(u16, ifaddrs::InterfaceAddress)> {
        self.interface_map
            .iter()
            .find(|(_idx, interface)| interface.interface_name == name)
            .map(|(idx, interface)| (*idx, interface.clone()))
    }

    async fn cleanup_routes(&mut self) -> Result<()> {
        self.cleanup_relay_routes().await;
        log::error!("CLEANED UP RELAY");
        let v4_default = self.restore_default_v4().await;
        log::error!("CLEANED UP v4");
        let v6_default = self.restore_default_v6().await;
        log::error!("CLEANED UP v6");
        v4_default.and(v6_default)
    }

    async fn cleanup_relay_routes(&mut self) {
        let old_routes = std::mem::replace(&mut self.applied_routes, BTreeMap::new());
        let mut routes_to_delete = old_routes
            .into_iter()
            .map(|(_, route)| route.route)
            .collect::<Vec<_>>();

        if let Some(iface) = &self.applied_interface {
            for v4_dest in v4_faux_destinations().chain(std::iter::once(v4_default())) {
                let route = RouteMessage::new_route(v4_dest.into())
                    .set_gateway_addr(iface.tunnel_routes_v4.tunnel_gateway.into());
                routes_to_delete.push(route);
            }
        }

        for route in routes_to_delete {
            match self.routing_table.delete_route(&route).await {
                Ok(_) | Err(watch::Error::RouteNotFound) | Err(watch::Error::Unreachable) => (),
                Err(err) => {
                    log::error!("Failed to remove relay route: {err:?}");
                }
            }
        }
    }

    async fn handle_add_address(&mut self, address: AddressMessage) {
        if self.interfaces.handle_add_address(address) {
            // TODO: recalculate best interface if need be
        }
    }

    async fn handle_delete_address(&mut self, address: AddressMessage) {
        if self.interfaces.handle_delete_address(address) {
            // TODO: recalculate best interface if need be
        }
    }

    /// Change routes for tunnel traffic, returns true if V4
    async fn try_update_best_interface(&self, relay_address: IpAddr) -> Result<bool> {
        match relay_address {
            IpAddr::V4(addr) => {}
            IpAddr::V6(addr) => {}
        };

        // Ok(v4_result? || v6_result?)
        Ok(true)
    }

    async fn try_update_best_interface_v4(&self, addr: Ipv4Addr) -> Result<bool> {
        if let Some(interface) = self.interfaces.get_best_default_interface_v4()? {}

        Ok(false)
    }

    async fn try_update_best_interface_v6(&self, addr: Ipv4Addr) -> Result<bool> {
        if let Some(interface) = self.interfaces.get_best_default_interface_v6()? {}

        Ok(false)
    }

    async fn add_route_to_tunnel(
        &self,
        addr: IpAddr,
        interface: &BestRoute,
    ) -> Result<()> {
        let mut route = RouteMessage::new_route(addr.into());

        Ok(())
    }

    async fn change_route_to_tunnel(
        &self,
        addr: &IpAddr,
        interface: &interfaces::Interface,
    ) -> Result<()> {
        Ok(())
    }

    async fn delete_route_to_tunnel(&self, addr: IpAddr) -> Result<()> {
        Ok(())
    }
}

fn v4_faux_destinations() -> impl Iterator<Item = IpNetwork> {
    let half_of_internet: IpNetwork = "0.0.0.0/1".parse().unwrap();
    let other_half_of_internet: IpNetwork = "128.0.0.0/1".parse().unwrap();
    [half_of_internet, other_half_of_internet].into_iter()
}

fn v6_faux_destinations() -> impl Iterator<Item = IpNetwork> {
    let half_of_internet: IpNetwork = "::/1".parse().unwrap();
    let other_half_of_internet: IpNetwork = "128::/1".parse().unwrap();
    [half_of_internet, other_half_of_internet].into_iter()
}

fn v4_default() -> IpNetwork {
    IpNetwork::new(Ipv4Addr::UNSPECIFIED.into(), 0).unwrap()
}

fn v6_default() -> IpNetwork {
    IpNetwork::new(Ipv6Addr::UNSPECIFIED.into(), 0).unwrap()
}

/// Returns a stream that produces an item whenever a default route is either added or deleted from
/// the routing table.
pub fn listen_for_default_route_changes() -> Result<impl Stream<Item = std::io::Result<()>>> {
    let mut cmd = Command::new("route");
    cmd.arg("-n")
        .arg("monitor")
        .arg("-")
        .stderr(Stdio::null())
        .stdout(Stdio::piped())
        .stdin(Stdio::null());

    let mut process = cmd.spawn().map_err(Error::FailedToMonitorRoutes)?;
    let reader = tokio::io::BufReader::new(process.stdout.take().unwrap());
    let lines = reader.lines();

    // route -n monitor will produce netlink messages in the following format
    // ```
    // got message of size 176 on Thu Jun  4 10:08:05 2020
    // RTM_DELETE: Delete Route: len 176, pid: 109, seq 1151, errno 3, ifscope 23,
    // flags:<UP,GATEWAY,STATIC,IFSCOPE>
    // locks:  inits:
    // sockaddrs: <DST,GATEWAY,NETMASK,IFP,IFA>
    //  default 192.168.44.1 default  192.168.44.90
    // ```
    // On the second line of the message, the message type is specified. Only messages with the
    // type 'RTM_ADD' or 'RTM_DELETE' are considered. On the 6th line, message attribute values are
    // shown. To detect a change for a default route in the routing table, check whether this line
    // contains 'default'.  Whenever an empty line is encountered, the message has been sent, so
    // the state can be reset.

    let mut add_or_delete_message = false;
    let mut contains_default = false;

    let monitor = LinesStream::new(lines).try_filter_map(move |line| {
        if add_or_delete_message {
            if line.contains("default") {
                contains_default = true;
            }
            if line.trim().is_empty() {
                add_or_delete_message = false;
                if contains_default {
                    contains_default = false;
                    return future::ready(Ok(Some(())));
                }
            }
        } else {
            add_or_delete_message = line.starts_with("RTM_ADD:") || line.starts_with("RTM_DELETE:");
        }
        future::ready(Ok(None))
    });

    Ok(monitor)
}

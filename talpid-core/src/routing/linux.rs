use crate::{
    routing::{imp::RouteManagerCommand, NetNode, Node, RequiredRoute, Route},
    split_tunnel,
};

use talpid_types::ErrorExt;

use ipnetwork::IpNetwork;
use std::{
    collections::{BTreeMap, HashSet},
    io,
    net::{IpAddr, Ipv4Addr},
};

use futures::{channel::mpsc::UnboundedReceiver, future::FutureExt, StreamExt, TryStreamExt};


use netlink_packet_route::{
    link::{nlas::Nla as LinkNla, LinkMessage},
    route::{nlas::Nla as RouteNla, RouteHeader, RouteMessage},
    rtnl::{
        constants::{RTN_UNICAST, RTPROT_STATIC, RT_SCOPE_UNIVERSE, RT_TABLE_MAIN},
        RouteFlags,
    },
    rule::{nlas::Nla as RuleNla, RuleHeader, RuleMessage},
    NetlinkMessage, NetlinkPayload, RtnlMessage,
};
use rtnetlink::{
    constants::{RTMGRP_IPV4_ROUTE, RTMGRP_IPV6_ROUTE, RTMGRP_LINK, RTMGRP_NOTIFY},
    sys::SocketAddr,
    Handle, IpVersion,
};

use libc::{AF_INET, AF_INET6};


pub type Result<T> = std::result::Result<T, Error>;

/// Errors that can happen in the Linux routing integration
#[derive(err_derive::Error, Debug)]
#[error(no_from)]
pub enum Error {
    #[error(display = "Failed to open a netlink connection")]
    ConnectError(#[error(source)] io::Error),

    #[error(display = "Failed to bind netlink socket")]
    BindError(#[error(source)] io::Error),

    #[error(display = "Netlink error")]
    NetlinkError(#[error(source)] rtnetlink::Error),

    #[error(display = "Route without a valid node")]
    InvalidRoute,

    #[error(display = "Invalid length of byte buffer for IP address")]
    InvalidIpBytes,

    #[error(display = "Invalid network prefix")]
    InvalidNetworkPrefix(#[error(source)] ipnetwork::IpNetworkError),

    #[error(display = "Failed to initialize event loop")]
    EventLoopError(#[error(source)] io::Error),

    #[error(display = "Unknown device index - {}", _0)]
    UnknownDeviceIndex(u32),

    /// Unable to create routing table for tagged connections and packets.
    #[error(display = "Cannot find a free routing table ID")]
    NoFreeRoutingTableId,

    #[error(display = "Shutting down route manager")]
    Shutdown,

    /// Failed to run the process.
    #[error(display = "Unable to execute process")]
    ExecFailed(#[error(source)] io::Error),

    /// ip command returned an error status.
    #[error(display = "ip command failed")]
    IpFailed,
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
struct RequiredDefaultRoute {
    table_id: u8,
    destination: IpNetwork,
}

pub struct RouteManagerImpl {
    handle: Handle,
    messages: UnboundedReceiver<(NetlinkMessage<RtnlMessage>, SocketAddr)>,
    iface_map: BTreeMap<u32, String>,

    // currently added routes
    added_routes: HashSet<Route>,
    // default route tracking
    // destinations that should be routed through the default route
    required_default_routes: HashSet<RequiredDefaultRoute>,
    default_routes: HashSet<Route>,
    best_default_node_v4: Option<Node>,
    best_default_node_v6: Option<Node>,

    split_table_id: u32,
}

impl RouteManagerImpl {
    pub async fn new(required_routes: HashSet<RequiredRoute>) -> Result<Self> {
        let (mut connection, handle, messages) =
            rtnetlink::new_connection().map_err(Error::ConnectError)?;

        let mgroup_flags = RTMGRP_IPV4_ROUTE | RTMGRP_IPV6_ROUTE | RTMGRP_LINK | RTMGRP_NOTIFY;
        let addr = SocketAddr::new(0, mgroup_flags);
        connection
            .socket_mut()
            .bind(&addr)
            .map_err(Error::BindError)?;

        tokio::spawn(connection);

        let iface_map = Self::initialize_link_map(&handle).await?;
        let split_table_id = Self::find_free_table_id(&handle).await?;

        log::trace!("Using table id {} for excluded apps", split_table_id);

        let mut monitor = Self {
            iface_map,
            handle,
            messages,

            required_default_routes: HashSet::new(),
            added_routes: HashSet::new(),

            default_routes: HashSet::new(),
            best_default_node_v4: None,
            best_default_node_v6: None,

            split_table_id,
        };

        monitor.default_routes = monitor.get_default_routes().await?;
        monitor.best_default_node_v4 =
            Self::pick_best_default_node(&monitor.default_routes, IpVersion::V4);
        monitor.best_default_node_v6 =
            Self::pick_best_default_node(&monitor.default_routes, IpVersion::V6);

        monitor.add_required_routes(required_routes).await?;

        Ok(monitor)
    }

    async fn find_free_table_id(handle: &rtnetlink::Handle) -> Result<u32> {
        let mut request = handle.route().get(IpVersion::V4).execute();
        let mut used_ids = HashSet::new();

        while let Some(route) = request.try_next().await.map_err(Error::NetlinkError)? {
            let mut id_found = false;
            for nla in route.nlas {
                match nla {
                    RouteNla::Table(id) => {
                        used_ids.insert(id);
                        id_found = true;
                        break;
                    }
                    _ => (),
                }
            }

            if !id_found {
                // Use old header ID
                used_ids.insert(u32::from(route.header.table));
            }
        }

        for id in 1..u32::MAX {
            if id == RT_TABLE_COMPAT as u32 {
                continue;
            }
            if !used_ids.contains(&id) {
                return Ok(id);
            }
        }

        Err(Error::NoFreeRoutingTableId)
    }

    /// Route PID-associated packets through the physical interface.
    async fn enable_exclusions_routes(&mut self) -> Result<()> {
        // TODO: IPv6
        use netlink_packet_route::constants::*;

        let mut req = NetlinkMessage::from(RtnlMessage::NewRule(self.fwmark_rule_message()));
        req.header.flags = NLM_F_REQUEST | NLM_F_ACK | NLM_F_CREATE | NLM_F_REPLACE;

        let mut response = self.handle.request(req).map_err(Error::NetlinkError)?;

        while let Some(message) = response.next().await {
            if let NetlinkPayload::Error(error) = message.payload {
                return Err(Error::NetlinkError(rtnetlink::Error::NetlinkError(error)));
            }
        }

        // Add default route for the exclusions table
        let zero_network =
            ipnetwork::IpNetwork::new(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)), 0).unwrap();
        let mut required_routes = HashSet::new();
        required_routes.insert(
            RequiredRoute::new(zero_network, NetNode::DefaultNode).table(self.split_table_id as u8),
        );
        self.add_required_routes(required_routes).await
    }

    /// Stop routing PID-associated packets through the physical interface.
    async fn disable_exclusions_routes(&mut self) {
        // TODO: IPv6
        use netlink_packet_route::constants::*;

        let mut req = NetlinkMessage::from(RtnlMessage::DelRule(self.fwmark_rule_message()));
        req.header.flags = NLM_F_REQUEST | NLM_F_ACK;

        match self.handle.request(req).map_err(Error::NetlinkError) {
            Ok(mut response) => {
                while let Some(message) = response.next().await {
                    if let NetlinkPayload::Error(error) = message.payload {
                        if error.to_io().kind() != io::ErrorKind::NotFound {
                            log::warn!("Failed to delete routing policy: {}", error);
                        }
                    }
                }
            }
            Err(error) => log::warn!("Failed to delete routing policy: {}", error),
        }
    }

    fn fwmark_rule_message(&self) -> RuleMessage {
        use netlink_packet_route::constants::*;

        RuleMessage {
            header: RuleHeader {
                family: AF_INET as u8,
                action: FR_ACT_TO_TBL,
                src_len: 0,
                ..RuleHeader::default()
            },
            nlas: vec![
                RuleNla::Source(ip_to_bytes(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)))),
                RuleNla::FwMark(split_tunnel::MARK as u32),
                RuleNla::Table(self.split_table_id as u32),
            ],
        }
    }

    /// Route DNS requests through the tunnel interface.
    #[cfg(target_os = "linux")]
    async fn route_exclusions_dns(
        &mut self,
        tunnel_alias: &str,
        dns_servers: &[IpAddr],
    ) -> Result<()> {
        let mut dns_routes = HashSet::new();

        for server in dns_servers {
            dns_routes.insert(
                RequiredRoute::new(
                    IpNetwork::from(*server),
                    Node::device(tunnel_alias.to_string()),
                )
                .table(self.split_table_id as u8),
            );
        }

        self.add_required_routes(dns_routes).await
    }

    async fn add_required_default_routes(
        &mut self,
        required_default_routes: HashSet<RequiredDefaultRoute>,
    ) -> Result<()> {
        for route in required_default_routes.into_iter() {
            if let (false, _, Some(default_node)) | (true, Some(default_node), _) = (
                route.destination.is_ipv4(),
                &self.best_default_node_v4,
                &self.best_default_node_v6,
            ) {
                // best to pick a single node identifier rather than device + ip
                let new_route =
                    Route::new(default_node.clone(), route.destination).table(route.table_id);
                self.add_route(new_route).await?;
            }
            self.required_default_routes.insert(route);
        }
        Ok(())
    }

    async fn add_required_routes(&mut self, required_routes: HashSet<RequiredRoute>) -> Result<()> {
        let mut required_normal_routes = HashSet::new();
        let mut required_default_routes = HashSet::new();

        for route in required_routes {
            match route.node {
                NetNode::RealNode(node) => {
                    required_normal_routes
                        .insert(Route::new(node, route.prefix).table(route.table_id));
                }
                NetNode::DefaultNode => {
                    required_default_routes.insert(RequiredDefaultRoute {
                        table_id: route.table_id,
                        destination: route.prefix,
                    });
                }
            }
        }

        for normal_route in required_normal_routes.into_iter() {
            self.add_route(normal_route).await?;
        }

        if self
            .add_required_default_routes(required_default_routes.clone())
            .await
            .is_err()
        {
            log::trace!("Refreshing default routes which may be stale");

            self.default_routes = self.get_default_routes().await?;
            self.best_default_node_v4 =
                Self::pick_best_default_node(&self.default_routes, IpVersion::V4);
            self.best_default_node_v6 =
                Self::pick_best_default_node(&self.default_routes, IpVersion::V6);

            self.add_required_default_routes(required_default_routes)
                .await?;
        }

        Ok(())
    }

    async fn get_default_routes(&self) -> Result<HashSet<Route>> {
        let mut routes = self.get_default_routes_inner(IpVersion::V4).await?;
        routes.extend(self.get_default_routes_inner(IpVersion::V6).await?);
        Ok(routes)
    }

    async fn get_default_routes_inner(&self, version: IpVersion) -> Result<HashSet<Route>> {
        let mut routes = HashSet::new();
        let mut route_request = self.handle.route().get(version).execute();
        while let Some(route) = route_request
            .try_next()
            .await
            .map_err(Error::NetlinkError)?
        {
            if route.header.destination_prefix_length == 0 {
                if let Some(default_route) = self.parse_route_message(route)? {
                    routes.insert(default_route);
                }
            }
        }
        Ok(routes)
    }

    async fn initialize_link_map(handle: &rtnetlink::Handle) -> Result<BTreeMap<u32, String>> {
        let mut link_map = BTreeMap::new();
        let mut link_request = handle.link().get().execute();
        while let Some(link) = link_request.try_next().await.map_err(Error::NetlinkError)? {
            if let Some((idx, link_name)) = Self::map_iface_name_to_idx(link) {
                link_map.insert(idx, link_name);
            }
        }

        Ok(link_map)
    }

    fn find_iface_idx(&self, iface_name: &str) -> Option<u32> {
        self.iface_map
            .iter()
            .find(|(_idx, name)| name.as_str() == iface_name)
            .map(|(idx, _name)| *idx)
    }


    async fn process_new_route(&mut self, route: Route) -> Result<()> {
        if route.prefix.prefix() == 0 {
            self.default_routes.insert(route);
            self.update_default_routes().await?;
        }
        Ok(())
    }

    async fn process_deleted_route(&mut self, route: Route) -> Result<()> {
        if route.prefix.prefix() == 0 {
            self.default_routes.remove(&route);
            self.update_default_routes().await?;
        }
        if self.added_routes.contains(&route) {
            self.added_routes.remove(&route);
        }
        Ok(())
    }

    async fn update_default_routes(&mut self) -> Result<()> {
        let new_best_v4 = Self::pick_best_default_node(&self.default_routes, IpVersion::V4);
        if self.best_default_node_v4 != new_best_v4 && new_best_v4.is_some() {
            let new_node = new_best_v4.unwrap();
            let old_node = self.best_default_node_v4.take();
            let v4_routes: Vec<_> = self
                .required_default_routes
                .iter()
                .filter(|ip| ip.destination.is_ipv4())
                .cloned()
                .collect();
            for route in v4_routes {
                let new_route =
                    Route::new(new_node.clone(), route.destination).table(route.table_id);

                if let Some(old_node) = &old_node {
                    let old_route =
                        Route::new(old_node.clone(), route.destination).table(route.table_id);

                    if let Err(e) = self.delete_route(&old_route).await {
                        log::error!("Failed to remove old route {} - {}", &old_route, e);
                    }
                }
                if let Err(e) = self.add_route(new_route).await {
                    log::error!("Failed to add new route {} - {}", &new_node, e);
                }
            }
            self.best_default_node_v4 = Some(new_node);
        }

        let new_best_v6 = Self::pick_best_default_node(&self.default_routes, IpVersion::V6);
        if self.best_default_node_v6 != new_best_v6 && new_best_v6.is_some() {
            let new_node = new_best_v6.unwrap();
            let old_node = self.best_default_node_v6.take();
            let v6_routes: Vec<_> = self
                .required_default_routes
                .iter()
                .filter(|ip| !ip.destination.is_ipv4())
                .cloned()
                .collect();

            for route in v6_routes {
                let new_route =
                    Route::new(new_node.clone(), route.destination).table(route.table_id);

                if let Some(old_node) = &old_node {
                    let old_route =
                        Route::new(old_node.clone(), route.destination).table(route.table_id);

                    if let Err(e) = self.delete_route(&old_route).await {
                        log::error!("Failed to remove old route {} - {}", &old_route, e);
                    }
                }
                if let Err(e) = self.add_route(new_route).await {
                    log::error!("Failed to add new route {} - {}", &new_node, e);
                }
            }
            self.best_default_node_v6 = Some(new_node);
        }

        Ok(())
    }

    fn pick_best_default_node(routes: &HashSet<Route>, version: IpVersion) -> Option<Node> {
        // Pick the route with the lowest metric - thus the most favourable route.
        routes
            .iter()
            .filter(|route| route.prefix.is_ipv4() == (version == IpVersion::V4))
            .fold(
                None,
                |best_route: Option<Route>, next_route| match best_route {
                    Some(current_best) => {
                        if current_best.metric.unwrap_or(0) > next_route.metric.unwrap_or(0) {
                            Some(next_route.clone())
                        } else {
                            Some(current_best)
                        }
                    }
                    None => Some(next_route.clone()),
                },
            )
            .map(|route| route.node)
    }

    async fn cleanup_routes(&mut self) {
        for required_route in &self.required_default_routes {
            let best_node = if required_route.destination.is_ipv4() {
                self.best_default_node_v4.clone()
            } else {
                self.best_default_node_v6.clone()
            };

            let best_node = match best_node {
                None => continue,
                Some(node) => node,
            };

            let route =
                Route::new(best_node, required_route.destination).table(required_route.table_id);
            if let Err(e) = self.delete_route(&route).await {
                if let Error::NetlinkError(err) = &e {
                    if let rtnetlink::Error::NetlinkError(msg) = err {
                        // -3 means that the route doesn't exist anymore anyway
                        if msg.code == -3 {
                            continue;
                        }
                    }
                }
                log::error!("Failed to remove route - {} - {}", route, e);
            }
        }
        self.required_default_routes.clear();

        for route in self.added_routes.drain().collect::<Vec<_>>().iter() {
            if let Err(e) = self.delete_route(&route).await {
                if let Error::NetlinkError(err) = &e {
                    if let rtnetlink::Error::NetlinkError(msg) = err {
                        // -3 means that the route doesn't exist anymore anyway
                        if msg.code == -3 {
                            continue;
                        }
                    }
                }
                log::error!("Failed to remove route - {} - {}", route, e);
            }
        }
    }


    pub async fn run(mut self, manage_rx: UnboundedReceiver<RouteManagerCommand>) -> Result<()> {
        let mut manage_rx = manage_rx.fuse();
        loop {
            futures::select! {
                command = manage_rx.select_next_some() => {
                    self.process_command(command).await?;
                },
                (route_change, socket) = self.messages.select_next_some().fuse() => {
                    if let Err(error) = self.process_netlink_message(route_change).await {
                        log::error!("{}", error.display_chain_with_msg("Failed to process netlink message"));
                    }
                }
            };
        }
    }

    async fn process_command(&mut self, command: RouteManagerCommand) -> Result<()> {
        match command {
            RouteManagerCommand::Shutdown(shutdown_signal) => {
                log::trace!("Shutting down route manager");
                self.cleanup_routes().await;
                log::trace!("Route manager done");
                let _ = shutdown_signal.send(());
                return Err(Error::Shutdown);
            }
            RouteManagerCommand::AddRoutes(routes, result_rx) => {
                log::debug!("Adding routes: {:?}", routes);
                let _ = result_rx.send(self.add_required_routes(routes.clone()).await);
            }
            RouteManagerCommand::EnableExclusionsRoutes(result_rx) => {
                let _ = result_rx.send(self.enable_exclusions_routes().await);
            }
            RouteManagerCommand::DisableExclusionsRoutes => {
                self.disable_exclusions_routes().await;
            }
            RouteManagerCommand::RouteExclusionsDns(tunnel_alias, dns_servers, result_rx) => {
                let _ =
                    result_rx.send(self.route_exclusions_dns(&tunnel_alias, &dns_servers).await);
            }
            RouteManagerCommand::ClearRoutes => {
                log::debug!("Clearing routes");
                self.cleanup_routes().await;
            }
        }
        Ok(())
    }

    async fn process_netlink_message(&mut self, msg: NetlinkMessage<RtnlMessage>) -> Result<()> {
        match msg.payload {
            NetlinkPayload::InnerMessage(RtnlMessage::NewLink(new_link)) => {
                if let Some((idx, name)) = Self::map_iface_name_to_idx(new_link) {
                    self.iface_map.insert(idx, name);
                }
            }
            NetlinkPayload::InnerMessage(RtnlMessage::DelLink(old_link)) => {
                if let Some((idx, _)) = Self::map_iface_name_to_idx(old_link) {
                    self.iface_map.remove(&idx);
                }
            }

            NetlinkPayload::InnerMessage(RtnlMessage::NewRoute(new_route)) => {
                if let Some(new_route) = self.parse_route_message(new_route)? {
                    self.process_new_route(new_route).await?;
                }
            }
            NetlinkPayload::InnerMessage(RtnlMessage::DelRoute(old_route)) => {
                if let Some(deletion) = self.parse_route_message(old_route)? {
                    self.process_deleted_route(deletion).await?;
                }
            }
            _ => (),
        };
        Ok(())
    }

    // Tries to coax a Route out of a RouteMessage, but only if it's a route from the main routing
    // table
    // TODO: Change to account for different routing tables.
    fn parse_route_message(&self, msg: RouteMessage) -> Result<Option<Route>> {
        if msg.header.table != RT_TABLE_MAIN {
            return Ok(None);
        }


        let mut prefix = None;
        let mut node_addr = None;
        let mut device = None;
        let mut metric = None;
        let mut gateway = None;

        let destination_length = msg.header.destination_prefix_length;
        let af_spec = msg.header.address_family;

        for nla in msg.nlas.iter() {
            match nla {
                RouteNla::Oif(device_idx) => {
                    match self.iface_map.get(&device_idx) {
                        Some(device_name) => device = Some(device_name.to_string()),
                        None => {
                            return Err(Error::UnknownDeviceIndex(*device_idx));
                        }
                    };
                }

                RouteNla::Via(addr) => {
                    node_addr = Self::parse_ip(&addr).map(Some)?;
                }

                RouteNla::Destination(addr) => {
                    prefix = Self::parse_ip(&addr)
                        .and_then(|ip| {
                            ipnetwork::IpNetwork::new(ip, destination_length)
                                .map_err(Error::InvalidNetworkPrefix)
                        })
                        .map(Some)?;
                }

                // gateway NLAs indicate that this is actually a default route
                RouteNla::Gateway(gateway_ip) => {
                    gateway = Self::parse_ip(&gateway_ip).map(Some)?;
                }

                RouteNla::Priority(priority) => {
                    metric = Some(*priority);
                }
                _ => continue,
            }
        }

        // when a gateway is specified but prefix is none, then this is a default route
        if prefix.is_none() && gateway.is_some() {
            prefix = match af_spec as i32 {
                AF_INET => Some("0.0.0.0/0".parse().expect("failed to parse ipnetwork")),
                AF_INET6 => Some("::/0".parse().expect("failed to parse ipnetwork")),
                _ => None,
            };
        }

        if device.is_none() && node_addr.is_none() || prefix.is_none() {
            return Err(Error::InvalidRoute);
        }


        let node = Node {
            ip: node_addr.or(gateway),
            device,
        };

        Ok(Some(Route {
            node,
            prefix: prefix.unwrap(),
            metric,
            table_id: msg.header.table,
        }))
    }

    fn map_iface_name_to_idx(msg: LinkMessage) -> Option<(u32, String)> {
        let index = msg.header.index;
        for nla in msg.nlas {
            if let LinkNla::IfName(name) = nla {
                return Some((index, name));
            }
        }
        None
    }

    fn parse_ip(bytes: &[u8]) -> Result<IpAddr> {
        if bytes.len() == 4 {
            let mut ipv4_bytes = [0u8; 4];
            ipv4_bytes.copy_from_slice(bytes);
            Ok(IpAddr::from(ipv4_bytes))
        } else if bytes.len() == 16 {
            let mut ipv6_bytes = [0u8; 16];
            ipv6_bytes.copy_from_slice(bytes);
            Ok(IpAddr::from(ipv6_bytes))
        } else {
            log::error!("Expected either 4 or 16 bytes, got {} bytes", bytes.len());
            Err(Error::InvalidIpBytes)
        }
    }

    async fn delete_route(&self, route: &Route) -> Result<()> {
        let mut route_message = RouteMessage {
            header: RouteHeader {
                address_family: if route.prefix.is_ipv4() {
                    AF_INET as u8
                } else {
                    AF_INET6 as u8
                },
                source_prefix_length: 0,
                destination_prefix_length: route.prefix.prefix(),
                tos: 0u8,
                table: route.table_id,
                protocol: RTPROT_STATIC,
                scope: RT_SCOPE_UNIVERSE,
                kind: RTN_UNICAST,
                flags: RouteFlags::empty(),
            },
            nlas: vec![RouteNla::Destination(ip_to_bytes(route.prefix.ip()))],
        };
        if let Some(interface_name) = route.node.get_device() {
            if let Some(iface_idx) = self.find_iface_idx(interface_name) {
                route_message.nlas.push(RouteNla::Oif(iface_idx));
            }
        }

        if let Some(gateway) = route.node.get_address() {
            let gateway_nla = if route.node.get_device().is_some() {
                RouteNla::Gateway(ip_to_bytes(gateway))
            } else {
                RouteNla::Via(ip_to_bytes(gateway))
            };
            route_message.nlas.push(gateway_nla);
        }


        self.handle
            .route()
            .del(route_message)
            .execute()
            .await
            .map_err(Error::NetlinkError)
    }

    async fn add_route(&mut self, route: Route) -> Result<()> {
        let add_message = match &route.prefix {
            IpNetwork::V4(v4_prefix) => {
                let mut add_message = self
                    .handle
                    .route()
                    .add_v4()
                    .destination_prefix(v4_prefix.ip(), v4_prefix.prefix())
                    .table(route.table_id);

                if v4_prefix.prefix() > 0 && v4_prefix.prefix() < 32 {
                    add_message = add_message.scope(RT_SCOPE_LINK);
                }

                if let Some(IpAddr::V4(node_address)) = route.node.get_address() {
                    add_message = add_message.gateway(node_address);
                }

                if let Some(interface_name) = route.node.get_device() {
                    if let Some(iface_idx) = self.find_iface_idx(interface_name) {
                        add_message = add_message.output_interface(iface_idx);
                    }
                }

                add_message.message_mut().clone()
            }

            IpNetwork::V6(v6_prefix) => {
                let mut add_message = self
                    .handle
                    .route()
                    .add_v6()
                    .destination_prefix(v6_prefix.ip(), v6_prefix.prefix())
                    .table(route.table_id);

                if v6_prefix.prefix() > 0 && v6_prefix.prefix() < 128 {
                    add_message = add_message.scope(RT_SCOPE_LINK);
                }

                if let Some(IpAddr::V6(node_address)) = route.node.get_address() {
                    add_message = add_message.gateway(node_address);
                }

                if let Some(interface_name) = route.node.get_device() {
                    if let Some(iface_idx) = self.find_iface_idx(interface_name) {
                        add_message = add_message.output_interface(iface_idx);
                    }
                }

                add_message.message_mut().clone()
            }
        };

        // Need to modify the request in place to set the correct flags to be able to replace any
        // existing routes - self.handle.route().add_v4().execute() sets the NLM_F_EXCL flag which
        // will make the request fail if a route with the same destination already exists.
        use netlink_packet_route::constants::*;
        let mut req = NetlinkMessage::from(RtnlMessage::NewRoute(add_message));
        req.header.flags = NLM_F_REQUEST | NLM_F_ACK | NLM_F_CREATE | NLM_F_REPLACE;

        let mut response = self.handle.request(req).map_err(Error::NetlinkError)?;

        while let Some(message) = response.next().await {
            if let NetlinkPayload::Error(err) = message.payload {
                return Err(Error::NetlinkError(rtnetlink::Error::NetlinkError(err)));
            }
        }
        self.added_routes.insert(route.clone());
        Ok(())
    }
}

impl Drop for RouteManagerImpl {
    fn drop(&mut self) {
        futures::executor::block_on(self.cleanup_routes())
    }
}

fn ip_to_bytes(addr: IpAddr) -> Vec<u8> {
    match addr {
        IpAddr::V4(addr) => addr.octets().to_vec(),
        IpAddr::V6(addr) => addr.octets().to_vec(),
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::collections::HashSet;


    /// Tests if dropping inside a tokio runtime panics
    #[test]
    fn test_drop_in_executor() {
        let mut runtime = tokio::runtime::Runtime::new().expect("Failed to initialize runtime");
        runtime.block_on(async {
            let manager = RouteManagerImpl::new(HashSet::new())
                .await
                .expect("Failed to initialize route manager");
            std::mem::drop(manager);
        });
    }

    /// Tests if dropping outside a runtime panics
    #[test]
    fn test_drop() {
        let mut runtime = tokio::runtime::Runtime::new().expect("Failed to initialize runtime");
        let manager = runtime.block_on(async {
            RouteManagerImpl::new(HashSet::new())
                .await
                .expect("Failed to initialize route manager")
        });
        std::mem::drop(manager);
    }
}

use std::collections::{BTreeMap, HashSet};
use std::io;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};
use std::sync::LazyLock;

use crate::{
    NetNode, Node, RequiredRoute, Route,
    imp::{CallbackMessage, RouteManagerCommand},
};
use netlink_packet_core::{
    Emitable, NLM_F_ACK, NLM_F_CREATE, NLM_F_DUMP, NLM_F_REPLACE, NLM_F_REQUEST, NetlinkMessage,
    NetlinkPayload,
};
use netlink_packet_route::route::RouteFlags;
use netlink_sys::AsyncSocket;
use talpid_types::ErrorExt;

use futures::{
    StreamExt, TryStreamExt,
    channel::mpsc::{UnboundedReceiver, UnboundedSender},
    future::FutureExt,
};
use ipnetwork::IpNetwork;
use libc::{RT_TABLE_COMPAT, RT_TABLE_MAIN};
use netlink_packet_route::{
    AddressFamily, RouteNetlinkMessage,
    link::{LinkAttribute, LinkLayerType, LinkMessage},
    route::{
        RouteAddress, RouteAttribute, RouteMessage, RouteMetric, RouteProtocol, RouteScope,
        RouteType, RouteVia,
    },
    rule::{RuleAction, RuleAttribute, RuleFlags, RuleHeader, RuleMessage},
};
use rtnetlink::{
    Handle, RouteMessageBuilder,
    constants::{RTMGRP_IPV4_ROUTE, RTMGRP_IPV6_ROUTE, RTMGRP_LINK, RTMGRP_NOTIFY},
    sys::SocketAddr,
};

/// What this
static SUPPRESS_RULE_V4: LazyLock<RuleMessage> = LazyLock::new(|| {
    let mut rule_msg = RuleMessage::default();
    let header = RuleHeader {
        family: AddressFamily::Inet,
        action: RuleAction::ToTable, // FR_ACT_TO_TBL
        ..RuleHeader::default()
    };
    let attributes = vec![
        RuleAttribute::SuppressPrefixLen(0),
        RuleAttribute::Table(RT_TABLE_MAIN as u32),
    ];

    rule_msg.header = header;
    rule_msg.attributes = attributes;
    rule_msg
});

/// What this
static SUPPRESS_RULE_V6: LazyLock<RuleMessage> = LazyLock::new(|| {
    let mut v6_rule = SUPPRESS_RULE_V4.clone();
    v6_rule.header.family = AddressFamily::Inet6;
    v6_rule
});

fn all_rules(fwmark: u32, table: u32) -> [RuleMessage; 4] {
    [
        no_fwmark_rule_v4(fwmark, table),
        no_fwmark_rule_v6(fwmark, table),
        SUPPRESS_RULE_V4.clone(),
        SUPPRESS_RULE_V6.clone(),
    ]
}

/// What this
fn no_fwmark_rule_v4(fwmark: u32, table: u32) -> RuleMessage {
    let mut rule_msg = RuleMessage::default();
    let header = RuleHeader {
        family: AddressFamily::Inet, // AF_INET
        action: RuleAction::ToTable, // FR_ACT_TO_TBL
        flags: RuleFlags::Invert,    // FIB_RULE_INVERT
        ..RuleHeader::default()
    };
    let attributes = vec![RuleAttribute::FwMark(fwmark), RuleAttribute::Table(table)];

    rule_msg.header = header;
    rule_msg.attributes = attributes;
    rule_msg
}

/// What this
fn no_fwmark_rule_v6(fwmark: u32, table: u32) -> RuleMessage {
    let mut v6_rule = no_fwmark_rule_v4(fwmark, table);
    v6_rule.header.family = AddressFamily::Inet6;
    v6_rule
}

pub type Result<T> = std::result::Result<T, Error>;

/// Errors that can happen in the Linux routing integration
#[derive(thiserror::Error, Debug)]
#[allow(missing_docs)]
pub enum Error {
    #[error("Failed to open a netlink connection")]
    Connect(#[source] io::Error),

    #[error("Failed to bind netlink socket")]
    Bind(#[source] io::Error),

    #[error("Netlink error")]
    Netlink(#[source] rtnetlink::Error),

    #[error("Route without a valid node")]
    InvalidRoute,

    #[error("Invalid length of byte buffer for IP address")]
    InvalidIpBytes,

    #[error("Invalid network prefix")]
    InvalidNetworkPrefix(#[source] ipnetwork::IpNetworkError),

    #[error("Unknown device index: {0}")]
    UnknownDeviceIndex(u32),

    #[error("Failed to get a route for the given IP address")]
    GetRoute(#[source] rtnetlink::Error),

    #[error("No netlink response for route query")]
    NoRoute,

    #[error("Route node was malformed")]
    InvalidRouteNode,

    #[error("No link found")]
    LinkNotFound,

    /// Unable to create routing table for tagged connections and packets.
    #[error("Cannot find a free routing table ID")]
    NoFreeRoutingTableId,

    #[error("Shutting down route manager")]
    Shutdown,
}

pub struct RouteManagerImpl {
    handle: Handle,
    messages: UnboundedReceiver<(NetlinkMessage<RouteNetlinkMessage>, SocketAddr)>,
    iface_map: BTreeMap<u32, NetworkInterface>,
    listeners: Vec<UnboundedSender<CallbackMessage>>,

    // currently added routes
    added_routes: HashSet<Route>,

    /// Tunnel specific routing table, traffic not marked will be routed via this routing table.
    table_id: u32,
    /// Firewall mark identifies traffic which shouldn't be routed via the tunnel routing table. It
    /// is used to construct a routing rule.
    fwmark: u32,
}

impl RouteManagerImpl {
    pub async fn new(table_id: u32, fwmark: u32) -> Result<Self> {
        let (mut connection, handle, messages) =
            rtnetlink::new_connection().map_err(Error::Connect)?;

        let mgroup_flags = RTMGRP_IPV4_ROUTE | RTMGRP_IPV6_ROUTE | RTMGRP_LINK | RTMGRP_NOTIFY;
        let addr = SocketAddr::new(0, mgroup_flags);
        connection
            .socket_mut()
            .socket_mut()
            .bind(&addr)
            .map_err(Error::Bind)?;

        tokio::spawn(connection);

        let iface_map = Self::initialize_link_map(&handle).await?;

        let mut monitor = Self {
            handle,
            messages,
            iface_map,
            listeners: vec![],
            added_routes: HashSet::new(),
            table_id,
            fwmark,
        };

        monitor.clear_routing_rules().await?;

        Ok(monitor)
    }

    async fn create_routing_rules(&mut self, enable_ipv6: bool) -> Result<()> {
        self.clear_routing_rules().await?;

        for rule in all_rules(self.fwmark, self.table_id)
            .iter()
            .filter(|rule| rule.header.family == AddressFamily::Inet || enable_ipv6)
        {
            let mut req = NetlinkMessage::from(RouteNetlinkMessage::NewRule((*rule).clone()));
            req.header.flags = NLM_F_REQUEST | NLM_F_ACK | NLM_F_CREATE | NLM_F_REPLACE;

            let mut response = self.handle.request(req).map_err(Error::Netlink)?;

            while let Some(message) = response.next().await {
                if let NetlinkPayload::Error(error) = message.payload {
                    return Err(Error::Netlink(rtnetlink::Error::NetlinkError(error)));
                }
            }
        }
        Ok(())
    }

    async fn clear_routing_rules(&mut self) -> Result<()> {
        let rules = self.get_rules().await?;
        for rule in all_rules(self.fwmark, self.table_id) {
            let mut matching_rule = None;

            // `RTM_DELRULE` is way too picky about which rules are considered the same.
            // So iterate over all rules and ignore irrelevant attributes.
            for found_rule in &rules {
                // Match header
                if found_rule.header.family != rule.header.family {
                    continue;
                }
                if found_rule.header.action != rule.header.action {
                    continue;
                }
                if (found_rule.header.flags & rule.header.flags) != rule.header.flags {
                    continue;
                }
                // Match NLAs
                let mut contains_nlas = true;
                for nla in &rule.attributes {
                    if !found_rule.attributes.contains(nla) {
                        contains_nlas = false;
                        break;
                    }
                }
                if contains_nlas {
                    log::trace!("Existing routing rule matched: {:?}", found_rule);
                    matching_rule = Some(found_rule);
                    break;
                }
            }

            if let Some(rule) = matching_rule {
                self.delete_rule_if_exists((*rule).clone()).await?;
            }
        }
        Ok(())
    }

    async fn get_rules(&mut self) -> Result<Vec<RuleMessage>> {
        let mut req = NetlinkMessage::from(RouteNetlinkMessage::GetRule(RuleMessage::default()));
        req.header.flags = NLM_F_REQUEST | NLM_F_ACK | NLM_F_DUMP;

        let mut response = self.handle.request(req).map_err(Error::Netlink)?;

        let mut rules = vec![];

        while let Some(message) = response.next().await {
            match message.payload {
                NetlinkPayload::InnerMessage(RouteNetlinkMessage::NewRule(rule)) => {
                    rules.push(rule);
                }
                NetlinkPayload::Error(error) => {
                    return Err(Error::Netlink(rtnetlink::Error::NetlinkError(error)));
                }
                _ => (),
            }
        }
        Ok(rules)
    }

    async fn delete_rule_if_exists(&mut self, rule: RuleMessage) -> Result<()> {
        let mut req = NetlinkMessage::from(RouteNetlinkMessage::DelRule(rule));
        req.header.flags = NLM_F_REQUEST | NLM_F_ACK;

        let mut response = self.handle.request(req).map_err(Error::Netlink)?;

        while let Some(message) = response.next().await {
            if let NetlinkPayload::Error(error) = message.payload
                && error.to_io().kind() != io::ErrorKind::NotFound
            {
                return Err(Error::Netlink(rtnetlink::Error::NetlinkError(error)));
            }
        }
        Ok(())
    }

    async fn add_required_routes(&mut self, required_routes: HashSet<RequiredRoute>) -> Result<()> {
        let mut required_normal_routes = HashSet::new();

        for route in required_routes {
            match route.node {
                NetNode::RealNode(node) => {
                    let table = if route.main_table {
                        RT_TABLE_MAIN.into()
                    } else {
                        self.table_id
                    };
                    let mut new_route = Route::new(node, route.prefix).table(table);
                    new_route.mtu = route.mtu.map(u32::from);
                    required_normal_routes.insert(new_route);
                }
            }
        }

        for normal_route in required_normal_routes.into_iter() {
            self.add_route(normal_route).await?;
        }

        Ok(())
    }

    async fn initialize_link_map(
        handle: &rtnetlink::Handle,
    ) -> Result<BTreeMap<u32, NetworkInterface>> {
        let mut link_map = BTreeMap::new();
        let mut link_request = handle.link().get().execute();
        while let Some(link) = link_request.try_next().await.map_err(Error::Netlink)? {
            if let Some((idx, device)) = Self::map_interface(link) {
                link_map.insert(idx, device);
            }
        }

        Ok(link_map)
    }

    fn find_iface_idx(&self, iface_name: &str) -> Option<u32> {
        self.iface_map
            .iter()
            .find(|(_idx, iface)| iface.name.as_str() == iface_name)
            .map(|(idx, _name)| *idx)
    }

    fn process_deleted_route(&mut self, route: &Route) -> Result<()> {
        self.added_routes.remove(route);
        Ok(())
    }

    async fn cleanup_routes(&mut self) {
        for route in self.added_routes.drain().collect::<Vec<_>>().iter() {
            if let Err(e) = self.delete_route_if_exists(route).await {
                log::error!("Failed to remove route: {}: {}", route, e);
            }
        }
    }

    pub(crate) async fn run(
        mut self,
        manage_rx: UnboundedReceiver<RouteManagerCommand>,
    ) -> Result<()> {
        let mut manage_rx = manage_rx.fuse();
        loop {
            futures::select! {
                command = manage_rx.select_next_some() => {
                    self.process_command(command).await?;
                },
                (route_change, _socket) = self.messages.select_next_some().fuse() => {
                    if let Err(error) = self.process_netlink_message(route_change) {
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
                self.destructor().await;
                log::trace!("Route manager done");
                let _ = shutdown_signal.send(());
                return Err(Error::Shutdown);
            }
            RouteManagerCommand::AddRoutes(routes, result_tx) => {
                log::debug!("Adding routes: {:?}", routes);
                let _ = result_tx.send(self.add_required_routes(routes.clone()).await);
            }
            RouteManagerCommand::CreateRoutingRules(enable_ipv6, result_tx) => {
                let _ = result_tx.send(self.create_routing_rules(enable_ipv6).await);
            }
            RouteManagerCommand::ClearRoutingRules(result_tx) => {
                let _ = result_tx.send(self.clear_routing_rules().await);
            }
            RouteManagerCommand::NewChangeListener(result_tx) => {
                let _ = result_tx.send(self.listen());
            }
            RouteManagerCommand::GetDestinationRoute(destination, mark, result_tx) => {
                let _ = result_tx.send(self.get_destination_route(&destination, mark).await);
            }
            RouteManagerCommand::GetMtuForRoute(ip, result_tx) => {
                let _ = result_tx.send(self.get_mtu_for_route(ip).await);
            }
            RouteManagerCommand::ClearRoutes => {
                log::debug!("Clearing routes");
                self.cleanup_routes().await;
            }
        }
        Ok(())
    }

    fn process_netlink_message(&mut self, msg: NetlinkMessage<RouteNetlinkMessage>) -> Result<()> {
        match msg.payload {
            NetlinkPayload::InnerMessage(RouteNetlinkMessage::NewLink(new_link)) => {
                if let Some((idx, name)) = Self::map_interface(new_link) {
                    self.iface_map.insert(idx, name);
                }
            }
            NetlinkPayload::InnerMessage(RouteNetlinkMessage::DelLink(old_link)) => {
                if let Some((idx, _)) = Self::map_interface(old_link) {
                    self.iface_map.remove(&idx);
                }
            }
            NetlinkPayload::InnerMessage(RouteNetlinkMessage::NewRoute(new_route)) => {
                if let Some(addition) = self.parse_route_message(new_route)? {
                    self.notify_change_listeners(CallbackMessage::NewRoute(addition));
                }
            }
            NetlinkPayload::InnerMessage(RouteNetlinkMessage::DelRoute(old_route)) => {
                if let Some(deletion) = self.parse_route_message(old_route)? {
                    self.process_deleted_route(&deletion)?;
                    self.notify_change_listeners(CallbackMessage::DelRoute(deletion));
                }
            }
            _ => (),
        };
        Ok(())
    }

    fn notify_change_listeners(&mut self, message: CallbackMessage) {
        self.listeners
            .retain(|listener| listener.unbounded_send(message.clone()).is_ok());
    }

    // Tries to coax a Route out of a RouteMessage
    fn parse_route_message(&self, msg: RouteMessage) -> Result<Option<Route>> {
        let af = msg.header.address_family;
        let destination_length = msg.header.destination_prefix_length;
        let is_ipv4 = match af {
            AddressFamily::Inet => true,
            AddressFamily::Inet6 => false,
            af_spec => {
                log::error!("Unexpected routing protocol: {:?}", af_spec);
                return Ok(None);
            }
        };

        // By default, the prefix is unspecified.
        let mut prefix = IpNetwork::new(
            if is_ipv4 {
                Ipv4Addr::UNSPECIFIED.into()
            } else {
                Ipv6Addr::UNSPECIFIED.into()
            },
            destination_length,
        )
        .map_err(Error::InvalidNetworkPrefix)?;

        let mut node_addr = None;
        let mut device = None;
        let mut metric = None;
        let mut gateway: Option<IpAddr> = None;
        let mut table_id = u32::from(msg.header.table);
        let mut route_mtu = None;

        for nla in msg.attributes.iter() {
            match nla {
                RouteAttribute::Oif(device_idx) => {
                    match self.iface_map.get(device_idx) {
                        Some(route_device) => {
                            if !route_device.is_loopback() {
                                device = Some(route_device);
                            } else {
                                gateway = if is_ipv4 {
                                    Some(Ipv4Addr::LOCALHOST.into())
                                } else {
                                    Some(Ipv6Addr::LOCALHOST.into())
                                };
                            }
                        }
                        None => {
                            return Err(Error::UnknownDeviceIndex(*device_idx));
                        }
                    };
                }

                RouteAttribute::Via(addr) => {
                    node_addr = Some(Self::parse_ip_from_via(addr)?);
                }

                RouteAttribute::Destination(addr) => {
                    let ip = Self::parse_ip_from_route_address(addr)?;
                    let network = ipnetwork::IpNetwork::new(ip, destination_length)
                        .map_err(Error::InvalidNetworkPrefix)?;
                    prefix = network;
                }

                // gateway NLAs indicate that this is actually a default route
                RouteAttribute::Gateway(gateway_ip) => {
                    gateway = Some(Self::parse_ip_from_route_address(gateway_ip)?)
                }

                RouteAttribute::Priority(priority) => {
                    metric = Some(*priority);
                }

                RouteAttribute::Table(id) => {
                    table_id = *id;
                }

                RouteAttribute::Metrics(metrics) => {
                    let get_mtu = |metric: &RouteMetric| match metric {
                        RouteMetric::Mtu(mtu) => Some(*mtu),
                        _ => None,
                    };
                    let mtu = metrics.iter().find_map(get_mtu);
                    route_mtu = mtu;
                }
                _ => continue,
            }
        }

        if device.is_none() && node_addr.is_none() && gateway.is_none() {
            return Err(Error::InvalidRoute);
        }

        let node = Node {
            ip: node_addr.or(gateway),
            device: device.map(|dev| dev.name.clone()),
        };

        Ok(Some(Route {
            node,
            prefix,
            metric,
            table_id,
            mtu: route_mtu,
        }))
    }

    fn map_interface(msg: LinkMessage) -> Option<(u32, NetworkInterface)> {
        let index = msg.header.index;
        let link_layer_type = msg.header.link_layer_type;
        for nla in msg.attributes {
            if let LinkAttribute::IfName(name) = nla {
                return Some((
                    index,
                    NetworkInterface {
                        name,
                        link_layer_type,
                    },
                ));
            }
        }

        None
    }

    fn parse_ip_from_via(via: &RouteVia) -> Result<IpAddr> {
        let mut bytes = vec![0; via.buffer_len()];
        via.emit(&mut bytes);
        Self::parse_ip(&bytes)
    }

    fn parse_ip_from_route_address(route_address: &RouteAddress) -> Result<IpAddr> {
        let mut bytes = vec![0; route_address.buffer_len()];
        route_address.emit(&mut bytes);
        Self::parse_ip(&bytes)
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

    async fn delete_route_if_exists(&self, route: &Route) -> Result<()> {
        if let Err(error) = self.delete_route(route).await {
            if let Error::Netlink(rtnetlink::Error::NetlinkError(msg)) = &error
                && msg.raw_code() == -libc::ESRCH
            {
                return Ok(());
            }

            Err(error)
        } else {
            Ok(())
        }
    }

    async fn delete_route(&self, route: &Route) -> Result<()> {
        let compat_table = compat_table_id(route.table_id);
        let scope = match route.prefix {
            IpNetwork::V4(v4_prefix) => {
                if v4_prefix.prefix() > 0 && v4_prefix.prefix() < 32 {
                    RouteScope::Link // RT_SCOPE_LINK
                } else {
                    RouteScope::Universe // RT_SCOPE_UNIVERSE
                }
            }
            IpNetwork::V6(v6_prefix) => {
                if v6_prefix.prefix() > 0 && v6_prefix.prefix() < 128 {
                    RouteScope::Link // RT_SCOPE_LINK
                } else {
                    RouteScope::Universe // RT_SCOPE_UNIVERSE
                }
            }
        };

        let mut route_message = {
            RouteMessageBuilder::<IpAddr>::new()
                    .destination_prefix(route.prefix.ip(), route.prefix.prefix())
                    // NOTE: This will only panic if the prefix length is wrong.
                    .unwrap()
                    .protocol(RouteProtocol::Unspec) // RTPROT_UNSPEC
                    .kind(RouteType::Unspec) //RTN_UNSPEC
                    .scope(scope)
                    .table_id(compat_table as u32).build()

            // TODO: Are these important? v
            //source_prefix_length: 0,
            //tos: 0u8,
        };
        if compat_table == RT_TABLE_COMPAT {
            route_message
                .attributes
                .push(RouteAttribute::Table(route.table_id));
        }

        if let Some(interface_name) = route.node.get_device()
            && let Some(iface_idx) = self.find_iface_idx(interface_name)
        {
            route_message
                .attributes
                .push(RouteAttribute::Oif(iface_idx));
        }

        if let Some(gateway) = route.node.get_address() {
            let gateway_nla = if route.node.get_device().is_some() {
                RouteAttribute::Gateway(ip_to_route_address(gateway))
            } else {
                RouteAttribute::Via(ip_to_route_via(gateway))
            };
            route_message.attributes.push(gateway_nla);
        }

        if let Some(metric) = route.metric {
            route_message
                .attributes
                .push(RouteAttribute::Priority(metric));
        }

        self.handle
            .route()
            .del(route_message)
            .execute()
            .await
            .map_err(Error::Netlink)
    }

    async fn add_route_direct(&mut self, route: Route) -> Result<()> {
        let add_message = match &route.prefix {
            IpNetwork::V4(v4_prefix) => {
                let mut add_message = RouteMessageBuilder::<Ipv4Addr>::new()
                    .destination_prefix(v4_prefix.ip(), v4_prefix.prefix());

                if v4_prefix.prefix() > 0 && v4_prefix.prefix() < 32 {
                    add_message = add_message.scope(RouteScope::Link); // RT_SCOPE_LINK
                }

                if let Some(IpAddr::V4(node_address)) = route.node.get_address() {
                    add_message = add_message.gateway(node_address);
                }

                if let Some(interface_name) = route.node.get_device()
                    && let Some(iface_idx) = self.find_iface_idx(interface_name)
                {
                    add_message = add_message.output_interface(iface_idx);
                }

                // TODO: Unused ?
                let compat_table = compat_table_id(route.table_id);
                if compat_table == RT_TABLE_COMPAT {
                    add_message = add_message.table_id(route.table_id);
                }

                // TODO: Request support for route priority in RouteAddIpv{4,6}Request
                if let Some(metric) = route.metric {
                    add_message = add_message.priority(metric);
                }

                let mut msg = add_message.build();

                // Set route MTU
                if let Some(mtu) = route.mtu {
                    // TODO: This can be done before calling `add_message.build()` if
                    // https://github.com/rust-netlink/rtnetlink/pull/126 is merged & released.
                    let mtu = RouteMetric::Mtu(mtu);
                    msg.attributes.push(RouteAttribute::Metrics(vec![mtu]));
                }

                self.handle.route().add(msg)
            }

            IpNetwork::V6(v6_prefix) => {
                let mut add_message = RouteMessageBuilder::<Ipv6Addr>::new()
                    .destination_prefix(v6_prefix.ip(), v6_prefix.prefix());

                if v6_prefix.prefix() > 0 && v6_prefix.prefix() < 128 {
                    add_message = add_message.scope(RouteScope::Link); // RT_SCOPE_LINK
                }

                if let Some(IpAddr::V6(node_address)) = route.node.get_address() {
                    add_message = add_message.gateway(node_address);
                }

                if let Some(interface_name) = route.node.get_device()
                    && let Some(iface_idx) = self.find_iface_idx(interface_name)
                {
                    add_message = add_message.output_interface(iface_idx);
                }

                let compat_table = compat_table_id(route.table_id);
                if compat_table == RT_TABLE_COMPAT {
                    add_message = add_message.table_id(route.table_id);
                }

                // TODO: Request support for route priority in RouteAddIpv{4,6}Request
                if let Some(metric) = route.metric {
                    add_message = add_message.priority(metric);
                }

                let mut msg = add_message.build();

                // Set route MTU
                if let Some(mtu) = route.mtu {
                    // TODO: This can be done before calling `add_message.build()` if
                    // https://github.com/rust-netlink/rtnetlink/pull/126 is merged & released.
                    let mtu = RouteMetric::Mtu(mtu);
                    msg.attributes.push(RouteAttribute::Metrics(vec![mtu]));
                }

                self.handle.route().add(msg)
            }
        };

        add_message
            .replace()
            .execute()
            .await
            .map_err(Error::Netlink)?;

        Ok(())
    }

    async fn add_route(&mut self, route: Route) -> Result<()> {
        self.add_route_direct(route.clone()).await?;
        self.added_routes.insert(route);
        Ok(())
    }

    fn listen(&mut self) -> UnboundedReceiver<CallbackMessage> {
        let (tx, rx) = futures::channel::mpsc::unbounded();
        self.listeners.push(tx);
        rx
    }

    async fn destructor(&mut self) {
        self.cleanup_routes().await;

        if let Err(error) = self.clear_routing_rules().await {
            log::error!(
                "{}",
                error.display_chain_with_msg("Failed to remove routing rules")
            );
        }
    }

    async fn get_mtu_for_route(&self, ip: IpAddr) -> Result<u16> {
        // RECURSION_LIMIT controls how many times we recurse to find the device name by looking up
        // an IP with `get_destination_route`.
        // TODO: Check route MTU first
        const RECURSION_LIMIT: usize = 10;
        const STANDARD_MTU: u16 = 1500;
        let mut attempted_ip = ip;
        for _ in 0..RECURSION_LIMIT {
            let route = self
                .get_destination_route(&attempted_ip, Some(self.fwmark))
                .await?;
            match route {
                Some(route) => {
                    let node = route.get_node();
                    match (node.get_device(), node.get_address()) {
                        (Some(device), _) => {
                            let mtu = self.get_device_mtu(device.to_string()).await?;
                            if mtu != STANDARD_MTU {
                                log::info!(
                                    "Found MTU: {} on device {} which is different from the standard {}",
                                    mtu,
                                    device,
                                    STANDARD_MTU
                                );
                            }
                            return Ok(mtu);
                        }
                        (None, Some(address)) => attempted_ip = address,
                        (None, None) => {
                            log::error!(
                                "Route contains an invalid node which lacks both a device and an address"
                            );
                            return Err(Error::InvalidRouteNode);
                        }
                    }
                }
                None => {
                    log::error!("No route detected when assigning the mtu to the Wireguard tunnel");
                    return Err(Error::NoRoute);
                }
            }
        }
        log::error!(
            "Retried {} times looking for the correct device and could not find it",
            RECURSION_LIMIT
        );
        Err(Error::NoRoute)
    }

    async fn get_device_mtu(&self, device: String) -> Result<u16> {
        let mut links = self.handle.link().get().execute();
        let target_device = LinkAttribute::IfName(device);
        while let Some(msg) = links.try_next().await.map_err(|_| Error::LinkNotFound)? {
            let found = msg.attributes.contains(&target_device);
            if found
                && let Some(LinkAttribute::Mtu(mtu)) = msg
                    .attributes
                    .iter()
                    .find(|e| matches!(e, LinkAttribute::Mtu(_)))
            {
                return Ok(
                    u16::try_from(*mtu).expect("MTU returned by device does not fit into a u16")
                );
            }
        }
        Err(Error::LinkNotFound)
    }

    async fn get_destination_route(
        &self,
        destination: &IpAddr,
        fwmark: Option<u32>,
    ) -> Result<Option<Route>> {
        //let mut request = self.handle.route().get(route_msg);
        let request = {
            let mut builder = RouteMessageBuilder::<IpAddr>::new();
            builder = {
                let prefix_length = match destination {
                    IpAddr::V4(ipv4_addr) => 8u8 * (ipv4_addr.octets().len() as u8),
                    IpAddr::V6(ipv6_addr) => 8u8 * (ipv6_addr.octets().len() as u8),
                };

                // Note: This will only panic if `prefix_length` is wrong for the IP version.
                builder
                    .destination_prefix(*destination, prefix_length)
                    .unwrap()
            };

            let mut request = builder.build();
            if let Some(mark) = fwmark {
                // TODO: This can be done before calling `builder.build()` if
                // https://github.com/rust-netlink/rtnetlink/pull/127 is merged & released.
                let fwmark = RouteAttribute::Mark(mark);
                request.attributes.push(fwmark);
            }

            request.header.flags = RouteFlags::FibMatch; // RTM_F_FIB_MATCH;
            request
        };

        let mut stream = self.handle.route().get(request).execute();
        match stream.try_next().await {
            Ok(Some(route_msg)) => self.parse_route_message(route_msg),
            Ok(None) => Err(Error::NoRoute),
            Err(rtnetlink::Error::NetlinkError(nl_err))
                if nl_err.raw_code() == -libc::ENETUNREACH =>
            {
                Ok(None)
            }
            Err(err) => Err(Error::GetRoute(err)),
        }
    }
}

fn ip_to_route_address(addr: IpAddr) -> RouteAddress {
    match addr {
        IpAddr::V4(addr) => RouteAddress::Inet(addr),
        IpAddr::V6(addr) => RouteAddress::Inet6(addr),
    }
}

fn ip_to_route_via(addr: IpAddr) -> RouteVia {
    match addr {
        IpAddr::V4(addr) => RouteVia::Inet(addr),
        IpAddr::V6(addr) => RouteVia::Inet6(addr),
    }
}

fn compat_table_id(id: u32) -> u8 {
    // RT_TABLE_COMPAT must be combined with nla Table(id)
    if id > 255 { RT_TABLE_COMPAT } else { id as u8 }
}

#[derive(Debug)]
struct NetworkInterface {
    name: String,
    link_layer_type: LinkLayerType,
}

impl NetworkInterface {
    const fn is_loopback(&self) -> bool {
        matches!(self.link_layer_type, LinkLayerType::Loopback)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    /// Tests if dropping inside a tokio runtime panics
    #[test]
    fn test_drop_in_executor() {
        let runtime = tokio::runtime::Runtime::new().expect("Failed to initialize runtime");
        runtime.block_on(async {
            let manager = RouteManagerImpl::new(0, 0)
                .await
                .expect("Failed to initialize route manager");
            std::mem::drop(manager);
        });
    }

    /// Tests if dropping outside a runtime panics
    #[test]
    fn test_drop() {
        let runtime = tokio::runtime::Runtime::new().expect("Failed to initialize runtime");
        let manager = runtime.block_on(async {
            RouteManagerImpl::new(1000, 1000)
                .await
                .expect("Failed to initialize route manager")
        });
        std::mem::drop(manager);
    }
}

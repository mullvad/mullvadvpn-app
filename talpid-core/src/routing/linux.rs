use crate::routing::{imp::RouteManagerCommand, NetNode, Node, RequiredRoute, Route};
use std::{
    collections::{BTreeMap, HashSet},
    io,
    net::{IpAddr, Ipv4Addr, Ipv6Addr},
};
use talpid_types::ErrorExt;

use futures::{channel::mpsc::UnboundedReceiver, future::FutureExt, StreamExt, TryStreamExt};
use ipnetwork::IpNetwork;
use lazy_static::lazy_static;
use netlink_packet_route::{
    constants::{ARPHRD_LOOPBACK, FIB_RULE_INVERT, FR_ACT_TO_TBL},
    link::{nlas::Nla as LinkNla, LinkMessage},
    route::{nlas::Nla as RouteNla, RouteHeader, RouteMessage},
    rtnl::{
        constants::{
            RTN_UNSPEC, RTPROT_UNSPEC, RT_SCOPE_LINK, RT_SCOPE_UNIVERSE, RT_TABLE_COMPAT,
            RT_TABLE_MAIN,
        },
        RouteFlags,
    },
    rule::{nlas::Nla as RuleNla, RuleHeader, RuleMessage},
    NetlinkMessage, NetlinkPayload, RtnlMessage,
};
use rtnetlink::{
    constants::{RTMGRP_IPV4_ROUTE, RTMGRP_IPV6_ROUTE, RTMGRP_LINK, RTMGRP_NOTIFY},
    sys::SocketAddr,
    Handle,
};

use libc::{AF_INET, AF_INET6};


lazy_static! {
    static ref SUPPRESS_RULE_V4: RuleMessage = RuleMessage {
        header: RuleHeader {
            family: AF_INET as u8,
            action: FR_ACT_TO_TBL,
            ..RuleHeader::default()
        },
        nlas: vec![
            RuleNla::SuppressPrefixLen(0),
            RuleNla::Table(RT_TABLE_MAIN as u32),
        ],
    };
    static ref SUPPRESS_RULE_V6: RuleMessage = {
        let mut v6_rule = SUPPRESS_RULE_V4.clone();
        v6_rule.header.family = AF_INET6 as u8;
        v6_rule
    };
    static ref NO_FWMARK_RULE_V4: RuleMessage = RuleMessage {
        header: RuleHeader {
            family: AF_INET as u8,
            action: FR_ACT_TO_TBL,
            flags: FIB_RULE_INVERT,
            ..RuleHeader::default()
        },
        nlas: vec![
            RuleNla::FwMark(crate::linux::TUNNEL_FW_MARK),
            RuleNla::Table(crate::linux::TUNNEL_TABLE_ID),
        ],
    };
    static ref NO_FWMARK_RULE_V6: RuleMessage = {
        let mut v6_rule = NO_FWMARK_RULE_V4.clone();
        v6_rule.header.family = AF_INET6 as u8;
        v6_rule
    };
    static ref ALL_RULES: [&'static RuleMessage; 4] = [
        &*NO_FWMARK_RULE_V4,
        &*NO_FWMARK_RULE_V6,
        &*SUPPRESS_RULE_V4,
        &*SUPPRESS_RULE_V6,
    ];
}


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

    #[error(display = "Unknown device index - {}", _0)]
    UnknownDeviceIndex(u32),

    /// Unable to create routing table for tagged connections and packets.
    #[error(display = "Cannot find a free routing table ID")]
    NoFreeRoutingTableId,

    #[error(display = "Shutting down route manager")]
    Shutdown,
}


pub struct RouteManagerImpl {
    handle: Handle,
    messages: UnboundedReceiver<(NetlinkMessage<RtnlMessage>, SocketAddr)>,
    iface_map: BTreeMap<u32, NetworkInterface>,

    // currently added routes
    added_routes: HashSet<Route>,
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

        let mut monitor = Self {
            iface_map,
            handle,
            messages,
            added_routes: HashSet::new(),
        };

        monitor.clear_routing_rules().await?;
        monitor.add_required_routes(required_routes).await?;

        Ok(monitor)
    }

    async fn create_routing_rules(&mut self, enable_ipv6: bool) -> Result<()> {
        use netlink_packet_route::constants::*;

        self.clear_routing_rules().await?;

        for rule in ALL_RULES
            .iter()
            .filter(|rule| rule.header.family as u16 == AF_INET || enable_ipv6)
        {
            let mut req = NetlinkMessage::from(RtnlMessage::NewRule((*rule).clone()));
            req.header.flags = NLM_F_REQUEST | NLM_F_ACK | NLM_F_CREATE | NLM_F_REPLACE;

            let mut response = self.handle.request(req).map_err(Error::NetlinkError)?;

            while let Some(message) = response.next().await {
                if let NetlinkPayload::Error(error) = message.payload {
                    return Err(Error::NetlinkError(rtnetlink::Error::NetlinkError(error)));
                }
            }
        }
        Ok(())
    }

    async fn clear_routing_rules(&mut self) -> Result<()> {
        let rules = self.get_rules().await?;
        for rule in &*ALL_RULES {
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
                for nla in &rule.nlas {
                    if !found_rule.nlas.contains(nla) {
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
        use netlink_packet_route::constants::*;

        let mut req = NetlinkMessage::from(RtnlMessage::GetRule(RuleMessage::default()));
        req.header.flags = NLM_F_REQUEST | NLM_F_ACK | NLM_F_DUMP;

        let mut response = self.handle.request(req).map_err(Error::NetlinkError)?;

        let mut rules = vec![];

        while let Some(message) = response.next().await {
            match message.payload {
                NetlinkPayload::InnerMessage(RtnlMessage::NewRule(rule)) => {
                    rules.push(rule);
                }
                NetlinkPayload::Error(error) => {
                    return Err(Error::NetlinkError(rtnetlink::Error::NetlinkError(error)));
                }
                _ => (),
            }
        }
        Ok(rules)
    }

    async fn delete_rule_if_exists(&mut self, rule: RuleMessage) -> Result<()> {
        use netlink_packet_route::constants::*;

        let mut req = NetlinkMessage::from(RtnlMessage::DelRule(rule));
        req.header.flags = NLM_F_REQUEST | NLM_F_ACK;

        let mut response = self.handle.request(req).map_err(Error::NetlinkError)?;

        while let Some(message) = response.next().await {
            if let NetlinkPayload::Error(error) = message.payload {
                if error.to_io().kind() != io::ErrorKind::NotFound {
                    return Err(Error::NetlinkError(rtnetlink::Error::NetlinkError(error)));
                }
            }
        }
        Ok(())
    }

    async fn add_required_routes(&mut self, required_routes: HashSet<RequiredRoute>) -> Result<()> {
        let mut required_normal_routes = HashSet::new();

        for route in required_routes {
            match route.node {
                NetNode::RealNode(node) => {
                    required_normal_routes
                        .insert(Route::new(node, route.prefix).table(route.table_id));
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
        while let Some(link) = link_request.try_next().await.map_err(Error::NetlinkError)? {
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

    async fn process_deleted_route(&mut self, route: Route) -> Result<()> {
        self.added_routes.remove(&route);
        Ok(())
    }

    async fn cleanup_routes(&mut self) {
        for route in self.added_routes.drain().collect::<Vec<_>>().iter() {
            if let Err(e) = self.delete_route_if_exists(&route).await {
                log::error!("Failed to remove route - {} - {}", route, e);
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
                if let Some((idx, name)) = Self::map_interface(new_link) {
                    self.iface_map.insert(idx, name);
                }
            }
            NetlinkPayload::InnerMessage(RtnlMessage::DelLink(old_link)) => {
                if let Some((idx, _)) = Self::map_interface(old_link) {
                    self.iface_map.remove(&idx);
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
        self.parse_route_message_inner(msg)
    }

    // Tries to coax a Route out of a RouteMessage
    fn parse_route_message_inner(&self, msg: RouteMessage) -> Result<Option<Route>> {
        let af_spec = msg.header.address_family;
        let destination_length = msg.header.destination_prefix_length;
        let is_ipv4 = match af_spec as i32 {
            AF_INET => true,
            AF_INET6 => false,
            af_spec => {
                log::error!("Unexpected routing protocol: {}", af_spec);
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

        for nla in msg.nlas.iter() {
            match nla {
                RouteNla::Oif(device_idx) => {
                    match self.iface_map.get(&device_idx) {
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

                RouteNla::Via(addr) => {
                    node_addr = Self::parse_ip(&addr).map(Some)?;
                }

                RouteNla::Destination(addr) => {
                    prefix = Self::parse_ip(&addr).and_then(|ip| {
                        ipnetwork::IpNetwork::new(ip, destination_length)
                            .map_err(Error::InvalidNetworkPrefix)
                    })?;
                }

                // gateway NLAs indicate that this is actually a default route
                RouteNla::Gateway(gateway_ip) => {
                    gateway = Self::parse_ip(&gateway_ip).map(Some)?;
                }

                RouteNla::Priority(priority) => {
                    metric = Some(*priority);
                }

                RouteNla::Table(id) => {
                    table_id = *id;
                }
                _ => continue,
            }
        }

        if device.is_none() && node_addr.is_none() && gateway.is_none() {
            return Err(Error::InvalidRoute);
        }


        let node = Node {
            ip: node_addr.or(gateway.into()),
            device: device.map(|dev| dev.name.clone()),
        };

        let result = Ok(Some(Route {
            node,
            prefix,
            metric,
            table_id,
        }));
        result
    }

    fn map_interface(msg: LinkMessage) -> Option<(u32, NetworkInterface)> {
        let index = msg.header.index;
        let link_layer_type = msg.header.link_layer_type;
        for nla in msg.nlas {
            if let LinkNla::IfName(name) = nla {
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
            if let Error::NetlinkError(netlink_error) = &error {
                if let rtnetlink::Error::NetlinkError(msg) = &netlink_error {
                    if msg.code == -libc::ESRCH {
                        return Ok(());
                    }
                }
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
                    RT_SCOPE_LINK
                } else {
                    RT_SCOPE_UNIVERSE
                }
            }
            IpNetwork::V6(v6_prefix) => {
                if v6_prefix.prefix() > 0 && v6_prefix.prefix() < 128 {
                    RT_SCOPE_LINK
                } else {
                    RT_SCOPE_UNIVERSE
                }
            }
        };

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
                table: compat_table,
                protocol: RTPROT_UNSPEC,
                scope,
                kind: RTN_UNSPEC,
                flags: RouteFlags::empty(),
            },
            nlas: vec![RouteNla::Destination(ip_to_bytes(route.prefix.ip()))],
        };
        if compat_table == RT_TABLE_COMPAT {
            route_message.nlas.push(RouteNla::Table(route.table_id));
        }

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

        if let Some(metric) = route.metric {
            route_message.nlas.push(RouteNla::Priority(metric));
        }

        self.handle
            .route()
            .del(route_message)
            .execute()
            .await
            .map_err(Error::NetlinkError)
    }

    async fn add_route_direct(&mut self, route: Route) -> Result<()> {
        let mut add_message = match &route.prefix {
            IpNetwork::V4(v4_prefix) => {
                let mut add_message = self
                    .handle
                    .route()
                    .add_v4()
                    .destination_prefix(v4_prefix.ip(), v4_prefix.prefix());

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
                    .destination_prefix(v6_prefix.ip(), v6_prefix.prefix());

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

        let compat_table = compat_table_id(route.table_id);
        add_message.header.table = compat_table;
        if compat_table == RT_TABLE_COMPAT {
            add_message.nlas.push(RouteNla::Table(route.table_id));
        }

        // TODO: Request support for route priority in RouteAddIpv{4,6}Request
        if let Some(metric) = route.metric {
            add_message.nlas.push(RouteNla::Priority(metric));
        }

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
        Ok(())
    }

    async fn add_route(&mut self, route: Route) -> Result<()> {
        self.add_route_direct(route.clone()).await?;
        self.added_routes.insert(route);
        Ok(())
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
}

fn ip_to_bytes(addr: IpAddr) -> Vec<u8> {
    match addr {
        IpAddr::V4(addr) => addr.octets().to_vec(),
        IpAddr::V6(addr) => addr.octets().to_vec(),
    }
}

fn compat_table_id(id: u32) -> u8 {
    // RT_TABLE_COMPAT must be combined with nla Table(id)
    if id > 255 {
        RT_TABLE_COMPAT
    } else {
        id as u8
    }
}

#[derive(Debug)]
struct NetworkInterface {
    name: String,
    link_layer_type: u16,
}

impl NetworkInterface {
    fn is_loopback(&self) -> bool {
        self.link_layer_type == ARPHRD_LOOPBACK
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

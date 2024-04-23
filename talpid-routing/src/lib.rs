//! Manage routing tables on various platforms.
#![allow(rustdoc::private_intra_doc_links)]
#![deny(missing_docs)]

use ipnetwork::IpNetwork;
use std::{fmt, net::IpAddr};

#[cfg(any(target_os = "windows", target_os = "macos"))]
/// Burst guard
pub mod debounce;

#[cfg(target_os = "windows")]
#[path = "windows/mod.rs"]
mod imp;

#[cfg(target_os = "windows")]
pub use imp::{get_best_default_route, CallbackHandle, EventType, InterfaceAndGateway};

#[cfg(not(target_os = "windows"))]
#[path = "unix/mod.rs"]
mod imp;

#[cfg(target_os = "linux")]
use netlink_packet_route::rtnl::constants::RT_TABLE_MAIN;

#[cfg(target_os = "macos")]
pub use imp::{imp::RouteError, DefaultRouteEvent, PlatformError};

pub use imp::{Error, RouteManagerHandle};

/// Link-layer/MAC adress
#[cfg(target_os = "macos")]
#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub struct MacAddress(pub [u8; 6]);

#[cfg(target_os = "macos")]
impl MacAddress {
    /// Consume bytes that make up the link address
    pub fn into_bytes(self) -> [u8; 6] {
        self.0
    }
}

#[cfg(target_os = "macos")]
impl From<[u8; 6]> for MacAddress {
    fn from(addr: [u8; 6]) -> Self {
        Self(addr)
    }
}

#[cfg(target_os = "macos")]
impl fmt::Display for MacAddress {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{:<02X}{:<02X}{:<02X}{:<02X}{:<02X}{:<02X}",
            self.0[0], self.0[1], self.0[2], self.0[3], self.0[4], self.0[5]
        )
    }
}

/// Gateway, including IP address and MAC address
#[cfg(target_os = "macos")]
#[derive(Debug, Eq, PartialEq, Clone)]
pub struct Gateway {
    /// Network layer address for the gateway
    pub ip_address: IpAddr,
    /// Link layer address for the gateway
    pub mac_address: MacAddress,
}

/// A network route with a specific network node, destination and an optional metric.
#[derive(Debug, Hash, Eq, PartialEq, Clone)]
pub struct Route {
    node: Node,
    prefix: IpNetwork,
    metric: Option<u32>,
    #[cfg(target_os = "linux")]
    table_id: u32,
    #[cfg(any(target_os = "linux", target_os = "macos"))]
    mtu: Option<u32>,
}

impl Route {
    /// Construct a new Route
    pub fn new(node: Node, prefix: IpNetwork) -> Self {
        Self {
            node,
            prefix,
            metric: None,
            #[cfg(target_os = "linux")]
            table_id: u32::from(RT_TABLE_MAIN),
            #[cfg(any(target_os = "linux", target_os = "macos"))]
            mtu: None,
        }
    }

    #[cfg(target_os = "linux")]
    fn table(mut self, new_id: u32) -> Self {
        self.table_id = new_id;
        self
    }

    /// Returns the network node of the route.
    pub fn get_node(&self) -> &Node {
        &self.node
    }
}

impl fmt::Display for Route {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} via {}", self.prefix, self.node)?;
        if let Some(metric) = &self.metric {
            write!(f, " metric {}", *metric)?;
        }
        #[cfg(target_os = "linux")]
        write!(f, " table {}", self.table_id)?;
        #[cfg(target_os = "linux")]
        if let Some(mtu) = self.mtu {
            write!(f, " mtu {mtu}")?;
        }
        Ok(())
    }
}

/// A network route that should be applied by the route manager.
/// It can either be routed through a specific network node or it can be routed through the current
/// default route.
#[derive(Debug, Hash, Eq, PartialEq, Clone)]
pub struct RequiredRoute {
    /// Route's prefix
    pub prefix: IpNetwork,
    node: NetNode,
    /// Specifies whether the route should be added to the main routing table or not.
    #[cfg(target_os = "linux")]
    main_table: bool,
    /// Specifies route MTU
    #[cfg(any(target_os = "linux", target_os = "macos"))]
    mtu: Option<u16>,
}

impl RequiredRoute {
    /// Constructs a new required route.
    pub fn new(prefix: IpNetwork, node: impl Into<NetNode>) -> Self {
        Self {
            node: node.into(),
            prefix,
            #[cfg(target_os = "linux")]
            main_table: true,
            #[cfg(any(target_os = "linux", target_os = "macos"))]
            mtu: None,
        }
    }

    /// Sets the routing table ID of the route.
    #[cfg(target_os = "linux")]
    pub fn use_main_table(mut self, main_table: bool) -> Self {
        self.main_table = main_table;
        self
    }

    /// Set route MTU to the given value.
    #[cfg(any(target_os = "linux", target_os = "macos"))]
    pub fn mtu(mut self, mtu: u16) -> Self {
        self.mtu = Some(mtu);
        self
    }
}

/// A NetNode represents a network node - either a real one or a symbolic default one.
/// A route with a symbolic default node will be changed whenever a new default route is created.
#[derive(Debug, Hash, Eq, PartialEq, Clone)]
pub enum NetNode {
    /// A real node will be used to set a regular route that will remain unchanged for the lifetime
    /// of the route manager
    RealNode(Node),
    /// A default node is a symbolic node that will resolve to the network node used in the current
    /// most preferable default route
    #[cfg(not(target_os = "linux"))]
    DefaultNode,
}

impl From<Node> for NetNode {
    fn from(node: Node) -> NetNode {
        NetNode::RealNode(node)
    }
}

/// Node represents a real network node - it can be identified by a network interface name, an IP
/// address or both.
#[derive(Debug, Hash, Eq, PartialEq, Clone)]
pub struct Node {
    ip: Option<IpAddr>,
    device: Option<String>,
}

impl Node {
    /// Construct an Node with both an IP address and an interface name.
    pub fn new(address: IpAddr, iface_name: String) -> Self {
        Self {
            ip: Some(address),
            device: Some(iface_name),
        }
    }

    /// Construct an Node from an IP address.
    pub fn address(address: IpAddr) -> Node {
        Self {
            ip: Some(address),
            device: None,
        }
    }

    /// Construct a Node from a network interface name.
    pub fn device(iface_name: String) -> Node {
        Self {
            ip: None,
            device: Some(iface_name),
        }
    }

    /// Retrieve a node's IP address
    pub fn get_address(&self) -> Option<IpAddr> {
        self.ip
    }

    /// Retrieve a node's network interface name
    pub fn get_device(&self) -> Option<&str> {
        self.device.as_ref().map(|s| s.as_ref())
    }
}

impl fmt::Display for Node {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(ip) = &self.ip {
            write!(f, "{ip}")?;
        }
        if let Some(device) = &self.device {
            let extra_space = if self.ip.is_some() { " " } else { "" };
            write!(f, "{extra_space}dev {device}")?;
        }
        Ok(())
    }
}

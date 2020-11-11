#![cfg_attr(target_os = "android", allow(dead_code))]
#![cfg_attr(target_os = "windows", allow(dead_code))]

use ipnetwork::IpNetwork;
use std::{fmt, net::IpAddr};

#[cfg(target_os = "windows")]
#[path = "windows.rs"]
mod imp;

#[cfg(not(target_os = "windows"))]
#[path = "unix.rs"]
mod imp;

#[cfg(target_os = "linux")]
use netlink_packet_route::rtnl::constants::RT_TABLE_MAIN;

pub use imp::{Error, RouteManager};

#[cfg(target_os = "linux")]
pub use imp::RouteManagerCommand;

/// A netowrk route with a specific network node, destinaiton and an optional metric.
#[derive(Debug, Hash, Eq, PartialEq, Clone)]
pub struct Route {
    node: Node,
    prefix: IpNetwork,
    metric: Option<u32>,
    #[cfg(target_os = "linux")]
    table_id: u8,
}

impl Route {
    fn new(node: Node, prefix: IpNetwork) -> Self {
        Self {
            node,
            prefix,
            metric: None,
            #[cfg(target_os = "linux")]
            table_id: RT_TABLE_MAIN,
        }
    }

    /// Returns route that only contains the device node if a device node exists.
    #[cfg(target_os = "linux")]
    fn device_only_route(&self) -> Option<Self> {
        if let Some(device) = self.node.get_device() {
            Some(Self {
                node: Node::device(device.to_string()),
                ..self.clone()
            })
        } else {
            None
        }
    }

    #[cfg(target_os = "linux")]
    fn table(mut self, new_id: u8) -> Self {
        self.table_id = new_id;
        self
    }

    #[cfg(target_os = "linux")]
    fn is_loopback(&self) -> bool {
        self.node.ip.map(|ip| ip.is_loopback()).unwrap_or(false)
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
        Ok(())
    }
}

/// A network route that should be applied by the RouteManager.
/// It can either be routed through a specific network node or it can be routed through the current
/// default route.
#[derive(Debug, Hash, Eq, PartialEq, Clone)]
pub struct RequiredRoute {
    prefix: IpNetwork,
    node: NetNode,
    #[cfg(target_os = "linux")]
    table_id: u8,
}

impl RequiredRoute {
    /// Constructs a new required route.
    pub fn new(prefix: IpNetwork, node: impl Into<NetNode>) -> Self {
        Self {
            node: node.into(),
            prefix,
            #[cfg(target_os = "linux")]
            table_id: RT_TABLE_MAIN,
        }
    }

    /// Sets the routing table ID of the route.
    #[cfg(target_os = "linux")]
    pub fn table(mut self, new_id: u8) -> Self {
        self.table_id = new_id;
        self
    }
}

/// A NetNode represents a network node - either a real one or a symbolic default one.
/// A route with a symbolic default node will be changed whenever a new default route is created.
#[derive(Debug, Hash, Eq, PartialEq, Clone)]
pub enum NetNode {
    /// A real node will be used to set a regular route that will remain unchanged for the lifetime
    /// of the RouteManager
    RealNode(Node),
    /// A default node is a symbolic node that will resolve to the network node used in the current
    /// most preferable default route
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
            write!(f, "{}", ip)?;
        }
        if let Some(device) = &self.device {
            let extra_space = if self.ip.is_some() { " " } else { "" };
            write!(f, "{}dev {}", extra_space, device)?;
        }
        Ok(())
    }
}

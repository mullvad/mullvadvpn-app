use ipnetwork::IpNetwork;
use std::net::IpAddr;

mod subprocess;


/// A single route
#[derive(Hash, Eq, PartialEq)]
pub struct Route {
    /// Route prefix
    pub prefix: IpNetwork,
    /// Route node
    pub node: NetNode,
}

impl Route {
    /// Create a new route
    pub fn new(prefix: IpNetwork, node: NetNode) -> Self {
        Self { prefix, node }
    }
}

/// A network node for a given route
#[derive(Hash, Eq, PartialEq, Clone)]
pub enum NetNode {
    /// For routing something through a network host
    Address(IpAddr),
    /// For routing something through an interface
    Device(String),
}

/// Contains a set of routes to be added
pub struct RequiredRoutes {
    /// List of routes to be applied to the routing table.
    pub routes: Vec<Route>,
    /// Optionally apply the routes to a specific table and only apply routes when a firewall mark
    /// is not used. Currently only used on Linux.
    pub fwmark: Option<String>,
}

/// This trait unifies platform specific implementations of route managers
pub trait RoutingT: Sized {
    /// Error type of the implementation
    type Error: ::std::error::Error;

    /// Creates a new router
    fn new() -> Result<Self, Self::Error>;

    /// Adds routes to the system routing table.
    fn add_routes(&mut self, required_routes: RequiredRoutes) -> Result<(), Self::Error>;

    /// Removes previously set routes. If routes were set for specific tables, the whole tables
    /// will be removed.
    fn delete_routes(&mut self) -> Result<(), Self::Error>;

    /// Retrieves the gateway for the default route
    fn get_default_route_node(&mut self) -> Result<std::net::IpAddr, Self::Error>;
}

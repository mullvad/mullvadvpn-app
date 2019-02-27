use ipnetwork::IpNetwork;
use std::net::IpAddr;

#[cfg(target_os = "macos")]
#[path = "macos.rs"]
mod imp;

#[cfg(target_os = "linux")]
#[path = "linux.rs"]
mod imp;

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
}

/// Manages adding and removing routes from the routing table.
pub struct RouteManager {
    inner: imp::RouteManager,
}

impl RouteManager {
    /// Creates a new RouteManager.
    pub fn new() -> Result<Self, imp::Error> {
        Ok(RouteManager {
            inner: imp::RouteManager::new()?,
        })
    }

    /// Set routes in the routing table.
    pub fn add_routes(&mut self, required_routes: RequiredRoutes) -> Result<(), imp::Error> {
        self.inner.add_routes(required_routes)
    }

    /// Remove previously set routes from the routing table.
    pub fn delete_routes(&mut self) -> Result<(), imp::Error> {
        self.inner.delete_routes()
    }

    /// Retrieves the gateway for the default route.
    pub fn get_default_route_node(&mut self) -> Result<std::net::IpAddr, imp::Error> {
        // use routing::RoutingT;
        self.inner.get_default_route_node()
    }
}

impl Drop for RouteManager {
    fn drop(&mut self) {
        if let Err(e) = self.delete_routes() {
            log::error!("Failed to reset routes on drop - {}", e);
        }
    }
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

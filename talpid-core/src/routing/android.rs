use super::RequiredRoutes;
use std::net::{IpAddr, Ipv4Addr};

/// Stub error type for routing errors on Android.
#[derive(Debug, err_derive::Error)]
#[error(display = "Unknown Android routing error")]
pub struct Error;

pub struct RouteManager;

impl super::RoutingT for RouteManager {
    type Error = Error;

    fn new() -> Result<Self, Self::Error> {
        Ok(RouteManager)
    }

    fn add_routes(&mut self, _required_routes: RequiredRoutes) -> Result<(), Self::Error> {
        Ok(())
    }

    fn delete_routes(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }

    fn get_default_route_node(&mut self) -> Result<IpAddr, Self::Error> {
        Ok(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)))
    }
}

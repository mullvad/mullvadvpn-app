use super::{
    subprocess::{Exec, RunExpr},
    NetNode, RequiredRoutes,
};
use std::{
    collections::HashSet,
    net::{IpAddr, Ipv4Addr},
};

error_chain! {}

pub struct RouteManager;

impl super::RoutingT for RouteManager {
    type Error = Error;

    fn new() -> Result<Self> {
        Ok(RouteManager)
    }

    fn add_routes(&mut self, _required_routes: RequiredRoutes) -> Result<()> {
        Ok(())
    }

    fn delete_routes(&mut self) -> Result<()> {
        Ok(())
    }

    fn get_default_route_node(&mut self) -> Result<IpAddr> {
        Ok(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)))
    }
}

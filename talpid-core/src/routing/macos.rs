use super::{NetNode, RequiredRoutes, Route};

use super::subprocess::{Exec, RunExpr};
use std::{collections::HashSet, net::IpAddr};

error_chain! {
    errors {
        FailedToAddRoute {
            description("Failed to add route")
        }

        FailedToGetDefaultRoute {
            description("Failed to get default route")
        }

        FailedToRemoveRoute {
            description("Failed to remove route")
        }
    }
}

pub struct RouteManager {
    set_routes: HashSet<Route>,
}

impl RouteManager {
    fn add_route(&mut self, route: Route) -> Result<()> {
        if route.prefix.prefix() == 0 {
            return if route.prefix.is_ipv4() {
                self.add_route(Route::new("0.0.0.0/1".parse().unwrap(), route.node.clone()))?;
                self.add_route(Route::new(
                    "128.0.0.0/1".parse().unwrap(),
                    route.node.clone(),
                ))
            } else {
                self.add_route(Route::new("::/1".parse().unwrap(), route.node.clone()))?;
                self.add_route(Route::new("8000::/1".parse().unwrap(), route.node.clone()))
            };
        }

        let mut cmd = Exec::cmd("route")
            .arg("-q")
            .arg("-n")
            .arg("add")
            .arg(ip_vers(&route))
            .arg(route.prefix.to_string());
        cmd = match &route.node {
            NetNode::Address(ref addr) => cmd.arg("-gateway").arg(addr.to_string()),
            NetNode::Device(device) => cmd.arg("-interface").arg(&device),
        };

        cmd.into_expr()
            .run_expr()
            .chain_err(|| ErrorKind::FailedToAddRoute)?;
        self.set_routes.insert(route);
        Ok(())
    }
}

fn ip_vers(route: &Route) -> &'static str {
    if route.prefix.is_ipv4() {
        "-inet"
    } else {
        "-inet6"
    }
}

impl super::RoutingT for RouteManager {
    type Error = Error;

    fn new() -> Result<Self> {
        Ok(Self {
            set_routes: HashSet::new(),
        })
    }

    fn add_routes(&mut self, required_routes: RequiredRoutes) -> Result<()> {
        for route in required_routes.routes.into_iter() {
            if let Err(e) = self.add_route(route) {
                let _ = self.delete_routes();
                return Err(e);
            }
        }
        Ok(())
    }

    fn delete_routes(&mut self) -> Result<()> {
        let mut end_result = Ok(());
        for route in self.set_routes.drain() {
            let result = duct::cmd!(
                "route",
                "-q",
                "-n",
                "delete",
                ip_vers(&route),
                route.prefix.to_string()
            )
            .run_expr()
            .chain_err(|| ErrorKind::FailedToRemoveRoute);
            if let Err(e) = result {
                log::error!("failed to reset remove route: {}", e);
                end_result = Err(e);
            }
        }
        // returning the last error as to signal some kind of failure.
        end_result
    }


    fn get_default_route_node(&mut self) -> Result<IpAddr> {
        let output = duct::cmd!("route", "-n", "get", "default")
            .stdout()
            .chain_err(|| ErrorKind::FailedToGetDefaultRoute)?;
        let ip_str: &str = output
            .lines()
            .find(|line| line.trim().starts_with("gateway: "))
            .and_then(|line| line.trim().split_whitespace().skip(1).next())
            .ok_or(Error::from(ErrorKind::FailedToGetDefaultRoute))?;

        ip_str
            .parse()
            .map_err(|_| Error::from(ErrorKind::FailedToGetDefaultRoute))
    }
}

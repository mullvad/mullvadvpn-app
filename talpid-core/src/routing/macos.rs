use super::{NetNode, RequiredRoutes, Route};

use super::subprocess::{Exec, RunExpr};
use std::collections::HashSet;
use std::net::IpAddr;

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
    fn set_route(&mut self, route: Route) -> Result<()> {
        if route.prefix.prefix() == 0 {
            if route.prefix.is_ipv4() {
                self.set_route(Route::new("0.0.0.0/1".parse().unwrap(), route.node.clone()))?;
                self.set_route(Route::new("128.0.0.0/1".parse().unwrap(), route.node.clone()))?;
            } else {
                self.set_route(Route::new("::/1".parse().unwrap(), route.node.clone()))?;
                self.set_route(Route::new("8000::/1".parse().unwrap(), route.node.clone()))?;
            }
        };

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

        cmd.to_expr()
            .run_expr()
            .chain_err(|| ErrorKind::FailedToAddRoute)?;
        self.set_routes.insert(route);
        Ok(())
    }

    fn clear_routes(&mut self) -> Result<()> {
        for route in self.set_routes.drain() {
            Exec::cmd("route")
                .arg("-q")
                .arg("-n")
                .arg("delete")
                .arg(ip_vers(&route))
                .arg(route.prefix.to_string())
                .to_expr()
                .run_expr()
                .chain_err(|| ErrorKind::FailedToRemoveRoute)?;
        }
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

    fn set_routes(&mut self, required_routes: RequiredRoutes) -> Result<()> {
        for route in required_routes.routes.into_iter() {
            if let Err(e) = self.set_route(route) {
                let _ = self.reset_routes();
                return Err(e);
            }
        }
        Ok(())
    }

    fn reset_routes(&mut self) -> Result<()> {
        self.clear_routes()
    }


    fn get_default_route_node(&mut self) -> Result<IpAddr> {
        let output = Exec::cmd("route")
            .arg("-n")
            .arg("get")
            .arg("default")
            .to_expr()
            .stdout_capture()
            .run()
            .chain_err(|| ErrorKind::FailedToGetDefaultRoute)
            .map(|output| String::from_utf8_lossy(&output.stdout).into_owned())?;
        let ip_str: &str = output
            .lines()
            .find(|line| line.trim().starts_with("gateway: "))
            .and_then(|line| line.trim().split_whitespace().skip(1).next())
            .map(Ok)
            .unwrap_or(Err(Error::from(ErrorKind::FailedToGetDefaultRoute)))?;

        ip_str
            .parse()
            .map_err(|_| Error::from(ErrorKind::FailedToGetDefaultRoute))
    }
}

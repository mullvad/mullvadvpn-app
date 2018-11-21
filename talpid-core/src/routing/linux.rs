use super::{NetNode, RequiredRoutes, Route};

use super::subprocess::{Exec, RunExpr};
use ipnetwork::IpNetwork;
use std::collections::HashSet;
use std::net::IpAddr;


error_chain! {
    errors {
        FailedToAddRoute {
            description("Failed to add route")
        }
        FailedToRemoveRoute {
            description("Failed to remove route")
        }

        FailedToRemoveTable {
            description("Failed to remove table")
        }

        FailedToAdjustMainRoutingTable {
            description("Failed to adjust main routing table")
        }

        FailedToSetRuleForFwmark {
            description("Failed to set rule for fwmark")
        }
        NoDefaultRoute {
            description("No default route")
        }

        FailedToGetDefaultRoute {
            description("Failed to get default route")
        }
    }
}

#[derive(Hash, Eq, PartialEq)]
enum IpVersionArg {
    V4,
    V6,
}

impl IpVersionArg {
    fn new(ip: &IpAddr) -> Self {
        if ip.is_ipv4() {
            IpVersionArg::V4
        } else {
            IpVersionArg::V6
        }
    }

    fn is_ipv4(&self) -> bool {
        match self {
            IpVersionArg::V4 => true,
            _ => false,
        }
    }
}

impl AsRef<str> for IpVersionArg {
    fn as_ref(&self) -> &str {
        match self {
            IpVersionArg::V4 => "-4",
            IpVersionArg::V6 => "-6",
        }
    }
}

#[derive(Hash, Eq, PartialEq)]
struct SetTable {
    version: IpVersionArg,
    fwmark: String,
}

pub struct RouteManager {
    set_routes: HashSet<super::Route>,
    set_tables: HashSet<SetTable>,
    // the main routing table only has to be adjusted for default routes
    main_v4_table_adjusted: bool,
    main_v6_table_adjusted: bool,
}

impl RouteManager {
    fn adjust_main_routing_table(&mut self, route: &IpNetwork) -> Result<()> {
        let route_arg = IpVersionArg::new(&route.ip());
        if route_arg.is_ipv4() && self.main_v4_table_adjusted {
            return Ok(());
        } else if self.main_v6_table_adjusted {
            return Ok(());
        }

        Exec::cmd("ip")
            .arg(route_arg.as_ref())
            .arg("rule")
            .arg("add")
            .arg("table")
            .arg("main")
            .arg("suppress_prefixlength")
            .arg("0")
            .to_expr()
            .run_expr()
            .chain_err(|| ErrorKind::FailedToAdjustMainRoutingTable)?;
        Ok(())
    }

    fn set_route(&mut self, route: super::Route, fwmark: &Option<String>) -> Result<()> {
        if route.prefix.prefix() == 0 {
            self.adjust_main_routing_table(&route.prefix)?;
        }

        let version = IpVersionArg::new(&route.prefix.ip());

        let mut cmd = Exec::cmd("ip")
            .arg(version.as_ref())
            .arg("route")
            .arg("add")
            .arg(route.prefix.to_string());
        cmd = match &route.node {
            NetNode::Address(ref addr) => cmd.arg(addr.to_string()),
            NetNode::Device(ref device) => cmd.arg("dev").arg(device),
        };

        if let Some(ref fwmark) = &fwmark {
            cmd = cmd.arg("table").arg(fwmark);
        }

        cmd.to_expr()
            .run_expr()
            .chain_err(|| ErrorKind::FailedToAddRoute)?;

        if let Some(fwmark) = &fwmark {
            self.set_table_rules(SetTable {
                version,
                fwmark: fwmark.to_string(),
            })?;
        } else {
            self.set_routes.insert(route);
        }
        Ok(())
    }

    fn set_table_rules(&mut self, set_table: SetTable) -> Result<()> {
        if self.set_tables.contains(&set_table) {
            return Ok(());
        }
        Exec::cmd("ip")
            .arg(set_table.version.as_ref())
            .arg("rule")
            .arg("add")
            .arg("not")
            .arg("fwmark")
            .arg(&set_table.fwmark)
            .arg("table")
            .arg(&set_table.fwmark)
            .to_expr()
            .run_expr()
            .chain_err(|| ErrorKind::FailedToSetRuleForFwmark)?;


        self.set_tables.insert(set_table);
        Ok(())
    }

    fn clear_routes(&mut self) -> Result<()> {
        for route in self.set_routes.drain() {
            Exec::cmd("ip")
                .arg(ip_vers(&route))
                .arg("route")
                .arg("delete")
                .arg(route.prefix.to_string())
                .to_expr()
                .run_expr()
                .chain_err(|| ErrorKind::FailedToRemoveRoute)?;
        }
        Ok(())
    }

    fn clear_tables(&mut self) -> Result<()> {
        for table in self.set_tables.drain() {
            Exec::cmd("ip")
                .arg(table.version.as_ref())
                .arg("rule")
                .arg("delete")
                .arg("table")
                .arg(&table.fwmark)
                .to_expr()
                .run_expr()
                .chain_err(|| ErrorKind::FailedToRemoveTable)?;
        }

        if self.main_v4_table_adjusted {
            Self::delete_main_table_rule(IpVersionArg::V4)?;
            self.main_v4_table_adjusted = false;
        }

        if self.main_v6_table_adjusted {
            Self::delete_main_table_rule(IpVersionArg::V6)?;
            self.main_v6_table_adjusted = false;
        }

        Ok(())
    }

    fn delete_main_table_rule(ip_vers: IpVersionArg) -> Result<()> {
        Exec::cmd("ip")
            .arg(ip_vers.as_ref())
            .arg("rule")
            .arg("delete")
            .arg("table")
            .arg("main")
            .arg("suppress_prefixlength")
            .arg("0")
            .to_expr()
            .run_expr()
            .chain_err(|| ErrorKind::FailedToRemoveRoute)?;

        Ok(())
    }
}

fn ip_vers(route: &Route) -> &'static str {
    if route.prefix.is_ipv4() {
        "-4"
    } else {
        "-6"
    }
}


impl super::RoutingT for RouteManager {
    type Error = Error;
    fn new() -> Result<Self> {
        Ok(RouteManager {
            set_routes: HashSet::new(),
            set_tables: HashSet::new(),
            // the main routing table only has to be adjusted for default routes
            main_v4_table_adjusted: false,
            main_v6_table_adjusted: false,
        })
    }
    fn set_routes(&mut self, required_routes: RequiredRoutes) -> Result<()> {
        for route in required_routes.routes.into_iter() {
            if let Err(e) = self.set_route(route, &required_routes.fwmark) {
                let _ = self.reset_routes();
                return Err(e);
            }
        }
        Ok(())
    }

    fn reset_routes(&mut self) -> Result<()> {
        self.clear_routes()?;
        self.clear_tables()
    }

    /// Retrieves the gateway for the default route
    fn get_default_route_node(&mut self) -> Result<IpAddr> {
        let output = Exec::cmd("ip")
            .arg("route")
            .to_expr()
            .stdout_capture()
            .run()
            .chain_err(|| ErrorKind::FailedToGetDefaultRoute)
            .map(|output| String::from_utf8_lossy(&output.stdout).into_owned())?;
        let ip_str: &str = output
            .lines()
            .find(|line| line.trim().starts_with("default via "))
            .and_then(|line| line.trim().split_whitespace().skip(2).next())
            .map(Ok)
            .unwrap_or(Err(Error::from(ErrorKind::FailedToGetDefaultRoute)))?;

        ip_str
            .parse()
            .map_err(|_| Error::from(ErrorKind::FailedToGetDefaultRoute))
    }
}

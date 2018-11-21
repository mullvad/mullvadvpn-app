use super::{NetNode, RequiredRoutes};

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
enum IpVersion {
    V4,
    V6,
}

impl IpVersion {
    fn new(ip: &IpAddr) -> Self {
        if ip.is_ipv4() {
            IpVersion::V4
        } else {
            IpVersion::V6
        }
    }

    fn is_ipv4(&self) -> bool {
        match self {
            IpVersion::V4 => true,
            _ => false,
        }
    }
}

impl From<IpAddr> for IpVersion {
    fn from(ip: IpAddr) -> IpVersion {
        Self::new(&ip)
    }
}

impl AsRef<str> for IpVersion {
    fn as_ref(&self) -> &str {
        match self {
            IpVersion::V4 => "-4",
            IpVersion::V6 => "-6",
        }
    }
}

// A record of a table being set by a RouteManager.
#[derive(Hash, Eq, PartialEq)]
struct Table {
    version: IpVersion,
    fwmark: String,
}

pub struct RouteManager {
    added_routes: HashSet<super::Route>,
    added_tables: HashSet<Table>,
    // the main routing table only has to be adjusted for default routes
    main_v4_table_adjusted: bool,
    main_v6_table_adjusted: bool,
}

impl RouteManager {
    // This function adjusts main routing table to not make any routing decisions based on rules
    // with a prefix of 0. This is to bypass the main table for default routes.
    fn adjust_main_routing_table(&mut self, route: &IpNetwork) -> Result<()> {
        let route_arg = IpVersion::new(&route.ip());
        if route_arg.is_ipv4() && self.main_v4_table_adjusted || self.main_v6_table_adjusted {
            return Ok(());
        }
        duct::cmd!(
            "ip",
            route_arg.as_ref(),
            "rule",
            "add",
            "table",
            "main",
            "suppress_prefixlength",
            "0"
        )
        .run_expr()
        .chain_err(|| ErrorKind::FailedToAdjustMainRoutingTable)?;
        Ok(())
    }

    fn add_route(&mut self, route: super::Route, fwmark: &Option<String>) -> Result<()> {
        if route.prefix.prefix() == 0 {
            self.adjust_main_routing_table(&route.prefix)?;
        }

        let version = IpVersion::new(&route.prefix.ip());

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
            self.ensure_table_rules(Table {
                version,
                fwmark: fwmark.to_string(),
            })?;
        } else {
            self.added_routes.insert(route);
        }
        Ok(())
    }

    // if a route we're applying is set to a specific table, that table should have it's rules set
    fn ensure_table_rules(&mut self, added_table: Table) -> Result<()> {
        if self.added_tables.contains(&added_table) {
            return Ok(());
        }
        duct::cmd!(
            "ip",
            added_table.version.as_ref(),
            "rule",
            "add",
            "not",
            "fwmark",
            &added_table.fwmark,
            "table",
            &added_table.fwmark
        )
        .run_expr()
        .chain_err(|| ErrorKind::FailedToSetRuleForFwmark)?;


        self.added_tables.insert(added_table);
        Ok(())
    }

    fn clear_routes(&mut self) -> Result<()> {
        let mut end_result = Ok(());
        for route in self.added_routes.drain() {
            let ip_vers: IpVersion = route.prefix.ip().into();
            let result = duct::cmd!(
                "ip",
                ip_vers.as_ref(),
                "route",
                "delete",
                route.prefix.to_string()
            )
            .run_expr()
            .chain_err(|| ErrorKind::FailedToRemoveRoute);
            if let Err(e) = result {
                log::error!("Failed to remove route {} - {}", route.prefix, e);
                end_result = Err(e);
            }
        }
        end_result
    }

    fn clear_tables(&mut self) -> Result<()> {
        let mut end_result = Ok(());
        for table in self.added_tables.drain() {
            let result = duct::cmd!(
                "ip",
                table.version.as_ref(),
                "rule",
                "delete",
                "table",
                &table.fwmark
            )
            .run_expr()
            .chain_err(|| ErrorKind::FailedToRemoveTable);

            if let Err(e) = result {
                log::error!("Failed to remove routing table {} - {}", &table.fwmark, e);
                end_result = Err(e);
            }
        }

        if self.main_v4_table_adjusted {
            if let Err(e) = Self::delete_main_table_rule(IpVersion::V4) {
                log::error!(
                    "Failed to remove prefix limit for main routing table - {}",
                    e
                );
                end_result = Err(e);
            } else {
                self.main_v4_table_adjusted = false;
            }
        }

        if self.main_v6_table_adjusted {
            if let Err(e) = Self::delete_main_table_rule(IpVersion::V6) {
                log::error!(
                    "Failed to remove prefix limit for main routing table - {}",
                    e
                );
                end_result = Err(e);
            } else {
                self.main_v6_table_adjusted = false;
            }
        }

        end_result
    }

    fn delete_main_table_rule(ip_vers: IpVersion) -> Result<()> {
        duct::cmd!(
            "ip",
            ip_vers.as_ref(),
            "rule",
            "delete",
            "table",
            "main",
            "suppress_prefixlength",
            "0"
        )
        .run_expr()
        .chain_err(|| ErrorKind::FailedToRemoveRoute)?;

        Ok(())
    }
}

impl super::RoutingT for RouteManager {
    type Error = Error;
    fn new() -> Result<Self> {
        Ok(RouteManager {
            added_routes: HashSet::new(),
            added_tables: HashSet::new(),
            // the main routing table only has to be adjusted for default routes
            main_v4_table_adjusted: false,
            main_v6_table_adjusted: false,
        })
    }

    fn add_routes(&mut self, required_routes: RequiredRoutes) -> Result<()> {
        for route in required_routes.routes.into_iter() {
            if let Err(e) = self.add_route(route, &required_routes.fwmark) {
                let _ = self.delete_routes();
                return Err(e);
            }
        }
        Ok(())
    }

    fn delete_routes(&mut self) -> Result<()> {
        let result = self.clear_routes();
        let other_result = self.clear_tables();
        result.and_then(|_| other_result)
    }

    /// Retrieves the gateway for the default route
    fn get_default_route_node(&mut self) -> Result<IpAddr> {
        let output = duct::cmd!("ip", "route")
            .stdout()
            .chain_err(|| ErrorKind::FailedToGetDefaultRoute)?;
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

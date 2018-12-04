use super::{NetNode, RequiredRoutes};

use super::subprocess::{Exec, RunExpr};
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
    fn new(ip: IpAddr) -> Self {
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
        Self::new(ip)
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
    main_table_suppress_by_prefix_set_v4: bool,
    main_table_suppress_by_prefix_set_v6: bool,
}

impl RouteManager {
    // This function adjusts main routing table to not make any routing decisions based on rules
    // with a prefix of 0. This is to bypass the main table for default routes.
    fn set_suppress_prefix_length_on_main_routing_table(
        &mut self,
        version: IpVersion,
        set_rule: bool,
    ) -> Result<()> {
        if (version.is_ipv4() && (set_rule == self.main_table_suppress_by_prefix_set_v4))
            || (!version.is_ipv4() && (set_rule == self.main_table_suppress_by_prefix_set_v6))
        {
            return Ok(());
        }
        duct::cmd!(
            "ip",
            version.as_ref(),
            "rule",
            if set_rule { "add" } else { "delete" },
            "table",
            "main",
            "suppress_prefixlength",
            "0"
        )
        .run_expr()
        .chain_err(|| ErrorKind::FailedToAdjustMainRoutingTable)?;
        if version.is_ipv4() {
            self.main_table_suppress_by_prefix_set_v4 = set_rule;
        } else {
            self.main_table_suppress_by_prefix_set_v6 = set_rule;
        }
        Ok(())
    }

    fn add_route(&mut self, route: super::Route, fwmark: &Option<String>) -> Result<()> {
        if route.prefix.prefix() == 0 {
            self.set_suppress_prefix_length_on_main_routing_table(route.prefix.ip().into(), true)?;
        }

        let version = IpVersion::new(route.prefix.ip());

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

        cmd.into_expr()
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

        if self.main_table_suppress_by_prefix_set_v4 {
            if let Err(e) =
                self.set_suppress_prefix_length_on_main_routing_table(IpVersion::V4, false)
            {
                log::error!(
                    "Failed to remove prefix limit for main routing table - {}",
                    e
                );
                end_result = Err(e);
            } else {
                self.main_table_suppress_by_prefix_set_v4 = false;
            }
        }

        if self.main_table_suppress_by_prefix_set_v6 {
            if let Err(e) =
                self.set_suppress_prefix_length_on_main_routing_table(IpVersion::V6, false)
            {
                log::error!(
                    "Failed to remove prefix limit for main routing table - {}",
                    e
                );
                end_result = Err(e);
            } else {
                self.main_table_suppress_by_prefix_set_v6 = false;
            }
        }
        end_result
    }
}

impl super::RoutingT for RouteManager {
    type Error = Error;
    fn new() -> Result<Self> {
        Ok(RouteManager {
            added_routes: HashSet::new(),
            added_tables: HashSet::new(),
            // the main routing table only has to be adjusted for default routes
            main_table_suppress_by_prefix_set_v4: false,
            main_table_suppress_by_prefix_set_v6: false,
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
            .and_then(|line| line.trim().split_whitespace().nth(2))
            .ok_or_else(|| Error::from(ErrorKind::FailedToGetDefaultRoute))?;

        ip_str
            .parse()
            .map_err(|_| Error::from(ErrorKind::FailedToGetDefaultRoute))
    }
}

use crate::{imp::RouteManagerCommand, NetNode, Node, RequiredRoute, Route};

use futures::{
    channel::mpsc,
    future,
    stream::{FusedStream, Stream, StreamExt, TryStreamExt},
};
use ipnetwork::IpNetwork;
use std::{
    collections::HashSet,
    io,
    net::IpAddr,
    process::{ExitStatus, Stdio},
};
use talpid_types::net::IpVersion;
use tokio::{io::AsyncBufReadExt, process::Command};
use tokio_stream::wrappers::LinesStream;

pub type Result<T> = std::result::Result<T, Error>;

/// Errors that can happen in the macOS routing integration.
#[derive(err_derive::Error, Debug)]
#[error(no_from)]
pub enum Error {
    /// Failed to add route.
    #[error(display = "Failed to add route")]
    FailedToAddRoute(#[error(source)] io::Error),

    /// Failed to remove route.
    #[error(display = "Failed to remove route")]
    FailedToRemoveRoute(#[error(source)] io::Error),

    /// Error while running "ip route".
    #[error(display = "Error while running \"route get\"")]
    FailedToRunRoute(#[error(source)] io::Error),

    /// Error while monitoring routes with `route -nv monitor`
    #[error(display = "Error while running \"route -nv monitor\"")]
    FailedToMonitorRoutes(#[error(source)] io::Error),

    /// Unexpected output from netstat
    #[error(display = "Unexpected output from netstat")]
    BadOutputFromNetstat,
}

/// Route manager can be in 1 of 4 states -
///  - waiting for a route to be added or removed from the route table
///  - obtaining default routes
///  - applying changes to the route table
///  - shutting down
///
/// Only the _shutting down_ state can be reached from all other states, but during normal
/// operation, the route manager will add all the required routes during startup and will start
/// waiting for changes to the route table.  If any change is detected, it will stop listening for
/// new changes, obtain new default routes and reapply routes that should be routed through the
/// default nodes. Once the routes are reapplied, the route table changes are monitored again.
pub struct RouteManagerImpl {
    default_destinations: HashSet<IpNetwork>,
    applied_routes: HashSet<Route>,
    v4_gateway: Option<Node>,
    v6_gateway: Option<Node>,
    connectivity_change:
        Option<Box<dyn FusedStream<Item = std::io::Result<()>> + Unpin + Send + Sync>>,
}

impl RouteManagerImpl {
    pub async fn new(required_routes: HashSet<RequiredRoute>) -> Result<Self> {
        let v4_gateway = Self::get_default_node(IpVersion::V4).await?;
        let v6_gateway = Self::get_default_node(IpVersion::V6).await?;

        let monitor = listen_for_default_route_changes()?;

        let mut manager = Self {
            default_destinations: HashSet::new(),
            applied_routes: HashSet::new(),
            connectivity_change: Some(Box::new(monitor.fuse())),
            v4_gateway,
            v6_gateway,
        };

        manager.add_required_routes(required_routes).await?;

        Ok(manager)
    }

    pub(crate) async fn run(mut self, manage_rx: mpsc::UnboundedReceiver<RouteManagerCommand>) {
        let mut manage_rx = manage_rx.fuse();
        let mut connectivity_change = self.connectivity_change.take().unwrap();

        loop {
            futures::select! {
                command = manage_rx.next() => {
                    match command {
                        Some(RouteManagerCommand::Shutdown(tx)) => {
                            self.cleanup_routes().await;
                            let _ = tx.send(());
                            return;
                        },

                        Some(RouteManagerCommand::AddRoutes(routes, result_tx)) => {
                            let result = self.add_required_routes(routes).await;
                            let _ = result_tx.send(result);
                        },
                        Some(RouteManagerCommand::ClearRoutes) => {
                            self.cleanup_routes().await;
                        },
                        None => {
                            break;
                        }
                    }
                },

                _result = connectivity_change.select_next_some() => {
                    let v4_gateway = Self::get_default_node(IpVersion::V4).await.unwrap_or(None);
                    let v6_gateway = Self::get_default_node(IpVersion::V6).await.unwrap_or(None);

                    if v4_gateway != self.v4_gateway {
                        self.v4_gateway = v4_gateway;
                        self.apply_new_default_route(&self.v4_gateway, true).await;
                    }

                    if v6_gateway != self.v6_gateway {
                        self.v6_gateway = v6_gateway;
                        self.apply_new_default_route(&self.v6_gateway, false).await;
                    }
                },
                complete => {
                    break;
                }
            };
        }
        self.cleanup_routes().await;
    }

    async fn add_required_routes(&mut self, required_routes: HashSet<RequiredRoute>) -> Result<()> {
        let mut routes_to_apply = vec![];
        let mut default_destinations = HashSet::new();

        for route in required_routes {
            match route.node {
                NetNode::DefaultNode => {
                    default_destinations.insert(route.prefix);
                }

                NetNode::RealNode(node) => routes_to_apply.push(Route::new(node, route.prefix)),
            }
        }

        for route in routes_to_apply {
            Self::add_route(&route).await?;
            self.applied_routes.insert(route);
        }

        for destination in default_destinations.iter() {
            match (&self.v4_gateway, &self.v6_gateway, destination.is_ipv4()) {
                (Some(gateway), _, true) | (_, Some(gateway), false) => {
                    let route = Route::new(gateway.clone(), *destination);
                    Self::add_route(&route).await?;
                    self.applied_routes.insert(route);
                }
                _ => (),
            };
        }

        self.default_destinations = default_destinations;

        Ok(())
    }

    // Retrieves the node that's currently used to reach 0.0.0.0/0
    pub(crate) async fn get_default_node(ip_version: IpVersion) -> Result<Option<Node>> {
        let ip_version_arg = match ip_version {
            IpVersion::V4 => "-inet",
            IpVersion::V6 => "-inet6",
        };
        let mut cmd = Command::new("route");
        cmd.arg("-n").arg("get").arg(ip_version_arg).arg("default");

        let output = cmd.output().await.map_err(Error::FailedToRunRoute)?;
        let output = String::from_utf8(output.stdout).map_err(|e| {
            log::error!("Failed to parse utf-8 bytes from output of netstat: {}", e);
            Error::BadOutputFromNetstat
        })?;
        Ok(Self::parse_route(&output))
    }

    fn parse_route(route_output: &str) -> Option<Node> {
        let mut address: Option<IpAddr> = None;
        let mut device = None;
        for line in route_output.lines() {
            // we're looking for just 2 different lines:
            // interface: utun0
            // gateway: 192.168.3.1
            let tokens: Vec<_> = line.split_whitespace().collect();
            if tokens.len() == 2 {
                match tokens[0].trim() {
                    "interface:" => {
                        device = Some(tokens[1].to_string());
                    }
                    "gateway:" => {
                        address = Self::parse_gateway_line(tokens[1]);
                    }
                    _ => continue,
                }
            }
        }

        match (address, device) {
            (Some(address), Some(device)) => Some(Node::new(address, device)),
            (Some(address), None) => Some(Node::address(address)),
            (None, Some(device)) => Some(Node::device(device)),
            _ => None,
        }
    }

    fn parse_gateway_line(line: &str) -> Option<IpAddr> {
        // IPv6 addresses may contain interfaces
        // if line contains '%' it should be split off
        line.split('%')
            .next()
            .and_then(|ip_str| ip_str.parse().ok())
    }

    async fn delete_route(destination: IpNetwork) -> Result<ExitStatus> {
        let mut cmd = Command::new("route");
        cmd.arg("-q")
            .arg("-n")
            .arg("delete")
            .arg(ip_vers(destination))
            .arg(destination.to_string())
            .stderr(Stdio::null());

        cmd.status().await.map_err(Error::FailedToRemoveRoute)
    }

    async fn add_route(route: &Route) -> Result<ExitStatus> {
        let mut cmd = Command::new("route");
        cmd.arg("-q")
            .arg("-n")
            .arg("add")
            .arg(ip_vers(route.prefix))
            .arg(route.prefix.to_string());

        if let Some(addr) = route.node.get_address() {
            cmd.arg("-gateway").arg(addr.to_string());
        } else if let Some(device) = route.node.get_device() {
            cmd.arg("-interface").arg(device);
        }

        cmd.status().await.map_err(Error::FailedToAddRoute)
    }

    async fn cleanup_routes(&self) -> () {
        let destinations_to_remove = self
            .applied_routes
            .iter()
            .map(|route| &route.prefix)
            .chain(self.default_destinations.iter());

        for destination in destinations_to_remove {
            match Self::delete_route(*destination).await {
                Ok(status) => {
                    if !status.success() {
                        log::debug!("Failed to remove route during shutdown");
                    }
                }
                Err(e) => log::error!("Failed to remove route during shutdown: {}", e),
            };
        }
    }

    async fn apply_new_default_route(&self, new_node: &Option<Node>, v4: bool) {
        for destination in self.default_destinations.iter() {
            if destination.is_ipv4() == v4 {
                let _ = Self::delete_route(*destination).await;

                if let Some(node) = new_node {
                    log::error!("Resetting default route for {}", destination);
                    match Self::add_route(&Route::new(node.clone(), *destination)).await {
                        Ok(status) => {
                            if !status.success() {
                                log::error!("Failed to reapply route");
                            }
                        }
                        Err(e) => log::error!("Failed to reset route: {}", e),
                    }
                }
            }
        }
    }
}

fn ip_vers(prefix: IpNetwork) -> &'static str {
    if prefix.is_ipv4() {
        "-inet"
    } else {
        "-inet6"
    }
}

/// Returns a stream that produces an item whenever a default route is either added or deleted from
/// the routing table.
pub fn listen_for_default_route_changes() -> Result<impl Stream<Item = std::io::Result<()>>> {
    let mut cmd = Command::new("route");
    cmd.arg("-n")
        .arg("monitor")
        .arg("-")
        .stderr(Stdio::null())
        .stdout(Stdio::piped())
        .stdin(Stdio::null());

    let mut process = cmd.spawn().map_err(Error::FailedToMonitorRoutes)?;
    let reader = tokio::io::BufReader::new(process.stdout.take().unwrap());
    let lines = reader.lines();

    // route -n monitor will produce netlink messages in the following format
    // ```
    // got message of size 176 on Thu Jun  4 10:08:05 2020
    // RTM_DELETE: Delete Route: len 176, pid: 109, seq 1151, errno 3, ifscope 23,
    // flags:<UP,GATEWAY,STATIC,IFSCOPE>
    // locks:  inits:
    // sockaddrs: <DST,GATEWAY,NETMASK,IFP,IFA>
    //  default 192.168.44.1 default  192.168.44.90
    // ```
    // On the second line of the message, the message type is specified. Only messages with the
    // type 'RTM_ADD' or 'RTM_DELETE' are considered. On the 6th line, message attribute values are
    // shown. To detect a change for a default route in the routing table, check whether this line
    // contains 'default'.  Whenever an empty line is encountered, the message has been sent, so
    // the state can be reset.

    let mut add_or_delete_message = false;
    let mut contains_default = false;

    let monitor = LinesStream::new(lines).try_filter_map(move |line| {
        if add_or_delete_message {
            if line.contains("default") {
                contains_default = true;
            }
            if line.trim().is_empty() {
                add_or_delete_message = false;
                if contains_default {
                    contains_default = false;
                    return future::ready(Ok(Some(())));
                }
            }
        } else {
            add_or_delete_message = line.starts_with("RTM_ADD:") || line.starts_with("RTM_DELETE:");
        }
        future::ready(Ok(None))
    });

    Ok(monitor)
}

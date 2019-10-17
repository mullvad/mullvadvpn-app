use super::{NetNode, Node, Route};

use ipnetwork::IpNetwork;
use std::{
    collections::{HashMap, HashSet, VecDeque},
    io,
    process::{Command, Stdio},
};

mod change_listener;
use change_listener::{Error as RouteChangeListenerError, RouteChangeListener};

use futures::{sync::oneshot, Async, Future, IntoFuture, Stream};
use tokio_process::CommandExt;

pub type Result<T> = std::result::Result<T, Error>;

/// Errors that can happen in the Linux routing integration
#[derive(err_derive::Error, Debug)]
pub enum Error {
    /// Failed to add route.
    #[error(display = "Failed to add route")]
    FailedToAddRoute,

    /// Failed to remove route.
    #[error(display = "Failed to remove route")]
    FailedToRemoveRoute,

    /// Error while running "ip route".
    #[error(display = "Error while running \"ip route\"")]
    FailedToRunIp(#[error(source)] io::Error),

    /// Invocation of `ip route` ended with a non-zero exit code
    #[error(display = "ip returend a non-zero exit code")]
    ErrorIpFailed,

    /// Received unexpected output from `ip route`
    #[error(display = "Received unexpected output from \"ip\"")]
    UnexpectedOutput,

    /// No default route exists
    #[error(display = "No default route in \"ip route\" output")]
    NoDefaultRoute,

    /// Route table change stream failed.
    #[error(display = "Route change listener failed")]
    ChangeListenerError(#[error(source)] RouteChangeListenerError),

    /// Route table change stream failed.
    #[error(display = "Route change listener closed unexpectedly")]
    ChangeListenerClosed,
}

pub struct RouteManagerImpl {
    changes: RouteChangeListener,

    // currently added routes
    added_routes: HashSet<Route>,
    // default route tracking
    // destinations that should be routed through the default route
    required_default_routes: HashSet<IpNetwork>,
    default_routes: HashSet<Route>,
    best_default_node_v4: Option<Node>,
    best_default_node_v6: Option<Node>,

    // pending changes
    needed_changes: VecDeque<RouteChange>,
    pending_change: Option<PendingChange>,

    // if the stop channel is set, the future should wind down - remove added routes and send a
    // signal.
    shutdown_finished_tx: Option<oneshot::Sender<()>>,
    shutdown_rx: oneshot::Receiver<oneshot::Sender<()>>,
    should_shut_down: bool,
}

impl RouteManagerImpl {
    /// Creates a new RouteManager.
    pub fn new(
        required_routes: HashMap<IpNetwork, NetNode>,
        shutdown_rx: oneshot::Receiver<oneshot::Sender<()>>,
    ) -> Result<Self> {
        let changes = RouteChangeListener::new().map_err(Error::ChangeListenerError)?;

        let mut required_normal_routes = HashSet::new();
        let mut required_default_routes = HashSet::new();
        let mut added_routes = HashSet::new();

        for (destination, node) in required_routes {
            match node {
                NetNode::RealNode(node) => {
                    required_normal_routes.insert(Route::new(node, destination));
                }
                NetNode::DefaultNode => {
                    required_default_routes.insert(destination);
                }
            }
        }

        let default_routes = Self::get_default_routes().wait()?;

        let best_default_node_v4 = Self::pick_best_default_node(&default_routes, true);
        let best_default_node_v6 = Self::pick_best_default_node(&default_routes, false);

        let mut establish_baseline_fn = || -> Result<()> {
            for normal_route in required_normal_routes.iter() {
                Self::add_route(&normal_route).wait()?;
                added_routes.insert(normal_route.clone());
            }

            for prefix in required_default_routes.iter() {
                match (
                    prefix.is_ipv4(),
                    &best_default_node_v4,
                    &best_default_node_v6,
                ) {
                    (false, _, Some(default_node)) | (true, Some(default_node), _) => {
                        // best to pick a single node identifier rather than device + ip
                        let route = Route::new(default_node.clone(), *prefix);
                        Self::add_route(&route).wait()?;
                        added_routes.insert(route);
                    }
                    // at this point in time, there exists no default route for the given IP version
                    // so no routes will be added. The assumption is that routing ipv4 through ipv6
                    // nodes may or may not be bonkers
                    _ => continue,
                }
            }
            Ok(())
        };

        if let Err(e) = establish_baseline_fn() {
            for setup_route in added_routes {
                if let Err(removal_err) = Self::delete_route(&setup_route).wait() {
                    log::error!(
                        "Failed to remove route whilst cleaning up failed initialization
of route monitor -{}",
                        removal_err
                    );
                }
            }
            return Err(e);
        }


        Ok(Self {
            changes,

            required_default_routes,
            added_routes,

            default_routes,
            best_default_node_v4,
            best_default_node_v6,

            needed_changes: VecDeque::new(),
            pending_change: None,

            shutdown_finished_tx: None,
            shutdown_rx,
            should_shut_down: false,
        })
    }

    fn process_route_table_change(&mut self) -> Result<()> {
        loop {
            let change = self.changes.poll().map_err(Error::ChangeListenerError)?;
            match change {
                Async::NotReady => return Ok(()),
                Async::Ready(Some(RouteChange::Add(route))) => self.process_new_route(route),
                Async::Ready(Some(RouteChange::Remove(route))) => self.process_deleted_route(route),
                Async::Ready(None) => return Err(Error::ChangeListenerClosed),
            }
        }
    }

    fn process_new_route(&mut self, route: Route) {
        self.needed_changes.retain(|change| {
            if let RouteChange::Add(old_route) = change {
                old_route != &route
            } else {
                true
            }
        });
        if route.prefix.prefix() == 0 {
            self.default_routes.insert(route);
            self.update_default_rotues();
        }
    }

    fn process_deleted_route(&mut self, route: Route) {
        self.needed_changes.retain(|change| {
            if let RouteChange::Remove(old_route) = change {
                old_route != &route
            } else {
                true
            }
        });
        if route.prefix.prefix() == 0 {
            self.update_default_rotues();
        }
    }

    fn update_default_rotues(&mut self) {
        let new_best_v4 = Self::pick_best_default_node(&self.default_routes, true);
        if self.best_default_node_v4 != new_best_v4 && new_best_v4.is_some() {
            let new_node = new_best_v4.unwrap();
            let old_node = self.best_default_node_v4.take();
            let v4_destinations: Vec<_> = self
                .required_default_routes
                .iter()
                .filter(|ip| ip.is_ipv4())
                .cloned()
                .collect();
            for destination in v4_destinations {
                if let Some(old_node) = &old_node {
                    self.enque_route_change(RouteChange::Remove(Route::new(
                        old_node.clone(),
                        destination,
                    )));
                }

                self.enque_route_change(RouteChange::Add(Route::new(
                    new_node.clone(),
                    destination,
                )));
            }
            self.best_default_node_v4 = Some(new_node);
        }

        let new_best_v6 = Self::pick_best_default_node(&self.default_routes, false);
        if self.best_default_node_v6 != new_best_v6 && new_best_v6.is_some() {
            let new_node = new_best_v6.unwrap();
            let old_node = self.best_default_node_v6.take();
            let v6_destinations: Vec<_> = self
                .required_default_routes
                .iter()
                .filter(|ip| !ip.is_ipv4())
                .cloned()
                .collect();

            for destination in v6_destinations {
                if let Some(old_node) = &old_node {
                    self.enque_route_change(RouteChange::Remove(Route::new(
                        old_node.clone(),
                        destination,
                    )));
                }
                self.enque_route_change(RouteChange::Add(Route::new(
                    new_node.clone(),
                    destination,
                )));
            }
            self.best_default_node_v6 = Some(new_node);
        }
    }

    fn enque_route_change(&mut self, route_change: RouteChange) {
        // Only add a route change to the queue of changes if a change like this doesn't exist
        // already.
        if self
            .pending_change
            .as_ref()
            .map(|pending_change| pending_change.change != route_change)
            .unwrap_or(true)
            && self
                .needed_changes
                .iter()
                .all(|enqued_change| enqued_change != &route_change)
        {
            self.needed_changes.push_back(route_change);
        }
    }

    fn pick_best_default_node(routes: &HashSet<Route>, v4: bool) -> Option<Node> {
        // Pick the route with the lowest metric - thus the most favourable route.
        routes
            .iter()
            .filter(|route| route.prefix.is_ipv4() == v4)
            .fold(
                None,
                |best_route: Option<Route>, next_route| match best_route {
                    Some(current_best) => {
                        if current_best.metric.unwrap_or(0) > next_route.metric.unwrap_or(0) {
                            Some(next_route.clone())
                        } else {
                            Some(current_best)
                        }
                    }
                    None => Some(next_route.clone()),
                },
            )
            .map(|route| route.node)
    }

    // Try and apply changes to the routing table if any are necessary.
    // Returns true if no more changes are to be made.
    fn apply_route_table_changes(&mut self) -> Result<bool> {
        let mut should_stop = false;
        while !should_stop {
            if self.pending_change.is_none() {
                if let Some(change) = self.needed_changes.pop_front() {
                    let process = match &change {
                        RouteChange::Add(route) => Self::add_route(route),
                        RouteChange::Remove(route) => Self::delete_route(route),
                    };
                    self.pending_change = Some(PendingChange { change, process });
                }
            }

            if let Some(mut change) = self.pending_change.take() {
                match change.process.poll()? {
                    Async::NotReady => {
                        self.pending_change = Some(change);
                        should_stop = true;
                    }
                    Async::Ready(_) => {
                        match change.change {
                            RouteChange::Add(route) => {
                                self.added_routes.insert(route);
                            }
                            RouteChange::Remove(route) => {
                                self.added_routes.remove(&route);
                            }
                        };
                    }
                };
            } else {
                should_stop = true;
            }
        }

        Ok(self.pending_change.is_none() && self.needed_changes.is_empty())
    }

    fn route_cmd(action: &str, route: &Route) -> Command {
        let mut cmd = Command::new("ip");

        cmd.arg(ip_vers(&route))
            .arg("route")
            .arg(action)
            .arg(route.prefix.to_string());

        if let Some(addr) = route.node.get_address() {
            cmd.arg("via").arg(addr.to_string());
        };
        if let Some(device) = route.node.get_device() {
            cmd.arg("dev").arg(device);
        };
        if let Some(metric) = route.metric {
            cmd.arg("metric").arg(metric.to_string());
        };

        cmd
    }

    fn run_cmd(mut cmd: Command, err: Error) -> Box<dyn Future<Item = (), Error = Error> + Send> {
        log::trace!("running cmd - {:?}", &cmd);
        Box::new(
            cmd.spawn_async()
                .into_future()
                .flatten()
                .map_err(Error::FailedToRunIp)
                .and_then(|exit_status| {
                    if exit_status.success() {
                        Ok(())
                    } else {
                        Err(err)
                    }
                }),
        )
    }

    fn get_default_routes_inner(
        ip_version: IpVersion,
    ) -> impl Future<Item = Vec<Route>, Error = Error> {
        let mut cmd = Command::new("ip");
        cmd.arg(ip_version.to_route_arg()).arg("route").arg("show");

        Box::new(
            cmd.stdout(Stdio::piped())
                .spawn_async()
                .map_err(Error::FailedToRunIp)
                .into_future()
                .and_then(|proc| proc.wait_with_output().map_err(Error::FailedToRunIp))
                .and_then(move |output| {
                    let output_lines = String::from_utf8(output.stdout.clone())
                        .map_err(|_| Error::UnexpectedOutput)?;
                    Ok(output_lines
                        .lines()
                        .filter_map(|line| {
                            if line.starts_with("default") {
                                parse_ip_route_show_line(line, ip_version)
                            } else {
                                None
                            }
                        })
                        .collect())
                }),
        )
    }

    /// Adds routes to the system routing table.
    fn add_route(route: &Route) -> Box<dyn Future<Item = (), Error = Error> + Send> {
        let cmd = Self::route_cmd("replace", route);
        Self::run_cmd(cmd, Error::FailedToAddRoute)
    }

    /// Removes previously set routes. If routes were set for specific tables, the whole tables
    /// will be removed.
    fn delete_route(route: &Route) -> Box<dyn Future<Item = (), Error = Error> + Send> {
        let cmd = Self::route_cmd("delete", route);
        Self::run_cmd(cmd, Error::FailedToRemoveRoute)
    }

    /// Retrieves the gateway for the default route
    fn get_default_routes() -> Box<dyn Future<Item = HashSet<Route>, Error = Error> + Send> {
        Box::new(
            Self::get_default_routes_inner(IpVersion::V4)
                .join(Self::get_default_routes_inner(IpVersion::V6))
                .map(|(v4_routes, v6_routes)| {
                    v4_routes.into_iter().chain(v6_routes.into_iter()).collect()
                }),
        )
    }
}

#[derive(Debug, Copy, Clone)]
enum IpVersion {
    V4,
    V6,
}

impl IpVersion {
    fn to_route_arg(self) -> &'static str {
        match self {
            IpVersion::V4 => "-4",
            IpVersion::V6 => "-6",
        }
    }
}

impl Future for RouteManagerImpl {
    type Item = ();
    type Error = Error;
    fn poll(&mut self) -> Result<Async<()>> {
        if !self.should_shut_down {
            match self.shutdown_rx.poll() {
                Ok(Async::NotReady) => (),
                Ok(Async::Ready(tx)) => {
                    self.should_shut_down = true;
                    self.shutdown_finished_tx = Some(tx);
                }
                Err(_) => {
                    self.should_shut_down = true;
                }
            };
            self.process_route_table_change()?;
        }
        let all_changes_applied = self.apply_route_table_changes()?;
        if all_changes_applied && self.should_shut_down {
            if let Some(tx) = self.shutdown_finished_tx.take() {
                if tx.send(()).is_err() {
                    log::error!("RouteManagerHandle already stopped");
                }
            }
            Ok(Async::Ready(()))
        } else {
            Ok(Async::NotReady)
        }
    }
}

// intended to parse lines sucha as the following:
//      default via 192.168.1.1 dev wlp61s0 proto dhcp metric 600
fn parse_ip_route_show_line(line: &str, ip_version: IpVersion) -> Option<Route> {
    let mut node_ip = None;
    let mut device = None;
    let mut metric = None;

    let mut tokens = line.split_whitespace();
    let prefix_str = tokens.next()?;
    let prefix = match prefix_str {
        "default" => match ip_version {
            IpVersion::V4 => "0.0.0.0/0".parse().unwrap(),
            IpVersion::V6 => "::/0".parse().unwrap(),
        },
        prefix_str => prefix_str.parse().ok()?,
    };

    let tokens: Vec<&str> = tokens.collect();
    for pair in tokens.chunks(2) {
        if pair.len() != 2 {
            log::error!("unexpected output from ip");
            break;
        }
        let kind = pair[0];
        let value = pair[1];

        match kind {
            "via" => node_ip = value.parse().ok(),
            "dev" => device = Some(value.to_string()),
            "metric" => metric = value.parse().ok(),
            _ => continue,
        };
    }

    if node_ip.is_none() && device.is_none() {
        None
    } else {
        let node = super::Node {
            ip: node_ip,
            device,
        };

        Some(super::Route {
            node,
            prefix,
            metric,
        })
    }
}

fn ip_vers(route: &Route) -> &'static str {
    if route.prefix.is_ipv4() {
        "-4"
    } else {
        "-6"
    }
}

#[derive(Debug, PartialEq)]
enum RouteChange {
    Add(Route),
    Remove(Route),
}

struct PendingChange {
    change: RouteChange,
    process: Box<dyn Future<Item = (), Error = Error> + Send>,
}

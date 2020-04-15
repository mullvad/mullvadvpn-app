use crate::routing::{NetNode, Node, Route};

use ipnetwork::IpNetwork;
use std::{
    collections::{HashMap, HashSet},
    io,
    net::IpAddr,
    process::{Command, ExitStatus, Stdio},
};

use futures01::{stream, sync::oneshot, Async, Future, IntoFuture, Stream};
use tokio_process::{Child, CommandExt};


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

    /// No default route in "route -n get default" output.
    #[error(display = "No default route in \"route -n get default\" output")]
    NoDefaultRoute,

    /// Unexpected output from netstat
    #[error(display = "Unexpected output from netstat")]
    BadOutputFromNetstat,
}

enum RouteManagerState {
    Listening(ChangeListener),
    ObtainingDefaultRoutes(
        Box<dyn Future<Item = (Option<Node>, Option<Node>), Error = Error> + Send>,
    ),
    Applying(Box<dyn Future<Item = (), Error = ()> + Send>),
    Shutdown(Box<dyn Future<Item = (), Error = ()> + Send>),
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
    current_state: RouteManagerState,
    v4_gateway: Option<Node>,
    v6_gateway: Option<Node>,
    shutdown_rx: Option<oneshot::Receiver<oneshot::Sender<()>>>,
}


impl RouteManagerImpl {
    pub fn new(
        required_routes: HashMap<IpNetwork, NetNode>,
        shutdown_rx: oneshot::Receiver<oneshot::Sender<()>>,
    ) -> Result<Self> {
        let mut applied_routes = HashSet::new();
        let mut routes_to_apply = vec![];
        let mut default_destinations = HashSet::new();

        let v4_gateway = Self::get_default_node_cmd("-inet").wait()?;
        let v6_gateway = Self::get_default_node_cmd("-inet6").wait()?;

        if v4_gateway.is_none() && v6_gateway.is_none() {
            return Err(Error::NoDefaultRoute);
        }

        for (destination, node) in required_routes.into_iter() {
            match node {
                NetNode::DefaultNode => {
                    default_destinations.insert(destination);
                }

                NetNode::RealNode(node) => routes_to_apply.push(Route::new(node, destination)),
            }
        }

        let apply_routes_fn = || -> Result<()> {
            for route in routes_to_apply {
                Self::add_route(&route).wait()?;
                applied_routes.insert(route);
            }
            for destination in default_destinations.iter() {
                match (&v4_gateway, &v6_gateway, destination.is_ipv4()) {
                    (Some(gateway), _, true) | (_, Some(gateway), false) => {
                        let route = Route::new(gateway.clone(), *destination);
                        Self::add_route(&route).wait()?;
                        applied_routes.insert(route);
                    }
                    _ => (),
                };
            }

            Ok(())
        };

        if let Err(e) = apply_routes_fn() {
            log::error!("Failed to apply routes - {}", e);
            for applied_route in applied_routes.iter() {
                if let Err(removal_err) = Self::delete_route(applied_route.prefix).wait() {
                    log::error!(
                        "Failed to clean up routes after failing to set them up - {}",
                        removal_err
                    );
                }
            }
            return Err(e);
        }
        let change_listener = ChangeListener::new().map_err(Error::FailedToMonitorRoutes)?;

        Ok(Self {
            default_destinations,
            applied_routes,
            current_state: RouteManagerState::Listening(change_listener),
            shutdown_rx: Some(shutdown_rx),
            v4_gateway,
            v6_gateway,
        })
    }

    // Retrieves the node that's currently used to reach 0.0.0.0/0
    // Arguments can be either -inet or -inet6
    fn get_default_node_cmd(
        if_family: &'static str,
    ) -> impl Future<Item = Option<Node>, Error = Error> {
        let mut cmd = Command::new("route");
        cmd.arg("-n").arg("get").arg(if_family).arg("default");

        cmd.output_async()
            .map_err(Error::FailedToRunRoute)
            .and_then(|output| {
                let output = String::from_utf8(output.stdout).map_err(|e| {
                    log::error!("Failed to parse utf-8 bytes from output of netstat - {}", e);
                    Error::BadOutputFromNetstat
                })?;
                Ok(Self::parse_route(&output))
            })
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

    pub fn delete_route(
        destination: IpNetwork,
    ) -> impl Future<Item = ExitStatus, Error = Error> + Send {
        let mut cmd = Command::new("route");
        cmd.arg("-q")
            .arg("-n")
            .arg("delete")
            .arg(ip_vers(destination))
            .arg(destination.to_string());

        futures01::lazy(move || cmd.spawn_async().into_future().and_then(|f| f))
            .map_err(Error::FailedToRemoveRoute)
    }

    fn add_route(route: &Route) -> impl Future<Item = ExitStatus, Error = Error> + Send {
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

        futures01::lazy(move || cmd.spawn_async().into_future().and_then(|f| f))
            .map_err(Error::FailedToAddRoute)
    }

    fn shutdown_future(
        &self,
        shutdown_done_tx: Option<oneshot::Sender<()>>,
    ) -> impl Future<Item = (), Error = ()> + Send {
        let remove_route_future = |route: &Route| {
            Self::delete_route(route.prefix).then(|removal| {
                match removal {
                    Ok(status) => {
                        if !status.success() {
                            log::debug!("Failed to remove route during shutdown");
                        }
                    }
                    Err(e) => log::error!("Failed to remove route during shutdown - {}", e),
                };
                Ok(())
            })
        };
        let mut routes_to_remove: Vec<_> = self
            .applied_routes
            .iter()
            .map(remove_route_future)
            .collect();
        routes_to_remove.extend(self.default_destinations.iter().filter_map(|dest| {
            match (&self.v4_gateway, &self.v6_gateway, dest.is_ipv4()) {
                (Some(gateway), _, true) | (_, Some(gateway), false) => {
                    let route = Route::new(gateway.clone(), *dest);
                    Some(remove_route_future(&route))
                }
                _ => None,
            }
        }));
        stream::futures_ordered(routes_to_remove)
            .for_each(|_| Ok(()))
            .and_then(|_| {
                if let Some(tx) = shutdown_done_tx {
                    if tx.send(()).is_err() {
                        log::debug!("RouteManager already dropped")
                    }
                }
                Ok(())
            })
    }

    fn apply_new_default_routes(
        &self,
        new_v4: Option<Node>,
        new_v6: Option<Node>,
    ) -> impl Future<Item = (), Error = ()> + Send {
        let apply_route_future = |route: &Route| {
            let add_route_future = Self::add_route(route);
            // always try to remove old route first - if it's still set, the new one won't be
            // applied
            Self::delete_route(route.prefix)
                .then(|_| add_route_future)
                .then(|addition| {
                    match addition {
                        Ok(status) => {
                            if !status.success() {
                                log::info!("Failed to reapply route");
                            }
                        }
                        Err(e) => log::error!("Failed to reset route: {}", e),
                    }
                    Ok(())
                })
        };


        let add_new_routes = self.default_destinations.iter().filter_map(|dest| {
            match (&new_v4, &new_v6, dest.is_ipv4()) {
                (Some(gateway), _, true) | (_, Some(gateway), false) => {
                    let new_route = Route::new(gateway.clone(), *dest);
                    Some(apply_route_future(&new_route))
                }

                _ => None,
            }
        });

        stream::futures_ordered(add_new_routes).for_each(|_| Ok(()))
    }

    fn get_default_routes_future(
        &self,
    ) -> impl Future<Item = (Option<Node>, Option<Node>), Error = Error> + Send {
        Self::get_default_node_cmd("-inet").join(Self::get_default_node_cmd("-inet6"))
    }
}

impl Future for RouteManagerImpl {
    type Item = ();
    type Error = Error;
    fn poll(&mut self) -> Result<Async<()>> {
        if let Some(mut shutdown_rx) = self.shutdown_rx.take() {
            match shutdown_rx.poll() {
                Ok(Async::Ready(shutdown_tx)) => {
                    self.current_state = RouteManagerState::Shutdown(Box::new(
                        self.shutdown_future(Some(shutdown_tx)),
                    ));
                }
                // handle is already dropped
                Err(_) => {
                    self.current_state =
                        RouteManagerState::Shutdown(Box::new(self.shutdown_future(None)));
                }
                Ok(Async::NotReady) => {
                    self.shutdown_rx = Some(shutdown_rx);
                }
            };
        }


        loop {
            match &mut self.current_state {
                RouteManagerState::Listening(listener) => {
                    match listener.poll().map_err(Error::FailedToMonitorRoutes)? {
                        Async::Ready(()) => {
                            self.current_state = RouteManagerState::ObtainingDefaultRoutes(
                                Box::new(self.get_default_routes_future()),
                            );
                        }
                        Async::NotReady => break,
                    }
                }

                RouteManagerState::ObtainingDefaultRoutes(f) => match f.poll()? {
                    Async::Ready((v4_gateway, v6_gateway)) => {
                        self.current_state = RouteManagerState::Applying(Box::new(
                            self.apply_new_default_routes(v4_gateway, v6_gateway),
                        ));
                    }
                    Async::NotReady => break,
                },

                RouteManagerState::Applying(f) => {
                    match f.poll() {
                        // the future for reapplying routes never fails - just logs failures
                        Err(_) => unreachable!(),
                        Ok(Async::NotReady) => break,
                        Ok(Async::Ready(_)) => {
                            self.current_state = RouteManagerState::Listening(
                                ChangeListener::new().map_err(Error::FailedToMonitorRoutes)?,
                            );
                        }
                    }
                }

                RouteManagerState::Shutdown(shutdown_future) => {
                    return Ok(shutdown_future.poll().unwrap_or(Async::Ready(())));
                }
            }
        }

        Ok(Async::NotReady)
    }
}

fn ip_vers(prefix: IpNetwork) -> &'static str {
    if prefix.is_ipv4() {
        "-inet"
    } else {
        "-inet6"
    }
}


pub struct ChangeListener {
    process: Child,
    lines: tokio_io::io::Lines<std::io::BufReader<tokio_process::ChildStdout>>,
}

impl ChangeListener {
    pub fn new() -> std::io::Result<Self> {
        let mut cmd = Command::new("route");
        cmd.arg("-vn").arg("monitor").stdout(Stdio::piped());

        let mut process = cmd.spawn_async()?;

        let reader = std::io::BufReader::new(process.stdout().take().unwrap());
        let lines = tokio_io::io::lines(reader);

        Ok(Self { process, lines })
    }
}

impl Future for ChangeListener {
    type Item = ();
    type Error = std::io::Error;

    fn poll(&mut self) -> std::io::Result<Async<Self::Item>> {
        match self.process.poll() {
            Ok(Async::NotReady) => (),
            Ok(Async::Ready(status)) => {
                log::debug!("route listener exited - {:?}", status);
                return Ok(Async::Ready(()));
            }
            Err(e) => return Err(e),
        };
        loop {
            return match self.lines.poll()? {
                Async::NotReady => Ok(Async::NotReady),
                Async::Ready(Some(line)) => {
                    if line.starts_with("RTM_DELETE") || line.starts_with("RTM_ADD") {
                        Ok(Async::Ready(()))
                    } else {
                        continue;
                    }
                }
                Async::Ready(None) => Ok(Async::Ready(())),
            };
        }
    }
}

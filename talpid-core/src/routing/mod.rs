#![cfg_attr(target_os = "android", allow(dead_code))]
#![cfg_attr(target_os = "windows", allow(dead_code))]
// TODO: remove the allow(dead_code) for android once it's up to scratch.
use futures::{sync::oneshot, Future};
use ipnetwork::IpNetwork;
use std::{collections::HashMap, fmt, net::IpAddr, thread};

#[cfg(target_os = "macos")]
#[path = "macos.rs"]
mod imp;

#[cfg(target_os = "linux")]
#[path = "linux/mod.rs"]
mod imp;

#[cfg(target_os = "android")]
#[path = "android.rs"]
mod imp;

#[cfg(target_os = "windows")]
#[path = "windows.rs"]
mod imp;
#[cfg(target_os = "windows")]
use crate::winnet;

pub use imp::Error as PlatformError;

/// Errors that can be encountered whilst initializing RouteManager
#[derive(err_derive::Error, Debug)]
pub enum Error {
    /// Routing manager thread panicked before starting routing manager
    #[error(display = "Routing manager thread panicked before starting routing manager")]
    RoutingManagerThreadPanic,
    /// Platform sepcific error occured
    #[error(display = "Failed to create route manager")]
    FailedToInitializeManager(#[error(source)] imp::Error),
    /// Failed to spawn route manager future
    #[error(display = "Failed to spawn route manager on the provided executor")]
    FailedToSpawnManager,
}

/// RouteManager applies a set of routes to the route table.
/// If a destination has to be routed through the default node,
/// the route will be adjusted dynamically when the default route changes.
pub struct RouteManager {
    shutdown_tx: Option<oneshot::Sender<oneshot::Sender<()>>>,
    route_thread: Option<thread::JoinHandle<()>>,
    #[cfg(target_os = "windows")]
    callback_handles: Vec<winnet::WinNetCallbackHandle>,
}

impl RouteManager {
    /// Constructs a RouteManager and applies the required routes.
    /// Takes a map of network destinations and network nodes as an argument, and applies said
    /// routes.
    pub fn new(required_routes: HashMap<IpNetwork, NetNode>) -> Result<Self, Error> {
        let (shutdown_tx, shutdown_rx) = oneshot::channel();
        let (start_tx, start_rx) = oneshot::channel();

        let route_thread =
            thread::spawn(
                move || match imp::RouteManagerImpl::new(required_routes, shutdown_rx) {
                    Ok(route_manager) => {
                        let _ = start_tx.send(Ok(()));
                        if let Err(e) = route_manager.wait() {
                            log::error!("Route manager failed - {}", e);
                        }
                    }
                    Err(e) => {
                        let _ = start_tx.send(Err(Error::FailedToInitializeManager(e)));
                    }
                },
            );
        match start_rx.wait() {
            Ok(Ok(())) => Ok(Self {
                shutdown_tx: Some(shutdown_tx),
                route_thread: Some(route_thread),
                #[cfg(target_os = "windows")]
                callback_handles: vec![],
            }),
            Ok(Err(e)) => Err(e),
            Err(_) => Err(Error::RoutingManagerThreadPanic),
        }
    }

    /// Sets a callback that is called whenever the default route changes.
    #[cfg(target_os = "windows")]
    pub fn add_default_route_callback<T: 'static>(
        &mut self,
        callback: Option<winnet::DefaultRouteChangedCallback>,
        context: T,
    ) {
        match winnet::add_default_route_change_callback(callback, context) {
            Err(_e) => {
                // not sure if this should panic
                log::error!("Failed to add callback!");
            }
            Ok(handle) => {
                self.callback_handles.push(handle);
            }
        }
    }

    /// Stops RouteManager and removes all of the applied routes.
    pub fn stop(&mut self) {
        if let Some(shutdown_tx) = self.shutdown_tx.take() {
            let (wait_tx, wait_rx) = oneshot::channel();
            if shutdown_tx.send(wait_tx).is_err() {
                log::error!("RouteManager already down!");
            } else {
                if wait_rx.wait().is_err() {
                    log::error!("RouteManager paniced while shutting down");
                }
            }
        }
        if let Some(route_thread) = self.route_thread.take() {
            let _ = route_thread.join();
        }
    }
}

impl Drop for RouteManager {
    fn drop(&mut self) {
        // Ensuring callbacks are removed before the route manager is stopped
        #[cfg(target_os = "windows")]
        {
            self.callback_handles.clear();
        }
        self.stop();
    }
}


/// A netowrk route with a specific network node, destinaiton and an optional metric.
#[derive(Debug, Hash, Eq, PartialEq, Clone)]
struct Route {
    node: Node,
    prefix: IpNetwork,
    metric: Option<u32>,
}

impl Route {
    fn new(node: Node, prefix: IpNetwork) -> Self {
        Self {
            node,
            prefix,
            metric: None,
        }
    }
}

impl fmt::Display for Route {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} via {}", self.prefix, self.node)?;
        if let Some(metric) = &self.metric {
            write!(f, " metric {}", *metric)?;
        }
        Ok(())
    }
}

/// A network route that should be applied by the RouteManager.
/// It can either be routed through a specific network node or it can be routed through the current
/// default route.
#[derive(Debug, Hash, Eq, PartialEq, Clone)]
pub struct RequiredRoute {
    prefix: IpNetwork,
    node: NetNode,
}

impl RequiredRoute {
    /// Constructs a new required route.
    pub fn new(prefix: IpNetwork, node: impl Into<NetNode>) -> Self {
        Self {
            node: node.into(),
            prefix,
        }
    }
}

/// A NetNode represents a network node - either a real one or a symbolic default one.
/// A route with a symbolic default node will be changed whenever a new default route is created.
#[derive(Debug, Hash, Eq, PartialEq, Clone)]
pub enum NetNode {
    /// A real node will be used to set a regular route that will remain unchanged for the lifetime
    /// of the RouteManager
    RealNode(Node),
    /// A default node is a symbolic node that will resolve to the network node used in the current
    /// most preferable default route
    DefaultNode,
}

impl From<Node> for NetNode {
    fn from(node: Node) -> NetNode {
        NetNode::RealNode(node)
    }
}

/// Node represents a real network node - it can be identified by a network interface name, an IP
/// address or both.
#[derive(Debug, Hash, Eq, PartialEq, Clone)]
pub struct Node {
    ip: Option<IpAddr>,
    device: Option<String>,
}

impl Node {
    /// Construct an Node with both an IP address and an interface name.
    pub fn new(address: IpAddr, iface_name: String) -> Self {
        Self {
            ip: Some(address),
            device: Some(iface_name),
        }
    }

    /// Construct an Node from an IP address.
    pub fn address(address: IpAddr) -> Node {
        Self {
            ip: Some(address),
            device: None,
        }
    }

    /// Construct a Node from a network interface name.
    pub fn device(iface_name: String) -> Node {
        Self {
            ip: None,
            device: Some(iface_name),
        }
    }

    /// Retrieve a node's IP address
    pub fn get_address(&self) -> Option<IpAddr> {
        self.ip
    }

    /// Retrieve a node's network interface name
    pub fn get_device(&self) -> Option<&str> {
        self.device.as_ref().map(|s| s.as_ref())
    }
}

impl fmt::Display for Node {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(ip) = &self.ip {
            write!(f, "{}", ip)?;
        }
        if let Some(device) = &self.device {
            let extra_space = if self.ip.is_some() { " " } else { "" };
            write!(f, "{}dev {}", extra_space, device)?;
        }
        Ok(())
    }
}

#![allow(missing_docs)]

use ipnetwork::IpNetwork;
use std::{collections::HashMap, net::IpAddr};


use futures::{sync::oneshot, Future};
use tokio_executor::Executor;

#[cfg(target_os = "macos")]
#[path = "macos.rs"]
pub mod imp;

#[cfg(target_os = "linux")]
#[path = "linux.rs"]
pub mod imp;

#[cfg(target_os = "android")]
#[path = "android.rs"]
mod imp;

#[derive(err_derive::Error, Debug)]
pub enum Error {
    #[error(display = "Platform specific error: {}", _0)]
    PlatformError(#[error(cause)] imp::Error),
    #[error(display = "Failed to spawn route manager")]
    FailedToSpawnManager,
}

pub struct RouteManagerHandle {
    tx: Option<oneshot::Sender<oneshot::Sender<()>>>,
}

impl RouteManagerHandle {
    pub fn new(
        required_routes: HashMap<IpNetwork, NetNode>,
        exec: &mut impl Executor,
    ) -> Result<Self, Error> {
        let (tx, rx) = oneshot::channel();

        let route_manager = RouteManager::new(required_routes, rx).map_err(Error::PlatformError)?;
        exec.spawn(Box::new(
            route_manager.map_err(|e| log::error!("Routing manager failed - {}", e)),
        ))
        .map_err(|_| Error::FailedToSpawnManager)?;


        Ok(Self { tx: Some(tx) })
    }

    pub fn stop(&mut self) {
        if let Some(tx) = self.tx.take() {
            let (wait_tx, wait_rx) = oneshot::channel();
            if let Err(_) = tx.send(wait_tx) {
                log::error!("RouteManager already down!");
                return;
            }

            if wait_rx.wait().is_err() {
                log::error!("RouteManager already down!");
            }
        }
    }
}

impl Drop for RouteManagerHandle {
    fn drop(&mut self) {
        self.stop();
    }
}


type RouteManager = imp::RouteManager;

#[derive(Debug, Hash, Eq, PartialEq, Clone)]
pub struct Route {
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

#[derive(Debug, Hash, Eq, PartialEq, Clone)]
pub struct RequiredRoute {
    prefix: IpNetwork,
    node: NetNode,
}

impl RequiredRoute {
    pub fn new(prefix: IpNetwork, node: impl Into<NetNode>) -> Self {
        Self {
            node: node.into(),
            prefix,
        }
    }
}

#[derive(Debug, Hash, Eq, PartialEq, Clone)]
pub enum NetNode {
    RealNode(Node),
    DefaultNode,
}

impl From<Node> for NetNode {
    fn from(node: Node) -> NetNode {
        NetNode::RealNode(node)
    }
}

#[derive(Debug, Hash, Eq, PartialEq, Clone)]
pub struct Node {
    ip: Option<IpAddr>,
    device: Option<String>,
}

impl Node {
    pub fn new(address: IpAddr, iface_name: String) -> Self {
        Self {
            ip: Some(address),
            device: Some(iface_name),
        }
    }

    pub fn address(address: IpAddr) -> Node {
        Self {
            ip: Some(address),
            device: None,
        }
    }

    pub fn device(iface_name: String) -> Node {
        Self {
            ip: None,
            device: Some(iface_name),
        }
    }

    pub fn get_address(&self) -> Option<IpAddr> {
        self.ip
    }

    pub fn get_device(&self) -> Option<&str> {
        self.device.as_ref().map(|s| s.as_ref())
    }
}

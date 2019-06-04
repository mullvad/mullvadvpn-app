use super::NetNode;
use crate::winnet;
use futures::{sync::oneshot, Async, Future};
use ipnetwork::IpNetwork;
use std::collections::HashMap;

/// Windows routing errors.
#[derive(err_derive::Error, Debug)]
pub enum Error {
    /// Failure to apply a route
    #[error(display = "Failed to start route manager")]
    FailedToStartManager,
}

pub type Result<T> = std::result::Result<T, Error>;

pub struct RouteManagerImpl {
    shutdown_rx: oneshot::Receiver<oneshot::Sender<()>>,
}

impl RouteManagerImpl {
    pub fn new(
        required_routes: HashMap<IpNetwork, NetNode>,
        shutdown_rx: oneshot::Receiver<oneshot::Sender<()>>,
    ) -> Result<Self> {
        let routes: Vec<_> = required_routes
            .iter()
            .map(|(destination, node)| {
                let destination = winnet::WinNetIpNetwork::from(*destination);
                match node {
                    NetNode::DefaultNode => winnet::WinNetRoute::through_default_node(destination),
                    NetNode::RealNode(node) => {
                        winnet::WinNetRoute::new(winnet::WinNetNode::from(node), destination)
                    }
                }
            })
            .collect();

        if !winnet::activate_routing_manager(&routes) {
            return Err(Error::FailedToStartManager);
        }


        Ok(Self { shutdown_rx })
    }
}

impl Drop for RouteManagerImpl {
    fn drop(&mut self) {
        if !winnet::deactivate_routing_manager() {
            log::error!("Failed to deactivate routing manager");
        }
    }
}


impl Future for RouteManagerImpl {
    type Item = ();
    type Error = Error;
    fn poll(&mut self) -> Result<Async<()>> {
        match self.shutdown_rx.poll() {
            Ok(Async::Ready(result_tx)) => {
                if let Err(_e) = result_tx.send(()) {
                    log::error!("Receiver already down");
                }
                Ok(Async::Ready(()))
            }
            Ok(Async::NotReady) => Ok(Async::NotReady),
            Err(_) => Ok(Async::Ready(())),
        }
    }
}

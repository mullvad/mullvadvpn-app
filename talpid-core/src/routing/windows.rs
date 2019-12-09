use super::NetNode;
use crate::winnet;
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

/// Manages routes by calling into WinNet
pub struct RouteManager {
    callback_handles: Vec<winnet::WinNetCallbackHandle>,
    is_stopped: bool,
}

impl RouteManager {
    /// Creates a new route manager that will apply the provided routes and ensure they exist until
    /// it's stopped.
    pub fn new(required_routes: HashMap<IpNetwork, NetNode>) -> Result<Self> {
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

        Ok(Self {
            callback_handles: vec![],
            is_stopped: false,
        })
    }

    /// Sets a callback that is called whenever the default route changes.
    #[cfg(target_os = "windows")]
    pub fn add_default_route_callback<T: 'static>(
        &mut self,
        callback: Option<winnet::DefaultRouteChangedCallback>,
        context: T,
    ) {
        if self.is_stopped {
            return;
        }

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

    /// Stops the routing manager and invalidates the route manager - no new default route callbacks
    /// can be added
    pub fn stop(&mut self) {
        if !self.is_stopped {
            self.callback_handles.clear();
            winnet::deactivate_routing_manager();
            self.is_stopped = true;
        }
    }
}

impl Drop for RouteManager {
    fn drop(&mut self) {
        self.stop();
    }
}

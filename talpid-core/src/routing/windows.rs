use super::NetNode;
use crate::{routing::RequiredRoute, winnet};
use std::collections::HashSet;

/// Windows routing errors.
#[derive(err_derive::Error, Debug)]
pub enum Error {
    /// Failure to initialize route manager
    #[error(display = "Failed to start route manager")]
    FailedToStartManager,
    /// Failure to add routes
    #[error(display = "Failed to add routes")]
    AddRoutesFailed,
    /// Failure to clear routes
    #[error(display = "Failed to clear applied routes")]
    ClearRoutesFailed,
}

pub type Result<T> = std::result::Result<T, Error>;

/// Manages routes by calling into WinNet
pub struct RouteManager {
    callback_handles: Vec<winnet::WinNetCallbackHandle>,
    is_stopped: bool,
    runtime: tokio::runtime::Handle,
}

impl RouteManager {
    /// Creates a new route manager that will apply the provided routes and ensure they exist until
    /// it's stopped.
    pub fn new(
        runtime: tokio::runtime::Handle,
        required_routes: HashSet<RequiredRoute>,
    ) -> Result<Self> {
        if !winnet::activate_routing_manager() {
            return Err(Error::FailedToStartManager);
        }
        let manager = Self {
            callback_handles: vec![],
            is_stopped: false,
            runtime,
        };
        manager.add_routes(required_routes)?;
        Ok(manager)
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

    /// Removes all routes previously applied in [`RouteManager::new`] or
    /// [`RouteManager::add_routes`].
    pub fn clear_default_route_callbacks(&mut self) {
        // `WinNetCallbackHandle::drop` removes these callbacks.
        self.callback_handles.clear();
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

    /// Applies the given routes until [`RouteManager::stop`] is called.
    pub fn add_routes(&self, routes: HashSet<RequiredRoute>) -> Result<()> {
        let routes: Vec<_> = routes
            .iter()
            .map(|route| {
                let destination = winnet::WinNetIpNetwork::from(route.prefix);
                match &route.node {
                    NetNode::DefaultNode => winnet::WinNetRoute::through_default_node(destination),
                    NetNode::RealNode(node) => {
                        winnet::WinNetRoute::new(winnet::WinNetNode::from(node), destination)
                    }
                }
            })
            .collect();

        if winnet::routing_manager_add_routes(&routes) {
            Ok(())
        } else {
            Err(Error::AddRoutesFailed)
        }
    }

    /// Removes all routes previously applied in [`RouteManager::new`] or
    /// [`RouteManager::add_routes`].
    pub fn clear_routes(&self) -> Result<()> {
        if winnet::routing_manager_delete_applied_routes() {
            Ok(())
        } else {
            Err(Error::ClearRoutesFailed)
        }
    }
}

impl Drop for RouteManager {
    fn drop(&mut self) {
        self.stop();
    }
}

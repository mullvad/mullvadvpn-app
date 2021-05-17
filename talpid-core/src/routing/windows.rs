use super::NetNode;
use crate::{routing::RequiredRoute, winnet};
use futures::{
    channel::{
        mpsc::{self, UnboundedReceiver, UnboundedSender},
        oneshot,
    },
    StreamExt,
};
use std::collections::HashSet;

/// Windows routing errors.
#[derive(err_derive::Error, Debug)]
pub enum Error {
    /// The sender was dropped unexpectedly -- possible panic
    #[error(display = "The channel sender was dropped")]
    ManagerChannelDown,
    /// Failure to initialize route manager
    #[error(display = "Failed to start route manager")]
    FailedToStartManager,
    /// Failure to add routes
    #[error(display = "Failed to add routes")]
    AddRoutesFailed(#[error(source)] winnet::Error),
    /// Failure to clear routes
    #[error(display = "Failed to clear applied routes")]
    ClearRoutesFailed,
    /// WinNet returned an error while adding default route callback
    #[error(display = "Failed to set callback for default route")]
    FailedToAddDefaultRouteCallback,
    /// Attempt to use route manager that has been dropped
    #[error(display = "Cannot send message to route manager since it is down")]
    RouteManagerDown,
}

pub type Result<T> = std::result::Result<T, Error>;

/// Manages routes by calling into WinNet
pub struct RouteManager {
    runtime: tokio::runtime::Handle,
    manage_tx: Option<UnboundedSender<RouteManagerCommand>>,
}

/// Handle to a route manager.
#[derive(Clone)]
pub struct RouteManagerHandle {
    runtime: tokio::runtime::Handle,
    tx: UnboundedSender<RouteManagerCommand>,
}

impl RouteManagerHandle {
    /// Applies the given routes while the route manager is running.
    pub fn add_routes(&self, routes: HashSet<RequiredRoute>) -> Result<()> {
        let (response_tx, response_rx) = oneshot::channel();
        self.tx
            .unbounded_send(RouteManagerCommand::AddRoutes(routes, response_tx))
            .map_err(|_| Error::RouteManagerDown)?;
        self.runtime
            .block_on(response_rx)
            .map_err(|_| Error::ManagerChannelDown)?
    }
}

#[derive(Debug)]
pub enum RouteManagerCommand {
    AddRoutes(HashSet<RequiredRoute>, oneshot::Sender<Result<()>>),
    Shutdown,
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
        let (manage_tx, manage_rx) = mpsc::unbounded();
        let manager = Self {
            runtime: runtime.clone(),
            manage_tx: Some(manage_tx),
        };
        runtime.spawn(RouteManager::listen(manage_rx));
        manager.add_routes(required_routes)?;

        Ok(manager)
    }

    /// Retrieve a sender directly to the command channel.
    pub fn handle(&self) -> Result<RouteManagerHandle> {
        if let Some(tx) = &self.manage_tx {
            Ok(RouteManagerHandle {
                runtime: self.runtime.clone(),
                tx: tx.clone(),
            })
        } else {
            Err(Error::RouteManagerDown)
        }
    }

    async fn listen(mut manage_rx: UnboundedReceiver<RouteManagerCommand>) {
        while let Some(command) = manage_rx.next().await {
            match command {
                RouteManagerCommand::AddRoutes(routes, tx) => {
                    let routes: Vec<_> = routes
                        .iter()
                        .map(|route| {
                            let destination = winnet::WinNetIpNetwork::from(route.prefix);
                            match &route.node {
                                NetNode::DefaultNode => {
                                    winnet::WinNetRoute::through_default_node(destination)
                                }
                                NetNode::RealNode(node) => winnet::WinNetRoute::new(
                                    winnet::WinNetNode::from(node),
                                    destination,
                                ),
                            }
                        })
                        .collect();

                    let _ = tx.send(
                        winnet::routing_manager_add_routes(&routes).map_err(Error::AddRoutesFailed),
                    );
                }
                RouteManagerCommand::Shutdown => {
                    break;
                }
            }
        }
    }

    /// Sets a callback that is called whenever the default route changes.
    pub fn add_default_route_callback<T: 'static>(
        &mut self,
        callback: Option<winnet::DefaultRouteChangedCallback>,
        context: T,
    ) -> Result<winnet::WinNetCallbackHandle> {
        if self.manage_tx.is_none() {
            return Err(Error::RouteManagerDown);
        }
        winnet::add_default_route_change_callback(callback, context)
            .map_err(|_| Error::FailedToAddDefaultRouteCallback)
    }

    /// Stops the routing manager and invalidates the route manager - no new default route callbacks
    /// can be added
    pub fn stop(&mut self) {
        if let Some(tx) = self.manage_tx.take() {
            if tx.unbounded_send(RouteManagerCommand::Shutdown).is_err() {
                log::error!("RouteManager channel already down or thread panicked");
            }

            winnet::deactivate_routing_manager();
        }
    }

    /// Applies the given routes until [`RouteManager::stop`] is called.
    pub fn add_routes(&self, routes: HashSet<RequiredRoute>) -> Result<()> {
        if let Some(tx) = &self.manage_tx {
            let (result_tx, result_rx) = oneshot::channel();
            if tx
                .unbounded_send(RouteManagerCommand::AddRoutes(routes, result_tx))
                .is_err()
            {
                return Err(Error::RouteManagerDown);
            }
            self.runtime
                .block_on(result_rx)
                .map_err(|_| Error::ManagerChannelDown)?
        } else {
            Err(Error::RouteManagerDown)
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

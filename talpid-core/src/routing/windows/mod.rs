use crate::{routing::RequiredRoute, windows::AddressFamily};
use futures::channel::oneshot;
use std::{collections::HashSet, net::IpAddr, sync::{Arc, Weak, Mutex}};

pub use default_route_monitor::EventType;
pub use route_manager::{Route, RouteManagerInternal, Callback, CallbackHandle};
pub use get_best_default_route::{get_best_default_route,
    get_best_default_route_internal, InterfaceAndGateway, route_has_gateway,
};

mod get_best_default_route;
mod default_route_monitor;
mod route_manager;

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
    AddRoutesFailed,
    /// Failure to clear routes
    #[error(display = "Failed to clear applied routes")]
    ClearRoutesFailed,
    /// WinNet returned an error while adding default route callback
    #[error(display = "Failed to set callback for default route")]
    FailedToAddDefaultRouteCallback,
    /// Attempt to use route manager that has been dropped
    #[error(display = "Cannot send message to route manager since it is down")]
    RouteManagerDown,
    /// Something went wrong when getting the mtu of the interface
    #[error(display = "Could not get the mtu of the interface")]
    GetMtu,
    /// A windows API has errored
    #[error(display = "A windows API has errored")]
    WindowsApi,
    /// The SI family was of an unexpected value
    #[error(display = "The SI family was of an unexpected value")]
    InvalidSiFamily,
    /// Device name not found
    #[error(display = "The device name was not found")]
    DeviceNameNotFound,
    /// No default route
    #[error(display = "No default route found")]
    NoDefaultRoute,
    /// Conversion error between types
    #[error(display = "Conversion error")]
    Conversion,
    /// Could not find device gateway
    #[error(display = "Could not find device gateway")]
    DeviceGatewayNotFound,
    /// Could not get default route
    #[error(display = "Could not get default route")]
    GetDefaultRoute,
    /// Could not find device by name
    #[error(display = "Could not find device by name")]
    GetDeviceByName,
    /// Could not find device by gateway
    #[error(display = "Could not find device by gateway")]
    GetDeviceByGateway,
    /// Internal inconsistent state in RouteManager
    #[error(display = "Route manager inconsistent internal state")]
    InternalInconsistentState,
}

pub type Result<T> = std::result::Result<T, Error>;

/// Manages routes by calling into WinNet
pub struct RouteManager {
    //manage_tx: Option<UnboundedSender<RouteManagerCommand>>,
    internal: Arc<Mutex<RouteManagerInternal>>,
}

/// Handle to a route manager.
#[derive(Clone)]
pub struct RouteManagerHandle {
    //tx: UnboundedSender<RouteManagerCommand>,
    internal: Weak<Mutex<RouteManagerInternal>>,
}

impl RouteManagerHandle {
    /// Add a callback which will be called if the default route changes.
    pub fn add_default_route_change_callback(&self, callback: Callback) -> Result<CallbackHandle> {
        // TODO: Fix errors
        Ok(self.internal.upgrade().unwrap().lock().unwrap().register_default_route_changed_callback(callback).unwrap())
    }

    /// Applies the given routes while the route manager is running.
    pub async fn add_routes(&self, routes: HashSet<RequiredRoute>) -> Result<()> {
        let routes: Vec<_> = routes
            .into_iter()
            .map(|route| {
                Route {
                    network: route.prefix,
                    node: route.node,
                }
            })
            .collect();
        // TODO: remove unwraps
        Ok(self.internal.upgrade().unwrap().lock().unwrap().add_routes(routes).unwrap())
        //let (response_tx, response_rx) = oneshot::channel();
        //self.tx
        //    .unbounded_send(RouteManagerCommand::AddRoutes(routes, response_tx))
        //    .map_err(|_| Error::RouteManagerDown)?;
        //response_rx.await.map_err(|_| Error::ManagerChannelDown)?
    }

    /// Applies the given routes while the route manager is running.
    pub async fn get_mtu_for_route(&self, ip: IpAddr) -> Result<u16> {
        let addr_family = if ip.is_ipv4() {
            AddressFamily::Ipv4
        } else {
            AddressFamily::Ipv6
        };
        match get_mtu_for_route(addr_family) {
            Ok(Some(mtu)) => Ok(mtu),
            Ok(None) => Err(Error::GetMtu),
            Err(e) => Err(e),
        }

        //let (response_tx, response_rx) = oneshot::channel();
        //self.tx
        //    .unbounded_send(RouteManagerCommand::GetMtuForRoute(ip, response_tx))
        //    .map_err(|_| Error::RouteManagerDown)?;
        //response_rx.await.map_err(|_| Error::ManagerChannelDown)?
    }
}

#[derive(Debug)]
pub enum RouteManagerCommand {
    AddRoutes(HashSet<RequiredRoute>, oneshot::Sender<Result<()>>),
    GetMtuForRoute(IpAddr, oneshot::Sender<Result<u16>>),
    Shutdown,
}

impl RouteManager {
    /// Creates a new route manager that will apply the provided routes and ensure they exist until
    /// it's stopped.
    pub async fn new(required_routes: HashSet<RequiredRoute>) -> Result<Self> {
        //if !winnet::activate_routing_manager() {
        //    return Err(Error::FailedToStartManager);
        //}
        let internal = match RouteManagerInternal::new() {
            Ok(internal) => internal,
            Err(_) => return Err(Error::FailedToStartManager),
        };
        //let (manage_tx, manage_rx) = mpsc::unbounded();
        let manager = Self {
            //manage_tx: Some(manage_tx),
            internal: Arc::new(Mutex::new(internal))
        };
        //tokio::spawn(RouteManager::listen(manage_rx, internal));
        manager.add_routes(required_routes).await?;

        Ok(manager)
    }

    /// Add a callback which will be called if the default route changes.
    pub fn add_default_route_change_callback(&self, callback: Callback) -> Result<CallbackHandle> {
        // TODO: Fix errors
        Ok(self.internal.lock().unwrap().register_default_route_changed_callback(callback).unwrap())
    }

    /// Retrieve a sender directly to the command channel.
    pub fn handle(&self) -> Result<RouteManagerHandle> {
        Ok(RouteManagerHandle { internal: Arc::downgrade(&self.internal) })
        //if let Some(tx) = &self.manage_tx {
        //    Ok(RouteManagerHandle { tx: tx.clone() })
        //} else {
        //    Err(Error::RouteManagerDown)
        //}
    }

    //async fn listen(mut manage_rx: UnboundedReceiver<RouteManagerCommand>, internal: RouteManagerInternal) {
    //    while let Some(command) = manage_rx.next().await {
    //        match command {
    //            RouteManagerCommand::AddRoutes(routes, tx) => {
    //                let routes: Vec<_> = routes
    //                    .into_iter()
    //                    .map(|route| {
    //                        Route {
    //                            network: route.prefix,
    //                            node: route.node,
    //                        }
    //                        //let destination = winnet::WinNetIpNetwork::from(route.prefix);
    //                        //match &route.node {
    //                        //    NetNode::DefaultNode => {
    //                        //        winnet::WinNetRoute::through_default_node(destination)
    //                        //    }
    //                        //    NetNode::RealNode(node) => winnet::WinNetRoute::new(
    //                        //        winnet::WinNetNode::from(node),
    //                        //        destination,
    //                        //    ),
    //                        //}
    //                    })
    //                    .collect();

    //                let _ = tx.send(
    //                    internal.add_routes(routes).map_err(Error::AddRoutesFailed),
    //                );
    //            }
    //            RouteManagerCommand::GetMtuForRoute(ip, tx) => {
    //                let addr_family = if ip.is_ipv4() {
    //                    winnet::WinNetAddrFamily::IPV4
    //                } else {
    //                    winnet::WinNetAddrFamily::IPV6
    //                };
    //                let res = match get_mtu_for_route(addr_family) {
    //                    Ok(Some(mtu)) => Ok(mtu),
    //                    Ok(None) => Err(Error::GetMtu),
    //                    Err(e) => Err(e),
    //                };
    //                let _ = tx.send(res);
    //            }
    //            RouteManagerCommand::Shutdown => {
    //                break;
    //            }
    //        }
    //    }
    //}

    /// Stops the routing manager and invalidates the route manager - no new default route callbacks
    /// can be added
    pub fn stop(&mut self) {
        //if let Some(tx) = self.manage_tx.take() {
        //    if tx.unbounded_send(RouteManagerCommand::Shutdown).is_err() {
        //        log::error!("RouteManager channel already down or thread panicked");
        //    }

        //    self.internal.lock().unwrap().winnet::deactivate_routing_manager();
        //}
    }

    /// Applies the given routes until [`RouteManager::stop`] is called.
    pub async fn add_routes(&self, routes: HashSet<RequiredRoute>) -> Result<()> {
        let routes: Vec<_> = routes
            .into_iter()
            .map(|route| {
                Route {
                    network: route.prefix,
                    node: route.node,
                }
            })
            .collect();
        //TODO: No unwrap
        Ok(self.internal.lock().unwrap().add_routes(routes).unwrap())
        //if let Some(tx) = &self.manage_tx {
        //    let (result_tx, result_rx) = oneshot::channel();
        //    if tx
        //        .unbounded_send(RouteManagerCommand::AddRoutes(routes, result_tx))
        //        .is_err()
        //    {
        //        return Err(Error::RouteManagerDown);
        //    }
        //    result_rx.await.map_err(|_| Error::ManagerChannelDown)?
        //} else {
        //    Err(Error::RouteManagerDown)
        //}
    }

    /// Removes all routes previously applied in [`RouteManager::new`] or
    /// [`RouteManager::add_routes`].
    pub fn clear_routes(&self) -> Result<()> {
        self.internal.lock().unwrap().delete_applied_routes().map_err(|_| Error::ClearRoutesFailed)
        //if winnet::routing_manager_delete_applied_routes() {
        //    Ok(())
        //} else {
        //    Err(Error::ClearRoutesFailed)
        //}
    }
}

fn get_mtu_for_route(addr_family: AddressFamily) -> Result<Option<u16>> {
    match get_best_default_route(addr_family) {
        Ok(Some(route)) => {
            let interface_row = crate::windows::get_ip_interface_entry(addr_family, &route.interface_luid)
                .map_err(|e| {
                    log::error!("Could not get ip interface entry: {}", e);
                    Error::GetMtu
                })?;
            let mtu = interface_row.NlMtu;
            let mtu = u16::try_from(mtu).map_err(|_| Error::GetMtu)?;
            Ok(Some(mtu))
        }
        Ok(None) => Ok(None),
        Err(e) => {
            log::error!("Could not get best default route: {}", e);
            Err(Error::GetMtu)
        }
    }
}

impl Drop for RouteManager {
    fn drop(&mut self) {
        self.stop();
    }
}

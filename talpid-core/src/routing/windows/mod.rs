use crate::{routing::RequiredRoute, windows::AddressFamily};
use futures::channel::oneshot;
use std::{
    collections::HashSet,
    net::IpAddr,
    sync::{Arc, Mutex, Weak},
};
use tokio::sync::mpsc::{self, UnboundedReceiver, UnboundedSender};

pub use default_route_monitor::EventType;
pub use get_best_default_route::{get_best_default_route, route_has_gateway, InterfaceAndGateway};
pub use route_manager::{Callback, CallbackHandle, Route, RouteManagerInternal};

mod default_route_monitor;
mod get_best_default_route;
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
    manage_tx: Option<UnboundedSender<RouteManagerCommand>>,
}

/// Handle to a route manager.
#[derive(Clone)]
pub struct RouteManagerHandle {
    tx: UnboundedSender<RouteManagerCommand>,
}

impl RouteManagerHandle {
    /// Add a callback which will be called if the default route changes.
    pub async fn add_default_route_change_callback(
        &self,
        callback: Callback,
    ) -> Result<CallbackHandle> {
        let (response_tx, response_rx) = oneshot::channel();
        self.tx
            .send(RouteManagerCommand::RegisterDefaultRouteChangeCallback(
                callback,
                response_tx,
            ))
            .map_err(|_| Error::RouteManagerDown)?;
        response_rx.await.map_err(|_| Error::ManagerChannelDown)?
    }

    /// Applies the given routes while the route manager is running.
    pub async fn add_routes(&self, routes: HashSet<RequiredRoute>) -> Result<()> {
        let (response_tx, response_rx) = oneshot::channel();
        self.tx
            .send(RouteManagerCommand::AddRoutes(routes, response_tx))
            .map_err(|_| Error::RouteManagerDown)?;
        response_rx.await.map_err(|_| Error::ManagerChannelDown)?
    }

    /// Applies the given routes while the route manager is running.
    pub async fn get_mtu_for_route(&self, ip: IpAddr) -> Result<u16> {
        let (response_tx, response_rx) = oneshot::channel();
        self.tx
            .send(RouteManagerCommand::GetMtuForRoute(ip, response_tx))
            .map_err(|_| Error::RouteManagerDown)?;
        response_rx.await.map_err(|_| Error::ManagerChannelDown)?
    }
}

pub enum RouteManagerCommand {
    AddRoutes(HashSet<RequiredRoute>, oneshot::Sender<Result<()>>),
    GetMtuForRoute(IpAddr, oneshot::Sender<Result<u16>>),
    ClearRoutes(oneshot::Sender<Result<()>>),
    RegisterDefaultRouteChangeCallback(Callback, oneshot::Sender<Result<CallbackHandle>>),
    Shutdown,
}

impl RouteManager {
    /// Creates a new route manager that will apply the provided routes and ensure they exist until
    /// it's stopped.
    pub async fn new(required_routes: HashSet<RequiredRoute>) -> Result<Self> {
        let internal = match RouteManagerInternal::new() {
            Ok(internal) => internal,
            Err(_) => return Err(Error::FailedToStartManager),
        };
        let (manage_tx, manage_rx) = mpsc::unbounded_channel();
        let manager = Self {
            manage_tx: Some(manage_tx),
        };
        tokio::spawn(RouteManager::listen(manage_rx, internal));
        manager.add_routes(required_routes).await?;

        Ok(manager)
    }

    /// Add a callback which will be called if the default route changes.
    pub async fn add_default_route_change_callback(
        &self,
        callback: Callback,
    ) -> Result<CallbackHandle> {
        //Ok(self.internal.lock().unwrap().register_default_route_changed_callback(callback)?)
        if let Some(tx) = &self.manage_tx {
            let (result_tx, result_rx) = oneshot::channel();
            if tx
                .send(RouteManagerCommand::RegisterDefaultRouteChangeCallback(
                    callback, result_tx,
                ))
                .is_err()
            {
                return Err(Error::RouteManagerDown);
            }
            result_rx.await.map_err(|_| Error::ManagerChannelDown)?
        } else {
            Err(Error::RouteManagerDown)
        }
    }

    /// Retrieve a sender directly to the command channel.
    pub fn handle(&self) -> Result<RouteManagerHandle> {
        //Ok(RouteManagerHandle { internal: Arc::downgrade(&self.internal) })
        if let Some(tx) = &self.manage_tx {
            Ok(RouteManagerHandle { tx: tx.clone() })
        } else {
            Err(Error::RouteManagerDown)
        }
    }

    async fn listen(
        mut manage_rx: UnboundedReceiver<RouteManagerCommand>,
        mut internal: RouteManagerInternal,
    ) {
        while let Some(command) = manage_rx.recv().await {
            match command {
                RouteManagerCommand::AddRoutes(routes, tx) => {
                    let routes: Vec<_> = routes
                        .into_iter()
                        .map(|route| Route {
                            network: route.prefix,
                            node: route.node,
                        })
                        .collect();

                    let _ = tx.send(
                        internal
                            .add_routes(routes)
                            .map_err(|_| Error::AddRoutesFailed),
                    );
                }
                RouteManagerCommand::GetMtuForRoute(ip, tx) => {
                    let addr_family = if ip.is_ipv4() {
                        AddressFamily::Ipv4
                    } else {
                        AddressFamily::Ipv6
                    };
                    let res = match get_mtu_for_route(addr_family) {
                        Ok(Some(mtu)) => Ok(mtu),
                        Ok(None) => Err(Error::GetMtu),
                        Err(e) => Err(e),
                    };
                    let _ = tx.send(res);
                }
                RouteManagerCommand::ClearRoutes(tx) => {
                    let _ = tx.send(internal.delete_applied_routes());
                }
                RouteManagerCommand::RegisterDefaultRouteChangeCallback(callback, tx) => {
                    let _ = tx.send(internal.register_default_route_changed_callback(callback));
                }
                RouteManagerCommand::Shutdown => {
                    break;
                }
            }
        }
    }

    /// Stops the routing manager and invalidates the route manager - no new default route callbacks
    /// can be added
    pub fn stop(&mut self) {
        if let Some(tx) = self.manage_tx.take() {
            if tx.send(RouteManagerCommand::Shutdown).is_err() {
                log::error!("RouteManager channel already down or thread panicked");
            }
        }
    }

    /// Applies the given routes until [`RouteManager::stop`] is called.
    pub async fn add_routes(&self, routes: HashSet<RequiredRoute>) -> Result<()> {
        if let Some(tx) = &self.manage_tx {
            let (result_tx, result_rx) = oneshot::channel();
            if tx
                .send(RouteManagerCommand::AddRoutes(routes, result_tx))
                .is_err()
            {
                return Err(Error::RouteManagerDown);
            }
            result_rx.await.map_err(|_| Error::ManagerChannelDown)?
        } else {
            Err(Error::RouteManagerDown)
        }
    }

    /// Removes all routes previously applied in [`RouteManager::new`] or
    /// [`RouteManager::add_routes`].
    pub async fn clear_routes(&self) -> Result<()> {
        if let Some(tx) = &self.manage_tx {
            let (result_tx, result_rx) = oneshot::channel();
            if tx
                .send(RouteManagerCommand::ClearRoutes(result_tx))
                .is_err()
            {
                return Err(Error::RouteManagerDown);
            }
            result_rx
                .await
                .map_err(|_| Error::ManagerChannelDown)?
                .map_err(|_| Error::ClearRoutesFailed)
        } else {
            Err(Error::RouteManagerDown)
        }
    }
}

fn get_mtu_for_route(addr_family: AddressFamily) -> Result<Option<u16>> {
    match get_best_default_route(addr_family) {
        Ok(Some(route)) => {
            let interface_row = crate::windows::get_ip_interface_entry(addr_family, &route.iface)
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

use crate::RequiredRoute;
pub use default_route_monitor::EventType;
use futures::{
    channel::{
        mpsc::{self, UnboundedReceiver, UnboundedSender},
        oneshot,
    },
    StreamExt,
};
pub use get_best_default_route::{get_best_default_route, route_has_gateway, InterfaceAndGateway};
use net::AddressFamily;
pub use route_manager::{Callback, CallbackHandle, Route, RouteManagerInternal};
use std::{collections::HashSet, io, net::IpAddr};
use talpid_types::ErrorExt;
use talpid_windows_net as net;

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
    /// Attempt to use route manager that has been dropped
    #[error(display = "Cannot send message to route manager since it is down")]
    RouteManagerDown,
    /// Low level error caused by a failure to add to route table
    #[error(display = "Could not add route to route table")]
    AddToRouteTable(io::Error),
    /// Low level error caused by failure to delete route from route table
    #[error(display = "Failed to delete applied routes")]
    DeleteFromRouteTable(io::Error),
    /// GetIpForwardTable2 windows API call failed
    #[error(display = "Failed to retrieve the routing table")]
    GetIpForwardTableFailed(io::Error),
    /// GetIfEntry2 windows API call failed
    #[error(display = "Failed to retrieve network interface entry")]
    GetIfEntryFailed(io::Error),
    /// Low level error caused by failing to register the route callback
    #[error(display = "Attempt to register notify route change callback failed")]
    RegisterNotifyRouteCallback(io::Error),
    /// Low level error caused by failing to register the ip interface callback
    #[error(display = "Attempt to register notify ip interface change callback failed")]
    RegisterNotifyIpInterfaceCallback(io::Error),
    /// Low level error caused by failing to register the unicast ip address callback
    #[error(display = "Attempt to register notify unicast ip address change callback failed")]
    RegisterNotifyUnicastIpAddressCallback(io::Error),
    /// Low level error caused by windows Adapters API
    #[error(display = "Windows adapter error")]
    Adapter(io::Error),
    /// High level error caused by a failure to clear the routes in the route manager.
    /// Contains the lower error
    #[error(display = "Failed to clear applied routes")]
    ClearRoutesFailed(Box<Error>),
    /// High level error caused by a failure to add routes in the route manager.
    /// Contains the lower error
    #[error(display = "Failed to add routes")]
    AddRoutesFailed(Box<Error>),
    /// Something went wrong when getting the mtu of the interface
    #[error(display = "Could not get the mtu of the interface")]
    GetMtu,
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
            .unbounded_send(RouteManagerCommand::RegisterDefaultRouteChangeCallback(
                callback,
                response_tx,
            ))
            .map_err(|_| Error::RouteManagerDown)?;
        Ok(response_rx.await.map_err(|_| Error::ManagerChannelDown)?)
    }

    /// Applies the given routes while the route manager is running.
    pub async fn add_routes(&self, routes: HashSet<RequiredRoute>) -> Result<()> {
        let (response_tx, response_rx) = oneshot::channel();
        self.tx
            .unbounded_send(RouteManagerCommand::AddRoutes(routes, response_tx))
            .map_err(|_| Error::RouteManagerDown)?;
        response_rx.await.map_err(|_| Error::ManagerChannelDown)?
    }

    /// Applies the given routes while the route manager is running.
    pub async fn get_mtu_for_route(&self, ip: IpAddr) -> Result<u16> {
        let (response_tx, response_rx) = oneshot::channel();
        self.tx
            .unbounded_send(RouteManagerCommand::GetMtuForRoute(ip, response_tx))
            .map_err(|_| Error::RouteManagerDown)?;
        response_rx.await.map_err(|_| Error::ManagerChannelDown)?
    }
}

pub enum RouteManagerCommand {
    AddRoutes(HashSet<RequiredRoute>, oneshot::Sender<Result<()>>),
    GetMtuForRoute(IpAddr, oneshot::Sender<Result<u16>>),
    ClearRoutes,
    RegisterDefaultRouteChangeCallback(Callback, oneshot::Sender<CallbackHandle>),
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
        let (manage_tx, manage_rx) = mpsc::unbounded();
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
        if let Some(tx) = &self.manage_tx {
            let (result_tx, result_rx) = oneshot::channel();
            if tx
                .unbounded_send(RouteManagerCommand::RegisterDefaultRouteChangeCallback(
                    callback, result_tx,
                ))
                .is_err()
            {
                return Err(Error::RouteManagerDown);
            }
            Ok(result_rx.await.map_err(|_| Error::ManagerChannelDown)?)
        } else {
            Err(Error::RouteManagerDown)
        }
    }

    /// Retrieve a sender directly to the command channel.
    pub fn handle(&self) -> Result<RouteManagerHandle> {
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
        while let Some(command) = manage_rx.next().await {
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
                            .map_err(|e| Error::AddRoutesFailed(Box::new(e))),
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
                RouteManagerCommand::ClearRoutes => {
                    if let Err(e) = internal.delete_applied_routes() {
                        log::error!("{}", e.display_chain_with_msg("Could not clear routes"));
                    }
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
            if tx.unbounded_send(RouteManagerCommand::Shutdown).is_err() {
                log::error!("RouteManager channel already down or thread panicked");
            }
        }
    }

    /// Applies the given routes until [`RouteManager::stop`] is called.
    pub async fn add_routes(&self, routes: HashSet<RequiredRoute>) -> Result<()> {
        if let Some(tx) = &self.manage_tx {
            let (result_tx, result_rx) = oneshot::channel();
            if tx
                .unbounded_send(RouteManagerCommand::AddRoutes(routes, result_tx))
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
    pub fn clear_routes(&self) -> Result<()> {
        if let Some(tx) = &self.manage_tx {
            tx.unbounded_send(RouteManagerCommand::ClearRoutes)
                .map_err(|_| Error::RouteManagerDown)
        } else {
            Err(Error::RouteManagerDown)
        }
    }
}

fn get_mtu_for_route(addr_family: AddressFamily) -> Result<Option<u16>> {
    match get_best_default_route(addr_family) {
        Ok(Some(route)) => {
            let interface_row =
                talpid_windows_net::get_ip_interface_entry(addr_family, &route.iface).map_err(
                    |e| {
                        log::error!("Could not get ip interface entry: {}", e);
                        Error::GetMtu
                    },
                )?;
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

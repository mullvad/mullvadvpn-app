use crate::RequiredRoute;
pub use default_route_monitor::EventType;
use futures::{
    StreamExt,
    channel::{
        mpsc::{self, UnboundedReceiver, UnboundedSender},
        oneshot,
    },
};
pub use get_best_default_route::{InterfaceAndGateway, get_best_default_route};
use net::AddressFamily;
pub use route_manager::{Callback, CallbackHandle, Route, RouteManagerInternal};
use std::{collections::HashSet, io, net::IpAddr};
use talpid_types::ErrorExt;
use talpid_windows::net;

mod default_route_monitor;
mod get_best_default_route;
mod route_manager;

/// Windows routing errors.
#[derive(thiserror::Error, Debug)]
pub enum Error {
    /// Failure to initialize route manager
    #[error("Failed to start route manager")]
    FailedToStartManager,
    /// Attempt to use route manager that has been dropped
    #[error("Cannot send message to route manager since it is down")]
    RouteManagerDown,
    /// Low level error caused by a failure to add to route table
    #[error("Could not add route to route table")]
    AddToRouteTable(io::Error),
    /// Low level error caused by failure to delete route from route table
    #[error("Failed to delete applied routes")]
    DeleteFromRouteTable(io::Error),
    /// GetIpForwardTable2 windows API call failed
    #[error("Failed to retrieve the routing table")]
    GetIpForwardTableFailed(io::Error),
    /// GetIfEntry2 windows API call failed
    #[error("Failed to retrieve network interface entry")]
    GetIfEntryFailed(io::Error),
    /// Low level error caused by failing to register the route callback
    #[error("Attempt to register notify route change callback failed")]
    RegisterNotifyRouteCallback(io::Error),
    /// Low level error caused by failing to register the ip interface callback
    #[error("Attempt to register notify ip interface change callback failed")]
    RegisterNotifyIpInterfaceCallback(io::Error),
    /// Low level error caused by failing to register the unicast ip address callback
    #[error("Attempt to register notify unicast ip address change callback failed")]
    RegisterNotifyUnicastIpAddressCallback(io::Error),
    /// Low level error caused by windows Adapters API
    #[error("Windows adapter error")]
    Adapter(io::Error),
    /// High level error caused by a failure to clear the routes in the route manager.
    /// Contains the lower error
    #[error("Failed to clear applied routes")]
    ClearRoutesFailed(Box<Error>),
    /// High level error caused by a failure to add routes in the route manager.
    /// Contains the lower error
    #[error("Failed to add routes")]
    AddRoutesFailed(Box<Error>),
    /// Something went wrong when getting the mtu of the interface
    #[error("Could not get the mtu of the interface")]
    GetMtu,
    /// The SI family was of an unexpected value
    #[error("The SI family was of an unexpected value")]
    InvalidSiFamily,
    /// Device name not found
    #[error("The device name was not found")]
    DeviceNameNotFound,
    /// No default route
    #[error("No default route found")]
    NoDefaultRoute,
    /// Conversion error between types
    #[error("Conversion error")]
    Conversion,
    /// Could not find device gateway
    #[error("Could not find device gateway")]
    DeviceGatewayNotFound,
    /// Could not get default route
    #[error("Could not get default route")]
    GetDefaultRoute,
    /// Could not find device by name
    #[error("Could not find device by name")]
    GetDeviceByName,
    /// Could not find device by gateway
    #[error("Could not find device by gateway")]
    GetDeviceByGateway,
}

impl Error {
    /// Return whether retrying the operation that caused this error is likely to succeed.
    pub fn is_recoverable(&self) -> bool {
        matches!(self, Error::AddRoutesFailed(_))
    }
}

pub type Result<T> = std::result::Result<T, Error>;

/// Manages routes by calling into WinNet
#[derive(Debug, Clone)]
pub struct RouteManagerHandle {
    tx: UnboundedSender<RouteManagerCommand>,
}

pub enum RouteManagerCommand {
    AddRoutes(HashSet<RequiredRoute>, oneshot::Sender<Result<()>>),
    GetMtuForRoute(IpAddr, oneshot::Sender<Result<u16>>),
    ClearRoutes,
    RegisterDefaultRouteChangeCallback(Callback, oneshot::Sender<CallbackHandle>),
    Shutdown(oneshot::Sender<()>),
}

impl RouteManagerHandle {
    /// Create a new route manager
    #[expect(clippy::unused_async)]
    pub async fn spawn() -> Result<Self> {
        let internal = RouteManagerInternal::new().map_err(|_| Error::FailedToStartManager)?;
        let (tx, rx) = mpsc::unbounded();
        let handle = Self { tx };
        tokio::spawn(RouteManagerHandle::run(rx, internal));

        Ok(handle)
    }

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
        response_rx.await.map_err(|_| Error::RouteManagerDown)
    }

    /// Applies the given routes while the route manager is running.
    pub async fn add_routes(&self, routes: HashSet<RequiredRoute>) -> Result<()> {
        let (response_tx, response_rx) = oneshot::channel();
        self.tx
            .unbounded_send(RouteManagerCommand::AddRoutes(routes, response_tx))
            .map_err(|_| Error::RouteManagerDown)?;
        response_rx.await.map_err(|_| Error::RouteManagerDown)?
    }

    /// Retrieve MTU for the given destination/route.
    pub async fn get_mtu_for_route(&self, ip: IpAddr) -> Result<u16> {
        let (response_tx, response_rx) = oneshot::channel();
        self.tx
            .unbounded_send(RouteManagerCommand::GetMtuForRoute(ip, response_tx))
            .map_err(|_| Error::RouteManagerDown)?;
        response_rx.await.map_err(|_| Error::RouteManagerDown)?
    }

    /// Stop the routing manager actor and revert all changes to routing
    pub async fn stop(&self) {
        let (result_tx, result_rx) = oneshot::channel();
        _ = self
            .tx
            .unbounded_send(RouteManagerCommand::Shutdown(result_tx));
        _ = result_rx.await;
    }

    /// Removes all routes previously applied in [`RouteManagerInternal::add_routes`].
    pub fn clear_routes(&self) -> Result<()> {
        self.tx
            .unbounded_send(RouteManagerCommand::ClearRoutes)
            .map_err(|_| Error::RouteManagerDown)
    }

    async fn run(
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
                RouteManagerCommand::Shutdown(tx) => {
                    drop(internal);
                    let _ = tx.send(());
                    break;
                }
            }
        }
    }
}

fn get_mtu_for_route(addr_family: AddressFamily) -> Result<Option<u16>> {
    match get_best_default_route(addr_family) {
        Ok(Some(route)) => {
            let interface_row =
                net::get_ip_interface_entry(addr_family, &route.iface).map_err(|e| {
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

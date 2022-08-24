use super::NetNode;
use crate::{routing::RequiredRoute, winnet};
use futures::{
    channel::{
        mpsc::{self, UnboundedReceiver, UnboundedSender},
        oneshot,
    },
    StreamExt,
};
use std::{collections::HashSet, net::IpAddr};
use windows_sys::Win32::NetworkManagement::IpHelper::NET_LUID_LH;
use winnet::WinNetAddrFamily;

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
    /// Something went wrong when getting the mtu of the interface
    #[error(display = "Could not get the mtu of the interface")]
    GetMtu,
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
        if !winnet::activate_routing_manager() {
            return Err(Error::FailedToStartManager);
        }
        let (manage_tx, manage_rx) = mpsc::unbounded();
        let manager = Self {
            manage_tx: Some(manage_tx),
        };
        tokio::spawn(RouteManager::listen(manage_rx));
        manager.add_routes(required_routes).await?;

        Ok(manager)
    }

    /// Retrieve a sender directly to the command channel.
    pub fn handle(&self) -> Result<RouteManagerHandle> {
        if let Some(tx) = &self.manage_tx {
            Ok(RouteManagerHandle { tx: tx.clone() })
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
                RouteManagerCommand::GetMtuForRoute(ip, tx) => {
                    let addr_family = if ip.is_ipv4() {
                        winnet::WinNetAddrFamily::IPV4
                    } else {
                        winnet::WinNetAddrFamily::IPV6
                    };
                    let res = match get_mtu_for_route(addr_family) {
                        Ok(Some(mtu)) => Ok(mtu),
                        Ok(None) => Err(Error::GetMtu),
                        Err(e) => Err(e),
                    };
                    let _ = tx.send(res);
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

            winnet::deactivate_routing_manager();
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
        if winnet::routing_manager_delete_applied_routes() {
            Ok(())
        } else {
            Err(Error::ClearRoutesFailed)
        }
    }
}

fn get_mtu_for_route(addr_family: WinNetAddrFamily) -> Result<Option<u16>> {
    use crate::windows::AddressFamily;
    match winnet::get_best_default_route(addr_family) {
        Ok(Some(route)) => {
            let addr_family = match addr_family {
                WinNetAddrFamily::IPV4 => AddressFamily::Ipv4,
                WinNetAddrFamily::IPV6 => AddressFamily::Ipv6,
            };
            let luid = NET_LUID_LH {
                Value: route.interface_luid,
            };
            let interface_row = crate::windows::get_ip_interface_entry(addr_family, &luid)
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

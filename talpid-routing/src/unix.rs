#![cfg_attr(target_os = "android", allow(dead_code))]
#![cfg_attr(target_os = "windows", allow(dead_code))]
// TODO: remove the allow(dead_code) for android once it's up to scratch.
use super::RequiredRoute;
#[cfg(target_os = "linux")]
use super::Route;

use futures::channel::{
    mpsc::{self, UnboundedSender},
    oneshot,
};
use std::{
    collections::HashSet,
    io,
    net::{Ipv4Addr, Ipv6Addr},
};

#[cfg(any(target_os = "linux", target_os = "macos"))]
use futures::stream::Stream;

#[cfg(target_os = "linux")]
use std::net::IpAddr;

#[allow(clippy::module_inception)]
#[cfg(target_os = "macos")]
#[path = "macos.rs"]
pub mod imp;

#[allow(clippy::module_inception)]
#[cfg(target_os = "linux")]
#[path = "linux.rs"]
mod imp;

#[allow(clippy::module_inception)]
#[cfg(target_os = "android")]
#[path = "android.rs"]
mod imp;

pub use imp::Error as PlatformError;

/// Errors that can be encountered whilst initializing RouteManager
#[derive(err_derive::Error, Debug)]
pub enum Error {
    /// Route manager thread may have panicked
    #[error(display = "The channel sender was dropped")]
    ManagerChannelDown,
    /// Platform specific error occurred
    #[error(display = "Internal route manager error")]
    PlatformError(#[error(source)] imp::Error),
    /// Failed to spawn route manager future
    #[error(display = "Failed to spawn route manager on the provided executor")]
    FailedToSpawnManager,
    /// Failed to spawn route manager runtime
    #[error(display = "Failed to spawn route manager runtime")]
    FailedToSpawnRuntime(#[error(source)] io::Error),
    /// Attempt to use route manager that has been dropped
    #[error(display = "Cannot send message to route manager since it is down")]
    RouteManagerDown,
    /// Failed to obtain a default route
    // TODO: elaborate on this variant, possibly add more data
    #[error(display = "Failed to obtain default routes")]
    DefaultRoute,
}

/// Handle to a route manager.
#[derive(Clone)]
pub struct RouteManagerHandle {
    tx: UnboundedSender<RouteManagerCommand>,
}

impl RouteManagerHandle {
    /// Applies the given routes while the route manager is running.
    pub async fn add_routes(&self, routes: HashSet<RequiredRoute>) -> Result<(), Error> {
        let (response_tx, response_rx) = oneshot::channel();
        self.tx
            .unbounded_send(RouteManagerCommand::AddRoutes(routes, response_tx))
            .map_err(|_| Error::RouteManagerDown)?;
        response_rx
            .await
            .map_err(|_| Error::ManagerChannelDown)?
            .map_err(Error::PlatformError)
    }

    /// Listen for non-tunnel default route changes.
    #[cfg(target_os = "macos")]
    pub async fn default_route_listener(&self) -> Result<impl Stream<Item = DefaultRouteEvent>, Error> {
        let (response_tx, response_rx) = oneshot::channel();
        self.tx
            .unbounded_send(RouteManagerCommand::NewDefaultRouteListener(response_tx))
            .map_err(|_| Error::RouteManagerDown)?;
        response_rx.await.map_err(|_| Error::ManagerChannelDown)
    }

    /// Ensure that packets are routed using the correct tables.
    #[cfg(target_os = "linux")]
    pub async fn create_routing_rules(&self, enable_ipv6: bool) -> Result<(), Error> {
        let (response_tx, response_rx) = oneshot::channel();
        self.tx
            .unbounded_send(RouteManagerCommand::CreateRoutingRules(
                enable_ipv6,
                response_tx,
            ))
            .map_err(|_| Error::RouteManagerDown)?;
        response_rx
            .await
            .map_err(|_| Error::ManagerChannelDown)?
            .map_err(Error::PlatformError)
    }

    /// Remove any routing rules created by [Self::create_routing_rules].
    #[cfg(target_os = "linux")]
    pub async fn clear_routing_rules(&self) -> Result<(), Error> {
        let (response_tx, response_rx) = oneshot::channel();
        self.tx
            .unbounded_send(RouteManagerCommand::ClearRoutingRules(response_tx))
            .map_err(|_| Error::RouteManagerDown)?;
        response_rx
            .await
            .map_err(|_| Error::ManagerChannelDown)?
            .map_err(Error::PlatformError)
    }

    /// Listen for route changes.
    #[cfg(target_os = "linux")]
    pub async fn change_listener(&self) -> Result<impl Stream<Item = CallbackMessage>, Error> {
        let (response_tx, response_rx) = oneshot::channel();
        self.tx
            .unbounded_send(RouteManagerCommand::NewChangeListener(response_tx))
            .map_err(|_| Error::RouteManagerDown)?;
        response_rx.await.map_err(|_| Error::ManagerChannelDown)
    }

    /// Listen for route changes.
    #[cfg(target_os = "linux")]
    pub async fn get_destination_route(
        &self,
        destination: IpAddr,
        mark: Option<Fwmark>,
    ) -> Result<Option<Route>, Error> {
        let (response_tx, response_rx) = oneshot::channel();
        self.tx
            .unbounded_send(RouteManagerCommand::GetDestinationRoute(
                destination,
                mark,
                response_tx,
            ))
            .map_err(|_| Error::RouteManagerDown)?;
        response_rx
            .await
            .map_err(|_| Error::ManagerChannelDown)?
            .map_err(Error::PlatformError)
    }

    /// Listen for route changes.
    #[cfg(target_os = "linux")]
    pub async fn get_mtu_for_route(&self, ip: IpAddr) -> Result<u16, Error> {
        let (response_tx, response_rx) = oneshot::channel();
        self.tx
            .unbounded_send(RouteManagerCommand::GetMtuForRoute(ip, response_tx))
            .map_err(|_| Error::RouteManagerDown)?;
        response_rx
            .await
            .map_err(|_| Error::ManagerChannelDown)?
            .map_err(Error::PlatformError)
    }
}

/// Represents a firewall mark.
#[cfg(target_os = "linux")]
type Fwmark = u32;

/// IPv6 addresess for tunnel interface
#[cfg(target_os = "macos")]
#[derive(Clone, Copy, Debug)]
pub struct TunnelRoutesV4 {
    /// IPv4 gateway of the tunnel
    pub tunnel_gateway: Ipv4Addr,
    /// IPv4 interface address
    pub tunnel_ip: Ipv4Addr,
}

/// IPv6 addresess for tunnel interface
#[cfg(target_os = "macos")]
#[derive(Copy, Clone, Debug)]
pub struct TunnelRoutesV6 {
    /// IPv6 gateway of the tunnel
    pub tunnel_gateway: Ipv6Addr,
    /// IPv6 interface address
    pub tunnel_ip: Ipv6Addr,
}

/// Commands for the underlying route manager object.
#[derive(Debug)]
pub(crate) enum RouteManagerCommand {
    AddRoutes(
        HashSet<RequiredRoute>,
        oneshot::Sender<Result<(), PlatformError>>,
    ),
    ClearRoutes,
    Shutdown(oneshot::Sender<()>),
    #[cfg(target_os = "macos")]
    NewDefaultRouteListener(oneshot::Sender<mpsc::UnboundedReceiver<DefaultRouteEvent>>),
    #[cfg(target_os = "linux")]
    CreateRoutingRules(bool, oneshot::Sender<Result<(), PlatformError>>),
    #[cfg(target_os = "linux")]
    ClearRoutingRules(oneshot::Sender<Result<(), PlatformError>>),
    #[cfg(target_os = "linux")]
    NewChangeListener(oneshot::Sender<mpsc::UnboundedReceiver<CallbackMessage>>),
    #[cfg(target_os = "linux")]
    GetMtuForRoute(IpAddr, oneshot::Sender<Result<u16, PlatformError>>),
    /// Attempt to fetch a route for the given destination with an optional firewall mark.
    #[cfg(target_os = "linux")]
    GetDestinationRoute(
        IpAddr,
        Option<Fwmark>,
        oneshot::Sender<Result<Option<Route>, PlatformError>>,
    ),
}

/// Event that is sent when a preferred non-tunnel default route is
/// added or removed.
#[cfg(target_os = "macos")]
#[derive(Debug, Clone, Copy)]
pub enum DefaultRouteEvent {
    /// Added or updated a non-tunnel default route
    AddedOrChanged,
    /// Non-tunnel default route was removed
    Removed,
}

#[cfg(target_os = "linux")]
#[derive(Debug, Clone)]
pub enum CallbackMessage {
    NewRoute(Route),
    DelRoute(Route),
}

/// RouteManager applies a set of routes to the route table.
/// If a destination has to be routed through the default node,
/// the route will be adjusted dynamically when the default route changes.
pub struct RouteManager {
    manage_tx: Option<UnboundedSender<RouteManagerCommand>>,
    runtime: tokio::runtime::Handle,
}

impl RouteManager {
    /// Constructs a RouteManager and applies the required routes.
    /// Takes a set of network destinations and network nodes as an argument, and applies said
    /// routes.
    pub async fn new(
        #[cfg(target_os = "linux")] fwmark: u32,
        #[cfg(target_os = "linux")] table_id: u32,
    ) -> Result<Self, Error> {
        let (manage_tx, manage_rx) = mpsc::unbounded();
        let manager = imp::RouteManagerImpl::new(
            #[cfg(target_os = "linux")]
            fwmark,
            #[cfg(target_os = "linux")]
            table_id,
        )
        .await?;
        tokio::spawn(manager.run(manage_rx));

        Ok(Self {
            runtime: tokio::runtime::Handle::current(),
            manage_tx: Some(manage_tx),
        })
    }

    /// Stops RouteManager and removes all of the applied routes.
    pub async fn stop(&mut self) {
        if let Some(tx) = self.manage_tx.take() {
            let (wait_tx, wait_rx) = oneshot::channel();

            if tx
                .unbounded_send(RouteManagerCommand::Shutdown(wait_tx))
                .is_err()
            {
                log::error!("RouteManager already down!");
                return;
            }

            if wait_rx.await.is_err() {
                log::error!("{}", Error::ManagerChannelDown);
            }
        }
    }

    /// Applies the given routes until [`RouteManager::stop`] is called.
    pub async fn add_routes(&mut self, routes: HashSet<RequiredRoute>) -> Result<(), Error> {
        if let Some(tx) = &self.manage_tx {
            let (result_tx, result_rx) = oneshot::channel();
            if tx
                .unbounded_send(RouteManagerCommand::AddRoutes(routes, result_tx))
                .is_err()
            {
                return Err(Error::RouteManagerDown);
            }

            result_rx
                .await
                .map_err(|_| Error::ManagerChannelDown)?
                .map_err(Error::PlatformError)
        } else {
            Err(Error::RouteManagerDown)
        }
    }

    /// Removes all routes previously applied in [`RouteManager::add_routes`].
    pub fn clear_routes(&mut self) -> Result<(), Error> {
        if let Some(tx) = &self.manage_tx {
            if tx.unbounded_send(RouteManagerCommand::ClearRoutes).is_err() {
                return Err(Error::RouteManagerDown);
            }
            Ok(())
        } else {
            Err(Error::RouteManagerDown)
        }
    }

    /// Ensure that packets are routed using the correct tables.
    #[cfg(target_os = "linux")]
    pub async fn create_routing_rules(&mut self, enable_ipv6: bool) -> Result<(), Error> {
        self.handle()?.create_routing_rules(enable_ipv6).await
    }

    /// Remove any routing rules created by [Self::create_routing_rules].
    #[cfg(target_os = "linux")]
    pub async fn clear_routing_rules(&mut self) -> Result<(), Error> {
        self.handle()?.clear_routing_rules().await
    }

    /// Retrieve a sender directly to the command channel.
    pub fn handle(&self) -> Result<RouteManagerHandle, Error> {
        if let Some(tx) = &self.manage_tx {
            Ok(RouteManagerHandle { tx: tx.clone() })
        } else {
            Err(Error::RouteManagerDown)
        }
    }
}

impl Drop for RouteManager {
    fn drop(&mut self) {
        self.runtime.clone().block_on(self.stop());
    }
}

#[cfg(any(target_os = "linux", target_os = "macos"))]
use crate::Route;

use super::RequiredRoute;

use futures::channel::{
    mpsc::{self, UnboundedSender},
    oneshot,
};
use std::{collections::HashSet, io, sync::Arc};

#[cfg(any(target_os = "linux", target_os = "macos"))]
use futures::stream::Stream;

#[cfg(target_os = "linux")]
use std::net::IpAddr;

#[allow(clippy::module_inception)]
#[cfg(target_os = "macos")]
#[path = "macos/mod.rs"]
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
#[derive(thiserror::Error, Debug)]
pub enum Error {
    /// Route manager thread may have panicked
    #[error("The channel sender was dropped")]
    ManagerChannelDown,
    /// Platform specific error occurred
    #[error("Internal route manager error")]
    PlatformError(#[from] imp::Error),
    /// Failed to spawn route manager future
    #[error("Failed to spawn route manager on the provided executor")]
    FailedToSpawnManager,
    /// Failed to spawn route manager runtime
    #[error("Failed to spawn route manager runtime")]
    FailedToSpawnRuntime(#[from] io::Error),
    /// Attempt to use route manager that has been dropped
    #[error("Cannot send message to route manager since it is down")]
    RouteManagerDown,
}

impl Error {
    /// Return whether retrying the operation that caused this error is likely to succeed.
    #[cfg(target_os = "macos")]
    pub fn is_recoverable(&self) -> bool {
        // If the default route disappears while connecting but before it is caught by the offline
        // monitor, then the gateway will be unreachable. In this case, just retry.
        matches!(
            self,
            Error::PlatformError(PlatformError::AddRoute(imp::RouteError::Unreachable,))
        )
    }

    /// Return whether retrying the operation that caused this error is likely to succeed.
    #[cfg(not(target_os = "macos"))]
    pub fn is_recoverable(&self) -> bool {
        false
    }
}

/// Handle to a route manager.
#[derive(Clone)]
pub struct RouteManagerHandle {
    tx: Arc<UnboundedSender<RouteManagerCommand>>,
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
    pub async fn default_route_listener(
        &self,
    ) -> Result<impl Stream<Item = DefaultRouteEvent>, Error> {
        let (response_tx, response_rx) = oneshot::channel();
        self.tx
            .unbounded_send(RouteManagerCommand::NewDefaultRouteListener(response_tx))
            .map_err(|_| Error::RouteManagerDown)?;
        response_rx.await.map_err(|_| Error::ManagerChannelDown)
    }

    /// Get current non-tunnel default routes.
    #[cfg(target_os = "macos")]
    pub async fn get_default_routes(&self) -> Result<(Option<Route>, Option<Route>), Error> {
        let (response_tx, response_rx) = oneshot::channel();
        self.tx
            .unbounded_send(RouteManagerCommand::GetDefaultRoutes(response_tx))
            .map_err(|_| Error::RouteManagerDown)?;
        response_rx.await.map_err(|_| Error::ManagerChannelDown)
    }

    /// Get current non-tunnel default routes.
    #[cfg(target_os = "macos")]
    pub fn refresh_routes(&self) -> Result<(), Error> {
        self.tx
            .unbounded_send(RouteManagerCommand::RefreshRoutes)
            .map_err(|_| Error::RouteManagerDown)
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
    RefreshRoutes,
    #[cfg(target_os = "macos")]
    NewDefaultRouteListener(oneshot::Sender<mpsc::UnboundedReceiver<DefaultRouteEvent>>),
    #[cfg(target_os = "macos")]
    GetDefaultRoutes(oneshot::Sender<(Option<Route>, Option<Route>)>),
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
    /// Added or updated a non-tunnel default IPv4 route
    AddedOrChangedV4,
    /// Added or updated a non-tunnel default IPv6 route
    AddedOrChangedV6,
    /// Non-tunnel default IPv4 route was removed
    RemovedV4,
    /// Non-tunnel default IPv6 route was removed
    RemovedV6,
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
    manage_tx: Option<Arc<UnboundedSender<RouteManagerCommand>>>,
    runtime: tokio::runtime::Handle,
}

impl RouteManager {
    /// Construct a RouteManager.
    pub async fn new(
        #[cfg(target_os = "linux")] fwmark: u32,
        #[cfg(target_os = "linux")] table_id: u32,
    ) -> Result<Self, Error> {
        let (manage_tx, manage_rx) = mpsc::unbounded();
        let manage_tx = Arc::new(manage_tx);
        let manager = imp::RouteManagerImpl::new(
            #[cfg(target_os = "linux")]
            fwmark,
            #[cfg(target_os = "linux")]
            table_id,
            #[cfg(target_os = "macos")]
            Arc::downgrade(&manage_tx),
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

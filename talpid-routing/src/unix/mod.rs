#[cfg(target_os = "macos")]
pub use crate::{imp::imp::DefaultRoute, Gateway};

#[cfg(any(target_os = "linux", target_os = "macos"))]
use super::RequiredRoute;
#[cfg(target_os = "linux")]
use super::Route;

use futures::channel::{
    mpsc::{self, UnboundedSender},
    oneshot,
};
use std::sync::Arc;
#[cfg(target_os = "android")]
use talpid_types::android::AndroidContext;

#[cfg(any(target_os = "linux", target_os = "macos"))]
use futures::stream::Stream;
#[cfg(any(target_os = "linux", target_os = "macos"))]
use std::collections::HashSet;

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

#[cfg(target_os = "android")]
use crate::Route;
#[cfg(any(target_os = "macos", target_os = "linux"))]
pub use imp::Error as PlatformError;

/// Errors that can be encountered whilst interacting with a [RouteManagerHandle].
#[derive(thiserror::Error, Debug)]
pub enum Error {
    /// Route manager thread may have panicked
    #[error("The channel sender was dropped")]
    ManagerChannelDown,
    /// Platform specific error occurred
    #[error("Internal route manager error")]
    PlatformError(#[from] imp::Error),
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

/// Represents a firewall mark.
#[cfg(target_os = "linux")]
type Fwmark = u32;

/// Commands for the underlying route manager object.
#[cfg(target_os = "linux")]
#[derive(Debug)]
pub(crate) enum RouteManagerCommand {
    AddRoutes(
        HashSet<RequiredRoute>,
        oneshot::Sender<Result<(), PlatformError>>,
    ),
    ClearRoutes,
    Shutdown(oneshot::Sender<()>),
    CreateRoutingRules(bool, oneshot::Sender<Result<(), PlatformError>>),
    ClearRoutingRules(oneshot::Sender<Result<(), PlatformError>>),
    NewChangeListener(oneshot::Sender<mpsc::UnboundedReceiver<CallbackMessage>>),
    GetMtuForRoute(IpAddr, oneshot::Sender<Result<u16, PlatformError>>),
    /// Attempt to fetch a route for the given destination with an optional firewall mark.
    GetDestinationRoute(
        IpAddr,
        Option<Fwmark>,
        oneshot::Sender<Result<Option<Route>, PlatformError>>,
    ),
}

/// Commands for the underlying route manager object.
#[cfg(target_os = "android")]
#[derive(Debug)]
pub(crate) enum RouteManagerCommand {
    ClearRoutes(oneshot::Sender<()>),
    WaitForRoutes(oneshot::Sender<()>, Vec<Route>),
    Shutdown(oneshot::Sender<()>),
}

/// Commands for the underlying route manager object.
#[cfg(target_os = "macos")]
#[derive(Debug)]
pub(crate) enum RouteManagerCommand {
    AddRoutes(
        HashSet<RequiredRoute>,
        oneshot::Sender<Result<(), PlatformError>>,
    ),
    ClearRoutes,
    Shutdown(oneshot::Sender<()>),
    RefreshRoutes,
    NewDefaultRouteListener(oneshot::Sender<mpsc::UnboundedReceiver<DefaultRouteEvent>>),
    GetDefaultRoutes(oneshot::Sender<(Option<DefaultRoute>, Option<DefaultRoute>)>),
    NewInterfaceChangeListener(oneshot::Sender<mpsc::UnboundedReceiver<InterfaceEvent>>),
    /// Return gateway for V4 and V6
    GetDefaultGateway(oneshot::Sender<(Option<Gateway>, Option<Gateway>)>),
}

/// Event that is sent when interface details may have changed for some interface.
#[cfg(target_os = "macos")]
pub struct InterfaceEvent {
    pub interface_index: u16,
    pub mtu: u16,
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

/// Route manager applies a set of routes to the route table.
/// If a destination has to be routed through the default node,
/// the route will be adjusted dynamically when the default route changes.
#[derive(Debug, Clone)]
pub struct RouteManagerHandle {
    tx: Arc<UnboundedSender<RouteManagerCommand>>,
}

impl RouteManagerHandle {
    /// Construct a route manager.
    pub async fn spawn(
        #[cfg(target_os = "linux")] fwmark: u32,
        #[cfg(target_os = "linux")] table_id: u32,
        #[cfg(target_os = "android")] android_context: AndroidContext,
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
            #[cfg(target_os = "android")]
            android_context,
        )
        .await?;
        tokio::spawn(manager.run(manage_rx));

        Ok(Self { tx: manage_tx })
    }

    /// Stop route manager and revert all changes to routing
    pub async fn stop(&self) {
        let (wait_tx, wait_rx) = oneshot::channel();
        let _ = self
            .tx
            .unbounded_send(RouteManagerCommand::Shutdown(wait_tx));
        let _ = wait_rx.await;
    }

    /// Applies the given routes until they are cleared
    #[cfg(not(target_os = "android"))]
    pub async fn add_routes(&self, routes: HashSet<RequiredRoute>) -> Result<(), Error> {
        let (result_tx, result_rx) = oneshot::channel();
        self.tx
            .unbounded_send(RouteManagerCommand::AddRoutes(routes, result_tx))
            .map_err(|_| Error::RouteManagerDown)?;

        result_rx
            .await
            .map_err(|_| Error::ManagerChannelDown)?
            .map_err(Error::PlatformError)
    }

    /// Wait for routes to come up.
    ///
    /// This function is guaranteed to *not* wait for longer than 2 seconds.
    /// Please, see the implementation of this function for further details.
    #[cfg(target_os = "android")]
    pub async fn wait_for_routes(&self, expect_routes: Vec<Route>) -> Result<(), Error> {
        use std::time::Duration;
        use tokio::time::timeout;
        /// Maximum time to wait for routes to come up. The expected mean time is low (~200 ms), but
        /// we add some additional margin to give some slack to slower hardware primarily.
        const WAIT_FOR_ROUTES_TIMEOUT: Duration = Duration::from_secs(2);

        let (result_tx, result_rx) = oneshot::channel();
        self.tx
            .unbounded_send(RouteManagerCommand::WaitForRoutes(result_tx, expect_routes))
            .map_err(|_| Error::RouteManagerDown)?;

        timeout(WAIT_FOR_ROUTES_TIMEOUT, result_rx)
            .await
            .map_err(|_error| Error::PlatformError(imp::Error::RoutesTimedOut))?
            .map_err(|_| Error::ManagerChannelDown)
    }

    /// Removes all routes previously applied in [`RouteManagerHandle::add_routes`].
    #[cfg(not(target_os = "android"))]
    pub fn clear_routes(&self) -> Result<(), Error> {
        self.tx
            .unbounded_send(RouteManagerCommand::ClearRoutes)
            .map_err(|_| Error::RouteManagerDown)
    }

    /// (Android) This is a noop since we don't directly control the routes on Android.
    #[cfg(target_os = "android")]
    pub fn clear_routes(&self) -> Result<(), Error> {
        Ok(())
    }

    /// (Android) Clear the cached routes
    #[cfg(target_os = "android")]
    pub async fn clear_android_routes(&self) -> Result<(), Error> {
        let (result_tx, result_rx) = oneshot::channel();
        self.tx
            .unbounded_send(RouteManagerCommand::ClearRoutes(result_tx))
            .map_err(|_| Error::RouteManagerDown)?;
        let _ = result_rx.await;
        Ok(())
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
    pub async fn get_default_routes(
        &self,
    ) -> Result<(Option<DefaultRoute>, Option<DefaultRoute>), Error> {
        let (response_tx, response_rx) = oneshot::channel();
        self.tx
            .unbounded_send(RouteManagerCommand::GetDefaultRoutes(response_tx))
            .map_err(|_| Error::RouteManagerDown)?;
        response_rx.await.map_err(|_| Error::ManagerChannelDown)
    }

    /// Listen for interface changes.
    #[cfg(target_os = "macos")]
    pub async fn interface_change_listener(
        &self,
    ) -> Result<impl Stream<Item = InterfaceEvent>, Error> {
        let (response_tx, response_rx) = oneshot::channel();
        self.tx
            .unbounded_send(RouteManagerCommand::NewInterfaceChangeListener(response_tx))
            .map_err(|_| Error::RouteManagerDown)?;
        response_rx.await.map_err(|_| Error::ManagerChannelDown)
    }

    /// Get default gateway
    #[cfg(target_os = "macos")]
    pub async fn get_default_gateway(&self) -> Result<(Option<Gateway>, Option<Gateway>), Error> {
        let (response_tx, response_rx) = oneshot::channel();
        self.tx
            .unbounded_send(RouteManagerCommand::GetDefaultGateway(response_tx))
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
    pub async fn change_listener(
        &self,
    ) -> Result<impl Stream<Item = CallbackMessage> + use<>, Error> {
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

#![cfg_attr(target_os = "android", allow(dead_code))]
#![cfg_attr(target_os = "windows", allow(dead_code))]
// TODO: remove the allow(dead_code) for android once it's up to scratch.
use super::RequiredRoute;
use futures01::{
    sync::{
        mpsc::{unbounded, UnboundedSender},
        oneshot,
    },
    Future,
};
use std::{collections::HashSet, sync::mpsc::sync_channel};
use talpid_types::ErrorExt;

#[cfg(target_os = "linux")]
use std::net::IpAddr;

#[cfg(target_os = "macos")]
#[path = "macos.rs"]
mod imp;

#[cfg(target_os = "linux")]
#[path = "linux.rs"]
mod imp;

#[cfg(target_os = "android")]
#[path = "android.rs"]
mod imp;

pub use imp::Error as PlatformError;

/// Errors that can be encountered whilst initializing RouteManager
#[derive(err_derive::Error, Debug)]
pub enum Error {
    /// Routing manager thread panicked before starting routing manager
    #[error(display = "Routing manager thread panicked before starting routing manager")]
    RoutingManagerThreadPanic,
    /// Platform specific error occured
    #[error(display = "Internal route manager error")]
    PlatformError(#[error(source)] imp::Error),
    /// Failed to spawn route manager future
    #[error(display = "Failed to spawn route manager on the provided executor")]
    FailedToSpawnManager,
    /// Attempt to use route manager that has been dropped
    #[error(display = "Cannot send message to route manager since it is down")]
    RouteManagerDown,
}

#[derive(Debug)]
pub enum RouteManagerCommand {
    AddRoutes(
        HashSet<RequiredRoute>,
        oneshot::Sender<Result<(), PlatformError>>,
    ),
    ClearRoutes,
    Shutdown(oneshot::Sender<()>),
    #[cfg(target_os = "linux")]
    EnableExclusionsRoutes(oneshot::Sender<Result<(), PlatformError>>),
    #[cfg(target_os = "linux")]
    DisableExclusionsRoutes,
    #[cfg(target_os = "linux")]
    RouteExclusionsDns(
        String,
        Vec<IpAddr>,
        oneshot::Sender<Result<(), PlatformError>>,
    ),
}

/// RouteManager applies a set of routes to the route table.
/// If a destination has to be routed through the default node,
/// the route will be adjusted dynamically when the default route changes.
pub struct RouteManager {
    manage_tx: Option<UnboundedSender<RouteManagerCommand>>,
}

impl RouteManager {
    /// Constructs a RouteManager and applies the required routes.
    /// Takes a set of network destinations and network nodes as an argument, and applies said
    /// routes.
    pub fn new(required_routes: HashSet<RequiredRoute>) -> Result<Self, Error> {
        let (manage_tx, manage_rx) = unbounded();
        let (start_tx, start_rx) = sync_channel(1);

        std::thread::spawn(
            move || match imp::RouteManagerImpl::new(required_routes, manage_rx) {
                Ok(route_manager) => {
                    let _ = start_tx.send(Ok(()));
                    if let Err(e) = route_manager.wait() {
                        log::error!("Route manager failed - {}", e);
                    }
                }
                Err(e) => {
                    let _ = start_tx.send(Err(Error::PlatformError(e)));
                }
            },
        );
        match start_rx.recv() {
            Ok(Ok(())) => Ok(Self {
                manage_tx: Some(manage_tx),
            }),
            Ok(Err(e)) => Err(e),
            Err(_) => Err(Error::RoutingManagerThreadPanic),
        }
    }

    /// Stops RouteManager and removes all of the applied routes.
    pub fn stop(&mut self) {
        if let Some(tx) = self.manage_tx.take() {
            let (wait_tx, wait_rx) = oneshot::channel();

            if tx
                .unbounded_send(RouteManagerCommand::Shutdown(wait_tx))
                .is_err()
            {
                log::error!("RouteManager already down!");
                return;
            }

            if wait_rx.wait().is_err() {
                log::error!("RouteManager paniced while shutting down");
            }
        }
    }

    /// Applies the given routes until [`RouteManager::stop`] is called.
    pub fn add_routes(&mut self, routes: HashSet<RequiredRoute>) -> Result<(), Error> {
        if let Some(tx) = &self.manage_tx {
            let (result_tx, result_rx) = oneshot::channel();
            if tx
                .unbounded_send(RouteManagerCommand::AddRoutes(routes, result_tx))
                .is_err()
            {
                return Err(Error::RouteManagerDown);
            }

            match result_rx.wait() {
                Ok(result) => result.map_err(Error::PlatformError),
                Err(error) => {
                    log::trace!(
                        "{}",
                        error.display_chain_with_msg("oneshot channel is closed")
                    );
                    Ok(())
                }
            }
        } else {
            Err(Error::RouteManagerDown)
        }
    }

    /// Removes all routes previously applied in [`RouteManager::new`] or
    /// [`RouteManager::add_routes`].
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

    /// Route PID-associated packets through the physical interface.
    #[cfg(target_os = "linux")]
    pub fn enable_exclusions_routes(&self) -> Result<(), Error> {
        if let Some(tx) = &self.manage_tx {
            let (result_tx, result_rx) = oneshot::channel();
            if tx
                .unbounded_send(RouteManagerCommand::EnableExclusionsRoutes(result_tx))
                .is_err()
            {
                return Err(Error::RouteManagerDown);
            }

            match result_rx.wait() {
                Ok(result) => result.map_err(Error::PlatformError),
                Err(error) => {
                    log::trace!("{}", error.display_chain_with_msg("channel is closed"));
                    Ok(())
                }
            }
        } else {
            Err(Error::RouteManagerDown)
        }
    }

    /// Stop routing PID-associated packets through the physical interface.
    #[cfg(target_os = "linux")]
    pub fn disable_exclusions_routes(&self) -> Result<(), Error> {
        if let Some(tx) = &self.manage_tx {
            if tx
                .unbounded_send(RouteManagerCommand::DisableExclusionsRoutes)
                .is_err()
            {
                return Err(Error::RouteManagerDown);
            }
            Ok(())
        } else {
            Err(Error::RouteManagerDown)
        }
    }

    /// Route DNS requests through the tunnel interface.
    #[cfg(target_os = "linux")]
    pub fn route_exclusions_dns(
        &mut self,
        tunnel_alias: &str,
        dns_servers: &[IpAddr],
    ) -> Result<(), Error> {
        if let Some(tx) = &self.manage_tx {
            let (result_tx, result_rx) = oneshot::channel();
            if tx
                .unbounded_send(RouteManagerCommand::RouteExclusionsDns(
                    tunnel_alias.to_string(),
                    dns_servers.to_vec(),
                    result_tx,
                ))
                .is_err()
            {
                return Err(Error::RouteManagerDown);
            }

            match result_rx.wait() {
                Ok(result) => result.map_err(Error::PlatformError),
                Err(error) => {
                    log::trace!("{}", error.display_chain_with_msg("channel is closed"));
                    Ok(())
                }
            }
        } else {
            Err(Error::RouteManagerDown)
        }
    }
}

impl Drop for RouteManager {
    fn drop(&mut self) {
        self.stop();
    }
}

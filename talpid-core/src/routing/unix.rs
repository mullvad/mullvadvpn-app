#![cfg_attr(target_os = "android", allow(dead_code))]
#![cfg_attr(target_os = "windows", allow(dead_code))]
// TODO: remove the allow(dead_code) for android once it's up to scratch.
use super::RequiredRoute;

use futures::channel::{
    mpsc::{self, UnboundedSender},
    oneshot,
};
use std::{collections::HashSet, io};
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
    /// Failed to spawn route manager runtime
    #[error(display = "Failed to spawn route manager runtime")]
    FailedToSpawnRuntime(#[error(source)] io::Error),
    /// Attempt to use route manager that has been dropped
    #[error(display = "Cannot send message to route manager since it is down")]
    RouteManagerDown,
}

/// Commands for the underlying route manager object.
#[derive(Debug)]
pub enum RouteManagerCommand {
    /// Adds required routes
    AddRoutes(
        HashSet<RequiredRoute>,
        oneshot::Sender<Result<(), PlatformError>>,
    ),
    /// Clears required routes
    ClearRoutes,
    /// Shuts down the route manager
    Shutdown(oneshot::Sender<()>),
    /// Routes traffic with correct fwmark using the exclusions table
    #[cfg(target_os = "linux")]
    EnableExclusionsRoutes(oneshot::Sender<Result<(), PlatformError>>),
    /// Removes rule for routing marked traffic differently.
    #[cfg(target_os = "linux")]
    DisableExclusionsRoutes,
    /// Adds link to ignore in the exclusions table.
    #[cfg(target_os = "linux")]
    SetTunnelLink(String, oneshot::Sender<()>),
    /// Adds exclusions table route for sending DNS requests via the tunnel.
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
    runtime: tokio::runtime::Runtime,
}

impl RouteManager {
    /// Constructs a RouteManager and applies the required routes.
    /// Takes a set of network destinations and network nodes as an argument, and applies said
    /// routes.
    pub fn new(required_routes: HashSet<RequiredRoute>) -> Result<Self, Error> {
        let (manage_tx, manage_rx) = mpsc::unbounded();
        let mut runtime = tokio::runtime::Builder::new()
            .threaded_scheduler()
            .core_threads(1)
            .max_threads(1)
            .enable_all()
            .build()?;
        let manager = runtime.block_on(imp::RouteManagerImpl::new(required_routes))?;
        runtime.handle().spawn(manager.run(manage_rx));

        Ok(Self {
            runtime,
            manage_tx: Some(manage_tx),
        })
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

            if self.runtime.block_on(wait_rx).is_err() {
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

            match self.runtime.block_on(result_rx) {
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
    pub fn enable_exclusions_routes(&mut self) -> Result<(), Error> {
        if let Some(tx) = &self.manage_tx {
            let (result_tx, result_rx) = oneshot::channel();
            if tx
                .unbounded_send(RouteManagerCommand::EnableExclusionsRoutes(result_tx))
                .is_err()
            {
                return Err(Error::RouteManagerDown);
            }

            match self.runtime.block_on(result_rx) {
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

    /// Set the link to be ignored by the exclusions routing table.
    #[cfg(target_os = "linux")]
    pub fn set_tunnel_link(&mut self, tunnel_alias: &str) -> Result<(), Error> {
        if let Some(tx) = &self.manage_tx {
            let (result_tx, result_rx) = oneshot::channel();
            if tx
                .unbounded_send(RouteManagerCommand::SetTunnelLink(
                    tunnel_alias.to_string(),
                    result_tx,
                ))
                .is_err()
            {
                return Err(Error::RouteManagerDown);
            }
            match self.runtime.block_on(result_rx) {
                Ok(()) => Ok(()),
                Err(error) => {
                    log::trace!("{}", error.display_chain_with_msg("channel is closed"));
                    Ok(())
                }
            }
        } else {
            Err(Error::RouteManagerDown)
        }
    }

    /// Retrieve a sender directly to the command channel.
    #[cfg(target_os = "linux")]
    pub fn channel(&self) -> Result<UnboundedSender<RouteManagerCommand>, Error> {
        if let Some(tx) = &self.manage_tx {
            Ok(tx.clone())
        } else {
            Err(Error::RouteManagerDown)
        }
    }

    /// Exposes runtime handle
    #[cfg(target_os = "linux")]
    pub fn runtime_handle(&self) -> tokio::runtime::Handle {
        self.runtime.handle().clone()
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

            match self.runtime.block_on(result_rx) {
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

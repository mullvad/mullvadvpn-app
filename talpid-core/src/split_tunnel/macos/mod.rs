use std::path::PathBuf;
use talpid_routing::RouteManagerHandle;

use self::process::ExclusionStatus;

#[allow(non_camel_case_types)]
mod bindings;
mod bpf;
mod default;
mod process;
mod tun;

pub use tun::VpnInterface;

/// Errors caused by split tunneling
#[derive(thiserror::Error, Debug)]
pub enum Error {
    /// Process monitor error
    #[error("Process monitor error")]
    Process(#[from] process::Error),
    /// Failed to initialize split tunnel
    #[error("Failed to initialize split tunnel")]
    InitializeTunnel(#[from] tun::Error),
    /// Default interface unavailable
    #[error("Default interface unavailable")]
    Default(#[from] default::Error),
    /// Split tunnel is unavailable
    #[error("Split tunnel is unavailable")]
    Unavailable,
}

impl Error {
    /// Return whether the error is due to a missing default route
    pub fn is_offline(&self) -> bool {
        matches!(self, Error::Default(_))
    }
}

/// Handle for interacting with the split tunnel module
pub struct Handle {
    state: State,
}

enum State {
    NoExclusions {
        route_manager: RouteManagerHandle,
        vpn_interface: Option<VpnInterface>,
    },
    HasProcessMonitor {
        route_manager: RouteManagerHandle,
        process: process::ProcessMonitorHandle,
    },
    Initialized {
        route_manager: RouteManagerHandle,
        process: process::ProcessMonitorHandle,
        tun_handle: tun::SplitTunnelHandle,
    },
    /// State entered when anything at all fails. Users can force a transition out of this state
    /// by disabling/clearing the paths to use.
    Failed {
        route_manager: RouteManagerHandle,
        vpn_interface: Option<VpnInterface>,
    },
}

impl Handle {
    /// Create split tunneling handle
    pub async fn new(route_manager: RouteManagerHandle) -> Handle {
        Self {
            state: State::NoExclusions {
                route_manager,
                vpn_interface: None,
            },
        }
    }

    /// Shut down split tunnel
    pub async fn shutdown(self) {
        match self.state {
            State::HasProcessMonitor { mut process, .. } => {
                process.shutdown().await;
            }
            State::Initialized {
                mut process,
                tun_handle,
                ..
            } => {
                if let Err(error) = tun_handle.shutdown().await {
                    log::error!("Failed to stop split tunnel: {error}");
                }
                process.shutdown().await;
            }
            State::Failed { .. } | State::NoExclusions { .. } => (),
        }
    }

    /// Return name of split tunnel interface
    pub fn interface(&self) -> Option<&str> {
        match &self.state {
            State::Initialized { tun_handle, .. } => Some(tun_handle.name()),
            _ => None,
        }
    }

    /// Set paths to exclude
    pub async fn set_exclude_paths(&mut self, paths: Vec<PathBuf>) -> Result<(), Error> {
        match &mut self.state {
            // If there are currently no paths and no process monitor, initialize it
            State::NoExclusions {
                route_manager,
                vpn_interface,
            } if !paths.is_empty() => {
                log::debug!("Initializing process monitor");

                let route_manager = route_manager.clone();
                let vpn_interface = vpn_interface.clone();
                let prev_state = std::mem::replace(
                    &mut self.state,
                    State::Failed {
                        route_manager,
                        vpn_interface,
                    },
                );
                let State::NoExclusions {
                    route_manager,
                    vpn_interface,
                } = prev_state
                else {
                    unreachable!("unexpected state")
                };
                let process = process::ProcessMonitor::spawn().await?;
                process.states().exclude_paths(paths);

                self.state = State::HasProcessMonitor {
                    route_manager,
                    process,
                };

                self.set_tunnel(vpn_interface).await
            }
            // If 'paths' is empty, do nothing
            State::NoExclusions { .. } => Ok(()),
            // If split tunneling is already initialized, or only the process monitor is, update the paths only
            State::Initialized { process, .. } | State::HasProcessMonitor { process, .. } => {
                process.states().exclude_paths(paths);
                Ok(())
            }
            // If 'paths' is empty, transition out of the failed state
            State::Failed {
                route_manager,
                vpn_interface,
            } if paths.is_empty() => {
                log::debug!("Transitioning out of split tunnel error state");

                self.state = State::NoExclusions {
                    route_manager: route_manager.clone(),
                    vpn_interface: vpn_interface.clone(),
                };
                return Ok(());
            }
            // Otherwise, remain in the failed state
            State::Failed { .. } => Err(Error::Unavailable),
        }
    }

    /// Set VPN tunnel interface
    pub async fn set_tunnel(
        &mut self,
        new_vpn_interface: Option<VpnInterface>,
    ) -> Result<(), Error> {
        match &mut self.state {
            // If split tunneling is already initialized, just update the interfaces
            State::Initialized { route_manager, .. } => {
                // Try to update the default interface first
                // If this fails, remain in the current state and just fail
                let default_interface = default::get_default_interface(route_manager).await?;

                // Update the VPN interface
                let route_manager = route_manager.clone();
                let prev_state = std::mem::replace(
                    &mut self.state,
                    State::Failed {
                        route_manager,
                        vpn_interface: new_vpn_interface.clone(),
                    },
                );
                let State::Initialized {
                    route_manager,
                    mut process,
                    tun_handle,
                } = prev_state
                else {
                    unreachable!("unexpected state")
                };

                log::debug!("Updating split tunnel device");

                match tun_handle
                    .set_interfaces(default_interface, new_vpn_interface)
                    .await
                {
                    Ok(tun_handle) => {
                        self.state = State::Initialized {
                            route_manager,
                            process,
                            tun_handle,
                        };
                        Ok(())
                    }
                    Err(error) => {
                        process.shutdown().await;
                        Err(error.into())
                    }
                }
            }
            // If there is a process monitor, initialize split tunneling
            State::HasProcessMonitor { route_manager, .. } if new_vpn_interface.is_some() => {
                // Try to update the default interface first
                // If this fails, remain in the current state and just fail
                let default_interface = default::get_default_interface(route_manager).await?;

                let route_manager = route_manager.clone();
                let State::HasProcessMonitor {
                    route_manager,
                    mut process,
                } = std::mem::replace(
                    &mut self.state,
                    State::Failed {
                        route_manager,
                        vpn_interface: new_vpn_interface.clone(),
                    },
                )
                else {
                    unreachable!("unexpected state");
                };

                log::debug!("Initializing split tunnel device");

                let states = process.states().clone();
                let result =
                    tun::create_split_tunnel(default_interface, new_vpn_interface, move |packet| {
                        match states.get_process_status(packet.header.pth_pid as u32) {
                            ExclusionStatus::Excluded => tun::RoutingDecision::DefaultInterface,
                            ExclusionStatus::Included => tun::RoutingDecision::VpnTunnel,
                            ExclusionStatus::Unknown => {
                                // TODO: Delay decision until next exec
                                tun::RoutingDecision::Drop
                            }
                        }
                    })
                    .await;

                match result {
                    Ok(tun_handle) => {
                        self.state = State::Initialized {
                            route_manager,
                            process,
                            tun_handle,
                        };
                        Ok(())
                    }
                    Err(error) => {
                        process.shutdown().await;
                        Err(error.into())
                    }
                }
            }
            // No-op there's a process monitor but we didn't get a VPN interface
            State::HasProcessMonitor { .. } => Ok(()),
            // If there are no paths to exclude, or split tunneling failed, remain in the current state
            State::NoExclusions { vpn_interface, .. } | State::Failed { vpn_interface, .. } => {
                *vpn_interface = new_vpn_interface;
                Ok(())
            }
        }
    }
}

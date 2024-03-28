use std::collections::HashSet;
use std::path::PathBuf;
use std::sync::Weak;
use talpid_routing::RouteManagerHandle;
use talpid_types::tunnel::ErrorStateCause;
use talpid_types::ErrorExt;
use tokio::sync::{mpsc, oneshot};

use self::process::ExclusionStatus;

#[allow(non_camel_case_types)]
mod bindings;
mod bpf;
mod default;
mod process;
mod tun;

use crate::tunnel_state_machine::TunnelCommand;
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

/// Split tunneling actor
pub struct SplitTunnel {
    state: State,
    tunnel_tx: Weak<futures::channel::mpsc::UnboundedSender<TunnelCommand>>,
    rx: mpsc::UnboundedReceiver<Message>,
}

enum Message {
    GetInterface {
        result_tx: oneshot::Sender<Option<String>>,
    },
    Shutdown {
        result_tx: oneshot::Sender<()>,
    },
    SetExcludePaths {
        result_tx: oneshot::Sender<Result<(), Error>>,
        paths: HashSet<PathBuf>,
    },
    SetTunnel {
        result_tx: oneshot::Sender<Result<(), Error>>,
        new_vpn_interface: Option<VpnInterface>,
    },
}

/// Handle for interacting with the split tunnel module
#[derive(Clone)]
pub struct Handle {
    tx: mpsc::UnboundedSender<Message>,
}

enum State {
    /// The initial state: no paths have been provided
    NoExclusions {
        route_manager: RouteManagerHandle,
        vpn_interface: Option<VpnInterface>,
    },
    /// There is a process monitor (and paths) but no split tunnel utun yet
    ProcessMonitorOnly {
        route_manager: RouteManagerHandle,
        process: process::ProcessMonitorHandle,
    },
    /// There is a split tunnel utun as well as paths to exclude
    Initialized {
        route_manager: RouteManagerHandle,
        process: process::ProcessMonitorHandle,
        tun_handle: tun::SplitTunnelHandle,
        vpn_interface: Option<VpnInterface>,
    },
    /// State entered when anything at all fails. Users can force a transition out of this state
    /// by disabling/clearing the paths to use.
    Failed {
        route_manager: RouteManagerHandle,
        vpn_interface: Option<VpnInterface>,
    },
}

impl State {
    fn process_monitor(&mut self) -> Option<&mut process::ProcessMonitorHandle> {
        match self {
            State::ProcessMonitorOnly { process, .. } | State::Initialized { process, .. } => {
                Some(process)
            }
            _ => None,
        }
    }

    fn route_manager(&self) -> &RouteManagerHandle {
        match self {
            State::NoExclusions { route_manager, .. }
            | State::ProcessMonitorOnly { route_manager, .. }
            | State::Initialized { route_manager, .. }
            | State::Failed { route_manager, .. } => route_manager,
        }
    }
}

impl Handle {
    /// Shut down split tunnel
    pub async fn shutdown(&self) {
        let (result_tx, result_rx) = oneshot::channel();
        let _ = self.tx.send(Message::Shutdown { result_tx });
        if let Err(error) = result_rx.await {
            log::error!(
                "{}",
                error.display_chain_with_msg("Split tunnel is already down")
            );
        }
    }

    /// Return name of split tunnel interface
    pub async fn interface(&self) -> Option<String> {
        let (result_tx, result_rx) = oneshot::channel();
        let _ = self.tx.send(Message::GetInterface { result_tx });
        result_rx.await.ok()?
    }

    /// Set paths to exclude
    pub async fn set_exclude_paths(&self, paths: HashSet<PathBuf>) -> Result<(), Error> {
        let (result_tx, result_rx) = oneshot::channel();
        let _ = self.tx.send(Message::SetExcludePaths { result_tx, paths });
        result_rx.await.map_err(|_| Error::Unavailable)?
    }

    /// Set VPN tunnel interface
    pub async fn set_tunnel(&self, new_vpn_interface: Option<VpnInterface>) -> Result<(), Error> {
        let (result_tx, result_rx) = oneshot::channel();
        let _ = self.tx.send(Message::SetTunnel {
            result_tx,
            new_vpn_interface,
        });
        result_rx.await.map_err(|_| Error::Unavailable)?
    }
}

impl SplitTunnel {
    /// Initialize split tunneling
    pub async fn spawn(
        tunnel_tx: Weak<futures::channel::mpsc::UnboundedSender<TunnelCommand>>,
        route_manager: RouteManagerHandle,
    ) -> Handle {
        let (tx, rx) = mpsc::unbounded_channel();
        let split_tunnel = Self {
            state: State::NoExclusions {
                route_manager,
                vpn_interface: None,
            },
            tunnel_tx,
            rx,
        };

        tokio::spawn(Self::run(split_tunnel));

        Handle { tx }
    }

    async fn run(mut self) {
        loop {
            let process_monitor_stopped = async {
                match self.state.process_monitor() {
                    Some(process) => process.wait().await,
                    None => futures::future::pending().await,
                }
            };

            tokio::select! {
                // Handle process monitor being stopped
                result = process_monitor_stopped => {
                    match result {
                        Ok(()) => log::error!("Process monitor stopped unexpectedly with no error"),
                        Err(error) => {
                            log::error!("{}", error.display_chain_with_msg("Process monitor stopped unexpectedly"));
                        }
                    }

                    let route_manager = self.state.route_manager();

                    match &self.state {
                        // Enter the error state if split tunneling is active. Otherwise, we might make incorrect
                        // decisions for new processes
                        State::Initialized { vpn_interface, .. } if vpn_interface.is_some() => {
                            if let Some(tunnel_tx) = self.tunnel_tx.upgrade() {
                                let _ = tunnel_tx.unbounded_send(TunnelCommand::Block(ErrorStateCause::SplitTunnelError));
                            }
                        }
                        _ => (),
                    }

                    self.state = State::Failed { route_manager: route_manager.clone(), vpn_interface: None };
                }

                // Handle messages
                message = self.rx.recv() => {
                    let Some(message) = message else {
                        // Shut down split tunnel
                        break
                    };

                    match message {
                        Message::GetInterface {
                            result_tx,
                        } => {
                            let _ = result_tx.send(self.interface().map(str::to_owned));
                        }
                        Message::Shutdown {
                            result_tx,
                        } => {
                            // Shut down; early exit
                            let _ = result_tx.send(self.shutdown().await);
                            return;
                        }
                        Message::SetExcludePaths {
                            result_tx,
                            paths,
                        } => {
                            let _ = result_tx.send(self.set_exclude_paths(paths).await);
                        }
                        Message::SetTunnel {
                            result_tx,
                            new_vpn_interface,
                        } => {
                            let _ = result_tx.send(self.set_tunnel(new_vpn_interface).await);
                        }
                    }
                }
            }
        }

        self.shutdown().await;
    }

    /// Shut down split tunnel
    async fn shutdown(self) {
        match self.state {
            State::ProcessMonitorOnly { mut process, .. } => {
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
    fn interface(&self) -> Option<&str> {
        match &self.state {
            State::Initialized { tun_handle, .. } => Some(tun_handle.name()),
            _ => None,
        }
    }

    /// Set paths to exclude
    async fn set_exclude_paths(&mut self, paths: HashSet<PathBuf>) -> Result<(), Error> {
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

                self.state = State::ProcessMonitorOnly {
                    route_manager,
                    process,
                };

                self.set_tunnel(vpn_interface).await
            }
            // If 'paths' is empty, do nothing
            State::NoExclusions { .. } => Ok(()),
            // If split tunneling is already initialized, or only the process monitor is, update the paths only
            State::Initialized { process, .. } | State::ProcessMonitorOnly { process, .. } => {
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
    async fn set_tunnel(&mut self, new_vpn_interface: Option<VpnInterface>) -> Result<(), Error> {
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
                    vpn_interface: _,
                } = prev_state
                else {
                    unreachable!("unexpected state")
                };

                log::debug!("Updating split tunnel device");

                match tun_handle
                    .set_interfaces(default_interface, new_vpn_interface.clone())
                    .await
                {
                    Ok(tun_handle) => {
                        self.state = State::Initialized {
                            route_manager,
                            process,
                            tun_handle,
                            vpn_interface: new_vpn_interface,
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
            State::ProcessMonitorOnly { route_manager, .. } if new_vpn_interface.is_some() => {
                // Try to update the default interface first
                // If this fails, remain in the current state and just fail
                let default_interface = default::get_default_interface(route_manager).await?;

                let route_manager = route_manager.clone();
                let State::ProcessMonitorOnly {
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
                let result = tun::create_split_tunnel(
                    default_interface,
                    new_vpn_interface.clone(),
                    move |packet| {
                        match states.get_process_status(packet.header.pth_pid as u32) {
                            ExclusionStatus::Excluded => tun::RoutingDecision::DefaultInterface,
                            ExclusionStatus::Included => tun::RoutingDecision::VpnTunnel,
                            ExclusionStatus::Unknown => {
                                // TODO: Delay decision until next exec
                                tun::RoutingDecision::Drop
                            }
                        }
                    },
                )
                .await;

                match result {
                    Ok(tun_handle) => {
                        self.state = State::Initialized {
                            route_manager,
                            process,
                            tun_handle,
                            vpn_interface: new_vpn_interface,
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
            State::ProcessMonitorOnly { .. } => Ok(()),
            // If there are no paths to exclude, remain in the current state
            State::NoExclusions { vpn_interface, .. } => {
                *vpn_interface = new_vpn_interface;
                Ok(())
            }
            // Remain in the failed state and return error if VPN is up
            State::Failed { vpn_interface, .. } => {
                *vpn_interface = new_vpn_interface;
                if vpn_interface.is_some() {
                    Err(Error::Unavailable)
                } else {
                    Ok(())
                }
            }
        }
    }
}

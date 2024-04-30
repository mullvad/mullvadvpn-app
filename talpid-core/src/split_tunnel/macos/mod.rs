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
    shutdown_tx: Option<oneshot::Sender<()>>,
}

enum Message {
    /// Return the name of the split tunnel interface
    GetInterface {
        result_tx: oneshot::Sender<Option<String>>,
    },
    /// Shut down split tunnel service
    Shutdown { result_tx: oneshot::Sender<()> },
    /// Set paths to exclude from the VPN tunnel
    SetExcludePaths {
        result_tx: oneshot::Sender<Result<(), Error>>,
        paths: HashSet<PathBuf>,
    },
    /// Update VPN tunnel interface
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
    pub fn spawn(
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
            shutdown_tx: None,
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
                    self.handle_process_monitor_shutdown(result);
                }

                // Handle messages
                message = self.rx.recv() => {
                    let Some(message) = message else {
                        break
                    };
                    if !self.handle_message(message).await {
                        break;
                    }
                }
            }
        }

        self.shutdown().await;

        if let Some(tx) = self.shutdown_tx.take() {
            let _ = tx.send(());
        }
    }

    /// Handle process monitor unexpectedly stopping
    fn handle_process_monitor_shutdown(&mut self, result: Result<(), process::Error>) {
        match result {
            Ok(()) => log::error!("Process monitor stopped unexpectedly with no error"),
            Err(error) => {
                log::error!(
                    "{}",
                    error.display_chain_with_msg("Process monitor stopped unexpectedly")
                );
            }
        }

        // Enter the error state if split tunneling is active. Otherwise, we might make incorrect
        // decisions for new processes
        if self.state.active() {
            if let Some(tunnel_tx) = self.tunnel_tx.upgrade() {
                let _ = tunnel_tx
                    .unbounded_send(TunnelCommand::Block(ErrorStateCause::SplitTunnelError));
            }
        }

        self.state.fail();
    }

    /// Handle an incoming message
    /// Return whether the actor should continue running
    async fn handle_message(&mut self, message: Message) -> bool {
        match message {
            Message::GetInterface { result_tx } => {
                let _ = result_tx.send(self.interface().map(str::to_owned));
            }
            Message::Shutdown { result_tx } => {
                self.shutdown_tx = Some(result_tx);
                return false;
            }
            Message::SetExcludePaths { result_tx, paths } => {
                let _ = result_tx.send(self.state.set_exclude_paths(paths).await);
            }
            Message::SetTunnel {
                result_tx,
                new_vpn_interface,
            } => {
                let _ = result_tx.send(self.state.set_tunnel(new_vpn_interface).await);
            }
        }
        true
    }

    /// Shut down split tunnel
    async fn shutdown(&mut self) {
        match self.state.fail() {
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

    fn vpn_interface(&self) -> Option<&VpnInterface> {
        match self {
            State::NoExclusions { vpn_interface, .. }
            | State::Initialized { vpn_interface, .. }
            | State::Failed { vpn_interface, .. } => vpn_interface.as_ref(),
            State::ProcessMonitorOnly { .. } => None,
        }
    }

    /// Take `self`, leaving a failed state in its place. The original value is returned
    fn fail(&mut self) -> Self {
        std::mem::replace(
            self,
            State::Failed {
                route_manager: self.route_manager().clone(),
                vpn_interface: self.vpn_interface().cloned(),
            },
        )
    }

    /// Return whether split tunneling is currently engaged. That is, there's both a process monitor
    /// and a VPN tunnel present
    fn active(&self) -> bool {
        matches!(self, State::Initialized { vpn_interface, .. } if vpn_interface.is_some())
    }

    /// Set paths to exclude. For a non-empty path, this will initialize split tunneling if a tunnel
    /// device is also set.
    async fn set_exclude_paths(&mut self, paths: HashSet<PathBuf>) -> Result<(), Error> {
        let state = self.fail();
        *self = state.set_exclude_paths_inner(paths).await?;
        Ok(())
    }

    async fn set_exclude_paths_inner(mut self, paths: HashSet<PathBuf>) -> Result<Self, Error> {
        match self {
            // If there are currently no paths and no process monitor, initialize it
            State::NoExclusions {
                route_manager,
                vpn_interface,
            } if !paths.is_empty() => {
                log::debug!("Initializing process monitor");

                let process = process::ProcessMonitor::spawn().await?;
                process.states().exclude_paths(paths);

                State::ProcessMonitorOnly {
                    route_manager,
                    process,
                }
                .set_tunnel_inner(vpn_interface)
                .await
            }
            // If 'paths' is empty, do nothing
            State::NoExclusions { .. } => Ok(self),
            // If split tunneling is already initialized, or only the process monitor is, update the paths only
            State::Initialized {
                ref mut process, ..
            }
            | State::ProcessMonitorOnly {
                ref mut process, ..
            } => {
                process.states().exclude_paths(paths);
                Ok(self)
            }
            // If 'paths' is empty, transition out of the failed state
            State::Failed {
                route_manager,
                vpn_interface,
            } if paths.is_empty() => {
                log::debug!("Transitioning out of split tunnel error state");

                Ok(State::NoExclusions {
                    route_manager: route_manager.clone(),
                    vpn_interface: vpn_interface.clone(),
                })
            }
            // Otherwise, remain in the failed state
            State::Failed { .. } => Err(Error::Unavailable),
        }
    }

    /// Update VPN tunnel interface that non-excluded packets are sent on
    async fn set_tunnel(&mut self, new_vpn_interface: Option<VpnInterface>) -> Result<(), Error> {
        let state = self.fail();
        *self = state.set_tunnel_inner(new_vpn_interface).await?;
        Ok(())
    }

    async fn set_tunnel_inner(
        mut self,
        new_vpn_interface: Option<VpnInterface>,
    ) -> Result<Self, Error> {
        match self {
            // If split tunneling is already initialized, just update the interfaces
            State::Initialized {
                route_manager,
                mut process,
                tun_handle,
                vpn_interface: _,
            } => {
                // Try to update the default interface first
                // If this fails, remain in the current state and just fail
                let default_interface = default::get_default_interface(&route_manager).await?;

                log::debug!("Updating split tunnel device");

                match tun_handle
                    .set_interfaces(default_interface, new_vpn_interface.clone())
                    .await
                {
                    Ok(tun_handle) => Ok(State::Initialized {
                        route_manager,
                        process,
                        tun_handle,
                        vpn_interface: new_vpn_interface,
                    }),
                    Err(error) => {
                        process.shutdown().await;
                        Err(error.into())
                    }
                }
            }
            // If there is a process monitor, initialize split tunneling
            State::ProcessMonitorOnly {
                route_manager,
                mut process,
            } if new_vpn_interface.is_some() => {
                // Try to update the default interface first
                // If this fails, remain in the current state and just fail
                let default_interface = default::get_default_interface(&route_manager).await?;

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
                    Ok(tun_handle) => Ok(State::Initialized {
                        route_manager,
                        process,
                        tun_handle,
                        vpn_interface: new_vpn_interface,
                    }),
                    Err(error) => {
                        process.shutdown().await;
                        Err(error.into())
                    }
                }
            }
            // No-op there's a process monitor but we didn't get a VPN interface
            State::ProcessMonitorOnly { .. } => Ok(self),
            // If there are no paths to exclude, remain in the current state
            State::NoExclusions {
                ref mut vpn_interface,
                ..
            } => {
                *vpn_interface = new_vpn_interface;
                Ok(self)
            }
            // Remain in the failed state and return error if VPN is up
            State::Failed {
                ref mut vpn_interface,
                ..
            } => {
                *vpn_interface = new_vpn_interface;
                if vpn_interface.is_some() {
                    Err(Error::Unavailable)
                } else {
                    Ok(self)
                }
            }
        }
    }
}

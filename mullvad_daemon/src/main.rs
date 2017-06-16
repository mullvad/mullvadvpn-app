#[macro_use]
extern crate log;
extern crate env_logger;
#[macro_use]
extern crate error_chain;

extern crate serde;
#[macro_use]
extern crate serde_derive;

extern crate jsonrpc_core;
extern crate jsonrpc_pubsub;
#[macro_use]
extern crate jsonrpc_macros;
extern crate jsonrpc_ws_server;
extern crate uuid;
#[macro_use]
extern crate lazy_static;

extern crate talpid_core;
extern crate talpid_ipc;

mod management_interface;
mod states;
mod rpc_info;
mod shutdown;

use management_interface::{ManagementInterfaceServer, TunnelCommand};
use states::{SecurityState, TargetState};

use std::sync::{Arc, Mutex, mpsc};
use std::thread;

use talpid_core::mpsc::IntoSender;
use talpid_core::net::RemoteAddr;
use talpid_core::tunnel::{self, TunnelEvent, TunnelMonitor};

error_chain!{
    errors {
        /// The client is in the wrong state for the requested operation. Optimally the code should
        /// be written in such a way so such states can't exist.
        InvalidState {
            description("Client is in an invalid state for the requested operation")
        }
        TunnelError(msg: &'static str) {
            description("Error in the tunnel monitor")
            display("Tunnel monitor error: {}", msg)
        }
        ManagementInterfaceError(msg: &'static str) {
            description("Error in the management interface")
            display("Management interface error: {}", msg)
        }
    }
}

lazy_static! {
    // Temporary store of hardcoded remotes.
    static ref REMOTES: [RemoteAddr; 3] = [
        RemoteAddr::new("se5.mullvad.net", 1300),
        RemoteAddr::new("se6.mullvad.net", 1300),
        RemoteAddr::new("se7.mullvad.net", 1300),
    ];
}


pub enum DaemonEvent {
    TunnelEvent(TunnelEvent),
    TunnelExit(tunnel::Result<()>),
    ManagementInterfaceEvent(TunnelCommand),
    ManagementInterfaceExit(talpid_ipc::Result<()>),
    Shutdown,
}

impl From<TunnelEvent> for DaemonEvent {
    fn from(tunnel_event: TunnelEvent) -> Self {
        DaemonEvent::TunnelEvent(tunnel_event)
    }
}

impl From<TunnelCommand> for DaemonEvent {
    fn from(tunnel_command: TunnelCommand) -> Self {
        DaemonEvent::ManagementInterfaceEvent(tunnel_command)
    }
}

/// Represents the internal state of the actual tunnel.
// TODO(linus): Put the tunnel::CloseHandle into this state, so it can't exist when not running.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum TunnelState {
    /// No tunnel is running.
    NotRunning,
    /// The tunnel has been started, but it is not established/functional.
    Down,
    /// The tunnel is up and working.
    Up,
}

impl TunnelState {
    pub fn as_security_state(&self) -> SecurityState {
        match *self {
            TunnelState::Up => SecurityState::Secured,
            _ => SecurityState::Unsecured,
        }
    }
}

struct Exit(bool);

struct Daemon {
    state: TunnelState,
    last_broadcasted_state: SecurityState,
    target_state: TargetState,
    rx: mpsc::Receiver<DaemonEvent>,
    tx: mpsc::Sender<DaemonEvent>,
    tunnel_close_handle: Option<tunnel::CloseHandle>,
    management_interface_broadcaster: management_interface::EventBroadcaster,

    // Just for testing. A cyclic iterator iterating over the hardcoded remotes,
    // picking a new one for each retry.
    remote_iter: std::iter::Cycle<std::iter::Cloned<std::slice::Iter<'static, RemoteAddr>>>,
}

impl Daemon {
    pub fn new() -> Result<Self> {
        let (tx, rx) = mpsc::channel();
        let management_interface_broadcaster = Self::start_management_interface(tx.clone())?;
        Ok(
            Daemon {
                state: TunnelState::NotRunning,
                last_broadcasted_state: SecurityState::Unsecured,
                target_state: TargetState::Unsecured,
                rx,
                tx,
                tunnel_close_handle: None,
                management_interface_broadcaster,
                remote_iter: REMOTES.iter().cloned().cycle(),
            },
        )
    }

    // Starts the management interface and spawns a thread that will process it.
    // Returns a handle that allows notifying all subscribers on events.
    fn start_management_interface(event_tx: mpsc::Sender<DaemonEvent>)
                                  -> Result<management_interface::EventBroadcaster> {
        let multiplex_event_tx = IntoSender::from(event_tx.clone());
        let server = Self::start_management_interface_server(multiplex_event_tx)?;
        let event_broadcaster = server.event_broadcaster();
        Self::spawn_management_interface_wait_thread(server, event_tx);
        Ok(event_broadcaster)
    }

    fn start_management_interface_server(event_tx: IntoSender<TunnelCommand, DaemonEvent>)
                                         -> Result<ManagementInterfaceServer> {
        let server =
            ManagementInterfaceServer::start(event_tx)
                .chain_err(|| ErrorKind::ManagementInterfaceError("Failed to start server"),)?;
        info!(
            "Mullvad management interface listening on {}",
            server.address()
        );
        rpc_info::write(server.address()).chain_err(|| ErrorKind::ManagementInterfaceError(
                "Failed to write RPC address to file"))?;
        Ok(server)
    }

    fn spawn_management_interface_wait_thread(server: ManagementInterfaceServer,
                                              exit_tx: mpsc::Sender<DaemonEvent>) {
        thread::spawn(
            move || {
                let result = server.wait();
                debug!("Mullvad management interface shut down");
                let _ = exit_tx.send(DaemonEvent::ManagementInterfaceExit(result));
            },
        );
    }

    /// Consume the `Daemon` and run the main event loop. Blocks until an error happens or a
    /// shutdown event is received.
    pub fn run(mut self) -> Result<()> {
        while let Ok(event) = self.rx.recv() {
            if let Exit(true) = self.handle_event(event)? {
                break;
            }
        }
        Ok(())
    }

    fn handle_event(&mut self, event: DaemonEvent) -> Result<Exit> {
        use DaemonEvent::*;
        match event {
            TunnelEvent(event) => self.handle_tunnel_event(event),
            TunnelExit(result) => self.handle_tunnel_exit(result),
            ManagementInterfaceEvent(event) => self.handle_management_interface_event(event),
            ManagementInterfaceExit(result) => self.handle_management_interface_exit(result),
            Shutdown => Ok(Exit(true)),
        }
    }

    fn handle_tunnel_event(&mut self, tunnel_event: TunnelEvent) -> Result<Exit> {
        info!("Tunnel event: {:?}", tunnel_event);
        let new_state = match tunnel_event {
            TunnelEvent::Up => TunnelState::Up,
            TunnelEvent::Down => TunnelState::Down,
        };
        self.set_state(new_state);
        Ok(Exit(false))
    }

    fn handle_tunnel_exit(&mut self, result: tunnel::Result<()>) -> Result<Exit> {
        self.tunnel_close_handle = None;
        if let Err(e) = result.chain_err(|| "Tunnel exited in an unexpected way") {
            log_error(&e);
        }
        self.set_state(TunnelState::NotRunning);
        self.apply_target_state()?;
        Ok(Exit(false))
    }

    fn handle_management_interface_event(&mut self, event: TunnelCommand) -> Result<Exit> {
        match event {
            TunnelCommand::SetTargetState(state) => self.set_target_state(state)?,
            TunnelCommand::GetState(tx) => {
                if let Err(_) = tx.send(self.last_broadcasted_state) {
                    warn!("Unable to send current state to management interface client",);
                }
            }
        }
        Ok(Exit(false))
    }

    fn handle_management_interface_exit(&self, result: talpid_ipc::Result<()>) -> Result<Exit> {
        let error = ErrorKind::ManagementInterfaceError("Server exited unexpectedly");
        match result {
            Ok(()) => Err(error.into()),
            Err(e) => Err(e).chain_err(|| error),
        }
    }

    /// Update the state of the client. If it changed, notify the subscribers.
    fn set_state(&mut self, new_state: TunnelState) {
        if new_state != self.state {
            self.state = new_state;
            let new_security_state = self.state.as_security_state();
            if self.last_broadcasted_state != new_security_state {
                self.last_broadcasted_state = new_security_state;
                self.management_interface_broadcaster.notify_new_state(new_security_state);
            }
        }
    }

    /// Set the target state of the client. If it changed trigger the operations needed to progress
    /// towards that state.
    fn set_target_state(&mut self, new_state: TargetState) -> Result<()> {
        if new_state != self.target_state {
            self.target_state = new_state;
            self.apply_target_state()
        } else {
            Ok(())
        }
    }

    fn apply_target_state(&mut self) -> Result<()> {
        match (self.target_state, self.state) {
            (TargetState::Secured, TunnelState::NotRunning) => {
                debug!("Triggering tunnel start");
                if let Err(e) = self.start_tunnel().chain_err(|| "Failed to start tunnel") {
                    log_error(&e);
                    self.target_state = TargetState::Unsecured;
                }
                Ok(())
            }
            (TargetState::Unsecured, TunnelState::Down) |
            (TargetState::Unsecured, TunnelState::Up) => {
                if let Some(close_handle) = self.tunnel_close_handle.take() {
                    debug!("Triggering tunnel stop");
                    // This close operation will block until the tunnel is dead.
                    close_handle
                        .close()
                        .chain_err(|| ErrorKind::TunnelError("Unable to kill tunnel"))
                } else {
                    Ok(())
                }
            }
            (target_state, state) => {
                trace!(
                    "apply_target_state does nothing on TargetState::{:?} TunnelState::{:?}",
                    target_state,
                    state
                );
                Ok(())
            }
        }
    }

    fn start_tunnel(&mut self) -> Result<()> {
        ensure!(
            self.state == TunnelState::NotRunning,
            ErrorKind::InvalidState
        );
        let remote = self.remote_iter.next().unwrap();
        let tunnel_monitor = self.spawn_tunnel_monitor(remote)?;
        self.tunnel_close_handle = Some(tunnel_monitor.close_handle());
        self.spawn_tunnel_monitor_wait_thread(tunnel_monitor);

        self.set_state(TunnelState::Down);
        Ok(())
    }

    fn spawn_tunnel_monitor(&self, remote: RemoteAddr) -> Result<TunnelMonitor> {
        // Must wrap the channel in a Mutex because TunnelMonitor forces the closure to be Sync
        let event_tx = Arc::new(Mutex::new(self.tx.clone()));
        let on_tunnel_event = move |event| {
            let _ = event_tx.lock().unwrap().send(DaemonEvent::TunnelEvent(event));
        };
        TunnelMonitor::new(remote, on_tunnel_event)
            .chain_err(|| ErrorKind::TunnelError("Unable to start tunnel monitor"))
    }

    fn spawn_tunnel_monitor_wait_thread(&self, tunnel_monitor: TunnelMonitor) {
        let error_tx = self.tx.clone();
        thread::spawn(
            move || {
                let result = tunnel_monitor.wait();
                let _ = error_tx.send(DaemonEvent::TunnelExit(result));
                trace!("Tunnel monitor thread exit");
            },
        );
    }

    pub fn shutdown_handle(&self) -> DaemonShutdownHandle {
        DaemonShutdownHandle { tx: self.tx.clone() }
    }
}

struct DaemonShutdownHandle {
    tx: mpsc::Sender<DaemonEvent>,
}

impl DaemonShutdownHandle {
    pub fn shutdown(&self) {
        let _ = self.tx.send(DaemonEvent::Shutdown);
    }
}

impl Drop for Daemon {
    fn drop(self: &mut Daemon) {
        if let Err(e) = rpc_info::remove().chain_err(|| "Unable to clean up rpc address file") {
            log_error(&e);
        }
    }
}


fn log_error<E>(error: &E)
    where E: error_chain::ChainedError
{
    error!("{}", error);
    for e in error.iter().skip(1) {
        error!("Caused by {}", e);
    }
}


quick_main!(run);

fn run() -> Result<()> {
    init_logger()?;

    let daemon = Daemon::new().chain_err(|| "Unable to initialize daemon")?;

    let shutdown_handle = daemon.shutdown_handle();
    shutdown::set_shutdown_signal_handler(move || shutdown_handle.shutdown())
        .chain_err(|| "Unable to attach shutdown signal handler")?;

    daemon.run()?;

    debug!("Mullvad daemon is quitting");
    Ok(())
}

fn init_logger() -> Result<()> {
    env_logger::init().chain_err(|| "Failed to bootstrap logging system")
}

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
use std::io;

use std::sync::{Arc, Mutex, mpsc};
use std::thread;

use talpid_core::mpsc::IntoSender;
use talpid_core::net::{Endpoint, TransportProtocol};
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
        InvalidSettings(msg: &'static str) {
            description("Invalid settings")
            display("Invalid Settings: {}", msg)
        }
    }
}

lazy_static! {
    // Temporary store of hardcoded remotes.
    static ref REMOTES: [Endpoint; 3] = [
        Endpoint::new("se5.mullvad.net", 1300, TransportProtocol::Udp),
        Endpoint::new("se6.mullvad.net", 1300, TransportProtocol::Udp),
        Endpoint::new("se7.mullvad.net", 1300, TransportProtocol::Udp),
    ];
}


pub enum DaemonEvent {
    TunnelEvent(TunnelEvent),
    TunnelExit(tunnel::Result<()>),
    TunnelKill(io::Result<()>),
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
    Connecting,
    /// The tunnel is up and working.
    Connected,
    /// This state is active from when we manually trigger a tunnel kill until the tunnel wait
    /// operation (TunnelExit) returned.
    Exiting,
}

impl TunnelState {
    pub fn as_security_state(&self) -> SecurityState {
        match *self {
            TunnelState::Connected => SecurityState::Secured,
            _ => SecurityState::Unsecured,
        }
    }
}


struct Daemon {
    state: TunnelState,
    last_broadcasted_state: SecurityState,
    target_state: TargetState,
    shutdown: bool,
    rx: mpsc::Receiver<DaemonEvent>,
    tx: mpsc::Sender<DaemonEvent>,
    tunnel_close_handle: Option<tunnel::CloseHandle>,
    management_interface_broadcaster: management_interface::EventBroadcaster,

    // Just for testing. A cyclic iterator iterating over the hardcoded remotes,
    // picking a new one for each retry.
    remote_iter: std::iter::Cycle<std::iter::Cloned<std::slice::Iter<'static, Endpoint>>>,
    // The current account token for now. Should be moved into the settings later.
    account_token: Option<String>,
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
                shutdown: false,
                rx,
                tx,
                tunnel_close_handle: None,
                management_interface_broadcaster,
                remote_iter: REMOTES.iter().cloned().cycle(),
                account_token: None,
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
            self.handle_event(event)?;
            if self.shutdown {
                break;
            }
        }
        Ok(())
    }

    fn handle_event(&mut self, event: DaemonEvent) -> Result<()> {
        use DaemonEvent::*;
        match event {
            TunnelEvent(event) => self.handle_tunnel_event(event),
            TunnelExit(result) => self.handle_tunnel_exit(result),
            TunnelKill(result) => self.handle_tunnel_kill(result),
            ManagementInterfaceEvent(event) => self.handle_management_interface_event(event),
            ManagementInterfaceExit(result) => self.handle_management_interface_exit(result),
            Shutdown => self.handle_shutdown_event(),
        }
    }

    fn handle_tunnel_event(&mut self, tunnel_event: TunnelEvent) -> Result<()> {
        info!("Tunnel event: {:?}", tunnel_event);
        if self.state == TunnelState::Connecting && tunnel_event == TunnelEvent::Up {
            self.set_state(TunnelState::Connected)
        } else if self.state == TunnelState::Connected && tunnel_event == TunnelEvent::Down {
            self.kill_tunnel()
        } else {
            Ok(())
        }
    }

    fn handle_tunnel_exit(&mut self, result: tunnel::Result<()>) -> Result<()> {
        self.tunnel_close_handle = None;
        if let Err(e) = result.chain_err(|| "Tunnel exited in an unexpected way") {
            log_error(&e);
        }
        self.set_state(TunnelState::NotRunning)
    }

    fn handle_tunnel_kill(&mut self, result: io::Result<()>) -> Result<()> {
        result.chain_err(|| "Error while trying to close tunnel")
    }

    fn handle_management_interface_event(&mut self, event: TunnelCommand) -> Result<()> {
        use TunnelCommand::*;
        match event {
            SetTargetState(state) => {
                if !self.shutdown {
                    self.set_target_state(state)?;
                }
            }
            GetState(tx) => {
                if let Err(_) = tx.send(self.last_broadcasted_state) {
                    warn!("Unable to send current state to management interface client",);
                }
            }
            SetAccount(account_token) => self.account_token = account_token,
            GetAccount(tx) => {
                if let Err(_) = tx.send(self.account_token.clone()) {
                    warn!("Unable to send current account to management interface client");
                }
            }
        }
        Ok(())
    }

    fn handle_management_interface_exit(&self, result: talpid_ipc::Result<()>) -> Result<()> {
        let error = ErrorKind::ManagementInterfaceError("Server exited unexpectedly");
        match result {
            Ok(()) => Err(error.into()),
            Err(e) => Err(e).chain_err(|| error),
        }
    }

    fn handle_shutdown_event(&mut self) -> Result<()> {
        self.shutdown = true;
        self.set_target_state(TargetState::Unsecured)
    }

    /// Update the state of the client. If it changed, notify the subscribers and trigger
    /// appropriate actions.
    fn set_state(&mut self, new_state: TunnelState) -> Result<()> {
        if new_state != self.state {
            debug!("State {:?} => {:?}", self.state, new_state);
            self.state = new_state;
            let new_security_state = self.state.as_security_state();
            if self.last_broadcasted_state != new_security_state {
                self.last_broadcasted_state = new_security_state;
                self.management_interface_broadcaster.notify_new_state(new_security_state);
            }
            self.apply_target_state()
        } else {
            // Calling set_state with the same state we already have is an error. Should try to
            // mitigate this possibility completely with a better state machine later.
            Err(ErrorKind::InvalidState.into())
        }
    }

    /// Set the target state of the client. If it changed trigger the operations needed to progress
    /// towards that state.
    fn set_target_state(&mut self, new_state: TargetState) -> Result<()> {
        if new_state != self.target_state {
            debug!("Target state {:?} => {:?}", self.target_state, new_state);
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
                    self.management_interface_broadcaster.notify_error(&e);
                    self.set_target_state(TargetState::Unsecured)?;
                }
                Ok(())
            }
            (TargetState::Unsecured, TunnelState::Connecting) |
            (TargetState::Unsecured, TunnelState::Connected) => self.kill_tunnel(),
            (..) => Ok(()),
        }
    }

    fn start_tunnel(&mut self) -> Result<()> {
        ensure!(
            self.state == TunnelState::NotRunning,
            ErrorKind::InvalidState
        );
        let remote = self.remote_iter.next().unwrap();
        let account_token = self.account_token
            .as_ref()
            .ok_or(ErrorKind::InvalidSettings("No account token"))?
            .clone();
        let tunnel_monitor = self.spawn_tunnel_monitor(remote, &account_token)?;
        self.tunnel_close_handle = Some(tunnel_monitor.close_handle());
        self.spawn_tunnel_monitor_wait_thread(tunnel_monitor);
        self.set_state(TunnelState::Connecting)
    }

    fn spawn_tunnel_monitor(&self, remote: Endpoint, account_token: &str) -> Result<TunnelMonitor> {
        // Must wrap the channel in a Mutex because TunnelMonitor forces the closure to be Sync
        let event_tx = Arc::new(Mutex::new(self.tx.clone()));
        let on_tunnel_event = move |event| {
            let _ = event_tx.lock().unwrap().send(DaemonEvent::TunnelEvent(event));
        };
        TunnelMonitor::new(remote, account_token, on_tunnel_event)
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

    fn kill_tunnel(&mut self) -> Result<()> {
        ensure!(
            self.state == TunnelState::Connecting || self.state == TunnelState::Connected,
            ErrorKind::InvalidState
        );
        self.set_state(TunnelState::Exiting)?;
        let close_handle = self.tunnel_close_handle.take().unwrap();
        let result_tx = self.tx.clone();
        thread::spawn(
            move || {
                let result = close_handle.close();
                let _ = result_tx.send(DaemonEvent::TunnelKill(result));
                trace!("Tunnel kill thread exit");
            },
        );
        Ok(())
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
    let mut msg = error.to_string();
    for e in error.iter().skip(1) {
        msg.push_str("\n\tCaused by: ");
        msg.push_str(&e.to_string()[..]);
    }
    error!("{}", msg);
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

#[macro_use]
extern crate clap;
extern crate chrono;
#[macro_use]
extern crate log;
#[macro_use]
extern crate error_chain;
extern crate fern;

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

extern crate mullvad_types;
extern crate talpid_core;
extern crate talpid_ipc;

mod cli;
mod management_interface;
mod rpc_info;
mod settings;
mod shutdown;

use error_chain::ChainedError;
use jsonrpc_core::futures::sync::oneshot::Sender as OneshotSender;
use management_interface::{ManagementInterfaceServer, TunnelCommand};
use mullvad_types::states::{DaemonState, SecurityState, TargetState};
use std::io;
use std::net::Ipv4Addr;

use std::path::PathBuf;
use std::sync::{Arc, Mutex, mpsc};
use std::thread;

use talpid_core::firewall::{Firewall, FirewallProxy, SecurityPolicy};
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
        FirewallError {
            description("Firewall error")
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
        // se5.mullvad.net
        Endpoint::new(Ipv4Addr::new(193, 138, 219, 240), 1300, TransportProtocol::Udp),
        // se6.mullvad.net
        Endpoint::new(Ipv4Addr::new(193, 138, 219, 241), 1300, TransportProtocol::Udp),
        // se7.mullvad.net
        Endpoint::new(Ipv4Addr::new(185, 65, 132, 104), 1300, TransportProtocol::Udp),
    ];
}

const CRATE_NAME: &str = "mullvadd";


/// All events that can happen in the daemon. Sent from various threads and exposed interfaces.
pub enum DaemonEvent {
    /// An event coming from the tunnel software to indicate a change in state.
    TunnelEvent(TunnelEvent),
    /// Triggered by the thread waiting for the tunnel process. Means the tunnel process exited.
    TunnelExited(tunnel::Result<()>),
    /// Triggered by the thread waiting for a tunnel close operation to complete. Contains the
    /// result of trying to kill the tunnel.
    TunnelKillResult(io::Result<()>),
    /// An event coming from the JSONRPC-2.0 management interface.
    ManagementInterfaceEvent(TunnelCommand),
    /// Triggered if the server hosting the JSONRPC-2.0 management interface dies unexpectedly.
    ManagementInterfaceExited(talpid_ipc::Result<()>),
    /// Daemon shutdown triggered by a signal, ctrl-c or similar.
    TriggerShutdown,
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
    // The tunnel_close_handle must only exist in the Connecting and Connected states!
    tunnel_close_handle: Option<tunnel::CloseHandle>,
    last_broadcasted_state: DaemonState,
    target_state: TargetState,
    shutdown: bool,
    rx: mpsc::Receiver<DaemonEvent>,
    tx: mpsc::Sender<DaemonEvent>,
    management_interface_broadcaster: management_interface::EventBroadcaster,
    settings: settings::Settings,
    firewall: FirewallProxy,
    remote_endpoint: Option<Endpoint>,

    // Just for testing. A cyclic iterator iterating over the hardcoded remotes,
    // picking a new one for each retry.
    remote_iter: std::iter::Cycle<std::iter::Cloned<std::slice::Iter<'static, Endpoint>>>,
}

impl Daemon {
    pub fn new() -> Result<Self> {
        let (tx, rx) = mpsc::channel();
        let management_interface_broadcaster = Self::start_management_interface(tx.clone())?;
        let state = TunnelState::NotRunning;
        let target_state = TargetState::Unsecured;
        Ok(
            Daemon {
                state,
                tunnel_close_handle: None,
                target_state,
                last_broadcasted_state: DaemonState {
                    state: state.as_security_state(),
                    target_state,
                },
                shutdown: false,
                rx,
                tx,
                management_interface_broadcaster,
                firewall: FirewallProxy::new()
                    .chain_err(|| ErrorKind::FirewallError)?,
                settings: settings::Settings::load().chain_err(|| "Unable to read settings")?,
                remote_endpoint: None,
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
                let _ = exit_tx.send(DaemonEvent::ManagementInterfaceExited(result));
            },
        );
    }

    /// Consume the `Daemon` and run the main event loop. Blocks until an error happens or a
    /// shutdown event is received.
    pub fn run(mut self) -> Result<()> {
        while let Ok(event) = self.rx.recv() {
            self.handle_event(event)?;
            if self.shutdown && self.state == TunnelState::NotRunning {
                break;
            }
        }
        Ok(())
    }

    fn handle_event(&mut self, event: DaemonEvent) -> Result<()> {
        use DaemonEvent::*;
        match event {
            TunnelEvent(event) => self.handle_tunnel_event(event),
            TunnelExited(result) => self.handle_tunnel_exited(result),
            TunnelKillResult(result) => self.handle_tunnel_kill_result(result),
            ManagementInterfaceEvent(event) => self.handle_management_interface_event(event),
            ManagementInterfaceExited(result) => self.handle_management_interface_exited(result),
            TriggerShutdown => self.handle_trigger_shutdown_event(),
        }
    }

    fn handle_tunnel_event(&mut self, tunnel_event: TunnelEvent) -> Result<()> {
        info!("Tunnel event: {:?}", tunnel_event);
        if self.state == TunnelState::Connecting && tunnel_event == TunnelEvent::Up {
            let remote = self.remote_endpoint.unwrap();
            let tunnel_interface = "utun1".to_owned();
            self.set_security_policy(SecurityPolicy::Connected(remote, tunnel_interface))?;
            self.set_state(TunnelState::Connected)
        } else if self.state == TunnelState::Connected && tunnel_event == TunnelEvent::Down {
            self.kill_tunnel()
        } else {
            Ok(())
        }
    }

    fn handle_tunnel_exited(&mut self, result: tunnel::Result<()>) -> Result<()> {
        if let Err(e) = result.chain_err(|| "Tunnel exited in an unexpected way") {
            error!("{}", e.display());
        }
        self.remote_endpoint = None;
        self.reset_security_policy()?;
        self.tunnel_close_handle = None;
        self.set_state(TunnelState::NotRunning)
    }

    fn handle_tunnel_kill_result(&mut self, result: io::Result<()>) -> Result<()> {
        result.chain_err(|| "Error while trying to close tunnel")
    }

    fn handle_management_interface_event(&mut self, event: TunnelCommand) -> Result<()> {
        use TunnelCommand::*;
        match event {
            SetTargetState(state) => self.on_set_target_state(state),
            GetState(tx) => Ok(self.on_get_state(tx)),
            SetAccount(tx, account_token) => self.on_set_account(tx, account_token),
            GetAccount(tx) => Ok(self.on_get_account(tx)),
        }
    }

    fn on_set_target_state(&mut self, new_target_state: TargetState) -> Result<()> {
        if !self.shutdown {
            self.set_target_state(new_target_state)
        } else {
            warn!("Ignoring target state change request due to shutdown");
            Ok(())
        }
    }

    fn on_get_state(&self, tx: OneshotSender<DaemonState>) {
        if let Err(_) = tx.send(self.last_broadcasted_state) {
            warn!("Unable to send current state to management interface client",);
        }
    }

    fn on_set_account(&mut self,
                      tx: OneshotSender<()>,
                      account_token: Option<String>)
                      -> Result<()> {

        let save_result = self.settings.set_account_token(account_token);

        match save_result.chain_err(|| "Unable to save settings") {
            Ok(account_changed) => {
                if let Err(_) = tx.send(()) {
                    warn!("Unable to send response to management interface client");
                }

                let tunnel_needs_restart = self.state == TunnelState::Connecting ||
                                           self.state == TunnelState::Connected;
                if account_changed && tunnel_needs_restart {
                    info!("Initiating tunnel restart because the account token changed");
                    self.kill_tunnel()?;
                }
            }
            Err(e) => error!("{}", e.display()),
        }
        Ok(())
    }

    fn on_get_account(&self, tx: OneshotSender<Option<String>>) {
        if let Err(_) = tx.send(self.settings.get_account_token()) {
            warn!("Unable to send current account to management interface client");
        }
    }

    fn handle_management_interface_exited(&self, result: talpid_ipc::Result<()>) -> Result<()> {
        let error = ErrorKind::ManagementInterfaceError("Server exited unexpectedly");
        match result {
            Ok(()) => Err(error.into()),
            Err(e) => Err(e).chain_err(|| error),
        }
    }

    fn handle_trigger_shutdown_event(&mut self) -> Result<()> {
        self.shutdown = true;
        self.set_target_state(TargetState::Unsecured)
    }

    /// Update the state of the client. If it changed, notify the subscribers and trigger
    /// appropriate actions.
    fn set_state(&mut self, new_state: TunnelState) -> Result<()> {
        if new_state != self.state {
            debug!("State {:?} => {:?}", self.state, new_state);
            self.state = new_state;
            self.broadcast_state();
            self.verify_state_consistency()?;
            self.apply_target_state()
        } else {
            // Calling set_state with the same state we already have is an error. Should try to
            // mitigate this possibility completely with a better state machine later.
            Err(ErrorKind::InvalidState.into())
        }
    }

    fn broadcast_state(&mut self) {
        let new_daemon_state = DaemonState {
            state: self.state.as_security_state(),
            target_state: self.target_state,
        };
        if self.last_broadcasted_state != new_daemon_state {
            self.last_broadcasted_state = new_daemon_state;
            self.management_interface_broadcaster.notify_new_state(new_daemon_state);
        }
    }

    // Check that the current state is valid and consistent.
    fn verify_state_consistency(&self) -> Result<()> {
        use TunnelState::*;
        ensure!(
            match self.state {
                NotRunning => self.tunnel_close_handle.is_none(),
                Connecting => self.tunnel_close_handle.is_some(),
                Connected => self.tunnel_close_handle.is_some(),
                Exiting => self.tunnel_close_handle.is_none(),
            },
            ErrorKind::InvalidState
        );
        Ok(())
    }

    /// Set the target state of the client. If it changed trigger the operations needed to progress
    /// towards that state.
    fn set_target_state(&mut self, new_state: TargetState) -> Result<()> {
        if new_state != self.target_state {
            debug!("Target state {:?} => {:?}", self.target_state, new_state);
            self.target_state = new_state;
            self.broadcast_state();
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
                    error!("{}", e.display());
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
        let account_token = self.settings
            .get_account_token()
            .ok_or(ErrorKind::InvalidSettings("No account token"))?;
        self.set_security_policy(SecurityPolicy::Connecting(remote))?;
        let tunnel_monitor = self.spawn_tunnel_monitor(remote, &account_token)?;
        self.tunnel_close_handle = Some(tunnel_monitor.close_handle());
        self.spawn_tunnel_monitor_wait_thread(tunnel_monitor);
        self.set_state(TunnelState::Connecting)?;
        self.remote_endpoint = Some(remote);
        Ok(())
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
                let _ = error_tx.send(DaemonEvent::TunnelExited(result));
                trace!("Tunnel monitor thread exit");
            },
        );
    }

    fn kill_tunnel(&mut self) -> Result<()> {
        ensure!(
            self.state == TunnelState::Connecting || self.state == TunnelState::Connected,
            ErrorKind::InvalidState
        );
        let close_handle = self.tunnel_close_handle.take().unwrap();
        self.set_state(TunnelState::Exiting)?;
        let result_tx = self.tx.clone();
        thread::spawn(
            move || {
                let result = close_handle.close();
                let _ = result_tx.send(DaemonEvent::TunnelKillResult(result));
                trace!("Tunnel kill thread exit");
            },
        );
        Ok(())
    }

    pub fn shutdown_handle(&self) -> DaemonShutdownHandle {
        DaemonShutdownHandle { tx: self.tx.clone() }
    }

    fn set_security_policy(&mut self, policy: SecurityPolicy) -> Result<()> {
        debug!("Set security policy: {:?}", policy);
        self.firewall.apply_policy(policy).chain_err(|| ErrorKind::FirewallError)
    }

    fn reset_security_policy(&mut self) -> Result<()> {
        debug!("Reset security policy");
        self.firewall.reset_policy().chain_err(|| ErrorKind::FirewallError)
    }
}

struct DaemonShutdownHandle {
    tx: mpsc::Sender<DaemonEvent>,
}

impl DaemonShutdownHandle {
    pub fn shutdown(&self) {
        let _ = self.tx.send(DaemonEvent::TriggerShutdown);
    }
}

impl Drop for Daemon {
    fn drop(self: &mut Daemon) {
        if let Err(e) = rpc_info::remove().chain_err(|| "Unable to clean up rpc address file") {
            error!("{}", e.display());
        }
    }
}


quick_main!(run);

fn run() -> Result<()> {
    let config = cli::get_config();
    init_logger(config.log_level, config.log_file.as_ref())?;

    let daemon = Daemon::new().chain_err(|| "Unable to initialize daemon")?;

    let shutdown_handle = daemon.shutdown_handle();
    shutdown::set_shutdown_signal_handler(move || shutdown_handle.shutdown())
        .chain_err(|| "Unable to attach shutdown signal handler")?;

    daemon.run()?;

    debug!("Mullvad daemon is quitting");
    Ok(())
}

fn init_logger(log_level: log::LogLevelFilter, log_file: Option<&PathBuf>) -> Result<()> {
    let silenced_crates = [
        "jsonrpc_core",
        "tokio_core",
        "jsonrpc_ws_server",
        "ws",
        "mio",
    ];
    let mut config = fern::Dispatch::new()
        .format(
            |out, message, record| {
                out.finish(
                    format_args!(
                        "{}[{}][{}] {}",
                        chrono::Local::now().format("[%Y-%m-%d %H:%M:%S]"),
                        record.target(),
                        record.level(),
                        message
                    ),
                )
            },
        )
        .level(log_level)
        .chain(std::io::stdout());
    for silenced_crate in &silenced_crates {
        config = config.level_for(*silenced_crate, log::LogLevelFilter::Warn);
    }
    if let Some(ref log_file) = log_file {
        let f = fern::log_file(log_file).chain_err(|| "Failed to open log file for writing")?;
        config = config.chain(f);
    }
    config.apply().chain_err(|| "Failed to bootstrap logging system")
}

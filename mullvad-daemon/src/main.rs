//! # License
//!
//! Copyright (C) 2017  Amagicom AB
//!
//! This program is free software: you can redistribute it and/or modify it under the terms of the
//! GNU General Public License as published by the Free Software Foundation, either version 3 of
//! the License, or (at your option) any later version.


extern crate app_dirs;
extern crate chrono;
#[macro_use]
extern crate clap;
#[macro_use]
extern crate error_chain;
extern crate fern;
extern crate futures;
#[macro_use]
extern crate log;

#[macro_use]
extern crate serde;
extern crate serde_json;

extern crate jsonrpc_core;
#[macro_use]
extern crate jsonrpc_macros;
extern crate jsonrpc_pubsub;
extern crate jsonrpc_ws_server;
#[macro_use]
extern crate lazy_static;
extern crate rand;
extern crate tokio_timer;
extern crate uuid;

extern crate mullvad_rpc;
extern crate mullvad_types;
extern crate talpid_core;
extern crate talpid_ipc;
extern crate talpid_types;

mod cli;
mod management_interface;
mod relays;
mod rpc_info;
mod settings;
mod shutdown;
mod account_history;


use app_dirs::AppInfo;
use error_chain::ChainedError;
use futures::Future;
use jsonrpc_core::futures::sync::oneshot::Sender as OneshotSender;

use management_interface::{BoxFuture, ManagementInterfaceServer, TunnelCommand};
use mullvad_rpc::{AccountsProxy, HttpHandle};

use mullvad_types::account::{AccountData, AccountToken};
use mullvad_types::location::Location;
use mullvad_types::relay_constraints::{RelaySettings, RelaySettingsUpdate};
use mullvad_types::relay_list::{Relay, RelayList};
use mullvad_types::states::{DaemonState, SecurityState, TargetState};

use std::env;
use std::io;
use std::net::{IpAddr, Ipv4Addr};
use std::path::{Path, PathBuf};
use std::sync::{mpsc, Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

use talpid_core::firewall::{Firewall, FirewallProxy, SecurityPolicy};
use talpid_core::mpsc::IntoSender;
use talpid_core::tunnel::{self, TunnelEvent, TunnelMetadata, TunnelMonitor};
use talpid_types::net::TunnelEndpoint;

use std::fs;


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
        NoRelay {
            description("Found no valid relays to connect to")
        }
    }
}

lazy_static! {
    static ref MIN_TUNNEL_ALIVE_TIME_MS: Duration = Duration::from_millis(1000);
    static ref MAX_RELAY_CACHE_AGE: Duration = Duration::from_secs(3600);
    static ref RELAY_CACHE_UPDATE_TIMEOUT: Duration = Duration::from_millis(3000);
}

static APP_INFO: AppInfo = AppInfo {
    name: crate_name!(),
    author: "Mullvad",
};

const DATE_TIME_FORMAT_STR: &str = "%Y-%m-%d %H:%M:%S%.3f";


/// All events that can happen in the daemon. Sent from various threads and exposed interfaces.
pub enum DaemonEvent {
    /// An event coming from the tunnel software to indicate a change in state.
    TunnelEvent(TunnelEvent),
    /// Triggered by the thread waiting for the tunnel process. Means the tunnel process
    /// exited.
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
        use TunnelState::*;
        match *self {
            NotRunning | Connecting => SecurityState::Unsecured,
            Connected | Exiting => SecurityState::Secured,
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
    accounts_proxy: AccountsProxy<HttpHandle>,
    relay_selector: relays::RelaySelector,
    firewall: FirewallProxy,
    current_relay: Option<Relay>,
    tunnel_endpoint: Option<TunnelEndpoint>,
    tunnel_metadata: Option<TunnelMetadata>,
    tunnel_log: Option<PathBuf>,
    resource_dir: PathBuf,
}

impl Daemon {
    pub fn new(tunnel_log: Option<PathBuf>) -> Result<Self> {
        let resource_dir = get_resource_dir();
        let rpc_http_handle = mullvad_rpc::connect().chain_err(|| "Unable to connect to RPC API")?;

        let relay_selector = Self::create_relay_selector(rpc_http_handle.clone(), &resource_dir)?;

        let (tx, rx) = mpsc::channel();
        let management_interface_broadcaster = Self::start_management_interface(tx.clone())?;
        let state = TunnelState::NotRunning;
        let target_state = TargetState::Unsecured;
        Ok(Daemon {
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
            settings: settings::Settings::load().chain_err(|| "Unable to read settings")?,
            accounts_proxy: AccountsProxy::new(rpc_http_handle),
            relay_selector,
            firewall: FirewallProxy::new().chain_err(|| ErrorKind::FirewallError)?,
            current_relay: None,
            tunnel_endpoint: None,
            tunnel_metadata: None,
            tunnel_log: tunnel_log,
            resource_dir,
        })
    }

    fn create_relay_selector(
        rpc_http_handle: mullvad_rpc::HttpHandle,
        resource_dir: &Path,
    ) -> Result<relays::RelaySelector> {
        let mut relay_selector = relays::RelaySelector::new(rpc_http_handle, &resource_dir)
            .chain_err(|| "Unable to initialize relay list cache")?;
        if let Ok(elapsed) = relay_selector.get_last_updated().elapsed() {
            if elapsed > *MAX_RELAY_CACHE_AGE {
                if let Err(e) = relay_selector.update(*RELAY_CACHE_UPDATE_TIMEOUT) {
                    error!("Unable to update relay cache: {}", e.display_chain());
                }
            }
        }
        Ok(relay_selector)
    }

    // Starts the management interface and spawns a thread that will process it.
    // Returns a handle that allows notifying all subscribers on events.
    fn start_management_interface(
        event_tx: mpsc::Sender<DaemonEvent>,
    ) -> Result<management_interface::EventBroadcaster> {
        let multiplex_event_tx = IntoSender::from(event_tx.clone());
        let server = Self::start_management_interface_server(multiplex_event_tx)?;
        let event_broadcaster = server.event_broadcaster();
        Self::spawn_management_interface_wait_thread(server, event_tx);
        Ok(event_broadcaster)
    }

    fn start_management_interface_server(
        event_tx: IntoSender<TunnelCommand, DaemonEvent>,
    ) -> Result<ManagementInterfaceServer> {
        let shared_secret = uuid::Uuid::new_v4().to_string();
        let server = ManagementInterfaceServer::start(event_tx, shared_secret.clone())
            .chain_err(|| ErrorKind::ManagementInterfaceError("Failed to start server"))?;
        info!(
            "Mullvad management interface listening on {}",
            server.address()
        );
        rpc_info::write(server.address(), &shared_secret).chain_err(|| {
            ErrorKind::ManagementInterfaceError("Failed to write RPC connection info to file")
        })?;
        Ok(server)
    }

    fn spawn_management_interface_wait_thread(
        server: ManagementInterfaceServer,
        exit_tx: mpsc::Sender<DaemonEvent>,
    ) {
        thread::spawn(move || {
            let result = server.wait();
            debug!("Mullvad management interface shut down");
            let _ = exit_tx.send(DaemonEvent::ManagementInterfaceExited(result));
        });
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
        debug!("Tunnel event: {:?}", tunnel_event);
        if self.state == TunnelState::Connecting {
            if let TunnelEvent::Up(metadata) = tunnel_event {
                self.tunnel_metadata = Some(metadata);
                self.set_security_policy()?;
                self.set_state(TunnelState::Connected)
            } else {
                Ok(())
            }
        } else if self.state == TunnelState::Connected && tunnel_event == TunnelEvent::Down {
            self.kill_tunnel()
        } else {
            Ok(())
        }
    }

    fn handle_tunnel_exited(&mut self, result: tunnel::Result<()>) -> Result<()> {
        if let Err(e) = result.chain_err(|| "Tunnel exited in an unexpected way") {
            error!("{}", e.display_chain());
        }
        self.current_relay = None;
        self.tunnel_endpoint = None;
        self.tunnel_metadata = None;
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
            GetPublicIp(tx) => Ok(self.on_get_ip(tx)),
            GetCurrentLocation(tx) => Ok(self.on_get_current_location(tx)),
            GetAccountData(tx, account_token) => Ok(self.on_get_account_data(tx, account_token)),
            GetRelayLocations(tx) => Ok(self.on_get_relay_locations(tx)),
            SetAccount(tx, account_token) => self.on_set_account(tx, account_token),
            GetAccount(tx) => Ok(self.on_get_account(tx)),
            UpdateRelaySettings(tx, update) => self.on_update_relay_settings(tx, update),
            SetAllowLan(tx, allow_lan) => self.on_set_allow_lan(tx, allow_lan),
            GetAllowLan(tx) => Ok(self.on_get_allow_lan(tx)),
            GetRelaySettings(tx) => Ok(self.on_get_relay_settings(tx)),
            Shutdown => self.handle_trigger_shutdown_event(),
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
        Self::oneshot_send(tx, self.last_broadcasted_state, "current state");
    }

    fn on_get_ip(&self, tx: OneshotSender<IpAddr>) {
        let ip = if let Some(ref relay) = self.current_relay {
            IpAddr::V4(relay.ipv4_addr_exit)
        } else {
            IpAddr::V4(Ipv4Addr::new(1, 3, 3, 7))
        };
        Self::oneshot_send(tx, ip, "current ip");
    }

    fn on_get_current_location(&self, tx: OneshotSender<Location>) {
        let location = if let Some(ref relay) = self.current_relay {
            relay.location.as_ref().cloned().unwrap()
        } else {
            Location {
                country: String::from("Narnia"),
                country_code: String::from("na"),
                city: String::from("Le City"),
                city_code: String::from("le"),
                position: [13.37, 0.0],
            }
        };
        Self::oneshot_send(tx, location, "current location");
    }

    fn on_get_account_data(
        &mut self,
        tx: OneshotSender<BoxFuture<AccountData, mullvad_rpc::Error>>,
        account_token: AccountToken,
    ) {
        let rpc_call = self.accounts_proxy
            .get_expiry(account_token)
            .map(|expiry| AccountData { expiry });
        Self::oneshot_send(tx, Box::new(rpc_call), "account data")
    }

    fn on_get_relay_locations(&mut self, tx: OneshotSender<RelayList>) {
        Self::oneshot_send(
            tx,
            self.relay_selector.get_locations().clone(),
            "relay locations",
        );
    }


    fn on_set_account(
        &mut self,
        tx: OneshotSender<()>,
        account_token: Option<String>,
    ) -> Result<()> {
        let save_result = self.settings.set_account_token(account_token);

        match save_result.chain_err(|| "Unable to save settings") {
            Ok(account_changed) => {
                Self::oneshot_send(tx, (), "set_account response");
                let tunnel_needs_restart =
                    self.state == TunnelState::Connecting || self.state == TunnelState::Connected;
                if account_changed && tunnel_needs_restart {
                    info!("Initiating tunnel restart because the account token changed");
                    self.kill_tunnel()?;
                }
            }
            Err(e) => error!("{}", e.display_chain()),
        }
        Ok(())
    }

    fn on_get_account(&self, tx: OneshotSender<Option<String>>) {
        Self::oneshot_send(tx, self.settings.get_account_token(), "current account")
    }

    fn on_update_relay_settings(
        &mut self,
        tx: OneshotSender<()>,
        update: RelaySettingsUpdate,
    ) -> Result<()> {
        let save_result = self.settings.update_relay_settings(update);

        match save_result.chain_err(|| "Unable to save settings") {
            Ok(changed) => {
                Self::oneshot_send(tx, (), "update_relay_settings response");

                let tunnel_needs_restart =
                    self.state == TunnelState::Connecting || self.state == TunnelState::Connected;

                if changed && tunnel_needs_restart {
                    info!("Initiating tunnel restart because the relay settings changed");
                    self.kill_tunnel()?;
                }
            }
            Err(e) => error!("{}", e.display_chain()),
        }

        Ok(())
    }

    fn on_get_relay_settings(&self, tx: OneshotSender<RelaySettings>) {
        Self::oneshot_send(tx, self.settings.get_relay_settings(), "relay settings")
    }

    fn on_set_allow_lan(&mut self, tx: OneshotSender<()>, allow_lan: bool) -> Result<()> {
        let save_result = self.settings.set_allow_lan(allow_lan);
        match save_result.chain_err(|| "Unable to save settings") {
            Ok(settings_changed) => {
                if settings_changed && self.target_state == TargetState::Secured {
                    self.set_security_policy()?;
                }
                Self::oneshot_send(tx, (), "set_allow_lan response");
            }
            Err(e) => error!("{}", e.display_chain()),
        }
        Ok(())
    }

    fn on_get_allow_lan(&self, tx: OneshotSender<bool>) {
        Self::oneshot_send(tx, self.settings.get_allow_lan(), "allow lan")
    }

    fn oneshot_send<T>(tx: OneshotSender<T>, t: T, msg: &'static str) {
        if let Err(_) = tx.send(t) {
            warn!("Unable to send {} to management interface client", msg);
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
            self.management_interface_broadcaster
                .notify_new_state(new_daemon_state);
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

    /// Set the target state of the client. If it changed trigger the operations needed to
    /// progress towards that state.
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
                    error!("{}", e.display_chain());
                    self.current_relay = None;
                    self.tunnel_endpoint = None;
                    self.management_interface_broadcaster.notify_error(&e);
                    self.set_target_state(TargetState::Unsecured)?;
                }
                Ok(())
            }
            (TargetState::Unsecured, TunnelState::NotRunning) => self.reset_security_policy(),
            (TargetState::Unsecured, TunnelState::Connecting)
            | (TargetState::Unsecured, TunnelState::Connected) => self.kill_tunnel(),
            (..) => Ok(()),
        }
    }

    fn start_tunnel(&mut self) -> Result<()> {
        ensure!(
            self.target_state == TargetState::Secured && self.state == TunnelState::NotRunning,
            ErrorKind::InvalidState
        );

        match self.settings.get_relay_settings() {
            RelaySettings::CustomTunnelEndpoint(custom_relay) => {
                let tunnel_endpoint = custom_relay
                    .to_tunnel_endpoint()
                    .chain_err(|| ErrorKind::NoRelay)?;
                self.tunnel_endpoint = Some(tunnel_endpoint);
            }
            RelaySettings::Normal(constraints) => {
                let (relay, tunnel_endpoint) = self.relay_selector
                    .get_tunnel_endpoint(&constraints)
                    .chain_err(|| ErrorKind::NoRelay)?;
                self.tunnel_endpoint = Some(tunnel_endpoint);
                self.current_relay = Some(relay);
            }
        }

        let account_token = self.settings
            .get_account_token()
            .ok_or(ErrorKind::InvalidSettings("No account token"))?;

        self.set_security_policy()?;

        if let Some(ref file) = self.tunnel_log {
            let _ = fs::remove_file(file);
            fs::File::create(file).chain_err(|| "Unable to create the tunnel log file")?;
        }

        let tunnel_monitor =
            self.spawn_tunnel_monitor(self.tunnel_endpoint.unwrap(), &account_token)?;
        self.tunnel_close_handle = Some(tunnel_monitor.close_handle());
        self.spawn_tunnel_monitor_wait_thread(tunnel_monitor);

        self.set_state(TunnelState::Connecting)?;
        Ok(())
    }

    fn spawn_tunnel_monitor(
        &self,
        tunnel_endpoint: TunnelEndpoint,
        account_token: &str,
    ) -> Result<TunnelMonitor> {
        // Must wrap the channel in a Mutex because TunnelMonitor forces the closure to be Sync
        let event_tx = Arc::new(Mutex::new(self.tx.clone()));
        let on_tunnel_event = move |event| {
            let _ = event_tx
                .lock()
                .unwrap()
                .send(DaemonEvent::TunnelEvent(event));
        };
        TunnelMonitor::new(
            tunnel_endpoint,
            account_token,
            self.tunnel_log.as_ref().map(PathBuf::as_path),
            &self.resource_dir,
            on_tunnel_event,
        ).chain_err(|| ErrorKind::TunnelError("Unable to start tunnel monitor"))
    }

    fn spawn_tunnel_monitor_wait_thread(&self, tunnel_monitor: TunnelMonitor) {
        let error_tx = self.tx.clone();
        thread::spawn(move || {
            let start = Instant::now();
            let result = tunnel_monitor.wait();
            if let Some(sleep_dur) = MIN_TUNNEL_ALIVE_TIME_MS.checked_sub(start.elapsed()) {
                thread::sleep(sleep_dur);
            }
            let _ = error_tx.send(DaemonEvent::TunnelExited(result));
            trace!("Tunnel monitor thread exit");
        });
    }

    fn kill_tunnel(&mut self) -> Result<()> {
        ensure!(
            self.state == TunnelState::Connecting || self.state == TunnelState::Connected,
            ErrorKind::InvalidState
        );
        let close_handle = self.tunnel_close_handle.take().unwrap();
        self.set_state(TunnelState::Exiting)?;
        let result_tx = self.tx.clone();
        thread::spawn(move || {
            let result = close_handle.close();
            let _ = result_tx.send(DaemonEvent::TunnelKillResult(result));
            trace!("Tunnel kill thread exit");
        });
        Ok(())
    }

    pub fn shutdown_handle(&self) -> DaemonShutdownHandle {
        DaemonShutdownHandle {
            tx: self.tx.clone(),
        }
    }

    fn set_security_policy(&mut self) -> Result<()> {
        let policy = match (self.tunnel_endpoint, self.tunnel_metadata.as_ref()) {
            (Some(relay), None) => SecurityPolicy::Connecting {
                relay_endpoint: relay.to_endpoint(),
                allow_lan: self.settings.get_allow_lan(),
            },
            (Some(relay), Some(tunnel_metadata)) => SecurityPolicy::Connected {
                relay_endpoint: relay.to_endpoint(),
                tunnel: tunnel_metadata.clone(),
                allow_lan: self.settings.get_allow_lan(),
            },
            _ => bail!(ErrorKind::InvalidState),
        };
        debug!("Set security policy: {:?}", policy);
        self.firewall
            .apply_policy(policy)
            .chain_err(|| ErrorKind::FirewallError)
    }

    fn reset_security_policy(&mut self) -> Result<()> {
        debug!("Reset security policy");
        self.firewall
            .reset_policy()
            .chain_err(|| ErrorKind::FirewallError)
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
            error!("{}", e.display_chain());
        }
    }
}


quick_main!(run);

fn run() -> Result<()> {
    let config = cli::get_config();
    init_logger(config.log_level, config.log_file.as_ref())?;
    log_version();

    let daemon = Daemon::new(config.tunnel_log_file).chain_err(|| "Unable to initialize daemon")?;

    let shutdown_handle = daemon.shutdown_handle();
    shutdown::set_shutdown_signal_handler(move || shutdown_handle.shutdown())
        .chain_err(|| "Unable to attach shutdown signal handler")?;

    daemon.run()?;

    debug!("Mullvad daemon is quitting");
    thread::sleep(Duration::from_millis(500));
    Ok(())
}

fn init_logger(log_level: log::LogLevelFilter, log_file: Option<&PathBuf>) -> Result<()> {
    let silenced_crates = [
        "jsonrpc_core",
        "tokio_core",
        "tokio_proto",
        "jsonrpc_ws_server",
        "ws",
        "mio",
        "hyper",
    ];
    let mut config = fern::Dispatch::new()
        .format(|out, message, record| {
            out.finish(format_args!(
                "[{}][{}][{}] {}",
                chrono::Local::now().format(DATE_TIME_FORMAT_STR),
                record.target(),
                record.level(),
                message
            ))
        })
        .level(log_level)
        .chain(std::io::stdout());
    for silenced_crate in &silenced_crates {
        config = config.level_for(*silenced_crate, log::LogLevelFilter::Warn);
    }
    if let Some(ref log_file) = log_file {
        let f = fern::log_file(log_file).chain_err(|| "Failed to open log file for writing")?;
        config = config.chain(f);
    }
    config
        .apply()
        .chain_err(|| "Failed to bootstrap logging system")
}

fn log_version() {
    info!(
        "Starting {} {}",
        env!("CARGO_PKG_NAME"),
        include_str!(concat!(env!("OUT_DIR"), "/git-commit-info.txt"))
    )
}

fn get_resource_dir() -> PathBuf {
    match env::current_exe() {
        Ok(mut path) => {
            path.pop();
            path
        }
        Err(e) => {
            error!(
                "Failed finding the install directory. Using working directory: {}",
                e
            );
            PathBuf::from(".")
        }
    }
}

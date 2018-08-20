//! # License
//!
//! Copyright (C) 2017  Amagicom AB
//!
//! This program is free software: you can redistribute it and/or modify it under the terms of the
//! GNU General Public License as published by the Free Software Foundation, either version 3 of
//! the License, or (at your option) any later version.

extern crate chrono;
#[macro_use]
extern crate clap;
#[macro_use]
extern crate error_chain;
extern crate futures;
#[cfg(unix)]
extern crate libc;
#[macro_use]
extern crate log;
extern crate log_panics;

#[macro_use]
extern crate serde;
extern crate serde_json;

extern crate jsonrpc_core;
#[macro_use]
extern crate jsonrpc_macros;
extern crate jsonrpc_pubsub;
extern crate jsonrpc_ws_server;
extern crate rand;
extern crate tokio_core;
extern crate tokio_timer;
extern crate uuid;

extern crate mullvad_ipc_client;
extern crate mullvad_paths;
extern crate mullvad_rpc;
extern crate mullvad_types;
extern crate talpid_core;
extern crate talpid_ipc;
extern crate talpid_types;

#[cfg(windows)]
#[macro_use]
extern crate windows_service;

mod account_history;
mod cli;
mod geoip;
mod logging;
mod management_interface;
mod relays;
mod rpc_address_file;
mod rpc_uniqueness_check;
mod settings;
mod shutdown;
mod system_service;
mod tunnel_state_machine;
mod version;

use error_chain::ChainedError;
use futures::sync::mpsc::UnboundedSender;
use futures::{Future, Sink};
use jsonrpc_core::futures::sync::oneshot::Sender as OneshotSender;

use management_interface::{BoxFuture, ManagementCommand, ManagementInterfaceServer};
use mullvad_rpc::{AccountsProxy, AppVersionProxy, HttpHandle};
use tunnel_state_machine::{TunnelCommand, TunnelParameters, TunnelStateTransition};

use mullvad_types::account::{AccountData, AccountToken};
use mullvad_types::location::GeoIpLocation;
use mullvad_types::relay_constraints::{RelaySettings, RelaySettingsUpdate};
use mullvad_types::relay_list::{Relay, RelayList};
use mullvad_types::states::{DaemonState, SecurityState, TargetState};
use mullvad_types::version::{AppVersion, AppVersionInfo};

use std::net::IpAddr;
use std::path::PathBuf;
use std::sync::mpsc;
use std::time::Duration;
use std::{mem, thread};

use talpid_core::mpsc::IntoSender;
use talpid_types::net::TunnelOptions;


error_chain!{
    errors {
        LogError(msg: &'static str) {
            description("Error setting up log")
            display("Error setting up log: {}", msg)
        }
        NoCacheDir {
            description("Unable to create cache directory")
        }
        DaemonIsAlreadyRunning {
            description("Another instance of the daemon is already running")
        }
        ManagementInterfaceError(msg: &'static str) {
            description("Error in the management interface")
            display("Management interface error: {}", msg)
        }
        InvalidSettings(msg: &'static str) {
            description("Invalid settings")
            display("Invalid Settings: {}", msg)
        }
        NoRelay {
            description("Found no valid relays to connect to")
        }
    }

    links {
        TunnelError(tunnel_state_machine::Error, tunnel_state_machine::ErrorKind);
    }
}

const DAEMON_LOG_FILENAME: &str = "daemon.log";

type SyncUnboundedSender<T> = ::futures::sink::Wait<UnboundedSender<T>>;

/// All events that can happen in the daemon. Sent from various threads and exposed interfaces.
pub enum DaemonEvent {
    /// Tunnel has changed state.
    TunnelStateTransition(TunnelStateTransition),
    /// An event coming from the JSONRPC-2.0 management interface.
    ManagementInterfaceEvent(ManagementCommand),
    /// Triggered if the server hosting the JSONRPC-2.0 management interface dies unexpectedly.
    ManagementInterfaceExited(talpid_ipc::Result<()>),
    /// Daemon shutdown triggered by a signal, ctrl-c or similar.
    TriggerShutdown,
}

impl From<TunnelStateTransition> for DaemonEvent {
    fn from(tunnel_state_transition: TunnelStateTransition) -> Self {
        DaemonEvent::TunnelStateTransition(tunnel_state_transition)
    }
}

impl From<ManagementCommand> for DaemonEvent {
    fn from(command: ManagementCommand) -> Self {
        DaemonEvent::ManagementInterfaceEvent(command)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum DaemonExecutionState {
    Running,
    Exiting,
    Finished,
}

impl DaemonExecutionState {
    pub fn shutdown(&mut self, tunnel_state: TunnelStateTransition) {
        use self::DaemonExecutionState::*;

        match self {
            Running => {
                match tunnel_state {
                    TunnelStateTransition::Disconnected => mem::replace(self, Finished),
                    _ => mem::replace(self, Exiting),
                };
            }
            Exiting | Finished => {}
        };
    }

    pub fn disconnected(&mut self) {
        use self::DaemonExecutionState::*;

        match self {
            Exiting => {
                mem::replace(self, Finished);
            }
            Running | Finished => {}
        };
    }

    pub fn is_running(&mut self) -> bool {
        use self::DaemonExecutionState::*;

        match self {
            Running => true,
            Exiting | Finished => false,
        }
    }
}


struct Daemon {
    tunnel_command_tx: SyncUnboundedSender<TunnelCommand>,
    tunnel_state: TunnelStateTransition,
    security_state: SecurityState,
    last_broadcasted_state: DaemonState,
    target_state: TargetState,
    state: DaemonExecutionState,
    rx: mpsc::Receiver<DaemonEvent>,
    tx: mpsc::Sender<DaemonEvent>,
    management_interface_broadcaster: management_interface::EventBroadcaster,
    settings: settings::Settings,
    accounts_proxy: AccountsProxy<HttpHandle>,
    version_proxy: AppVersionProxy<HttpHandle>,
    https_handle: mullvad_rpc::rest::RequestSender,
    tokio_remote: tokio_core::reactor::Remote,
    relay_selector: relays::RelaySelector,
    current_relay: Option<Relay>,
    log_dir: Option<PathBuf>,
    resource_dir: PathBuf,
}

impl Daemon {
    pub fn new(
        log_dir: Option<PathBuf>,
        resource_dir: PathBuf,
        cache_dir: PathBuf,
    ) -> Result<Self> {
        ensure!(
            !rpc_uniqueness_check::is_another_instance_running(),
            ErrorKind::DaemonIsAlreadyRunning
        );
        let ca_path = resource_dir.join(mullvad_paths::resources::API_CA_FILENAME);

        let mut rpc_manager = mullvad_rpc::MullvadRpcFactory::with_cache_dir(&cache_dir, &ca_path);

        let (rpc_handle, https_handle, tokio_remote) =
            mullvad_rpc::event_loop::create(move |core| {
                let handle = core.handle();
                let rpc = rpc_manager.new_connection_on_event_loop(&handle);
                let https_handle = mullvad_rpc::rest::create_https_client(&ca_path, &handle);
                let remote = core.remote();
                (rpc, https_handle, remote)
            }).chain_err(|| "Unable to initialize network event loop")?;
        let rpc_handle = rpc_handle.chain_err(|| "Unable to create RPC client")?;
        let https_handle = https_handle.chain_err(|| "Unable to create am.i.mullvad client")?;

        let relay_selector =
            relays::RelaySelector::new(rpc_handle.clone(), &resource_dir, &cache_dir);

        let (tx, rx) = mpsc::channel();
        let tunnel_command_tx =
            tunnel_state_machine::spawn(cache_dir.clone(), IntoSender::from(tx.clone()))?;

        let target_state = TargetState::Unsecured;
        let management_interface_broadcaster =
            Self::start_management_interface(tx.clone(), cache_dir.clone())?;

        Ok(Daemon {
            tunnel_command_tx: Sink::wait(tunnel_command_tx),
            tunnel_state: TunnelStateTransition::Disconnected,
            security_state: SecurityState::Unsecured,
            target_state,
            last_broadcasted_state: DaemonState {
                state: SecurityState::Unsecured,
                target_state,
            },
            state: DaemonExecutionState::Running,
            rx,
            tx,
            management_interface_broadcaster,
            settings: settings::Settings::load().chain_err(|| "Unable to read settings")?,
            accounts_proxy: AccountsProxy::new(rpc_handle.clone()),
            version_proxy: AppVersionProxy::new(rpc_handle),
            https_handle,
            tokio_remote,
            relay_selector,
            current_relay: None,
            log_dir,
            resource_dir,
        })
    }

    // Starts the management interface and spawns a thread that will process it.
    // Returns a handle that allows notifying all subscribers on events.
    fn start_management_interface(
        event_tx: mpsc::Sender<DaemonEvent>,
        cache_dir: PathBuf,
    ) -> Result<management_interface::EventBroadcaster> {
        let multiplex_event_tx = IntoSender::from(event_tx.clone());
        let server = Self::start_management_interface_server(multiplex_event_tx, cache_dir)?;
        let event_broadcaster = server.event_broadcaster();
        Self::spawn_management_interface_wait_thread(server, event_tx);
        Ok(event_broadcaster)
    }

    fn start_management_interface_server(
        event_tx: IntoSender<ManagementCommand, DaemonEvent>,
        cache_dir: PathBuf,
    ) -> Result<ManagementInterfaceServer> {
        let shared_secret = uuid::Uuid::new_v4().to_string();

        let server = ManagementInterfaceServer::start(event_tx, shared_secret.clone(), cache_dir)
            .chain_err(|| ErrorKind::ManagementInterfaceError("Failed to start server"))?;
        info!(
            "Mullvad management interface listening on {}",
            server.address()
        );

        rpc_address_file::write(server.address(), &shared_secret).chain_err(|| {
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
            error!("Mullvad management interface shut down");
            let _ = exit_tx.send(DaemonEvent::ManagementInterfaceExited(result));
        });
    }

    /// Consume the `Daemon` and run the main event loop. Blocks until an error happens or a
    /// shutdown event is received.
    pub fn run(mut self) -> Result<()> {
        if self.settings.get_auto_connect() {
            info!("Automatically connecting since auto-connect is turned on");
            self.set_target_state(TargetState::Secured)?;
        }
        while let Ok(event) = self.rx.recv() {
            self.handle_event(event)?;
            if self.state == DaemonExecutionState::Finished {
                break;
            }
        }
        Ok(())
    }

    fn handle_event(&mut self, event: DaemonEvent) -> Result<()> {
        use DaemonEvent::*;
        match event {
            TunnelStateTransition(transition) => self.handle_tunnel_state_transition(transition),
            ManagementInterfaceEvent(event) => self.handle_management_interface_event(event),
            ManagementInterfaceExited(result) => self.handle_management_interface_exited(result),
            TriggerShutdown => self.handle_trigger_shutdown_event(),
        }
    }

    fn handle_tunnel_state_transition(
        &mut self,
        tunnel_state: TunnelStateTransition,
    ) -> Result<()> {
        use self::TunnelStateTransition::*;

        debug!("New tunnel state: {:?}", tunnel_state);

        if tunnel_state == Disconnected {
            self.state.disconnected();
        }

        self.tunnel_state = tunnel_state;
        self.security_state = match tunnel_state {
            Disconnected | Connecting => SecurityState::Unsecured,
            Connected | Disconnecting => SecurityState::Secured,
        };

        self.broadcast_state();

        Ok(())
    }

    fn handle_management_interface_event(&mut self, event: ManagementCommand) -> Result<()> {
        use ManagementCommand::*;
        match event {
            SetTargetState(state) => self.on_set_target_state(state),
            GetState(tx) => Ok(self.on_get_state(tx)),
            GetCurrentLocation(tx) => Ok(self.on_get_current_location(tx)),
            GetAccountData(tx, account_token) => Ok(self.on_get_account_data(tx, account_token)),
            GetRelayLocations(tx) => Ok(self.on_get_relay_locations(tx)),
            SetAccount(tx, account_token) => self.on_set_account(tx, account_token),
            GetAccount(tx) => Ok(self.on_get_account(tx)),
            UpdateRelaySettings(tx, update) => self.on_update_relay_settings(tx, update),
            SetAllowLan(tx, allow_lan) => self.on_set_allow_lan(tx, allow_lan),
            GetAllowLan(tx) => Ok(self.on_get_allow_lan(tx)),
            SetAutoConnect(tx, auto_connect) => self.on_set_auto_connect(tx, auto_connect),
            GetAutoConnect(tx) => Ok(self.on_get_auto_connect(tx)),
            SetOpenVpnMssfix(tx, mssfix_arg) => self.on_set_openvpn_mssfix(tx, mssfix_arg),
            SetOpenVpnEnableIpv6(tx, enable_ipv6) => {
                self.on_set_openvpn_enable_ipv6(tx, enable_ipv6)
            }
            GetTunnelOptions(tx) => self.on_get_tunnel_options(tx),
            GetRelaySettings(tx) => Ok(self.on_get_relay_settings(tx)),
            GetVersionInfo(tx) => Ok(self.on_get_version_info(tx)),
            GetCurrentVersion(tx) => Ok(self.on_get_current_version(tx)),
            Shutdown => self.handle_trigger_shutdown_event(),
        }
    }

    fn on_set_target_state(&mut self, new_target_state: TargetState) -> Result<()> {
        if self.state.is_running() {
            self.set_target_state(new_target_state)
        } else {
            warn!("Ignoring target state change request due to shutdown");
            Ok(())
        }
    }

    fn on_get_state(&self, tx: OneshotSender<DaemonState>) {
        Self::oneshot_send(tx, self.last_broadcasted_state, "current state");
    }

    fn on_get_current_location(&self, tx: OneshotSender<GeoIpLocation>) {
        if let Some(ref relay) = self.current_relay {
            let location = relay.location.as_ref().cloned().unwrap();
            let geo_ip_location = GeoIpLocation {
                ip: IpAddr::V4(relay.ipv4_addr_exit),
                country: location.country,
                city: Some(location.city),
                latitude: location.latitude,
                longitude: location.longitude,
                mullvad_exit_ip: true,
            };
            Self::oneshot_send(tx, geo_ip_location, "current location");
        } else {
            let https_handle = self.https_handle.clone();
            self.tokio_remote.spawn(move |_| {
                geoip::send_location_request(https_handle)
                    .map(move |location| Self::oneshot_send(tx, location, "current location"))
                    .map_err(|e| {
                        warn!("Unable to fetch GeoIP location: {}", e.display_chain());
                    })
            });
        }
    }

    fn on_get_account_data(
        &mut self,
        tx: OneshotSender<BoxFuture<AccountData, mullvad_rpc::Error>>,
        account_token: AccountToken,
    ) {
        let rpc_call = self
            .accounts_proxy
            .get_expiry(account_token)
            .map(|expiry| AccountData { expiry });
        Self::oneshot_send(tx, Box::new(rpc_call), "account data")
    }

    fn on_get_relay_locations(&mut self, tx: OneshotSender<RelayList>) {
        Self::oneshot_send(tx, self.relay_selector.get_locations(), "relay locations");
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
                if account_changed {
                    info!("Initiating tunnel restart because the account token changed");
                    self.connect_tunnel()?;
                }
            }
            Err(e) => error!("{}", e.display_chain()),
        }
        Ok(())
    }

    fn on_get_version_info(
        &mut self,
        tx: OneshotSender<BoxFuture<AppVersionInfo, mullvad_rpc::Error>>,
    ) {
        let current_version = version::CURRENT.to_owned();
        let fut = self
            .version_proxy
            .latest_app_version()
            .join(
                self.version_proxy
                    .is_app_version_supported(&current_version),
            ).map(|(latest_versions, is_supported)| AppVersionInfo {
                current_is_supported: is_supported,
                latest: latest_versions,
            });
        Self::oneshot_send(tx, Box::new(fut), "get_version_info response");
    }

    fn on_get_current_version(&mut self, tx: OneshotSender<AppVersion>) {
        let current_version = version::CURRENT.to_owned();
        Self::oneshot_send(tx, current_version, "get_current_version response");
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

                if changed {
                    info!("Initiating tunnel restart because the relay settings changed");
                    self.connect_tunnel()?;
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
                if settings_changed {
                    self.tunnel_command_tx
                        .send(TunnelCommand::AllowLan(allow_lan))
                        .expect("Tunnel state machine has stopped");
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

    fn on_set_auto_connect(&mut self, tx: OneshotSender<()>, auto_connect: bool) -> Result<()> {
        let save_result = self.settings.set_auto_connect(auto_connect);
        match save_result.chain_err(|| "Unable to save settings") {
            Ok(_settings_changed) => Self::oneshot_send(tx, (), "set auto-connect response"),
            Err(e) => error!("{}", e.display_chain()),
        }
        Ok(())
    }

    fn on_get_auto_connect(&self, tx: OneshotSender<bool>) {
        Self::oneshot_send(
            tx,
            self.settings.get_auto_connect(),
            "get auto-connect response",
        )
    }

    fn on_set_openvpn_mssfix(
        &mut self,
        tx: OneshotSender<()>,
        mssfix_arg: Option<u16>,
    ) -> Result<()> {
        let save_result = self.settings.set_openvpn_mssfix(mssfix_arg);
        match save_result.chain_err(|| "Unable to save settings") {
            Ok(_) => Self::oneshot_send(tx, (), "set_openvpn_mssfix response"),
            Err(e) => error!("{}", e.display_chain()),
        };
        Ok(())
    }

    fn on_set_openvpn_enable_ipv6(
        &mut self,
        tx: OneshotSender<()>,
        enable_ipv6: bool,
    ) -> Result<()> {
        let save_result = self.settings.set_openvpn_enable_ipv6(enable_ipv6);

        match save_result.chain_err(|| "Unable to save settings") {
            Ok(settings_changed) => {
                Self::oneshot_send(tx, (), "set_openvpn_enable_ipv6 response");

                if settings_changed {
                    info!("Initiating tunnel restart because the enable IPv6 setting changed");
                    self.connect_tunnel()?;
                }
            }
            Err(e) => error!("{}", e.display_chain()),
        };

        Ok(())
    }

    fn on_get_tunnel_options(&self, tx: OneshotSender<TunnelOptions>) -> Result<()> {
        let tunnel_options = self.settings.get_tunnel_options().clone();
        Self::oneshot_send(tx, tunnel_options, "get_tunnel_options response");
        Ok(())
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
        self.state.shutdown(self.tunnel_state);
        self.disconnect_tunnel();

        Ok(())
    }

    fn broadcast_state(&mut self) {
        let new_daemon_state = DaemonState {
            state: self.security_state,
            target_state: self.target_state,
        };
        if self.last_broadcasted_state != new_daemon_state {
            self.last_broadcasted_state = new_daemon_state;
            self.management_interface_broadcaster
                .notify_new_state(new_daemon_state);
        }
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
        match self.target_state {
            TargetState::Secured => {
                debug!("Triggering tunnel start");
                if let Err(e) = self.connect_tunnel().chain_err(|| "Failed to start tunnel") {
                    error!("{}", e.display_chain());
                    self.current_relay = None;
                    self.management_interface_broadcaster.notify_error(&e);
                    self.set_target_state(TargetState::Unsecured)?;
                }
            }
            TargetState::Unsecured => self.disconnect_tunnel(),
        }

        Ok(())
    }

    fn connect_tunnel(&mut self) -> Result<()> {
        let parameters = self.build_tunnel_parameters()?;

        self.tunnel_command_tx
            .send(TunnelCommand::Connect(parameters))
            .expect("Tunnel state machine has stopped");

        Ok(())
    }

    fn disconnect_tunnel(&mut self) {
        self.tunnel_command_tx
            .send(TunnelCommand::Disconnect)
            .expect("Tunnel state machine has stopped");
    }

    fn build_tunnel_parameters(&mut self) -> Result<TunnelParameters> {
        let endpoint = match self.settings.get_relay_settings() {
            RelaySettings::CustomTunnelEndpoint(custom_relay) => custom_relay
                .to_tunnel_endpoint()
                .chain_err(|| ErrorKind::NoRelay)?,
            RelaySettings::Normal(constraints) => {
                let (relay, tunnel_endpoint) = self
                    .relay_selector
                    .get_tunnel_endpoint(&constraints)
                    .chain_err(|| ErrorKind::NoRelay)?;
                self.current_relay = Some(relay);
                tunnel_endpoint
            }
        };

        let account_token = self
            .settings
            .get_account_token()
            .ok_or(ErrorKind::InvalidSettings("No account token"))?;

        Ok(TunnelParameters {
            endpoint,
            options: self.settings.get_tunnel_options().clone(),
            log_dir: self.log_dir.clone(),
            resource_dir: self.resource_dir.clone(),
            account_token,
            allow_lan: self.settings.get_allow_lan(),
        })
    }

    pub fn shutdown_handle(&self) -> DaemonShutdownHandle {
        DaemonShutdownHandle {
            tx: self.tx.clone(),
        }
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
        if let Err(e) =
            rpc_address_file::remove().chain_err(|| "Unable to clean up rpc address file")
        {
            error!("{}", e.display_chain());
        }
    }
}


fn main() {
    let exit_code = match run() {
        Ok(_) => 0,
        Err(error) => {
            if let &ErrorKind::LogError(_) = error.kind() {
                eprintln!("{}", error.display_chain());
            } else {
                error!("{}", error.display_chain());
            }
            1
        }
    };
    debug!("Process exiting with code {}", exit_code);
    ::std::process::exit(exit_code);
}

fn run() -> Result<()> {
    let config = cli::get_config();
    let log_dir = if config.log_to_file {
        Some(
            mullvad_paths::log_dir()
                .chain_err(|| ErrorKind::LogError("Unable to get log directory"))?,
        )
    } else {
        None
    };
    let log_file = log_dir.as_ref().map(|dir| dir.join(DAEMON_LOG_FILENAME));

    logging::init_logger(
        config.log_level,
        log_file.as_ref(),
        config.log_stdout_timestamps,
    ).chain_err(|| ErrorKind::LogError("Unable to initialize logger"))?;
    log_panics::init();
    log_version();
    if let Some(ref log_dir) = log_dir {
        info!("Logging to {}", log_dir.display());
    }

    run_platform(config)
}

#[cfg(windows)]
fn run_platform(config: cli::Config) -> Result<()> {
    if config.run_as_service {
        system_service::run()
    } else {
        if config.register_service {
            let install_result =
                system_service::install_service().chain_err(|| "Unable to install the service");
            if install_result.is_ok() {
                println!("Installed the service.");
            }
            install_result
        } else {
            run_standalone(config)
        }
    }
}

#[cfg(not(windows))]
fn run_platform(config: cli::Config) -> Result<()> {
    run_standalone(config)
}

fn run_standalone(config: cli::Config) -> Result<()> {
    if !running_as_admin() {
        warn!("Running daemon as a non-administrator user, clients might refuse to connect");
    }

    let daemon = create_daemon(config)?;

    let shutdown_handle = daemon.shutdown_handle();
    shutdown::set_shutdown_signal_handler(move || shutdown_handle.shutdown())
        .chain_err(|| "Unable to attach shutdown signal handler")?;

    daemon.run()?;

    info!("Mullvad daemon is quitting");
    thread::sleep(Duration::from_millis(500));
    Ok(())
}

fn create_daemon(config: cli::Config) -> Result<Daemon> {
    let log_dir = if config.log_to_file {
        Some(mullvad_paths::log_dir().chain_err(|| "Unable to get log directory")?)
    } else {
        None
    };
    let resource_dir = mullvad_paths::get_resource_dir();
    let cache_dir = mullvad_paths::cache_dir().chain_err(|| "Unable to get cache dir")?;

    Daemon::new(log_dir, resource_dir, cache_dir).chain_err(|| "Unable to initialize daemon")
}

fn log_version() {
    info!(
        "Starting {} - {} {}",
        env!("CARGO_PKG_NAME"),
        version::CURRENT,
        version::COMMIT_DATE,
    )
}

#[cfg(unix)]
fn running_as_admin() -> bool {
    let uid = unsafe { libc::getuid() };
    uid == 0
}

#[cfg(windows)]
fn running_as_admin() -> bool {
    // TODO: Check if user is administrator correctly on Windows.
    true
}

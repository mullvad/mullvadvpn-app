//! # License
//!
//! Copyright (C) 2017  Amagicom AB
//!
//! This program is free software: you can redistribute it and/or modify it under the terms of the
//! GNU General Public License as published by the Free Software Foundation, either version 3 of
//! the License, or (at your option) any later version.

#![deny(rust_2018_idioms)]

#[macro_use]
extern crate serde;


mod account_history;
mod geoip;
pub mod logging;
mod management_interface;
mod relays;
mod rpc_uniqueness_check;
pub mod version;

pub use crate::management_interface::ManagementCommand;
use crate::management_interface::{
    BoxFuture, ManagementInterfaceEventBroadcaster, ManagementInterfaceServer,
};
use futures::{
    future,
    sync::{mpsc::UnboundedSender, oneshot},
    Future, Sink,
};
use log::{debug, error, info, warn};
use mullvad_rpc::{AccountsProxy, AppVersionProxy, HttpHandle, WireguardKeyProxy};
use mullvad_types::{
    account::{AccountData, AccountToken},
    endpoint::MullvadEndpoint,
    location::GeoIpLocation,
    relay_constraints::{
        BridgeSettings, BridgeState, Constraint, InternalBridgeConstraints, OpenVpnConstraints,
        RelayConstraintsUpdate, RelaySettings, RelaySettingsUpdate, TunnelConstraints,
    },
    relay_list::{Relay, RelayList},
    settings::{self, Settings},
    states::TargetState,
    version::{AppVersion, AppVersionInfo},
};
use std::{io, mem, path::PathBuf, sync::mpsc, thread, time::Duration};
use talpid_core::{
    mpsc::IntoSender,
    tunnel_state_machine::{self, TunnelCommand, TunnelParametersGenerator},
};
use talpid_types::{
    net::{openvpn, wireguard, TransportProtocol, TunnelParameters},
    tunnel::{BlockReason, TunnelStateTransition},
    ErrorExt,
};

pub type Result<T> = std::result::Result<T, Error>;

#[derive(err_derive::Error, Debug)]
pub enum Error {
    #[error(display = "Another instance of the daemon is already running")]
    DaemonIsAlreadyRunning,

    #[error(display = "Failed to send command to daemon because it is not running")]
    DaemonUnavailable,

    #[error(display = "Unable to initialize network event loop")]
    InitIoEventLoop(#[error(cause)] io::Error),

    #[error(display = "Unable to create RPC client")]
    InitRpcClient(#[error(cause)] mullvad_rpc::HttpError),

    #[error(display = "Unable to create am.i.mullvad client")]
    InitHttpsClient(#[error(cause)] mullvad_rpc::rest::Error),

    #[error(display = "Unable to load settings")]
    LoadSettings(#[error(cause)] settings::Error),

    #[error(display = "Unable to load account history with wireguard key cache")]
    LoadAccountHistory(#[error(cause)] account_history::Error),

    /// Error in the management interface
    #[error(display = "Unable to start management interface server")]
    StartManagementInterface(#[error(cause)] talpid_ipc::Error),

    #[error(display = "Management interface server exited unexpectedly")]
    ManagementInterfaceExited,

    #[error(display = "No wireguard private key available")]
    NoKeyAvailable,

    #[error(display = "No bridge available")]
    NoBridgeAvailable,

    #[error(display = "Account history problems")]
    AccountHistory(#[error(cause)] account_history::Error),

    #[error(display = "Tunnel state machine error")]
    TunnelError(#[error(cause)] tunnel_state_machine::Error),
}

type SyncUnboundedSender<T> = ::futures::sink::Wait<UnboundedSender<T>>;

/// All events that can happen in the daemon. Sent from various threads and exposed interfaces.
enum InternalDaemonEvent {
    /// Tunnel has changed state.
    TunnelStateTransition(TunnelStateTransition),
    /// Request from the `MullvadTunnelParametersGenerator` to obtain a new relay.
    GenerateTunnelParameters(mpsc::Sender<TunnelParameters>, u32),
    /// An event coming from the JSONRPC-2.0 management interface.
    ManagementInterfaceEvent(ManagementCommand),
    /// Triggered if the server hosting the JSONRPC-2.0 management interface dies unexpectedly.
    ManagementInterfaceExited,
    /// Daemon shutdown triggered by a signal, ctrl-c or similar.
    TriggerShutdown,
}

impl From<TunnelStateTransition> for InternalDaemonEvent {
    fn from(tunnel_state_transition: TunnelStateTransition) -> Self {
        InternalDaemonEvent::TunnelStateTransition(tunnel_state_transition)
    }
}

impl From<ManagementCommand> for InternalDaemonEvent {
    fn from(command: ManagementCommand) -> Self {
        InternalDaemonEvent::ManagementInterfaceEvent(command)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum DaemonExecutionState {
    Running,
    Exiting,
    Finished,
}

impl DaemonExecutionState {
    pub fn shutdown(&mut self, tunnel_state: &TunnelStateTransition) {
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

    pub fn is_running(&self) -> bool {
        use self::DaemonExecutionState::*;

        match self {
            Running => true,
            Exiting | Finished => false,
        }
    }
}

pub struct DaemonCommandSender(IntoSender<ManagementCommand, InternalDaemonEvent>);

impl DaemonCommandSender {
    pub(crate) fn new(internal_event_sender: mpsc::Sender<InternalDaemonEvent>) -> Self {
        DaemonCommandSender(IntoSender::from(internal_event_sender))
    }

    pub fn send(&self, command: ManagementCommand) -> Result<()> {
        self.0.send(command).map_err(|_| Error::DaemonUnavailable)
    }
}

/// Trait representing something that can broadcast daemon events.
pub trait EventListener {
    /// Notify that the tunnel state changed.
    fn notify_new_state(&self, new_state: TunnelStateTransition);

    /// Notify that the settings changed.
    fn notify_settings(&self, settings: Settings);

    /// Notify that the relay list changed.
    fn notify_relay_list(&self, relay_list: RelayList);
}

pub struct Daemon<L: EventListener = ManagementInterfaceEventBroadcaster> {
    tunnel_command_tx: SyncUnboundedSender<TunnelCommand>,
    tunnel_state: TunnelStateTransition,
    target_state: TargetState,
    state: DaemonExecutionState,
    rx: mpsc::Receiver<InternalDaemonEvent>,
    tx: mpsc::Sender<InternalDaemonEvent>,
    reconnection_loop_tx: Option<mpsc::Sender<()>>,
    event_listener: L,
    settings: Settings,
    account_history: account_history::AccountHistory,
    wg_key_proxy: WireguardKeyProxy<HttpHandle>,
    accounts_proxy: AccountsProxy<HttpHandle>,
    version_proxy: AppVersionProxy<HttpHandle>,
    https_handle: mullvad_rpc::rest::RequestSender,
    tokio_remote: tokio_core::reactor::Remote,
    relay_selector: relays::RelaySelector,
    last_generated_relay: Option<Relay>,
    version: String,
}

impl Daemon<ManagementInterfaceEventBroadcaster> {
    pub fn start(
        log_dir: Option<PathBuf>,
        resource_dir: PathBuf,
        cache_dir: PathBuf,
        version: String,
    ) -> Result<Self> {
        if rpc_uniqueness_check::is_another_instance_running() {
            return Err(Error::DaemonIsAlreadyRunning);
        }
        let (tx, rx) = mpsc::channel();
        let management_interface_broadcaster = Self::start_management_interface(tx.clone())?;

        Self::start_internal(
            tx,
            rx,
            management_interface_broadcaster,
            log_dir,
            resource_dir,
            cache_dir,
            version,
        )
    }

    // Starts the management interface and spawns a thread that will process it.
    // Returns a handle that allows notifying all subscribers on events.
    fn start_management_interface(
        event_tx: mpsc::Sender<InternalDaemonEvent>,
    ) -> Result<ManagementInterfaceEventBroadcaster> {
        let multiplex_event_tx = IntoSender::from(event_tx.clone());
        let server = Self::start_management_interface_server(multiplex_event_tx)?;
        let event_broadcaster = server.event_broadcaster();
        Self::spawn_management_interface_wait_thread(server, event_tx);
        Ok(event_broadcaster)
    }

    fn start_management_interface_server(
        event_tx: IntoSender<ManagementCommand, InternalDaemonEvent>,
    ) -> Result<ManagementInterfaceServer> {
        let server =
            ManagementInterfaceServer::start(event_tx).map_err(Error::StartManagementInterface)?;
        info!("Management interface listening on {}", server.socket_path());

        Ok(server)
    }

    fn spawn_management_interface_wait_thread(
        server: ManagementInterfaceServer,
        exit_tx: mpsc::Sender<InternalDaemonEvent>,
    ) {
        thread::spawn(move || {
            server.wait();
            info!("Management interface shut down");
            let _ = exit_tx.send(InternalDaemonEvent::ManagementInterfaceExited);
        });
    }
}

impl<L> Daemon<L>
where
    L: EventListener + Clone + Send + 'static,
{
    pub fn start_with_event_listener(
        event_listener: L,
        log_dir: Option<PathBuf>,
        resource_dir: PathBuf,
        cache_dir: PathBuf,
        version: String,
    ) -> Result<Self> {
        let (tx, rx) = mpsc::channel();

        Self::start_internal(
            tx,
            rx,
            event_listener,
            log_dir,
            resource_dir,
            cache_dir,
            version,
        )
    }

    fn start_internal(
        internal_event_tx: mpsc::Sender<InternalDaemonEvent>,
        internal_event_rx: mpsc::Receiver<InternalDaemonEvent>,
        event_listener: L,
        log_dir: Option<PathBuf>,
        resource_dir: PathBuf,
        cache_dir: PathBuf,
        version: String,
    ) -> Result<Self> {
        let ca_path = resource_dir.join(mullvad_paths::resources::API_CA_FILENAME);

        let mut rpc_manager = mullvad_rpc::MullvadRpcFactory::with_cache_dir(&cache_dir, &ca_path);

        let (rpc_handle, https_handle, tokio_remote) =
            mullvad_rpc::event_loop::create(move |core| {
                let handle = core.handle();
                let rpc = rpc_manager.new_connection_on_event_loop(&handle);
                let https_handle = mullvad_rpc::rest::create_https_client(&ca_path, &handle);
                let remote = core.remote();
                (rpc, https_handle, remote)
            })
            .map_err(Error::InitIoEventLoop)?;
        let rpc_handle = rpc_handle.map_err(Error::InitRpcClient)?;
        let https_handle = https_handle.map_err(Error::InitHttpsClient)?;

        let relay_list_listener = event_listener.clone();
        let on_relay_list_update = move |relay_list: &RelayList| {
            relay_list_listener.notify_relay_list(relay_list.clone());
        };

        let relay_selector = relays::RelaySelector::new(
            rpc_handle.clone(),
            on_relay_list_update,
            &resource_dir,
            &cache_dir,
        );
        let settings = Settings::load().map_err(Error::LoadSettings)?;
        let account_history =
            account_history::AccountHistory::new(&cache_dir).map_err(Error::LoadAccountHistory)?;

        let tunnel_parameters_generator = MullvadTunnelParametersGenerator {
            tx: internal_event_tx.clone(),
        };
        let tunnel_command_tx = tunnel_state_machine::spawn(
            settings.get_allow_lan(),
            settings.get_block_when_disconnected(),
            tunnel_parameters_generator,
            log_dir,
            resource_dir,
            cache_dir.clone(),
            IntoSender::from(internal_event_tx.clone()),
        )
        .map_err(Error::TunnelError)?;

        // Attempt to download a fresh relay list
        relay_selector.update();

        Ok(Daemon {
            tunnel_command_tx: Sink::wait(tunnel_command_tx),
            tunnel_state: TunnelStateTransition::Disconnected,
            target_state: TargetState::Unsecured,
            state: DaemonExecutionState::Running,
            rx: internal_event_rx,
            tx: internal_event_tx,
            reconnection_loop_tx: None,
            event_listener,
            settings,
            account_history,
            wg_key_proxy: WireguardKeyProxy::new(rpc_handle.clone()),
            accounts_proxy: AccountsProxy::new(rpc_handle.clone()),
            version_proxy: AppVersionProxy::new(rpc_handle),
            https_handle,
            tokio_remote,
            relay_selector,
            last_generated_relay: None,
            version,
        })
    }

    /// Retrieve a channel for sending daemon commands.
    pub fn command_sender(&self) -> DaemonCommandSender {
        DaemonCommandSender::new(self.tx.clone())
    }

    /// Consume the `Daemon` and run the main event loop. Blocks until an error happens or a
    /// shutdown event is received.
    pub fn run(mut self) -> Result<()> {
        if self.settings.get_auto_connect() && self.settings.get_account_token().is_some() {
            info!("Automatically connecting since auto-connect is turned on");
            self.set_target_state(TargetState::Secured);
        }
        while let Ok(event) = self.rx.recv() {
            self.handle_event(event)?;
            if self.state == DaemonExecutionState::Finished {
                break;
            }
        }
        Ok(())
    }

    fn handle_event(&mut self, event: InternalDaemonEvent) -> Result<()> {
        use self::InternalDaemonEvent::*;
        match event {
            TunnelStateTransition(transition) => self.handle_tunnel_state_transition(transition),
            GenerateTunnelParameters(tunnel_parameters_tx, retry_attempt) => {
                self.handle_generate_tunnel_parameters(&tunnel_parameters_tx, retry_attempt)
            }
            ManagementInterfaceEvent(event) => self.handle_management_interface_event(event),
            ManagementInterfaceExited => {
                return Err(Error::ManagementInterfaceExited);
            }
            TriggerShutdown => self.handle_trigger_shutdown_event(),
        }
        Ok(())
    }

    fn handle_tunnel_state_transition(&mut self, tunnel_state: TunnelStateTransition) {
        use self::TunnelStateTransition::*;

        self.unschedule_reconnect();

        debug!("New tunnel state: {:?}", tunnel_state);
        match tunnel_state {
            Disconnected => self.state.disconnected(),
            Blocked(ref reason) => {
                info!("Blocking all network connections, reason: {}", reason);

                if let BlockReason::AuthFailed(_) = reason {
                    self.schedule_reconnect(Duration::from_secs(60))
                }
            }
            _ => {}
        }

        self.tunnel_state = tunnel_state.clone();
        self.event_listener.notify_new_state(tunnel_state);
    }

    fn handle_generate_tunnel_parameters(
        &mut self,
        tunnel_parameters_tx: &mpsc::Sender<TunnelParameters>,
        retry_attempt: u32,
    ) {
        if let Some(account_token) = self.settings.get_account_token() {
            if let Err(error_str) = match self.settings.get_relay_settings() {
                RelaySettings::CustomTunnelEndpoint(custom_relay) => {
                    self.last_generated_relay = None;
                    custom_relay
                        .to_tunnel_parameters(self.settings.get_tunnel_options().clone())
                        .map_err(|e| {
                            e.display_chain_with_msg("Custom tunnel endpoint could not be resolved")
                        })
                }
                RelaySettings::Normal(constraints) => self
                    .relay_selector
                    .get_tunnel_endpoint(&constraints, retry_attempt)
                    .map_err(|e| {
                        e.display_chain_with_msg(
                            "No valid relay servers match the current settings",
                        )
                    })
                    .and_then(|(relay, endpoint)| {
                        let result = self
                            .create_tunnel_parameters(
                                &relay,
                                endpoint,
                                account_token,
                                retry_attempt,
                            )
                            .map_err(|e| e.display_chain());
                        self.last_generated_relay = Some(relay);
                        result
                    }),
            }
            .and_then(|tunnel_params| {
                tunnel_parameters_tx.send(tunnel_params).map_err(|e| {
                    e.display_chain_with_msg("Tunnel parameters receiver stopped listening")
                })
            }) {
                error!("{}", error_str);
            }
        } else {
            error!("No account token configured");
        }
    }

    fn create_tunnel_parameters(
        &mut self,
        relay: &Relay,
        endpoint: MullvadEndpoint,
        account_token: String,
        retry_attempt: u32,
    ) -> Result<TunnelParameters> {
        let mut tunnel_options = self.settings.get_tunnel_options().clone();
        let location = relay.location.as_ref().expect("Relay has no location set");
        match endpoint {
            MullvadEndpoint::OpenVpn(endpoint) => {
                let proxy_settings = match self.settings.get_bridge_settings() {
                    BridgeSettings::Normal(settings) => {
                        let bridge_constraints = InternalBridgeConstraints {
                            location: settings.location.clone(),
                            transport_protocol: Constraint::Only(endpoint.protocol),
                        };
                        match self.settings.get_bridge_state() {
                            BridgeState::On => Some(
                                self.relay_selector
                                    .get_proxy_settings(&bridge_constraints, location)
                                    .ok_or(Error::NoBridgeAvailable)?,
                            ),
                            BridgeState::Auto => self.relay_selector.get_auto_proxy_settings(
                                &bridge_constraints,
                                location,
                                retry_attempt,
                            ),
                            BridgeState::Off => None,
                        }
                    }
                    BridgeSettings::Custom(proxy_settings) => {
                        match self.settings.get_bridge_state() {
                            BridgeState::On => Some(proxy_settings.clone()),
                            BridgeState::Auto => {
                                if self.relay_selector.should_use_bridge(retry_attempt) {
                                    Some(proxy_settings.clone())
                                } else {
                                    None
                                }
                            }
                            BridgeState::Off => None,
                        }
                    }
                };
                tunnel_options.openvpn.proxy = proxy_settings;

                Ok(openvpn::TunnelParameters {
                    config: openvpn::ConnectionConfig::new(
                        endpoint,
                        account_token,
                        "-".to_string(),
                    ),
                    options: tunnel_options.openvpn,
                    generic_options: tunnel_options.generic,
                }
                .into())
            }
            MullvadEndpoint::Wireguard {
                peer,
                ipv4_gateway,
                ipv6_gateway,
            } => {
                let wg_data = self
                    .account_history
                    .get(&account_token)
                    .map_err(Error::AccountHistory)?
                    .and_then(|entry| entry.wireguard)
                    .ok_or(Error::NoKeyAvailable)?;
                let tunnel = wireguard::TunnelConfig {
                    private_key: wg_data.private_key,
                    addresses: vec![
                        wg_data.addresses.ipv4_address.ip().into(),
                        wg_data.addresses.ipv6_address.ip().into(),
                    ],
                };
                Ok(wireguard::TunnelParameters {
                    connection: wireguard::ConnectionConfig {
                        tunnel,
                        peer,
                        ipv4_gateway,
                        ipv6_gateway: Some(ipv6_gateway),
                    },
                    options: tunnel_options.wireguard,
                    generic_options: tunnel_options.generic,
                }
                .into())
            }
        }
    }

    fn schedule_reconnect(&mut self, delay: Duration) {
        let tunnel_command_tx = self.tx.clone();
        let (tx, rx) = mpsc::channel();

        self.reconnection_loop_tx = Some(tx);

        thread::spawn(move || {
            let (result_tx, _result_rx) = oneshot::channel();

            if let Err(mpsc::RecvTimeoutError::Timeout) = rx.recv_timeout(delay) {
                debug!("Attempting to reconnect");
                let _ = tunnel_command_tx.send(InternalDaemonEvent::ManagementInterfaceEvent(
                    ManagementCommand::SetTargetState(result_tx, TargetState::Secured),
                ));
            }
        });
    }

    fn unschedule_reconnect(&mut self) {
        if let Some(tx) = self.reconnection_loop_tx.take() {
            let _ = tx.send(());
        }
    }

    fn handle_management_interface_event(&mut self, event: ManagementCommand) {
        use self::ManagementCommand::*;
        match event {
            SetTargetState(tx, state) => self.on_set_target_state(tx, state),
            GetState(tx) => self.on_get_state(tx),
            GetCurrentLocation(tx) => self.on_get_current_location(tx),
            GetAccountData(tx, account_token) => self.on_get_account_data(tx, account_token),
            GetRelayLocations(tx) => self.on_get_relay_locations(tx),
            UpdateRelayLocations => self.on_update_relay_locations(),
            SetAccount(tx, account_token) => self.on_set_account(tx, account_token),
            GetAccountHistory(tx) => self.on_get_account_history(tx),
            RemoveAccountFromHistory(tx, account_token) => {
                self.on_remove_account_from_history(tx, account_token)
            }
            UpdateRelaySettings(tx, update) => self.on_update_relay_settings(tx, update),
            SetAllowLan(tx, allow_lan) => self.on_set_allow_lan(tx, allow_lan),
            SetBlockWhenDisconnected(tx, block_when_disconnected) => {
                self.on_set_block_when_disconnected(tx, block_when_disconnected)
            }
            SetAutoConnect(tx, auto_connect) => self.on_set_auto_connect(tx, auto_connect),
            SetOpenVpnMssfix(tx, mssfix_arg) => self.on_set_openvpn_mssfix(tx, mssfix_arg),
            SetOpenVpnProxy(tx, proxy) => self.on_set_openvpn_proxy(tx, proxy),
            SetEnableIpv6(tx, enable_ipv6) => self.on_set_enable_ipv6(tx, enable_ipv6),
            SetWireguardMtu(tx, mtu) => self.on_set_wireguard_mtu(tx, mtu),
            GetSettings(tx) => self.on_get_settings(tx),
            GenerateWireguardKey(tx) => self.on_generate_wireguard_key(tx),
            GetWireguardKey(tx) => self.on_get_wireguard_key(tx),
            VerifyWireguardKey(tx) => self.on_verify_wireguard_key(tx),
            GetVersionInfo(tx) => self.on_get_version_info(tx),
            GetCurrentVersion(tx) => self.on_get_current_version(tx),
            Shutdown => self.handle_trigger_shutdown_event(),
        }
    }

    fn on_set_target_state(
        &mut self,
        tx: oneshot::Sender<::std::result::Result<(), ()>>,
        new_target_state: TargetState,
    ) {
        if self.state.is_running() {
            self.set_target_state(new_target_state);
        } else {
            warn!("Ignoring target state change request due to shutdown");
        }
        Self::oneshot_send(tx, Ok(()), "target state");
    }

    fn on_get_state(&self, tx: oneshot::Sender<TunnelStateTransition>) {
        Self::oneshot_send(tx, self.tunnel_state.clone(), "current state");
    }

    fn on_get_current_location(&self, tx: oneshot::Sender<Option<GeoIpLocation>>) {
        use self::TunnelStateTransition::*;
        let get_location: Box<dyn Future<Item = Option<GeoIpLocation>, Error = ()> + Send> =
            match self.tunnel_state {
                Disconnected => Box::new(self.get_geo_location().map(Some)),
                Connecting(_) | Disconnecting(..) => match self.build_location_from_relay() {
                    Some(relay_location) => Box::new(future::result(Ok(Some(relay_location)))),
                    // Custom relay is set, no location is known
                    None => Box::new(future::result(Ok(None))),
                },
                Connected(_) => match self.build_location_from_relay() {
                    Some(location_from_relay) => Box::new(
                        self.get_geo_location()
                            .map(|fetched_location| GeoIpLocation {
                                ipv4: fetched_location.ipv4,
                                ipv6: fetched_location.ipv6,
                                ..location_from_relay
                            })
                            .map(Some),
                    ),
                    // Custom relay is set, no location is known intrinsicly
                    None => Box::new(self.get_geo_location().map(Some)),
                },
                Blocked(..) => {
                    // We are not online at all at this stage so no location data is available.
                    Box::new(future::result(Ok(None)))
                }
            };

        self.tokio_remote.spawn(move |_| {
            get_location.map(|location| Self::oneshot_send(tx, location, "current location"))
        });
    }

    fn get_geo_location(&self) -> impl Future<Item = GeoIpLocation, Error = ()> {
        let https_handle = self.https_handle.clone();

        geoip::send_location_request(https_handle).map_err(|e| {
            warn!("Unable to fetch GeoIP location: {}", e.display_chain());
        })
    }

    fn build_location_from_relay(&self) -> Option<GeoIpLocation> {
        let relay = self.last_generated_relay.as_ref()?;
        let location = relay.location.as_ref().cloned().unwrap();
        let hostname = relay.hostname.clone();

        Some(GeoIpLocation {
            ipv4: None,
            ipv6: None,
            country: location.country,
            city: Some(location.city),
            latitude: location.latitude,
            longitude: location.longitude,
            mullvad_exit_ip: true,
            hostname: Some(hostname),
        })
    }

    fn on_get_account_data(
        &mut self,
        tx: oneshot::Sender<BoxFuture<AccountData, mullvad_rpc::Error>>,
        account_token: AccountToken,
    ) {
        let rpc_call = self
            .accounts_proxy
            .get_expiry(account_token)
            .map(|expiry| AccountData { expiry });
        Self::oneshot_send(tx, Box::new(rpc_call), "account data")
    }

    fn on_get_relay_locations(&mut self, tx: oneshot::Sender<RelayList>) {
        Self::oneshot_send(tx, self.relay_selector.get_locations(), "relay locations");
    }

    fn on_update_relay_locations(&mut self) {
        self.relay_selector.update();
    }

    fn on_set_account(&mut self, tx: oneshot::Sender<()>, account_token: Option<String>) {
        let save_result = self.settings.set_account_token(account_token.clone());

        match save_result {
            Ok(account_changed) => {
                Self::oneshot_send(tx, (), "set_account response");
                if account_changed {
                    self.event_listener.notify_settings(self.settings.clone());
                    match account_token {
                        Some(token) => {
                            if let Err(e) = self.account_history.bump_history(&token) {
                                log::error!("Failed to bump account history: {}", e);
                            }
                            info!("Initiating tunnel restart because the account token changed");
                            self.reconnect_tunnel();
                        }
                        None => {
                            info!("Disconnecting because account token was cleared");
                            self.set_target_state(TargetState::Unsecured);
                        }
                    }
                }
            }
            Err(e) => error!("{}", e.display_chain_with_msg("Unable to save settings")),
        }
    }

    fn on_get_account_history(&mut self, tx: oneshot::Sender<Vec<AccountToken>>) {
        Self::oneshot_send(
            tx,
            self.account_history.get_account_history(),
            "get_account_history response",
        );
    }

    fn on_remove_account_from_history(
        &mut self,
        tx: oneshot::Sender<()>,
        account_token: AccountToken,
    ) {
        if let Ok(_) = self.account_history.remove_account(&account_token) {
            Self::oneshot_send(tx, (), "remove_account_from_history response");
        }
    }

    fn on_get_version_info(
        &mut self,
        tx: oneshot::Sender<BoxFuture<AppVersionInfo, mullvad_rpc::Error>>,
    ) {
        #[cfg(target_os = "linux")]
        const PLATFORM: &str = "linux";
        #[cfg(target_os = "macos")]
        const PLATFORM: &str = "macos";
        #[cfg(target_os = "windows")]
        const PLATFORM: &str = "windows";
        #[cfg(target_os = "android")]
        const PLATFORM: &str = "android";

        let fut = self
            .version_proxy
            .app_version_check(&self.version, PLATFORM);
        Self::oneshot_send(tx, Box::new(fut), "get_version_info response");
    }

    fn on_get_current_version(&mut self, tx: oneshot::Sender<AppVersion>) {
        Self::oneshot_send(tx, self.version.clone(), "get_current_version response");
    }

    fn on_update_relay_settings(&mut self, tx: oneshot::Sender<()>, update: RelaySettingsUpdate) {
        let save_result = self.settings.update_relay_settings(update);
        match save_result {
            Ok(settings_changed) => {
                Self::oneshot_send(tx, (), "update_relay_settings response");
                if settings_changed {
                    self.event_listener.notify_settings(self.settings.clone());
                    info!("Initiating tunnel restart because the relay settings changed");
                    self.reconnect_tunnel();
                }
            }
            Err(e) => error!("{}", e.display_chain_with_msg("Unable to save settings")),
        }
    }

    fn on_set_allow_lan(&mut self, tx: oneshot::Sender<()>, allow_lan: bool) {
        let save_result = self.settings.set_allow_lan(allow_lan);
        match save_result {
            Ok(settings_changed) => {
                Self::oneshot_send(tx, (), "set_allow_lan response");
                if settings_changed {
                    self.event_listener.notify_settings(self.settings.clone());
                    self.send_tunnel_command(TunnelCommand::AllowLan(allow_lan));
                }
            }
            Err(e) => error!("{}", e.display_chain_with_msg("Unable to save settings")),
        }
    }

    fn on_set_block_when_disconnected(
        &mut self,
        tx: oneshot::Sender<()>,
        block_when_disconnected: bool,
    ) {
        let save_result = self
            .settings
            .set_block_when_disconnected(block_when_disconnected);
        match save_result {
            Ok(settings_changed) => {
                Self::oneshot_send(tx, (), "set_block_when_disconnected response");
                if settings_changed {
                    self.event_listener.notify_settings(self.settings.clone());
                    self.send_tunnel_command(TunnelCommand::BlockWhenDisconnected(
                        block_when_disconnected,
                    ));
                }
            }
            Err(e) => error!("{}", e.display_chain_with_msg("Unable to save settings")),
        }
    }

    fn on_set_auto_connect(&mut self, tx: oneshot::Sender<()>, auto_connect: bool) {
        let save_result = self.settings.set_auto_connect(auto_connect);
        match save_result {
            Ok(settings_changed) => {
                Self::oneshot_send(tx, (), "set auto-connect response");
                if settings_changed {
                    self.event_listener.notify_settings(self.settings.clone());
                }
            }
            Err(e) => error!("{}", e.display_chain_with_msg("Unable to save settings")),
        }
    }

    fn on_set_openvpn_mssfix(&mut self, tx: oneshot::Sender<()>, mssfix_arg: Option<u16>) {
        let save_result = self.settings.set_openvpn_mssfix(mssfix_arg);
        match save_result {
            Ok(settings_changed) => {
                Self::oneshot_send(tx, (), "set_openvpn_mssfix response");
                if settings_changed {
                    self.event_listener.notify_settings(self.settings.clone());
                    info!("Initiating tunnel restart because the OpenVPN mssfix setting changed");
                    self.reconnect_tunnel();
                }
            }
            Err(e) => error!("{}", e.display_chain_with_msg("Unable to save settings")),
        }
    }

    fn on_set_openvpn_proxy(
        &mut self,
        tx: oneshot::Sender<::std::result::Result<(), settings::Error>>,
        proxy: Option<openvpn::ProxySettings>,
    ) {
        let constraints_result = match proxy {
            Some(_) => self.apply_proxy_constraints(),
            _ => Ok(false),
        };
        let proxy_result = self.settings.set_openvpn_proxy(proxy);

        match (proxy_result, constraints_result) {
            (Ok(proxy_changed), Ok(constraints_changed)) => {
                Self::oneshot_send(tx, Ok(()), "set_openvpn_proxy response");
                if proxy_changed || constraints_changed {
                    self.event_listener.notify_settings(self.settings.clone());
                    info!("Initiating tunnel restart because the OpenVPN proxy setting changed");
                    self.reconnect_tunnel();
                }
            }
            (Ok(_), Err(error)) | (Err(error), Ok(_)) => {
                error!("{}", error.display_chain());
                Self::oneshot_send(tx, Err(error), "set_openvpn_proxy response");
            }
            (Err(error), Err(_)) => {
                error!("{}", error.display_chain());
                Self::oneshot_send(tx, Err(error), "set_openvpn_proxy response");
            }
        }
    }

    // Set the OpenVPN tunnel to use TCP.
    fn apply_proxy_constraints(&mut self) -> settings::Result<bool> {
        let openvpn_constraints = OpenVpnConstraints {
            port: Constraint::Any,
            protocol: Constraint::Only(TransportProtocol::Tcp),
        };

        let tunnel_constraints = TunnelConstraints::OpenVpn(openvpn_constraints);

        let constraints_update = RelayConstraintsUpdate {
            location: None,
            tunnel: Some(Constraint::Only(tunnel_constraints)),
        };

        let settings_update = RelaySettingsUpdate::Normal(constraints_update);

        self.settings.update_relay_settings(settings_update)
    }

    fn on_set_enable_ipv6(&mut self, tx: oneshot::Sender<()>, enable_ipv6: bool) {
        let save_result = self.settings.set_enable_ipv6(enable_ipv6);
        match save_result {
            Ok(settings_changed) => {
                Self::oneshot_send(tx, (), "set_enable_ipv6 response");
                if settings_changed {
                    self.event_listener.notify_settings(self.settings.clone());
                    info!("Initiating tunnel restart because the enable IPv6 setting changed");
                    self.reconnect_tunnel();
                }
            }
            Err(e) => error!("{}", e.display_chain_with_msg("Unable to save settings")),
        }
    }

    fn on_set_wireguard_mtu(&mut self, tx: oneshot::Sender<()>, mtu: Option<u16>) {
        let save_result = self.settings.set_wireguard_mtu(mtu);
        match save_result {
            Ok(settings_changed) => {
                Self::oneshot_send(tx, (), "set_wireguard_mtu response");
                if settings_changed {
                    self.event_listener.notify_settings(self.settings.clone());
                    info!("Initiating tunnel restart because the WireGuard MTU setting changed");
                    self.reconnect_tunnel();
                }
            }
            Err(e) => error!("{}", e.display_chain_with_msg("Unable to save settings")),
        }
    }

    fn on_generate_wireguard_key(
        &mut self,
        tx: oneshot::Sender<::std::result::Result<(), mullvad_rpc::Error>>,
    ) {
        let mut result = || -> ::std::result::Result<(), String> {
            let account_token = self
                .settings
                .get_account_token()
                .ok_or("No account token set".to_string())?;

            let mut account_entry = self
                .account_history
                .get(&account_token)
                .map_err(|e| format!("Failed to read account entry from history: {}", e))
                .map(|data| {
                    data.unwrap_or_else(|| {
                        log::error!("Account token set in settings but not in account history");
                        account_history::AccountEntry {
                            account: account_token.clone(),
                            wireguard: None,
                        }
                    })
                })?;

            let private_key = wireguard::PrivateKey::new_from_random()
                .map_err(|e| format!("Failed to generate new key - {}", e))?;

            let fut = self
                .wg_key_proxy
                .push_wg_key(account_token, private_key.public_key());

            let mut core = tokio_core::reactor::Core::new()
                .map_err(|e| format!("Failed to spawn future for pushing wg key - {}", e))?;

            let addresses = core
                .run(fut)
                .map_err(|e| format!("Failed to push new wireguard key: {}", e))?;

            account_entry.wireguard = Some(mullvad_types::wireguard::WireguardData {
                private_key,
                addresses,
            });

            self.account_history
                .insert(account_entry)
                .map_err(|e| format!("Failed to add new wireguard key to account data: {}", e))
        };
        match result() {
            Ok(()) => {
                Self::oneshot_send(tx, Ok(()), "generate_wireguard_key response");
            }
            Err(e) => {
                log::error!("Failed to generate new wireguard key - {}", e);
            }
        }
    }

    fn on_get_wireguard_key(&mut self, tx: oneshot::Sender<Option<wireguard::PublicKey>>) {
        let key = self
            .settings
            .get_account_token()
            .and_then(|account| self.account_history.get(&account).ok()?)
            .and_then(|account_entry| {
                account_entry
                    .wireguard
                    .map(|wg| wg.private_key.public_key())
            });

        Self::oneshot_send(tx, key, "get_wireguard_key response");
    }

    fn on_verify_wireguard_key(&mut self, tx: oneshot::Sender<bool>) {
        use futures::future::Executor;
        let account = match self.settings.get_account_token() {
            Some(account) => account,
            None => {
                Self::oneshot_send(tx, false, "verify_wireguard_key response");
                return;
            }
        };

        let key = self
            .account_history
            .get(&account)
            .map(|entry| entry.and_then(|e| e.wireguard.map(|wg| wg.private_key.public_key())));

        let public_key = match key {
            Ok(Some(public_key)) => public_key,
            Ok(None) => {
                Self::oneshot_send(tx, false, "verify_wireguard_key response");
                return;
            }
            Err(e) => {
                log::error!("Failed to read key data: {}", e);
                return;
            }
        };

        let fut = self
            .wg_key_proxy
            .check_wg_key(account, public_key.clone())
            .map(|is_valid| {
                Self::oneshot_send(tx, is_valid, "verify_wireguard_key response");
                ()
            })
            .map_err(|e| log::error!("Failed to verify wireguard key - {}", e));
        if let Err(e) = self.tokio_remote.execute(fut) {
            log::error!("Failed to spawn a future to verify wireguard key: {:?}", e);
        }
    }

    fn on_get_settings(&self, tx: oneshot::Sender<Settings>) {
        Self::oneshot_send(tx, self.settings.clone(), "get_settings response");
    }

    fn oneshot_send<T>(tx: oneshot::Sender<T>, t: T, msg: &'static str) {
        if tx.send(t).is_err() {
            warn!("Unable to send {} to management interface client", msg);
        }
    }

    fn handle_trigger_shutdown_event(&mut self) {
        self.state.shutdown(&self.tunnel_state);
        self.disconnect_tunnel();
    }

    /// Set the target state of the client. If it changed trigger the operations needed to
    /// progress towards that state.
    /// Returns an error if trying to set secured state, but no account token is present.
    fn set_target_state(&mut self, new_state: TargetState) {
        if new_state != self.target_state || self.tunnel_state.is_blocked() {
            debug!("Target state {:?} => {:?}", self.target_state, new_state);
            self.target_state = new_state;
            match self.target_state {
                TargetState::Secured => self.connect_tunnel(),
                TargetState::Unsecured => self.disconnect_tunnel(),
            }
        }
    }

    fn connect_tunnel(&mut self) {
        self.send_tunnel_command(TunnelCommand::Connect);
    }

    fn disconnect_tunnel(&mut self) {
        self.send_tunnel_command(TunnelCommand::Disconnect);
    }

    fn reconnect_tunnel(&mut self) {
        if self.target_state == TargetState::Secured {
            self.connect_tunnel();
        }
    }

    fn send_tunnel_command(&mut self, command: TunnelCommand) {
        self.tunnel_command_tx
            .send(command)
            .expect("Tunnel state machine has stopped");
    }

    pub fn shutdown_handle(&self) -> DaemonShutdownHandle {
        DaemonShutdownHandle {
            tx: self.tx.clone(),
        }
    }
}

pub struct DaemonShutdownHandle {
    tx: mpsc::Sender<InternalDaemonEvent>,
}

impl DaemonShutdownHandle {
    pub fn shutdown(&self) {
        let _ = self.tx.send(InternalDaemonEvent::TriggerShutdown);
    }
}

struct MullvadTunnelParametersGenerator {
    tx: mpsc::Sender<InternalDaemonEvent>,
}

impl TunnelParametersGenerator for MullvadTunnelParametersGenerator {
    fn generate(&mut self, retry_attempt: u32) -> Option<TunnelParameters> {
        let (response_tx, response_rx) = mpsc::channel();
        self.tx
            .send(InternalDaemonEvent::GenerateTunnelParameters(
                response_tx,
                retry_attempt,
            ))
            .ok()
            .and_then(|_| response_rx.recv().ok())
    }
}

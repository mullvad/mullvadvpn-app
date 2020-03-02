use crate::{BoxFuture, DaemonCommand, DaemonCommandSender, EventListener};
use jsonrpc_core::{
    futures::{future, sync, Future},
    Error, ErrorCode, MetaIoHandler, Metadata,
};
use jsonrpc_ipc_server;
use jsonrpc_macros::{build_rpc_trait, metadata, pubsub};
use jsonrpc_pubsub::{PubSubHandler, PubSubMetadata, Session, SubscriptionId};
use mullvad_paths;
use mullvad_rpc;
use mullvad_types::{
    account::{AccountData, AccountToken, VoucherSubmission},
    location::GeoIpLocation,
    relay_constraints::{BridgeSettings, BridgeState, RelaySettingsUpdate},
    relay_list::RelayList,
    settings::{self, Settings},
    states::{TargetState, TunnelState},
    version, wireguard, DaemonEvent,
};
use parking_lot::RwLock;
use std::{
    collections::{hash_map::Entry, HashMap},
    sync::Arc,
};
use talpid_ipc;
use talpid_types::ErrorExt;
use uuid;

build_rpc_trait! {
    pub trait ManagementInterfaceApi {
        type Metadata;

        /// Creates and sets a new account
        #[rpc(meta, name = "create_new_account")]
        fn create_new_account(&self, Self::Metadata) -> BoxFuture<String, Error>;

        /// Fetches and returns metadata about an account. Returns an error on non-existing
        /// accounts.
        #[rpc(meta, name = "get_account_data")]
        fn get_account_data(&self, Self::Metadata, AccountToken) -> BoxFuture<AccountData, Error>;

        #[rpc(meta, name = "get_www_auth_token")]
        fn get_www_auth_token(&self, Self::Metadata) -> BoxFuture<String, Error>;

        /// Submit voucher to add time to account
        #[rpc(meta, name = "submit_voucher")]
        fn submit_voucher(&self, Self::Metadata, String) -> BoxFuture<VoucherSubmission, Error>;

        /// Returns available countries.
        #[rpc(meta, name = "get_relay_locations")]
        fn get_relay_locations(&self, Self::Metadata) -> BoxFuture<RelayList, Error>;

        /// Triggers a relay list update
        #[rpc(meta, name = "update_relay_locations")]
        fn update_relay_locations(&self, Self::Metadata) -> BoxFuture<(), Error>;

        /// Set which account to connect with.
        #[rpc(meta, name = "set_account")]
        fn set_account(&self, Self::Metadata, Option<AccountToken>) -> BoxFuture<(), Error>;

        /// Update constraints put on the type of tunnel connection to use
        #[rpc(meta, name = "update_relay_settings")]
        fn update_relay_settings(
            &self,
            Self::Metadata, RelaySettingsUpdate
            ) -> BoxFuture<(), Error>;

        /// Set if the client should allow communication with the LAN while in secured state.
        #[rpc(meta, name = "set_allow_lan")]
        fn set_allow_lan(&self, Self::Metadata, bool) -> BoxFuture<(), Error>;

        /// Set whether to enable the beta program.
        #[rpc(meta, name = "set_show_beta_releases")]
        fn set_show_beta_releases(&self, Self::Metadata, bool) -> BoxFuture<(), Error>;

        /// Set if the client should allow network communication when in the disconnected state.
        #[rpc(meta, name = "set_block_when_disconnected")]
        fn set_block_when_disconnected(&self, Self::Metadata, bool) -> BoxFuture<(), Error>;

        /// Set if the daemon should automatically establish a tunnel on start or not.
        #[rpc(meta, name = "set_auto_connect")]
        fn set_auto_connect(&self, Self::Metadata, bool) -> BoxFuture<(), Error>;

        /// Try to connect if disconnected, or do nothing if already connecting/connected.
        #[rpc(meta, name = "connect")]
        fn connect(&self, Self::Metadata) -> BoxFuture<(), Error>;

        /// Disconnect the VPN tunnel if it is connecting/connected. Does nothing if already
        /// disconnected.
        #[rpc(meta, name = "disconnect")]
        fn disconnect(&self, Self::Metadata) -> BoxFuture<(), Error>;

        /// Reconnect if connecting/connected, or do nothing if disconnected.
        #[rpc(meta, name = "reconnect")]
        fn reconnect(&self, Self::Metadata) -> BoxFuture<(), Error>;

        /// Returns the current state of the Mullvad client. Changes to this state will
        /// be announced to subscribers of `new_state`.
        #[rpc(meta, name = "get_state")]
        fn get_state(&self, Self::Metadata) -> BoxFuture<TunnelState, Error>;

        /// Performs a geoIP lookup and returns the current location as perceived by the public
        /// internet.
        #[rpc(meta, name = "get_current_location")]
        fn get_current_location(&self, Self::Metadata) -> BoxFuture<Option<GeoIpLocation>, Error>;

        /// Makes the daemon exit its main loop and quit.
        #[rpc(meta, name = "shutdown")]
        fn shutdown(&self, Self::Metadata) -> BoxFuture<(), Error>;

        /// Saves the target tunnel state and enters a blocking state. The state is restored
        /// upon restart.
        #[rpc(meta, name = "prepare_restart")]
        fn prepare_restart(&self, Self::Metadata) -> BoxFuture<(), Error>;

        /// Get previously used account tokens from the account history
        #[rpc(meta, name = "get_account_history")]
        fn get_account_history(&self, Self::Metadata) -> BoxFuture<Vec<AccountToken>, Error>;

        /// Remove given account token from the account history
        #[rpc(meta, name = "remove_account_from_history")]
        fn remove_account_from_history(&self, Self::Metadata, AccountToken) -> BoxFuture<(), Error>;

        /// Sets openvpn's mssfix parameter
        #[rpc(meta, name = "set_openvpn_mssfix")]
        fn set_openvpn_mssfix(&self, Self::Metadata, Option<u16>) -> BoxFuture<(), Error>;

        /// Sets proxy details for OpenVPN
        #[rpc(meta, name = "set_bridge_settings")]
        fn set_bridge_settings(&self, Self::Metadata, BridgeSettings) -> BoxFuture<(), Error>;

        /// Sets bridge state
        #[rpc(meta, name = "set_bridge_state")]
        fn set_bridge_state(&self, Self::Metadata, BridgeState) -> BoxFuture<(), Error>;

        /// Set if IPv6 is enabled in the tunnel
        #[rpc(meta, name = "set_enable_ipv6")]
        fn set_enable_ipv6(&self, Self::Metadata, bool) -> BoxFuture<(), Error>;

        /// Set MTU for wireguard tunnels
        #[rpc(meta, name = "set_wireguard_mtu")]
        fn set_wireguard_mtu(&self, Self::Metadata, Option<u16>) -> BoxFuture<(), Error>;

        /// Set automatic key rotation interval for wireguard tunnels
        #[rpc(meta, name = "set_wireguard_rotation_interval")]
        fn set_wireguard_rotation_interval(&self, Self::Metadata, Option<u32>) -> BoxFuture<(), Error>;

        /// Returns the current daemon settings
        #[rpc(meta, name = "get_settings")]
        fn get_settings(&self, Self::Metadata) -> BoxFuture<Settings, Error>;

        /// Generates new wireguard key for current account
        #[rpc(meta, name = "generate_wireguard_key")]
        fn generate_wireguard_key(&self, Self::Metadata) -> BoxFuture<wireguard::KeygenEvent, Error>;

        /// Retrieve a public key for current account if the account has one.
        #[rpc(meta, name = "get_wireguard_key")]
        fn get_wireguard_key(&self, Self::Metadata) -> BoxFuture<Option<wireguard::PublicKey>, Error>;

        /// Verify if current wireguard key is still valid
        #[rpc(meta, name = "verify_wireguard_key")]
        fn verify_wireguard_key(&self, Self::Metadata) -> BoxFuture<bool, Error>;

        /// Retreive version of the app
        #[rpc(meta, name = "get_current_version")]
        fn get_current_version(&self, Self::Metadata) -> BoxFuture<String, Error>;

        /// Retrieve information about the currently running and latest versions of the app
        #[rpc(meta, name = "get_version_info")]
        fn get_version_info(&self, Self::Metadata) -> BoxFuture<version::AppVersionInfo, Error>;

        /// Remove all configuration and cache files
        #[rpc(meta, name = "factory_reset")]
        fn factory_reset(&self, Self::Metadata) -> BoxFuture<(), Error>;

        /// Retrieve PIDs to exclude from the tunnel
        #[rpc(meta, name = "get_split_tunnel_processes")]
        fn get_split_tunnel_processes(&self, Self::Metadata) -> BoxFuture<Vec<i32>, Error>;

        /// Add a process to exclude from the tunnel
        #[rpc(meta, name = "add_split_tunnel_process")]
        fn add_split_tunnel_process(&self, Self::Metadata, i32) -> BoxFuture<(), Error>;

        /// Remove a process excluded from the tunnel
        #[rpc(meta, name = "remove_split_tunnel_process")]
        fn remove_split_tunnel_process(&self, Self::Metadata, i32) -> BoxFuture<(), Error>;

        /// Clear list of processes to exclude from the tunnel
        #[rpc(meta, name = "clear_split_tunnel_processes")]
        fn clear_split_tunnel_processes(&self, Self::Metadata) -> BoxFuture<(), Error>;

        #[pubsub(name = "daemon_event")] {
            /// Subscribes to events from the daemon.
            #[rpc(name = "daemon_event_subscribe")]
            fn daemon_event_subscribe(
                &self,
                Self::Metadata,
                pubsub::Subscriber<DaemonEvent>
            );

            /// Unsubscribes from the `daemon_event` event notifications.
            #[rpc(name = "daemon_event_unsubscribe")]
            fn daemon_event_unsubscribe(&self, SubscriptionId) -> BoxFuture<(), Error>;
        }
    }
}

pub struct ManagementInterfaceServer {
    server: talpid_ipc::IpcServer,
    subscriptions: Arc<RwLock<HashMap<SubscriptionId, pubsub::Sink<DaemonEvent>>>>,
}

impl ManagementInterfaceServer {
    pub fn start(tunnel_tx: DaemonCommandSender) -> Result<Self, talpid_ipc::Error> {
        let rpc = ManagementInterface::new(tunnel_tx);
        let subscriptions = rpc.subscriptions.clone();

        let mut io = PubSubHandler::default();
        io.extend_with(rpc.to_delegate());
        let meta_io: MetaIoHandler<Meta> = io.into();
        let path = mullvad_paths::get_rpc_socket_path();
        let server = talpid_ipc::IpcServer::start_with_metadata(
            meta_io,
            meta_extractor,
            &path.to_string_lossy(),
        )?;
        Ok(ManagementInterfaceServer {
            server,
            subscriptions,
        })
    }

    pub fn socket_path(&self) -> &str {
        self.server.path()
    }

    pub fn event_broadcaster(&self) -> ManagementInterfaceEventBroadcaster {
        ManagementInterfaceEventBroadcaster {
            subscriptions: self.subscriptions.clone(),
            close_handle: Some(self.server.close_handle()),
        }
    }

    /// Consumes the server and waits for it to finish. Returns an error if the server exited
    /// due to an error.
    pub fn wait(self) {
        self.server.wait()
    }
}

/// A handle that allows broadcasting messages to all subscribers of the management interface.
#[derive(Clone)]
pub struct ManagementInterfaceEventBroadcaster {
    subscriptions: Arc<RwLock<HashMap<SubscriptionId, pubsub::Sink<DaemonEvent>>>>,
    close_handle: Option<talpid_ipc::CloseHandle>,
}

impl EventListener for ManagementInterfaceEventBroadcaster {
    /// Sends a new state update to all `new_state` subscribers of the management interface.
    fn notify_new_state(&self, new_state: TunnelState) {
        self.notify(DaemonEvent::TunnelState(new_state));
    }

    /// Sends settings to all `settings` subscribers of the management interface.
    fn notify_settings(&self, settings: Settings) {
        log::debug!("Broadcasting new settings");
        self.notify(DaemonEvent::Settings(settings));
    }

    /// Sends settings to all `settings` subscribers of the management interface.
    fn notify_relay_list(&self, relay_list: RelayList) {
        log::debug!("Broadcasting new relay list");
        self.notify(DaemonEvent::RelayList(relay_list));
    }

    fn notify_app_version(&self, app_version_info: version::AppVersionInfo) {
        log::debug!("Broadcasting new app version info");
        self.notify(DaemonEvent::AppVersionInfo(app_version_info));
    }

    fn notify_key_event(&self, key_event: mullvad_types::wireguard::KeygenEvent) {
        log::debug!("Broadcasting new wireguard key event");
        self.notify(DaemonEvent::WireguardKey(key_event));
    }
}

impl ManagementInterfaceEventBroadcaster {
    fn notify(&self, value: DaemonEvent) {
        let subscriptions = self.subscriptions.read();
        for sink in subscriptions.values() {
            let _ = sink.notify(Ok(value.clone())).wait();
        }
    }
}

impl Drop for ManagementInterfaceEventBroadcaster {
    fn drop(&mut self) {
        if let Some(close_handle) = self.close_handle.take() {
            close_handle.close();
        }
    }
}

struct ManagementInterface {
    subscriptions: Arc<RwLock<HashMap<SubscriptionId, pubsub::Sink<DaemonEvent>>>>,
    tx: DaemonCommandSender,
}

impl ManagementInterface {
    pub fn new(tx: DaemonCommandSender) -> Self {
        ManagementInterface {
            subscriptions: Default::default(),
            tx,
        }
    }

    /// Sends a command to the daemon and maps the error to an RPC error.
    fn send_command_to_daemon(
        &self,
        command: DaemonCommand,
    ) -> impl Future<Item = (), Error = Error> {
        future::result(self.tx.send(command)).map_err(|_| Error::internal_error())
    }

    /// Converts the given error to an error that can be given to the caller of the API.
    /// Will let any actual RPC error through as is, any other error is changed to an internal
    /// error.
    fn map_rpc_error(error: &mullvad_rpc::Error) -> Error {
        match error.kind() {
            mullvad_rpc::ErrorKind::JsonRpcError(ref rpc_error) => {
                // We have to manually copy the error since we have different
                // versions of the jsonrpc_core library at the moment.
                Error {
                    code: ErrorCode::from(rpc_error.code.code()),
                    message: rpc_error.message.clone(),
                    data: rpc_error.data.clone(),
                }
            }
            _ => Error::internal_error(),
        }
    }
}

impl ManagementInterfaceApi for ManagementInterface {
    type Metadata = Meta;

    fn create_new_account(&self, _: Self::Metadata) -> BoxFuture<String, Error> {
        let (tx, rx) = sync::oneshot::channel();
        let future = self
            .send_command_to_daemon(DaemonCommand::CreateNewAccount(tx))
            .and_then(|_| rx.map_err(|_| Error::internal_error()))
            .and_then(|result| match result {
                Ok(account_token) => Ok(account_token),
                Err(e) => Err(Self::map_rpc_error(&e)),
            });

        Box::new(future)
    }

    fn get_account_data(
        &self,
        _: Self::Metadata,
        account_token: AccountToken,
    ) -> BoxFuture<AccountData, Error> {
        log::debug!("get_account_data");
        let (tx, rx) = sync::oneshot::channel();
        let future = self
            .send_command_to_daemon(DaemonCommand::GetAccountData(tx, account_token))
            .and_then(|_| rx.map_err(|_| Error::internal_error()))
            .and_then(|rpc_future| {
                rpc_future.map_err(|error: mullvad_rpc::Error| {
                    log::error!(
                        "Unable to get account data from API: {}",
                        error.display_chain()
                    );
                    Self::map_rpc_error(&error)
                })
            });
        Box::new(future)
    }

    fn get_www_auth_token(&self, _: Self::Metadata) -> BoxFuture<String, Error> {
        log::debug!("get_account_data");
        let (tx, rx) = sync::oneshot::channel();
        let future = self
            .send_command_to_daemon(DaemonCommand::GetWwwAuthToken(tx))
            .and_then(|_| rx.map_err(|_| Error::internal_error()))
            .and_then(|rpc_future| {
                rpc_future.map_err(|error: mullvad_rpc::Error| {
                    log::error!(
                        "Unable to get account data from API: {}",
                        error.display_chain()
                    );
                    Self::map_rpc_error(&error)
                })
            });
        Box::new(future)
    }

    fn submit_voucher(
        &self,
        _: Self::Metadata,
        voucher: String,
    ) -> BoxFuture<VoucherSubmission, Error> {
        log::debug!("submit_voucher");
        let (tx, rx) = sync::oneshot::channel();
        let future = self
            .send_command_to_daemon(DaemonCommand::SubmitVoucher(tx, voucher))
            .and_then(|_| rx.map_err(|_| Error::internal_error()))
            .and_then(|f| f.map_err(|e| Self::map_rpc_error(&e)));
        Box::new(future)
    }

    fn get_relay_locations(&self, _: Self::Metadata) -> BoxFuture<RelayList, Error> {
        log::debug!("get_relay_locations");
        let (tx, rx) = sync::oneshot::channel();
        let future = self
            .send_command_to_daemon(DaemonCommand::GetRelayLocations(tx))
            .and_then(|_| rx.map_err(|_| Error::internal_error()));
        Box::new(future)
    }

    fn update_relay_locations(&self, _: Self::Metadata) -> BoxFuture<(), Error> {
        log::debug!("update_relay_locations");
        Box::new(self.send_command_to_daemon(DaemonCommand::UpdateRelayLocations))
    }

    fn set_account(
        &self,
        _: Self::Metadata,
        account_token: Option<AccountToken>,
    ) -> BoxFuture<(), Error> {
        log::debug!("set_account");
        let (tx, rx) = sync::oneshot::channel();
        let future = self
            .send_command_to_daemon(DaemonCommand::SetAccount(tx, account_token))
            .and_then(|_| rx.map_err(|_| Error::internal_error()));
        Box::new(future)
    }

    fn update_relay_settings(
        &self,
        _: Self::Metadata,
        constraints_update: RelaySettingsUpdate,
    ) -> BoxFuture<(), Error> {
        log::debug!("update_relay_settings");
        let (tx, rx) = sync::oneshot::channel();

        let message = DaemonCommand::UpdateRelaySettings(tx, constraints_update);
        let future = self
            .send_command_to_daemon(message)
            .and_then(|_| rx.map_err(|_| Error::internal_error()));
        Box::new(future)
    }

    fn set_allow_lan(&self, _: Self::Metadata, allow_lan: bool) -> BoxFuture<(), Error> {
        log::debug!("set_allow_lan({})", allow_lan);
        let (tx, rx) = sync::oneshot::channel();
        let future = self
            .send_command_to_daemon(DaemonCommand::SetAllowLan(tx, allow_lan))
            .and_then(|_| rx.map_err(|_| Error::internal_error()));
        Box::new(future)
    }

    fn set_show_beta_releases(&self, _: Self::Metadata, enabled: bool) -> BoxFuture<(), Error> {
        log::debug!("set_show_beta_releases({})", enabled);
        let (tx, rx) = sync::oneshot::channel();
        let future = self
            .send_command_to_daemon(DaemonCommand::SetShowBetaReleases(tx, enabled))
            .and_then(|_| rx.map_err(|_| Error::internal_error()));
        Box::new(future)
    }

    fn set_block_when_disconnected(
        &self,
        _: Self::Metadata,
        block_when_disconnected: bool,
    ) -> BoxFuture<(), Error> {
        log::debug!("set_block_when_disconnected({})", block_when_disconnected);
        let (tx, rx) = sync::oneshot::channel();
        let future = self
            .send_command_to_daemon(DaemonCommand::SetBlockWhenDisconnected(
                tx,
                block_when_disconnected,
            ))
            .and_then(|_| rx.map_err(|_| Error::internal_error()));
        Box::new(future)
    }

    fn set_auto_connect(&self, _: Self::Metadata, auto_connect: bool) -> BoxFuture<(), Error> {
        log::debug!("set_auto_connect({})", auto_connect);
        let (tx, rx) = sync::oneshot::channel();
        let future = self
            .send_command_to_daemon(DaemonCommand::SetAutoConnect(tx, auto_connect))
            .and_then(|_| rx.map_err(|_| Error::internal_error()));
        Box::new(future)
    }

    fn connect(&self, _: Self::Metadata) -> BoxFuture<(), Error> {
        log::debug!("connect");
        let (tx, rx) = sync::oneshot::channel();
        let future = self
            .send_command_to_daemon(DaemonCommand::SetTargetState(tx, TargetState::Secured))
            .and_then(|_| rx.map_err(|_| Error::internal_error()))
            .and_then(|result| match result {
                Ok(()) => future::ok(()),
                Err(()) => future::err(Error {
                    code: ErrorCode::ServerError(-900),
                    message: "No account token configured".to_owned(),
                    data: None,
                }),
            });
        Box::new(future)
    }

    fn disconnect(&self, _: Self::Metadata) -> BoxFuture<(), Error> {
        log::debug!("disconnect");
        let (tx, _) = sync::oneshot::channel();
        let future = self
            .send_command_to_daemon(DaemonCommand::SetTargetState(tx, TargetState::Unsecured))
            .then(|_| future::ok(()));
        Box::new(future)
    }

    fn reconnect(&self, _: Self::Metadata) -> BoxFuture<(), Error> {
        log::debug!("reconnect");
        let future = self.send_command_to_daemon(DaemonCommand::Reconnect);
        Box::new(future)
    }

    fn get_state(&self, _: Self::Metadata) -> BoxFuture<TunnelState, Error> {
        log::debug!("get_state");
        let (state_tx, state_rx) = sync::oneshot::channel();
        let future = self
            .send_command_to_daemon(DaemonCommand::GetState(state_tx))
            .and_then(|_| state_rx.map_err(|_| Error::internal_error()));
        Box::new(future)
    }

    fn get_current_location(&self, _: Self::Metadata) -> BoxFuture<Option<GeoIpLocation>, Error> {
        log::debug!("get_current_location");
        let (tx, rx) = sync::oneshot::channel();
        let future = self
            .send_command_to_daemon(DaemonCommand::GetCurrentLocation(tx))
            .and_then(|_| rx.map_err(|_| Error::internal_error()));
        Box::new(future)
    }

    fn shutdown(&self, _: Self::Metadata) -> BoxFuture<(), Error> {
        log::debug!("shutdown");
        Box::new(self.send_command_to_daemon(DaemonCommand::Shutdown))
    }

    fn prepare_restart(&self, _: Self::Metadata) -> BoxFuture<(), Error> {
        log::debug!("prepare_restart");
        Box::new(self.send_command_to_daemon(DaemonCommand::PrepareRestart))
    }

    fn get_account_history(&self, _: Self::Metadata) -> BoxFuture<Vec<AccountToken>, Error> {
        log::debug!("get_account_history");
        let (tx, rx) = sync::oneshot::channel();
        let future = self
            .send_command_to_daemon(DaemonCommand::GetAccountHistory(tx))
            .and_then(|_| rx.map_err(|_| Error::internal_error()));
        Box::new(future)
    }

    fn remove_account_from_history(
        &self,
        _: Self::Metadata,
        account_token: AccountToken,
    ) -> BoxFuture<(), Error> {
        log::debug!("remove_account_from_history");
        let (tx, rx) = sync::oneshot::channel();
        let future = self
            .send_command_to_daemon(DaemonCommand::RemoveAccountFromHistory(tx, account_token))
            .and_then(|_| rx.map_err(|_| Error::internal_error()));
        Box::new(future)
    }

    fn set_openvpn_mssfix(&self, _: Self::Metadata, mssfix: Option<u16>) -> BoxFuture<(), Error> {
        log::debug!("set_openvpn_mssfix({:?})", mssfix);
        let (tx, rx) = sync::oneshot::channel();
        let future = self
            .send_command_to_daemon(DaemonCommand::SetOpenVpnMssfix(tx, mssfix))
            .and_then(|_| rx.map_err(|_| Error::internal_error()));

        Box::new(future)
    }

    fn set_bridge_settings(
        &self,
        _: Self::Metadata,
        bridge_settings: BridgeSettings,
    ) -> BoxFuture<(), Error> {
        log::debug!("set_bridge_settings({:?})", bridge_settings);
        let (tx, rx) = sync::oneshot::channel();
        let future = self
            .send_command_to_daemon(DaemonCommand::SetBridgeSettings(tx, bridge_settings))
            .and_then(|_| rx.map_err(|_| Error::internal_error()))
            .and_then(|settings_result| {
                settings_result.map_err(|error| match error {
                    settings::Error::InvalidProxyData(reason) => Error::invalid_params(reason),
                    _ => Error::internal_error(),
                })
            });

        Box::new(future)
    }

    fn set_bridge_state(
        &self,
        _: Self::Metadata,
        bridge_state: BridgeState,
    ) -> BoxFuture<(), Error> {
        log::debug!("set_bridge_state({:?})", bridge_state);
        let (tx, rx) = sync::oneshot::channel();
        let future = self
            .send_command_to_daemon(DaemonCommand::SetBridgeState(tx, bridge_state))
            .and_then(|_| rx.map_err(|_| Error::internal_error()))
            .and_then(|settings_result| settings_result.map_err(|_| Error::internal_error()));

        Box::new(future)
    }

    fn set_enable_ipv6(&self, _: Self::Metadata, enable_ipv6: bool) -> BoxFuture<(), Error> {
        log::debug!("set_enable_ipv6({})", enable_ipv6);
        let (tx, rx) = sync::oneshot::channel();
        let future = self
            .send_command_to_daemon(DaemonCommand::SetEnableIpv6(tx, enable_ipv6))
            .and_then(|_| rx.map_err(|_| Error::internal_error()));

        Box::new(future)
    }

    /// Set MTU for wireguard tunnels
    fn set_wireguard_mtu(&self, _: Self::Metadata, mtu: Option<u16>) -> BoxFuture<(), Error> {
        log::debug!("set_wireguard_mtu({:?})", mtu);
        let (tx, rx) = sync::oneshot::channel();
        let future = self
            .send_command_to_daemon(DaemonCommand::SetWireguardMtu(tx, mtu))
            .and_then(|_| rx.map_err(|_| Error::internal_error()));
        Box::new(future)
    }

    /// Set automatic key rotation interval for wireguard tunnels
    fn set_wireguard_rotation_interval(
        &self,
        _: Self::Metadata,
        interval: Option<u32>,
    ) -> BoxFuture<(), Error> {
        log::debug!("set_wireguard_rotation_interval({:?})", interval);
        let (tx, rx) = sync::oneshot::channel();
        let future = self
            .send_command_to_daemon(DaemonCommand::SetWireguardRotationInterval(tx, interval))
            .and_then(|_| rx.map_err(|_| Error::internal_error()));
        Box::new(future)
    }

    fn get_settings(&self, _: Self::Metadata) -> BoxFuture<Settings, Error> {
        log::debug!("get_settings");
        let (tx, rx) = sync::oneshot::channel();
        let future = self
            .send_command_to_daemon(DaemonCommand::GetSettings(tx))
            .and_then(|_| rx.map_err(|_| Error::internal_error()));
        Box::new(future)
    }

    fn generate_wireguard_key(
        &self,
        _: Self::Metadata,
    ) -> BoxFuture<mullvad_types::wireguard::KeygenEvent, Error> {
        log::debug!("generate_wireguard_key");
        let (tx, rx) = sync::oneshot::channel();
        let future = self
            .send_command_to_daemon(DaemonCommand::GenerateWireguardKey(tx))
            .and_then(|_| rx.map_err(|_| Error::internal_error()));
        Box::new(future)
    }

    fn get_wireguard_key(
        &self,
        _: Self::Metadata,
    ) -> BoxFuture<Option<wireguard::PublicKey>, Error> {
        log::debug!("get_wireguard_key");
        let (tx, rx) = sync::oneshot::channel();
        let future = self
            .send_command_to_daemon(DaemonCommand::GetWireguardKey(tx))
            .and_then(|_| rx.map_err(|_| Error::internal_error()));
        Box::new(future)
    }

    fn verify_wireguard_key(&self, _: Self::Metadata) -> BoxFuture<bool, Error> {
        log::debug!("verify_wireguard_key");
        let (tx, rx) = sync::oneshot::channel();
        let future = self
            .send_command_to_daemon(DaemonCommand::VerifyWireguardKey(tx))
            .and_then(|_| rx.map_err(|_| Error::internal_error()));
        Box::new(future)
    }

    fn get_current_version(&self, _: Self::Metadata) -> BoxFuture<String, Error> {
        log::debug!("get_current_version");
        let (tx, rx) = sync::oneshot::channel();
        let future = self
            .send_command_to_daemon(DaemonCommand::GetCurrentVersion(tx))
            .and_then(|_| rx.map_err(|_| Error::internal_error()));

        Box::new(future)
    }

    fn get_version_info(&self, _: Self::Metadata) -> BoxFuture<version::AppVersionInfo, Error> {
        log::debug!("get_version_info");
        let (tx, rx) = sync::oneshot::channel();
        let future = self
            .send_command_to_daemon(DaemonCommand::GetVersionInfo(tx))
            .and_then(|_| rx.map_err(|_| Error::internal_error()));

        Box::new(future)
    }

    fn factory_reset(&self, _: Self::Metadata) -> BoxFuture<(), Error> {
        #[cfg(not(target_os = "android"))]
        {
            log::debug!("factory_reset");
            let (tx, rx) = sync::oneshot::channel();
            let future = self
                .send_command_to_daemon(DaemonCommand::FactoryReset(tx))
                .and_then(|_| rx.map_err(|_| Error::internal_error()));

            Box::new(future)
        }
        #[cfg(target_os = "android")]
        {
            Box::new(future::ok(()))
        }
    }

    fn get_split_tunnel_processes(&self, _: Self::Metadata) -> BoxFuture<Vec<i32>, Error> {
        #[cfg(target_os = "linux")]
        {
            log::debug!("get_split_tunnel_processes");
            let (tx, rx) = sync::oneshot::channel();
            let future = self
                .send_command_to_daemon(DaemonCommand::GetSplitTunnelProcesses(tx))
                .and_then(|_| rx.map_err(|_| Error::internal_error()));
            Box::new(future)
        }
        #[cfg(not(target_os = "linux"))]
        {
            Box::new(future::ok(Vec::with_capacity(0)))
        }
    }

    #[cfg(target_os = "linux")]
    fn add_split_tunnel_process(&self, _: Self::Metadata, pid: i32) -> BoxFuture<(), Error> {
        log::debug!("add_split_tunnel_process");
        let (tx, rx) = sync::oneshot::channel();
        let future = self
            .send_command_to_daemon(DaemonCommand::AddSplitTunnelProcess(tx, pid))
            .and_then(|_| rx.map_err(|_| Error::internal_error()));
        Box::new(future)
    }
    #[cfg(not(target_os = "linux"))]
    fn add_split_tunnel_process(&self, _: Self::Metadata, _: i32) -> BoxFuture<(), Error> {
        Box::new(future::ok(()))
    }

    #[cfg(target_os = "linux")]
    fn remove_split_tunnel_process(&self, _: Self::Metadata, pid: i32) -> BoxFuture<(), Error> {
        log::debug!("remove_split_tunnel_process");
        let (tx, rx) = sync::oneshot::channel();
        let future = self
            .send_command_to_daemon(DaemonCommand::RemoveSplitTunnelProcess(tx, pid))
            .and_then(|_| rx.map_err(|_| Error::internal_error()));
        Box::new(future)
    }
    #[cfg(not(target_os = "linux"))]
    fn remove_split_tunnel_process(&self, _: Self::Metadata, _: i32) -> BoxFuture<(), Error> {
        Box::new(future::ok(()))
    }

    fn clear_split_tunnel_processes(&self, _: Self::Metadata) -> BoxFuture<(), Error> {
        #[cfg(target_os = "linux")]
        {
            log::debug!("clear_split_tunnel_processes");
            let (tx, rx) = sync::oneshot::channel();
            let future = self
                .send_command_to_daemon(DaemonCommand::ClearSplitTunnelProcesses(tx))
                .and_then(|_| rx.map_err(|_| Error::internal_error()));
            Box::new(future)
        }
        #[cfg(not(target_os = "linux"))]
        {
            Box::new(future::ok(()))
        }
    }


    fn daemon_event_subscribe(
        &self,
        _: Self::Metadata,
        subscriber: pubsub::Subscriber<DaemonEvent>,
    ) {
        log::debug!("daemon_event_subscribe");
        let mut subscriptions = self.subscriptions.write();
        loop {
            let id = SubscriptionId::String(uuid::Uuid::new_v4().to_string());
            if let Entry::Vacant(entry) = subscriptions.entry(id.clone()) {
                if let Ok(sink) = subscriber.assign_id(id.clone()) {
                    log::debug!("Accepting new subscription with id {:?}", id);
                    entry.insert(sink);
                }
                break;
            }
        }
    }

    fn daemon_event_unsubscribe(&self, id: SubscriptionId) -> BoxFuture<(), Error> {
        log::debug!("daemon_event_unsubscribe");
        let was_removed = self.subscriptions.write().remove(&id).is_some();
        let result = if was_removed {
            log::debug!("Unsubscribing id {:?}", id);
            future::ok(())
        } else {
            future::err(Error {
                code: ErrorCode::InvalidParams,
                message: "Invalid subscription".to_owned(),
                data: None,
            })
        };
        Box::new(result)
    }
}


/// The metadata type. There is one instance associated with each connection. In this pubsub
/// scenario they are created by `meta_extractor` by the server on each new incoming
/// connection.
#[derive(Clone, Debug, Default)]
pub struct Meta {
    session: Option<Arc<Session>>,
}

/// Make the `Meta` type possible to use as jsonrpc metadata type.
impl Metadata for Meta {}

/// Make the `Meta` type possible to use as a pubsub metadata type.
impl PubSubMetadata for Meta {
    fn session(&self) -> Option<Arc<Session>> {
        self.session.clone()
    }
}

/// Metadata extractor function for `Meta`.
fn meta_extractor(context: &jsonrpc_ipc_server::RequestContext<'_>) -> Meta {
    Meta {
        session: Some(Arc::new(Session::new(context.sender.clone()))),
    }
}

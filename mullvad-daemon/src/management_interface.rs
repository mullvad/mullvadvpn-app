use crate::{
    account_history, custom_lists, device, settings, DaemonCommand, DaemonCommandSender,
    EventListener,
};
use futures::{
    channel::{mpsc, oneshot},
    StreamExt,
};
use mullvad_api::{rest::Error as RestError, StatusCode};
use mullvad_management_interface::{
    types::{self, daemon_event, management_service_server::ManagementService},
    Code, Request, Response, Status,
};
use mullvad_paths;
#[cfg(not(target_os = "android"))]
use mullvad_types::settings::DnsOptions;
use mullvad_types::{
    account::AccountToken,
    relay_constraints::{BridgeSettings, BridgeState, ObfuscationSettings, RelaySettingsUpdate},
    relay_list::RelayList,
    settings::Settings,
    states::{TargetState, TunnelState},
    version,
    wireguard::{RotationInterval, RotationIntervalError},
};
#[cfg(windows)]
use std::path::PathBuf;
use std::{
    convert::{TryFrom, TryInto},
    sync::{Arc, Mutex},
    time::Duration,
};
use talpid_types::ErrorExt;
use tokio_stream::wrappers::UnboundedReceiverStream;

#[derive(err_derive::Error, Debug)]
#[error(no_from)]
pub enum Error {
    // Unable to start the management interface server
    #[error(display = "Unable to start management interface server")]
    SetupError(#[error(source)] mullvad_management_interface::Error),
}

struct ManagementServiceImpl {
    daemon_tx: DaemonCommandSender,
    subscriptions: Arc<Mutex<Vec<EventsListenerSender>>>,
}

pub type ServiceResult<T> = std::result::Result<Response<T>, Status>;
type EventsListenerReceiver = UnboundedReceiverStream<Result<types::DaemonEvent, Status>>;
type EventsListenerSender = tokio::sync::mpsc::UnboundedSender<Result<types::DaemonEvent, Status>>;

const INVALID_VOUCHER_MESSAGE: &str = "This voucher code is invalid";
const USED_VOUCHER_MESSAGE: &str = "This voucher code has already been used";

#[mullvad_management_interface::async_trait]
impl ManagementService for ManagementServiceImpl {
    type GetSplitTunnelProcessesStream = UnboundedReceiverStream<Result<i32, Status>>;
    type EventsListenStream = EventsListenerReceiver;

    // Control and get the tunnel state
    //

    async fn connect_tunnel(&self, _: Request<()>) -> ServiceResult<bool> {
        log::debug!("connect_tunnel");

        let (tx, rx) = oneshot::channel();
        self.send_command_to_daemon(DaemonCommand::SetTargetState(tx, TargetState::Secured))?;
        let connect_issued = self.wait_for_result(rx).await?;
        Ok(Response::new(connect_issued))
    }

    async fn disconnect_tunnel(&self, _: Request<()>) -> ServiceResult<bool> {
        log::debug!("disconnect_tunnel");

        let (tx, rx) = oneshot::channel();
        self.send_command_to_daemon(DaemonCommand::SetTargetState(tx, TargetState::Unsecured))?;
        let disconnect_issued = self.wait_for_result(rx).await?;
        Ok(Response::new(disconnect_issued))
    }

    async fn reconnect_tunnel(&self, _: Request<()>) -> ServiceResult<bool> {
        log::debug!("reconnect_tunnel");
        let (tx, rx) = oneshot::channel();
        self.send_command_to_daemon(DaemonCommand::Reconnect(tx))?;
        let reconnect_issued = self.wait_for_result(rx).await?;
        Ok(Response::new(reconnect_issued))
    }

    async fn get_tunnel_state(&self, _: Request<()>) -> ServiceResult<types::TunnelState> {
        log::debug!("get_tunnel_state");
        let (tx, rx) = oneshot::channel();
        self.send_command_to_daemon(DaemonCommand::GetState(tx))?;
        let state = self.wait_for_result(rx).await?;
        Ok(Response::new(types::TunnelState::from(state)))
    }

    // Control the daemon and receive events
    //

    async fn events_listen(&self, _: Request<()>) -> ServiceResult<Self::EventsListenStream> {
        let (tx, rx) = tokio::sync::mpsc::unbounded_channel();

        let mut subscriptions = self.subscriptions.lock().unwrap();
        subscriptions.push(tx);

        Ok(Response::new(UnboundedReceiverStream::new(rx)))
    }

    async fn prepare_restart(&self, _: Request<()>) -> ServiceResult<()> {
        log::debug!("prepare_restart");
        self.send_command_to_daemon(DaemonCommand::PrepareRestart)?;
        Ok(Response::new(()))
    }

    async fn factory_reset(&self, _: Request<()>) -> ServiceResult<()> {
        #[cfg(not(target_os = "android"))]
        {
            log::debug!("factory_reset");
            let (tx, rx) = oneshot::channel();
            self.send_command_to_daemon(DaemonCommand::FactoryReset(tx))?;
            self.wait_for_result(rx)
                .await?
                .map(Response::new)
                .map_err(map_daemon_error)
        }
        #[cfg(target_os = "android")]
        {
            Ok(Response::new(()))
        }
    }

    async fn get_current_version(&self, _: Request<()>) -> ServiceResult<String> {
        log::debug!("get_current_version");
        let (tx, rx) = oneshot::channel();
        self.send_command_to_daemon(DaemonCommand::GetCurrentVersion(tx))?;
        let version = self.wait_for_result(rx).await?;
        Ok(Response::new(version))
    }

    async fn get_version_info(&self, _: Request<()>) -> ServiceResult<types::AppVersionInfo> {
        log::debug!("get_version_info");

        let (tx, rx) = oneshot::channel();
        self.send_command_to_daemon(DaemonCommand::GetVersionInfo(tx))?;
        self.wait_for_result(rx)
            .await?
            .ok_or_else(|| Status::not_found("no version cache"))
            .map(types::AppVersionInfo::from)
            .map(Response::new)
    }

    async fn is_performing_post_upgrade(&self, _: Request<()>) -> ServiceResult<bool> {
        log::debug!("is_performing_post_upgrade");
        let (tx, rx) = oneshot::channel();
        self.send_command_to_daemon(DaemonCommand::IsPerformingPostUpgrade(tx))?;
        Ok(Response::new(self.wait_for_result(rx).await?))
    }

    // Relays and tunnel constraints
    //

    async fn update_relay_locations(&self, _: Request<()>) -> ServiceResult<()> {
        log::debug!("update_relay_locations");
        self.send_command_to_daemon(DaemonCommand::UpdateRelayLocations)?;
        Ok(Response::new(()))
    }

    async fn update_relay_settings(
        &self,
        request: Request<types::RelaySettingsUpdate>,
    ) -> ServiceResult<()> {
        log::debug!("update_relay_settings");
        let (tx, rx) = oneshot::channel();
        let constraints_update =
            RelaySettingsUpdate::try_from(request.into_inner()).map_err(map_protobuf_type_err)?;

        let message = DaemonCommand::UpdateRelaySettings(tx, constraints_update);
        self.send_command_to_daemon(message)?;
        self.wait_for_result(rx)
            .await?
            .map(Response::new)
            .map_err(map_settings_error)
    }

    async fn get_relay_locations(&self, _: Request<()>) -> ServiceResult<types::RelayList> {
        log::debug!("get_relay_locations");

        let (tx, rx) = oneshot::channel();
        self.send_command_to_daemon(DaemonCommand::GetRelayLocations(tx))?;
        self.wait_for_result(rx)
            .await
            .map(|relays| Response::new(types::RelayList::from(relays)))
    }

    async fn get_current_location(&self, _: Request<()>) -> ServiceResult<types::GeoIpLocation> {
        log::debug!("get_current_location");
        let (tx, rx) = oneshot::channel();
        self.send_command_to_daemon(DaemonCommand::GetCurrentLocation(tx))?;
        let result = self.wait_for_result(rx).await?;
        match result {
            Some(geoip) => Ok(Response::new(types::GeoIpLocation::from(geoip))),
            None => Err(Status::not_found("no location was found")),
        }
    }

    async fn set_bridge_settings(
        &self,
        request: Request<types::BridgeSettings>,
    ) -> ServiceResult<()> {
        let settings =
            BridgeSettings::try_from(request.into_inner()).map_err(map_protobuf_type_err)?;

        log::debug!("set_bridge_settings({:?})", settings);

        let (tx, rx) = oneshot::channel();
        self.send_command_to_daemon(DaemonCommand::SetBridgeSettings(tx, settings))?;
        let settings_result = self.wait_for_result(rx).await?;
        settings_result
            .map(Response::new)
            .map_err(map_settings_error)
    }

    async fn set_obfuscation_settings(
        &self,
        request: Request<types::ObfuscationSettings>,
    ) -> ServiceResult<()> {
        let settings =
            ObfuscationSettings::try_from(request.into_inner()).map_err(map_protobuf_type_err)?;
        log::debug!("set_obfuscation_settings({:?})", settings);
        let (tx, rx) = oneshot::channel();
        self.send_command_to_daemon(DaemonCommand::SetObfuscationSettings(tx, settings))?;
        let settings_result = self.wait_for_result(rx).await?;
        settings_result
            .map(Response::new)
            .map_err(map_settings_error)
    }

    async fn set_bridge_state(&self, request: Request<types::BridgeState>) -> ServiceResult<()> {
        let bridge_state =
            BridgeState::try_from(request.into_inner()).map_err(map_protobuf_type_err)?;

        log::debug!("set_bridge_state({:?})", bridge_state);
        let (tx, rx) = oneshot::channel();
        self.send_command_to_daemon(DaemonCommand::SetBridgeState(tx, bridge_state))?;
        let settings_result = self.wait_for_result(rx).await?;
        settings_result
            .map(Response::new)
            .map_err(map_settings_error)
    }

    // Settings
    //

    async fn get_settings(&self, _: Request<()>) -> ServiceResult<types::Settings> {
        log::debug!("get_settings");
        let (tx, rx) = oneshot::channel();
        self.send_command_to_daemon(DaemonCommand::GetSettings(tx))?;
        self.wait_for_result(rx)
            .await
            .map(|settings| Response::new(types::Settings::from(&settings)))
    }

    async fn set_allow_lan(&self, request: Request<bool>) -> ServiceResult<()> {
        let allow_lan = request.into_inner();
        log::debug!("set_allow_lan({})", allow_lan);
        let (tx, rx) = oneshot::channel();
        self.send_command_to_daemon(DaemonCommand::SetAllowLan(tx, allow_lan))?;
        self.wait_for_result(rx)
            .await?
            .map(Response::new)
            .map_err(map_settings_error)
    }

    async fn set_show_beta_releases(&self, request: Request<bool>) -> ServiceResult<()> {
        let enabled = request.into_inner();
        log::debug!("set_show_beta_releases({})", enabled);
        let (tx, rx) = oneshot::channel();
        self.send_command_to_daemon(DaemonCommand::SetShowBetaReleases(tx, enabled))?;
        self.wait_for_result(rx)
            .await?
            .map(Response::new)
            .map_err(map_settings_error)
    }

    async fn set_block_when_disconnected(&self, request: Request<bool>) -> ServiceResult<()> {
        let block_when_disconnected = request.into_inner();
        log::debug!("set_block_when_disconnected({})", block_when_disconnected);
        let (tx, rx) = oneshot::channel();
        self.send_command_to_daemon(DaemonCommand::SetBlockWhenDisconnected(
            tx,
            block_when_disconnected,
        ))?;
        self.wait_for_result(rx)
            .await?
            .map(Response::new)
            .map_err(map_settings_error)
    }

    async fn set_auto_connect(&self, request: Request<bool>) -> ServiceResult<()> {
        let auto_connect = request.into_inner();
        log::debug!("set_auto_connect({})", auto_connect);
        let (tx, rx) = oneshot::channel();
        self.send_command_to_daemon(DaemonCommand::SetAutoConnect(tx, auto_connect))?;
        self.wait_for_result(rx)
            .await?
            .map(Response::new)
            .map_err(map_settings_error)
    }

    async fn set_openvpn_mssfix(&self, request: Request<u32>) -> ServiceResult<()> {
        let mssfix = request.into_inner();
        let mssfix = if mssfix != 0 {
            Some(mssfix as u16)
        } else {
            None
        };
        log::debug!("set_openvpn_mssfix({:?})", mssfix);
        let (tx, rx) = oneshot::channel();
        self.send_command_to_daemon(DaemonCommand::SetOpenVpnMssfix(tx, mssfix))?;
        self.wait_for_result(rx)
            .await?
            .map(Response::new)
            .map_err(map_settings_error)
    }

    async fn set_wireguard_mtu(&self, request: Request<u32>) -> ServiceResult<()> {
        let mtu = request.into_inner();
        let mtu = if mtu != 0 { Some(mtu as u16) } else { None };
        log::debug!("set_wireguard_mtu({:?})", mtu);
        let (tx, rx) = oneshot::channel();
        self.send_command_to_daemon(DaemonCommand::SetWireguardMtu(tx, mtu))?;
        self.wait_for_result(rx)
            .await?
            .map(Response::new)
            .map_err(map_settings_error)
    }

    async fn set_enable_ipv6(&self, request: Request<bool>) -> ServiceResult<()> {
        let enable_ipv6 = request.into_inner();
        log::debug!("set_enable_ipv6({})", enable_ipv6);
        let (tx, rx) = oneshot::channel();
        self.send_command_to_daemon(DaemonCommand::SetEnableIpv6(tx, enable_ipv6))?;
        self.wait_for_result(rx)
            .await?
            .map(Response::new)
            .map_err(map_settings_error)
    }

    async fn set_quantum_resistant_tunnel(
        &self,
        request: Request<types::QuantumResistantState>,
    ) -> ServiceResult<()> {
        let state = mullvad_types::wireguard::QuantumResistantState::try_from(request.into_inner())
            .map_err(map_protobuf_type_err)?;

        log::debug!("set_quantum_resistant_tunnel({state:?})");
        let (tx, rx) = oneshot::channel();
        self.send_command_to_daemon(DaemonCommand::SetQuantumResistantTunnel(tx, state))?;
        self.wait_for_result(rx)
            .await?
            .map(Response::new)
            .map_err(map_settings_error)
    }

    #[cfg(not(target_os = "android"))]
    async fn set_dns_options(&self, request: Request<types::DnsOptions>) -> ServiceResult<()> {
        let options = DnsOptions::try_from(request.into_inner()).map_err(map_protobuf_type_err)?;
        log::debug!("set_dns_options({:?})", options);

        let (tx, rx) = oneshot::channel();
        self.send_command_to_daemon(DaemonCommand::SetDnsOptions(tx, options))?;
        self.wait_for_result(rx)
            .await?
            .map(Response::new)
            .map_err(map_settings_error)
    }

    #[cfg(target_os = "android")]
    async fn set_dns_options(&self, _: Request<types::DnsOptions>) -> ServiceResult<()> {
        Ok(Response::new(()))
    }

    // Account management
    //

    async fn create_new_account(&self, _: Request<()>) -> ServiceResult<String> {
        log::debug!("create_new_account");
        let (tx, rx) = oneshot::channel();
        self.send_command_to_daemon(DaemonCommand::CreateNewAccount(tx))?;
        self.wait_for_result(rx)
            .await?
            .map(Response::new)
            .map_err(map_daemon_error)
    }

    async fn login_account(&self, request: Request<AccountToken>) -> ServiceResult<()> {
        log::debug!("login_account");
        let account_token = request.into_inner();
        let (tx, rx) = oneshot::channel();
        self.send_command_to_daemon(DaemonCommand::LoginAccount(tx, account_token))?;
        self.wait_for_result(rx)
            .await?
            .map(Response::new)
            .map_err(map_daemon_error)
    }

    async fn logout_account(&self, _: Request<()>) -> ServiceResult<()> {
        log::debug!("logout_account");
        let (tx, rx) = oneshot::channel();
        self.send_command_to_daemon(DaemonCommand::LogoutAccount(tx))?;
        self.wait_for_result(rx)
            .await?
            .map(Response::new)
            .map_err(map_daemon_error)
    }

    async fn get_account_data(
        &self,
        request: Request<AccountToken>,
    ) -> ServiceResult<types::AccountData> {
        log::debug!("get_account_data");
        let account_token = request.into_inner();
        let (tx, rx) = oneshot::channel();
        self.send_command_to_daemon(DaemonCommand::GetAccountData(tx, account_token))?;
        let result = self.wait_for_result(rx).await?;
        result
            .map(|account_data| Response::new(types::AccountData::from(account_data)))
            .map_err(|error: RestError| {
                log::error!(
                    "Unable to get account data from API: {}",
                    error.display_chain()
                );
                map_rest_error(&error)
            })
    }

    async fn get_account_history(&self, _: Request<()>) -> ServiceResult<types::AccountHistory> {
        log::debug!("get_account_history");
        let (tx, rx) = oneshot::channel();
        self.send_command_to_daemon(DaemonCommand::GetAccountHistory(tx))?;
        self.wait_for_result(rx)
            .await
            .map(|history| Response::new(types::AccountHistory { token: history }))
    }

    async fn clear_account_history(&self, _: Request<()>) -> ServiceResult<()> {
        log::debug!("clear_account_history");
        let (tx, rx) = oneshot::channel();
        self.send_command_to_daemon(DaemonCommand::ClearAccountHistory(tx))?;
        self.wait_for_result(rx)
            .await?
            .map(Response::new)
            .map_err(map_daemon_error)
    }

    async fn get_www_auth_token(&self, _: Request<()>) -> ServiceResult<String> {
        log::debug!("get_www_auth_token");
        let (tx, rx) = oneshot::channel();
        self.send_command_to_daemon(DaemonCommand::GetWwwAuthToken(tx))?;
        let result = self.wait_for_result(rx).await?;
        result.map(Response::new).map_err(|error| {
            log::error!(
                "Unable to get account data from API: {}",
                error.display_chain()
            );
            map_daemon_error(error)
        })
    }

    async fn submit_voucher(
        &self,
        request: Request<String>,
    ) -> ServiceResult<types::VoucherSubmission> {
        log::debug!("submit_voucher");
        let voucher = request.into_inner();
        let (tx, rx) = oneshot::channel();
        self.send_command_to_daemon(DaemonCommand::SubmitVoucher(tx, voucher))?;
        let result = self.wait_for_result(rx).await?;
        result
            .map(|submission| Response::new(types::VoucherSubmission::from(submission)))
            .map_err(map_daemon_error)
    }

    // Device management
    async fn get_device(&self, _: Request<()>) -> ServiceResult<types::DeviceState> {
        log::debug!("get_device");
        let (tx, rx) = oneshot::channel();
        self.send_command_to_daemon(DaemonCommand::GetDevice(tx))?;
        let device = self.wait_for_result(rx).await?.map_err(map_daemon_error)?;
        Ok(Response::new(types::DeviceState::from(device)))
    }

    async fn update_device(&self, _: Request<()>) -> ServiceResult<()> {
        log::debug!("update_device");
        let (tx, rx) = oneshot::channel();
        self.send_command_to_daemon(DaemonCommand::UpdateDevice(tx))?;
        self.wait_for_result(rx)
            .await?
            .map_err(map_daemon_error)
            .map(Response::new)
    }

    async fn list_devices(
        &self,
        request: Request<AccountToken>,
    ) -> ServiceResult<types::DeviceList> {
        log::debug!("list_devices");
        let (tx, rx) = oneshot::channel();
        let token = request.into_inner();
        self.send_command_to_daemon(DaemonCommand::ListDevices(tx, token))?;
        let device = self.wait_for_result(rx).await?.map_err(map_daemon_error)?;
        Ok(Response::new(types::DeviceList::from(device)))
    }

    async fn remove_device(&self, request: Request<types::DeviceRemoval>) -> ServiceResult<()> {
        log::debug!("remove_device");
        let (tx, rx) = oneshot::channel();
        let removal = request.into_inner();
        self.send_command_to_daemon(DaemonCommand::RemoveDevice(
            tx,
            removal.account_token,
            removal.device_id,
        ))?;
        self.wait_for_result(rx).await?.map_err(map_daemon_error)?;
        Ok(Response::new(()))
    }

    // WireGuard key management
    //

    async fn set_wireguard_rotation_interval(
        &self,
        request: Request<types::Duration>,
    ) -> ServiceResult<()> {
        let interval: RotationInterval = Duration::try_from(request.into_inner())
            .map_err(|_| Status::invalid_argument("unexpected negative rotation interval"))?
            .try_into()
            .map_err(|error: RotationIntervalError| {
                Status::invalid_argument(error.display_chain())
            })?;

        log::debug!("set_wireguard_rotation_interval({:?})", interval);
        let (tx, rx) = oneshot::channel();
        self.send_command_to_daemon(DaemonCommand::SetWireguardRotationInterval(
            tx,
            Some(interval),
        ))?;
        self.wait_for_result(rx)
            .await?
            .map(Response::new)
            .map_err(map_settings_error)
    }

    async fn reset_wireguard_rotation_interval(&self, _: Request<()>) -> ServiceResult<()> {
        log::debug!("reset_wireguard_rotation_interval");
        let (tx, rx) = oneshot::channel();
        self.send_command_to_daemon(DaemonCommand::SetWireguardRotationInterval(tx, None))?;
        self.wait_for_result(rx)
            .await?
            .map(Response::new)
            .map_err(map_settings_error)
    }

    async fn rotate_wireguard_key(&self, _: Request<()>) -> ServiceResult<()> {
        log::debug!("rotate_wireguard_key");
        let (tx, rx) = oneshot::channel();
        self.send_command_to_daemon(DaemonCommand::RotateWireguardKey(tx))?;
        self.wait_for_result(rx)
            .await?
            .map(Response::new)
            .map_err(map_daemon_error)
    }

    async fn get_wireguard_key(&self, _: Request<()>) -> ServiceResult<types::PublicKey> {
        log::debug!("get_wireguard_key");
        let (tx, rx) = oneshot::channel();
        self.send_command_to_daemon(DaemonCommand::GetWireguardKey(tx))?;
        let key = self.wait_for_result(rx).await?.map_err(map_daemon_error)?;
        match key {
            Some(key) => Ok(Response::new(types::PublicKey::from(key))),
            None => Err(Status::not_found("no WireGuard key was found")),
        }
    }

    // Custom lists
    //

    async fn list_custom_lists(
        &self,
        _: Request<()>,
    ) -> ServiceResult<mullvad_management_interface::types::CustomLists> {
        log::debug!("list_custom_lists");
        let (tx, rx) = oneshot::channel();
        self.send_command_to_daemon(DaemonCommand::ListCustomLists(tx))?;
        self.wait_for_result(rx)
            .await?
            .map(|custom_lists| {
                Response::new(mullvad_management_interface::types::CustomLists::from(
                    custom_lists,
                ))
            })
            .map_err(map_daemon_error)
    }

    async fn get_custom_list(
        &self,
        request: Request<String>,
    ) -> ServiceResult<mullvad_management_interface::types::CustomList> {
        log::debug!("get_custom_list");
        let (tx, rx) = oneshot::channel();
        self.send_command_to_daemon(DaemonCommand::GetCustomList(tx, request.into_inner()))?;
        self.wait_for_result(rx)
            .await?
            .map(|custom_list| {
                Response::new(mullvad_management_interface::types::CustomList::from(
                    custom_list,
                ))
            })
            .map_err(map_daemon_error)
    }

    async fn get_api_access_methods(
        &self,
        _: Request<()>,
    ) -> ServiceResult<mullvad_management_interface::types::ApiAccessMethods> {
        log::debug!("get_api_access_methods");
        let (tx, rx) = oneshot::channel();
        self.send_command_to_daemon(DaemonCommand::GetApiAccessMethods(tx))?;
        self.wait_for_result(rx)
            .await?
            .map(From::from)
            .map(Response::new)
            .map_err(map_daemon_error)
    }

    async fn add_api_access_method(
        &self,
        request: Request<types::ApiAccessMethod>,
    ) -> ServiceResult<()> {
        log::debug!("add_api_access_method");
        let access_method =
            mullvad_types::api_access_method::AccessMethod::try_from(request.into_inner())?;
        let (tx, rx) = oneshot::channel();
        self.send_command_to_daemon(DaemonCommand::AddApiAccessMethod(tx, access_method))?;
        self.wait_for_result(rx)
            .await?
            .map(Response::new)
            .map_err(map_daemon_error)
    }

    async fn remove_api_access_method(
        &self,
        request: Request<types::ApiAccessMethod>,
    ) -> ServiceResult<()> {
        log::debug!("remove_api_access_method");
        let access_method =
            mullvad_types::api_access_method::AccessMethod::try_from(request.into_inner())?;

        match access_method.as_custom() {
            None => Err(Status::not_found(
                "Can not remove built-in API access mtehod",
            )),
            Some(access_method) => {
                let (tx, rx) = oneshot::channel();
                self.send_command_to_daemon(DaemonCommand::RemoveApiAccessMethod(
                    tx,
                    access_method.clone(),
                ))?;
                self.wait_for_result(rx)
                    .await?
                    .map(Response::new)
                    .map_err(map_daemon_error)
            }
        }
    }

    async fn replace_api_access_method(
        &self,
        request: Request<types::ApiAccessMethodReplace>,
    ) -> ServiceResult<()> {
        log::debug!("edit_api_access_method");
        let access_method_replace =
            mullvad_types::api_access_method::daemon::ApiAccessMethodReplace::try_from(
                request.into_inner(),
            )?;
        let (tx, rx) = oneshot::channel();
        self.send_command_to_daemon(DaemonCommand::ReplaceApiAccessMethod(
            tx,
            access_method_replace,
        ))?;
        self.wait_for_result(rx)
            .await?
            .map(Response::new)
            .map_err(map_daemon_error)
    }

    async fn toggle_api_access_method(
        &self,
        request: Request<types::ApiAccessMethodToggle>,
    ) -> ServiceResult<()> {
        log::debug!("toggle_api_access_method");
        let access_method_toggle =
            mullvad_types::api_access_method::daemon::ApiAccessMethodToggle::try_from(
                request.into_inner(),
            )?;
        let (tx, rx) = oneshot::channel();
        self.send_command_to_daemon(DaemonCommand::ToggleApiAccessMethod(
            tx,
            access_method_toggle,
        ))?;
        self.wait_for_result(rx)
            .await?
            .map(Response::new)
            .map_err(map_daemon_error)
    }

    async fn set_api_access_method(
        &self,
        request: Request<types::ApiAccessMethod>,
    ) -> ServiceResult<()> {
        log::debug!("set_api_access_method");
        let access_method =
            mullvad_types::api_access_method::AccessMethod::try_from(request.into_inner())?;
        let (tx, rx) = oneshot::channel();
        self.send_command_to_daemon(DaemonCommand::SetApiAccessMethod(tx, access_method))?;
        self.wait_for_result(rx)
            .await?
            .map(Response::new)
            .map_err(map_daemon_error)
    }

    async fn create_custom_list(&self, request: Request<String>) -> ServiceResult<()> {
        log::debug!("create_custom_list");
        let (tx, rx) = oneshot::channel();
        self.send_command_to_daemon(DaemonCommand::CreateCustomList(tx, request.into_inner()))?;
        self.wait_for_result(rx)
            .await?
            .map(Response::new)
            .map_err(map_daemon_error)
    }

    async fn delete_custom_list(&self, request: Request<String>) -> ServiceResult<()> {
        log::debug!("delete_custom_list");
        let (tx, rx) = oneshot::channel();
        self.send_command_to_daemon(DaemonCommand::DeleteCustomList(tx, request.into_inner()))?;
        self.wait_for_result(rx)
            .await?
            .map(Response::new)
            .map_err(map_daemon_error)
    }

    async fn update_custom_list_location(
        &self,
        request: Request<types::CustomListLocationUpdate>,
    ) -> ServiceResult<()> {
        log::debug!("update_custom_list_location");
        let custom_list =
            mullvad_types::custom_list::CustomListLocationUpdate::try_from(request.into_inner())?;
        let (tx, rx) = oneshot::channel();
        self.send_command_to_daemon(DaemonCommand::UpdateCustomListLocation(tx, custom_list))?;
        self.wait_for_result(rx)
            .await?
            .map(Response::new)
            .map_err(map_daemon_error)
    }
    async fn rename_custom_list(
        &self,
        request: Request<types::CustomListRename>,
    ) -> ServiceResult<()> {
        log::debug!("rename_custom_list");
        let names: (String, String) = From::from(request.into_inner());
        let (tx, rx) = oneshot::channel();
        self.send_command_to_daemon(DaemonCommand::RenameCustomList(tx, names.0, names.1))?;
        self.wait_for_result(rx)
            .await?
            .map(Response::new)
            .map_err(map_daemon_error)
    }

    // Split tunneling
    //

    async fn get_split_tunnel_processes(
        &self,
        _: Request<()>,
    ) -> ServiceResult<Self::GetSplitTunnelProcessesStream> {
        #[cfg(target_os = "linux")]
        {
            log::debug!("get_split_tunnel_processes");
            let (tx, rx) = oneshot::channel();
            self.send_command_to_daemon(DaemonCommand::GetSplitTunnelProcesses(tx))?;
            let pids = self
                .wait_for_result(rx)
                .await?
                .map_err(|error| Status::failed_precondition(error.to_string()))?;

            let (tx, rx) = tokio::sync::mpsc::unbounded_channel();
            tokio::spawn(async move {
                for pid in pids {
                    let _ = tx.send(Ok(pid));
                }
            });

            Ok(Response::new(UnboundedReceiverStream::new(rx)))
        }
        #[cfg(not(target_os = "linux"))]
        {
            let (_, rx) = tokio::sync::mpsc::unbounded_channel();
            Ok(Response::new(UnboundedReceiverStream::new(rx)))
        }
    }

    #[cfg(target_os = "linux")]
    async fn add_split_tunnel_process(&self, request: Request<i32>) -> ServiceResult<()> {
        let pid = request.into_inner();
        log::debug!("add_split_tunnel_process");
        let (tx, rx) = oneshot::channel();
        self.send_command_to_daemon(DaemonCommand::AddSplitTunnelProcess(tx, pid))?;
        self.wait_for_result(rx)
            .await?
            .map_err(|error| Status::failed_precondition(error.to_string()))?;
        Ok(Response::new(()))
    }
    #[cfg(not(target_os = "linux"))]
    async fn add_split_tunnel_process(&self, _: Request<i32>) -> ServiceResult<()> {
        Ok(Response::new(()))
    }

    #[cfg(target_os = "linux")]
    async fn remove_split_tunnel_process(&self, request: Request<i32>) -> ServiceResult<()> {
        let pid = request.into_inner();
        log::debug!("remove_split_tunnel_process");
        let (tx, rx) = oneshot::channel();
        self.send_command_to_daemon(DaemonCommand::RemoveSplitTunnelProcess(tx, pid))?;
        self.wait_for_result(rx)
            .await?
            .map_err(|error| Status::failed_precondition(error.to_string()))?;
        Ok(Response::new(()))
    }
    #[cfg(not(target_os = "linux"))]
    async fn remove_split_tunnel_process(&self, _: Request<i32>) -> ServiceResult<()> {
        Ok(Response::new(()))
    }

    async fn clear_split_tunnel_processes(&self, _: Request<()>) -> ServiceResult<()> {
        #[cfg(target_os = "linux")]
        {
            log::debug!("clear_split_tunnel_processes");
            let (tx, rx) = oneshot::channel();
            self.send_command_to_daemon(DaemonCommand::ClearSplitTunnelProcesses(tx))?;
            self.wait_for_result(rx)
                .await?
                .map_err(|error| Status::failed_precondition(error.to_string()))?;
            Ok(Response::new(()))
        }
        #[cfg(not(target_os = "linux"))]
        {
            Ok(Response::new(()))
        }
    }

    #[cfg(windows)]
    async fn add_split_tunnel_app(&self, request: Request<String>) -> ServiceResult<()> {
        log::debug!("add_split_tunnel_app");
        let path = PathBuf::from(request.into_inner());
        let (tx, rx) = oneshot::channel();
        self.send_command_to_daemon(DaemonCommand::AddSplitTunnelApp(tx, path))?;
        self.wait_for_result(rx)
            .await?
            .map_err(map_daemon_error)
            .map(Response::new)
    }
    #[cfg(not(windows))]
    async fn add_split_tunnel_app(&self, _: Request<String>) -> ServiceResult<()> {
        Ok(Response::new(()))
    }

    #[cfg(windows)]
    async fn remove_split_tunnel_app(&self, request: Request<String>) -> ServiceResult<()> {
        log::debug!("remove_split_tunnel_app");
        let path = PathBuf::from(request.into_inner());
        let (tx, rx) = oneshot::channel();
        self.send_command_to_daemon(DaemonCommand::RemoveSplitTunnelApp(tx, path))?;
        self.wait_for_result(rx)
            .await?
            .map_err(map_daemon_error)
            .map(Response::new)
    }
    #[cfg(not(windows))]
    async fn remove_split_tunnel_app(&self, _: Request<String>) -> ServiceResult<()> {
        Ok(Response::new(()))
    }

    #[cfg(windows)]
    async fn clear_split_tunnel_apps(&self, _: Request<()>) -> ServiceResult<()> {
        log::debug!("clear_split_tunnel_apps");
        let (tx, rx) = oneshot::channel();
        self.send_command_to_daemon(DaemonCommand::ClearSplitTunnelApps(tx))?;
        self.wait_for_result(rx)
            .await?
            .map_err(map_daemon_error)
            .map(Response::new)
    }
    #[cfg(not(windows))]
    async fn clear_split_tunnel_apps(&self, _: Request<()>) -> ServiceResult<()> {
        Ok(Response::new(()))
    }

    #[cfg(windows)]
    async fn set_split_tunnel_state(&self, request: Request<bool>) -> ServiceResult<()> {
        log::debug!("set_split_tunnel_state");
        let enabled = request.into_inner();
        let (tx, rx) = oneshot::channel();
        self.send_command_to_daemon(DaemonCommand::SetSplitTunnelState(tx, enabled))?;
        self.wait_for_result(rx)
            .await?
            .map_err(map_daemon_error)
            .map(Response::new)
    }
    #[cfg(not(windows))]
    async fn set_split_tunnel_state(&self, _: Request<bool>) -> ServiceResult<()> {
        Ok(Response::new(()))
    }

    #[cfg(windows)]
    async fn get_excluded_processes(
        &self,
        _: Request<()>,
    ) -> ServiceResult<types::ExcludedProcessList> {
        log::debug!("get_excluded_processes");
        let (tx, rx) = oneshot::channel();
        self.send_command_to_daemon(DaemonCommand::GetSplitTunnelProcesses(tx))?;
        self.wait_for_result(rx)
            .await?
            .map_err(map_split_tunnel_error)
            .map(|processes| {
                Response::new(types::ExcludedProcessList {
                    processes: processes
                        .into_iter()
                        .map(types::ExcludedProcess::from)
                        .collect(),
                })
            })
    }

    #[cfg(not(windows))]
    async fn get_excluded_processes(
        &self,
        _: Request<()>,
    ) -> ServiceResult<types::ExcludedProcessList> {
        Ok(Response::new(types::ExcludedProcessList {
            processes: vec![],
        }))
    }

    #[cfg(windows)]
    async fn check_volumes(&self, _: Request<()>) -> ServiceResult<()> {
        log::debug!("check_volumes");
        let (tx, rx) = oneshot::channel();
        self.send_command_to_daemon(DaemonCommand::CheckVolumes(tx))?;
        self.wait_for_result(rx)
            .await?
            .map_err(map_daemon_error)
            .map(Response::new)
    }

    #[cfg(not(windows))]
    async fn check_volumes(&self, _: Request<()>) -> ServiceResult<()> {
        Ok(Response::new(()))
    }
}

impl ManagementServiceImpl {
    /// Sends a command to the daemon and maps the error to an RPC error.
    fn send_command_to_daemon(&self, command: DaemonCommand) -> Result<(), Status> {
        self.daemon_tx
            .send(command)
            .map_err(|_| Status::internal("the daemon channel receiver has been dropped"))
    }

    async fn wait_for_result<T>(&self, rx: oneshot::Receiver<T>) -> Result<T, Status> {
        rx.await.map_err(|_| Status::internal("sender was dropped"))
    }
}

pub struct ManagementInterfaceServer(());

impl ManagementInterfaceServer {
    pub fn start(
        tunnel_tx: DaemonCommandSender,
    ) -> Result<(String, ManagementInterfaceEventBroadcaster), Error> {
        let subscriptions = Arc::<Mutex<Vec<EventsListenerSender>>>::default();

        let socket_path = mullvad_paths::get_rpc_socket_path()
            .to_string_lossy()
            .to_string();

        let (server_abort_tx, server_abort_rx) = mpsc::channel(0);
        let server = ManagementServiceImpl {
            daemon_tx: tunnel_tx,
            subscriptions: subscriptions.clone(),
        };
        let join_handle = mullvad_management_interface::spawn_rpc_server(server, async move {
            server_abort_rx.into_future().await;
        })
        .map_err(Error::SetupError)?;

        tokio::spawn(async move {
            if let Err(error) = join_handle.await {
                log::error!("Management server panic: {}", error);
            }
            log::info!("Management interface shut down");
        });

        Ok((
            socket_path,
            ManagementInterfaceEventBroadcaster {
                subscriptions,
                _close_handle: server_abort_tx,
            },
        ))
    }
}

/// A handle that allows broadcasting messages to all subscribers of the management interface.
#[derive(Clone)]
pub struct ManagementInterfaceEventBroadcaster {
    subscriptions: Arc<Mutex<Vec<EventsListenerSender>>>,
    _close_handle: mpsc::Sender<()>,
}

impl EventListener for ManagementInterfaceEventBroadcaster {
    /// Sends a new state update to all `new_state` subscribers of the management interface.
    fn notify_new_state(&self, new_state: TunnelState) {
        self.notify(types::DaemonEvent {
            event: Some(daemon_event::Event::TunnelState(types::TunnelState::from(
                new_state,
            ))),
        })
    }

    /// Sends settings to all `settings` subscribers of the management interface.
    fn notify_settings(&self, settings: Settings) {
        log::debug!("Broadcasting new settings");
        self.notify(types::DaemonEvent {
            event: Some(daemon_event::Event::Settings(types::Settings::from(
                &settings,
            ))),
        })
    }

    /// Sends relays to all subscribers of the management interface.
    fn notify_relay_list(&self, relay_list: RelayList) {
        log::debug!("Broadcasting new relay list");
        self.notify(types::DaemonEvent {
            event: Some(daemon_event::Event::RelayList(types::RelayList::from(
                relay_list,
            ))),
        })
    }

    fn notify_app_version(&self, app_version_info: version::AppVersionInfo) {
        log::debug!("Broadcasting new app version info");
        self.notify(types::DaemonEvent {
            event: Some(daemon_event::Event::VersionInfo(
                types::AppVersionInfo::from(app_version_info),
            )),
        })
    }

    fn notify_device_event(&self, device: mullvad_types::device::DeviceEvent) {
        log::debug!("Broadcasting device event");
        self.notify(types::DaemonEvent {
            event: Some(daemon_event::Event::Device(types::DeviceEvent::from(
                device,
            ))),
        })
    }

    fn notify_remove_device_event(&self, remove_event: mullvad_types::device::RemoveDeviceEvent) {
        log::debug!("Broadcasting remove device event");
        self.notify(types::DaemonEvent {
            event: Some(daemon_event::Event::RemoveDevice(
                types::RemoveDeviceEvent::from(remove_event),
            )),
        })
    }
}

impl ManagementInterfaceEventBroadcaster {
    fn notify(&self, value: types::DaemonEvent) {
        let mut subscriptions = self.subscriptions.lock().unwrap();
        subscriptions.retain(|tx| tx.send(Ok(value.clone())).is_ok());
    }
}

/// Converts [`mullvad_daemon::Error`] into a tonic status.
fn map_daemon_error(error: crate::Error) -> Status {
    use crate::Error as DaemonError;

    match error {
        DaemonError::RestError(error) => map_rest_error(&error),
        DaemonError::SettingsError(error) => map_settings_error(error),
        DaemonError::AlreadyLoggedIn => Status::already_exists(error.to_string()),
        DaemonError::LoginError(error) => map_device_error(&error),
        DaemonError::LogoutError(error) => map_device_error(&error),
        DaemonError::KeyRotationError(error) => map_device_error(&error),
        DaemonError::ListDevicesError(error) => map_device_error(&error),
        DaemonError::RemoveDeviceError(error) => map_device_error(&error),
        DaemonError::UpdateDeviceError(error) => map_device_error(&error),
        DaemonError::VoucherSubmission(error) => map_device_error(&error),
        #[cfg(windows)]
        DaemonError::SplitTunnelError(error) => map_split_tunnel_error(error),
        DaemonError::AccountHistory(error) => map_account_history_error(error),
        DaemonError::NoAccountToken | DaemonError::NoAccountTokenHistory => {
            Status::unauthenticated(error.to_string())
        }
        DaemonError::CustomListError(error) => map_custom_list_error(error),
        error => Status::unknown(error.to_string()),
    }
}

#[cfg(windows)]
/// Converts [`talpid_core::split_tunnel::Error`] into a tonic status.
fn map_split_tunnel_error(error: talpid_core::split_tunnel::Error) -> Status {
    use talpid_core::split_tunnel::Error;

    match &error {
        Error::RegisterIps(io_error) | Error::SetConfiguration(io_error) => {
            if io_error.kind() == std::io::ErrorKind::NotFound {
                Status::not_found(format!("{}: {}", error, io_error))
            } else {
                Status::unknown(error.to_string())
            }
        }
        _ => Status::unknown(error.to_string()),
    }
}

/// Converts a REST API error into a tonic status.
fn map_rest_error(error: &RestError) -> Status {
    match error {
        RestError::ApiError(status, message)
            if *status == StatusCode::UNAUTHORIZED || *status == StatusCode::FORBIDDEN =>
        {
            Status::new(Code::Unauthenticated, message)
        }
        RestError::TimeoutError(_elapsed) => Status::deadline_exceeded("API request timed out"),
        RestError::HyperError(_) => Status::unavailable("Cannot reach the API"),
        error => Status::unknown(format!("REST error: {error}")),
    }
}

/// Converts an instance of [`mullvad_daemon::settings::Error`] into a tonic status.
fn map_settings_error(error: settings::Error) -> Status {
    match error {
        settings::Error::DeleteError(..)
        | settings::Error::WriteError(..)
        | settings::Error::ReadError(..) => {
            Status::new(Code::FailedPrecondition, error.to_string())
        }
        settings::Error::SerializeError(..) | settings::Error::ParseError(..) => {
            Status::new(Code::Internal, error.to_string())
        }
    }
}

/// Converts an instance of [`mullvad_daemon::device::Error`] into a tonic status.
fn map_device_error(error: &device::Error) -> Status {
    match error {
        device::Error::MaxDevicesReached => Status::new(Code::ResourceExhausted, error.to_string()),
        device::Error::InvalidAccount => Status::new(Code::Unauthenticated, error.to_string()),
        device::Error::InvalidDevice | device::Error::NoDevice => {
            Status::new(Code::NotFound, error.to_string())
        }
        device::Error::InvalidVoucher => Status::new(Code::NotFound, INVALID_VOUCHER_MESSAGE),
        device::Error::UsedVoucher => Status::new(Code::ResourceExhausted, USED_VOUCHER_MESSAGE),
        device::Error::DeviceIoError(ref _error) => {
            Status::new(Code::Unavailable, error.to_string())
        }
        device::Error::OtherRestError(error) => map_rest_error(error),
        device::Error::ResponseFailure(error) => map_device_error(error.unpack()),
        _ => Status::new(Code::Unknown, error.to_string()),
    }
}

/// Converts an instance of [`mullvad_daemon::account_history::Error`] into a tonic status.
fn map_account_history_error(error: account_history::Error) -> Status {
    match error {
        account_history::Error::Read(..) | account_history::Error::Write(..) => {
            Status::new(Code::FailedPrecondition, error.to_string())
        }
        account_history::Error::Serialize(..) | account_history::Error::WriteCancelled(..) => {
            Status::new(Code::Internal, error.to_string())
        }
    }
}

/// Converts an instance of [`mullvad_daemon::account_history::Error`] into a tonic status.
fn map_custom_list_error(error: custom_lists::Error) -> Status {
    match error {
        custom_lists::Error::ListExists => Status::with_details(
            Code::AlreadyExists,
            error.to_string(),
            mullvad_management_interface::CUSTOM_LIST_LIST_EXISTS_DETAILS.into(),
        ),
        custom_lists::Error::ListNotFound => Status::with_details(
            Code::NotFound,
            error.to_string(),
            mullvad_management_interface::CUSTOM_LIST_LIST_NOT_FOUND_DETAILS.into(),
        ),
        custom_lists::Error::CannotAddOrRemoveAny => {
            Status::new(Code::InvalidArgument, error.to_string())
        }
        custom_lists::Error::LocationExists => Status::with_details(
            Code::AlreadyExists,
            error.to_string(),
            mullvad_management_interface::CUSTOM_LIST_LOCATION_EXISTS_DETAILS.into(),
        ),
        custom_lists::Error::LocationNotFoundInlist => Status::with_details(
            Code::NotFound,
            error.to_string(),
            mullvad_management_interface::CUSTOM_LIST_LOCATION_NOT_FOUND_DETAILS.into(),
        ),
        custom_lists::Error::Settings(error) => map_settings_error(error),
    }
}

fn map_protobuf_type_err(err: types::FromProtobufTypeError) -> Status {
    match err {
        types::FromProtobufTypeError::InvalidArgument(err) => Status::invalid_argument(err),
    }
}

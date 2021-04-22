use crate::{account_history, settings, DaemonCommand, DaemonCommandSender, EventListener};
use futures::channel::oneshot;
use mullvad_management_interface::{
    types::{self, daemon_event, management_service_server::ManagementService},
    Code, Request, Response, Status,
};
use mullvad_paths;
use mullvad_rpc::{rest::Error as RestError, StatusCode};
#[cfg(not(target_os = "android"))]
use mullvad_types::settings::DnsOptions;
use mullvad_types::{
    account::AccountToken,
    location::GeoIpLocation,
    relay_constraints::{
        BridgeConstraints, BridgeSettings, BridgeState, Constraint, LocationConstraint,
        OpenVpnConstraints, Providers, RelayConstraintsUpdate, RelaySettingsUpdate,
        WireguardConstraints,
    },
    relay_list::{Relay, RelayList, RelayListCountry},
    settings::Settings,
    states::{TargetState, TunnelState},
    version,
    wireguard::{self, RotationInterval, RotationIntervalError},
    ConnectionConfig,
};
use parking_lot::RwLock;
use std::{
    cmp,
    convert::{TryFrom, TryInto},
    sync::{mpsc, Arc},
    time::Duration,
};
use talpid_types::{
    net::{IpVersion, TunnelType},
    ErrorExt,
};

#[derive(err_derive::Error, Debug)]
#[error(no_from)]
pub enum Error {
    // Unable to start the management interface server
    #[error(display = "Unable to start management interface server")]
    SetupError(#[error(source)] mullvad_management_interface::Error),
}

struct ManagementServiceImpl {
    daemon_tx: DaemonCommandSender,
    subscriptions: Arc<RwLock<Vec<EventsListenerSender>>>,
}

pub type ServiceResult<T> = std::result::Result<Response<T>, Status>;
type EventsListenerReceiver =
    tokio::sync::mpsc::UnboundedReceiver<Result<types::DaemonEvent, Status>>;
type EventsListenerSender = tokio::sync::mpsc::UnboundedSender<Result<types::DaemonEvent, Status>>;

const INVALID_VOUCHER_MESSAGE: &str = "This voucher code is invalid";
const USED_VOUCHER_MESSAGE: &str = "This voucher code has already been used";

#[mullvad_management_interface::async_trait]
impl ManagementService for ManagementServiceImpl {
    type GetRelayLocationsStream =
        tokio::sync::mpsc::Receiver<Result<types::RelayListCountry, Status>>;
    type GetSplitTunnelProcessesStream = tokio::sync::mpsc::UnboundedReceiver<Result<i32, Status>>;
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
        Ok(Response::new(convert_state(state)))
    }

    // Control the daemon and receive events
    //

    async fn events_listen(&self, _: Request<()>) -> ServiceResult<Self::EventsListenStream> {
        let (tx, rx) = tokio::sync::mpsc::unbounded_channel();

        let mut subscriptions = self.subscriptions.write();
        subscriptions.push(tx);

        Ok(Response::new(rx))
    }

    async fn prepare_restart(&self, _: Request<()>) -> ServiceResult<()> {
        log::debug!("prepare_restart");
        self.send_command_to_daemon(DaemonCommand::PrepareRestart)?;
        Ok(Response::new(()))
    }

    async fn shutdown(&self, _: Request<()>) -> ServiceResult<()> {
        log::debug!("shutdown");
        self.send_command_to_daemon(DaemonCommand::Shutdown)?;
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
            .ok_or(Status::not_found("no version cache"))
            .map(types::AppVersionInfo::from)
            .map(Response::new)
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
        let constraints_update = convert_relay_settings_update(&request.into_inner())?;

        let message = DaemonCommand::UpdateRelaySettings(tx, constraints_update);
        self.send_command_to_daemon(message)?;
        self.wait_for_result(rx)
            .await?
            .map(Response::new)
            .map_err(map_settings_error)
    }

    async fn get_relay_locations(
        &self,
        _: Request<()>,
    ) -> ServiceResult<Self::GetRelayLocationsStream> {
        log::debug!("get_relay_locations");

        let (tx, rx) = oneshot::channel();
        self.send_command_to_daemon(DaemonCommand::GetRelayLocations(tx))?;
        let locations = self.wait_for_result(rx).await?;

        let (mut stream_tx, stream_rx) =
            tokio::sync::mpsc::channel(cmp::max(1, locations.countries.len()));

        tokio::spawn(async move {
            for country in &locations.countries {
                if let Err(error) = stream_tx
                    .send(Ok(convert_relay_list_country(country)))
                    .await
                {
                    log::error!(
                        "Error while sending relays to client: {}",
                        error.display_chain()
                    );
                }
            }
        });

        Ok(Response::new(stream_rx))
    }

    async fn get_current_location(&self, _: Request<()>) -> ServiceResult<types::GeoIpLocation> {
        log::debug!("get_current_location");
        let (tx, rx) = oneshot::channel();
        self.send_command_to_daemon(DaemonCommand::GetCurrentLocation(tx))?;
        let result = self.wait_for_result(rx).await?;
        match result {
            Some(geoip) => Ok(Response::new(convert_geoip_location(geoip))),
            None => Err(Status::not_found("no location was found")),
        }
    }

    async fn set_bridge_settings(
        &self,
        request: Request<types::BridgeSettings>,
    ) -> ServiceResult<()> {
        use talpid_types::net;
        use types::bridge_settings::Type as BridgeSettingType;

        let settings = request
            .into_inner()
            .r#type
            .ok_or(Status::invalid_argument("no settings provided"))?;

        let settings = match settings {
            BridgeSettingType::Normal(constraints) => {
                let location = match constraints.location {
                    None => Constraint::Any,
                    Some(location) => convert_proto_location(location),
                };
                let providers = if constraints.providers.is_empty() {
                    Constraint::Any
                } else {
                    Constraint::Only(
                        Providers::new(constraints.providers.clone().into_iter()).map_err(
                            |_| Status::invalid_argument("must specify at least one provider"),
                        )?,
                    )
                };

                BridgeSettings::Normal(BridgeConstraints {
                    location,
                    providers,
                })
            }
            BridgeSettingType::Local(proxy_settings) => {
                let peer = proxy_settings
                    .peer
                    .parse()
                    .map_err(|_| Status::invalid_argument("failed to parse peer address"))?;
                let proxy_settings =
                    net::openvpn::ProxySettings::Local(net::openvpn::LocalProxySettings {
                        port: proxy_settings.port as u16,
                        peer,
                    });
                BridgeSettings::Custom(proxy_settings)
            }
            BridgeSettingType::Remote(proxy_settings) => {
                let address = proxy_settings
                    .address
                    .parse()
                    .map_err(|_| Status::invalid_argument("failed to parse IP address"))?;
                let auth = proxy_settings.auth.map(|auth| net::openvpn::ProxyAuth {
                    username: auth.username,
                    password: auth.password,
                });
                let proxy_settings =
                    net::openvpn::ProxySettings::Remote(net::openvpn::RemoteProxySettings {
                        address,
                        auth,
                    });
                BridgeSettings::Custom(proxy_settings)
            }
            BridgeSettingType::Shadowsocks(proxy_settings) => {
                let peer = proxy_settings
                    .peer
                    .parse()
                    .map_err(|_| Status::invalid_argument("failed to parse peer address"))?;
                let proxy_settings = net::openvpn::ProxySettings::Shadowsocks(
                    net::openvpn::ShadowsocksProxySettings {
                        peer,
                        password: proxy_settings.password,
                        cipher: proxy_settings.cipher,
                    },
                );
                BridgeSettings::Custom(proxy_settings)
            }
        };

        log::debug!("set_bridge_settings({:?})", settings);

        let (tx, rx) = oneshot::channel();
        self.send_command_to_daemon(DaemonCommand::SetBridgeSettings(tx, settings))?;
        let settings_result = self.wait_for_result(rx).await?;
        settings_result
            .map(Response::new)
            .map_err(map_settings_error)
    }

    async fn set_bridge_state(&self, request: Request<types::BridgeState>) -> ServiceResult<()> {
        use types::bridge_state::State;

        let bridge_state = match State::from_i32(request.into_inner().state) {
            Some(State::Auto) => BridgeState::Auto,
            Some(State::On) => BridgeState::On,
            Some(State::Off) => BridgeState::Off,
            None => return Err(Status::invalid_argument("unknown bridge state")),
        };

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

    #[cfg(not(target_os = "android"))]
    async fn set_dns_options(&self, request: Request<types::DnsOptions>) -> ServiceResult<()> {
        let options = request.into_inner();
        log::debug!(
            "set_dns_options({}, {:?})",
            options.custom,
            options.addresses
        );

        let mut servers_ip = vec![];
        for server in options.addresses.into_iter() {
            if let Ok(addr) = server.parse() {
                servers_ip.push(addr);
            } else {
                let err_msg = format!("failed to parse IP address: {}", server);
                return Err(Status::invalid_argument(err_msg));
            }
        }

        let (tx, rx) = oneshot::channel();
        self.send_command_to_daemon(DaemonCommand::SetDnsOptions(
            tx,
            DnsOptions {
                custom: options.custom,
                addresses: servers_ip,
            },
        ))?;
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
        let (tx, rx) = oneshot::channel();
        self.send_command_to_daemon(DaemonCommand::CreateNewAccount(tx))?;
        self.wait_for_result(rx)
            .await?
            .map(Response::new)
            .map_err(map_daemon_error)
    }

    async fn set_account(&self, request: Request<AccountToken>) -> ServiceResult<()> {
        log::debug!("set_account");
        let account_token = request.into_inner();
        let account_token = if account_token == "" {
            None
        } else {
            Some(account_token)
        };
        let (tx, rx) = oneshot::channel();
        self.send_command_to_daemon(DaemonCommand::SetAccount(tx, account_token))?;
        self.wait_for_result(rx)
            .await?
            .map(Response::new)
            .map_err(map_settings_error)
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
            .map(|account_data| {
                Response::new(types::AccountData {
                    expiry: Some(types::Timestamp {
                        seconds: account_data.expiry.timestamp(),
                        nanos: 0,
                    }),
                })
            })
            .map_err(|error: RestError| {
                log::error!(
                    "Unable to get account data from API: {}",
                    error.display_chain()
                );
                map_rest_error(error)
            })
    }

    async fn get_account_history(&self, _: Request<()>) -> ServiceResult<types::AccountHistory> {
        // TODO: this might be a stream
        log::debug!("get_account_history");
        let (tx, rx) = oneshot::channel();
        self.send_command_to_daemon(DaemonCommand::GetAccountHistory(tx))?;
        self.wait_for_result(rx)
            .await
            .map(|history| Response::new(types::AccountHistory { token: history }))
    }

    async fn remove_account_from_history(
        &self,
        request: Request<AccountToken>,
    ) -> ServiceResult<()> {
        log::debug!("remove_account_from_history");
        let account_token = request.into_inner();
        let (tx, rx) = oneshot::channel();
        self.send_command_to_daemon(DaemonCommand::RemoveAccountFromHistory(tx, account_token))?;
        self.wait_for_result(rx)
            .await?
            .map(Response::new)
            .map_err(map_daemon_error)
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
            .map(|submission| {
                Response::new(types::VoucherSubmission {
                    seconds_added: submission.time_added,
                    new_expiry: Some(types::Timestamp {
                        seconds: submission.new_expiry.timestamp(),
                        nanos: 0,
                    }),
                })
            })
            .map_err(|error| match error {
                crate::Error::RestError(error) => map_rest_voucher_error(error),
                error => map_daemon_error(error),
            })
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

    async fn generate_wireguard_key(&self, _: Request<()>) -> ServiceResult<types::KeygenEvent> {
        // TODO: return error for TooManyKeys, GenerationFailure
        // on success, simply return the new key or nil
        log::debug!("generate_wireguard_key");
        let (tx, rx) = oneshot::channel();
        self.send_command_to_daemon(DaemonCommand::GenerateWireguardKey(tx))?;
        self.wait_for_result(rx)
            .await?
            .map(|event| Response::new(convert_wireguard_key_event(&event)))
            .map_err(map_daemon_error)
    }

    async fn get_wireguard_key(&self, _: Request<()>) -> ServiceResult<types::PublicKey> {
        log::debug!("get_wireguard_key");
        let (tx, rx) = oneshot::channel();
        self.send_command_to_daemon(DaemonCommand::GetWireguardKey(tx))?;
        let key = self.wait_for_result(rx).await?.map_err(map_daemon_error)?;
        match key {
            Some(key) => Ok(Response::new(convert_public_key(&key))),
            None => Err(Status::not_found("no WireGuard key was found")),
        }
    }

    async fn verify_wireguard_key(&self, _: Request<()>) -> ServiceResult<bool> {
        log::debug!("verify_wireguard_key");
        let (tx, rx) = oneshot::channel();
        self.send_command_to_daemon(DaemonCommand::VerifyWireguardKey(tx))?;
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

            Ok(Response::new(rx))
        }
        #[cfg(not(target_os = "linux"))]
        {
            let (_, rx) = tokio::sync::mpsc::unbounded_channel();
            Ok(Response::new(rx))
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

fn convert_relay_settings_update(
    settings: &types::RelaySettingsUpdate,
) -> Result<RelaySettingsUpdate, Status> {
    use mullvad_types::CustomTunnelEndpoint;
    use talpid_types::net::{self, openvpn, wireguard};
    use types::{
        connection_config::Config as ProtoConnectionConfig,
        relay_settings_update::Type as ProtoUpdateType,
    };

    let update_value = settings
        .r#type
        .clone()
        .ok_or(Status::invalid_argument("missing relay settings"))?;

    match update_value {
        ProtoUpdateType::Custom(settings) => {
            let config = settings
                .config
                .ok_or(Status::invalid_argument("missing relay settings"))?;
            let config = config
                .config
                .ok_or(Status::invalid_argument("missing relay settings"))?;
            let config = match config {
                ProtoConnectionConfig::Openvpn(config) => {
                    let address = match config.address.parse() {
                        Ok(address) => address,
                        Err(_) => return Err(Status::invalid_argument("invalid address")),
                    };

                    ConnectionConfig::OpenVpn(openvpn::ConnectionConfig {
                        endpoint: net::Endpoint {
                            address,
                            protocol: types::TransportProtocol::from_i32(config.protocol)
                                .ok_or(Status::invalid_argument("invalid transport protocol"))?
                                .into(),
                        },
                        username: config.username.clone(),
                        password: config.password.clone(),
                    })
                }
                ProtoConnectionConfig::Wireguard(config) => {
                    let tunnel = config
                        .tunnel
                        .ok_or(Status::invalid_argument("missing tunnel config"))?;

                    // Copy the private key to an array
                    if tunnel.private_key.len() != 32 {
                        return Err(Status::invalid_argument("invalid private key"));
                    }

                    let mut private_key = [0; 32];
                    let buffer = &tunnel.private_key[..private_key.len()];
                    private_key.copy_from_slice(buffer);

                    let peer = config
                        .peer
                        .ok_or(Status::invalid_argument("missing peer config"))?;

                    // Copy the public key to an array
                    if peer.public_key.len() != 32 {
                        return Err(Status::invalid_argument("invalid public key"));
                    }

                    let mut public_key = [0; 32];
                    let buffer = &peer.public_key[..public_key.len()];
                    public_key.copy_from_slice(buffer);

                    let ipv4_gateway = match config.ipv4_gateway.parse() {
                        Ok(address) => address,
                        Err(_) => return Err(Status::invalid_argument("invalid IPv4 gateway")),
                    };
                    let ipv6_gateway = if !config.ipv6_gateway.is_empty() {
                        let address = match config.ipv6_gateway.parse() {
                            Ok(address) => address,
                            Err(_) => return Err(Status::invalid_argument("invalid IPv6 gateway")),
                        };
                        Some(address)
                    } else {
                        None
                    };

                    let endpoint = match peer.endpoint.parse() {
                        Ok(address) => address,
                        Err(_) => return Err(Status::invalid_argument("invalid peer address")),
                    };

                    let mut tunnel_addresses = Vec::new();
                    for address in tunnel.addresses {
                        let address = address
                            .parse()
                            .map_err(|_| Status::invalid_argument("invalid address"))?;
                        tunnel_addresses.push(address);
                    }

                    let mut allowed_ips = Vec::new();
                    for address in peer.allowed_ips {
                        let address = address
                            .parse()
                            .map_err(|_| Status::invalid_argument("invalid address"))?;
                        allowed_ips.push(address);
                    }

                    ConnectionConfig::Wireguard(wireguard::ConnectionConfig {
                        tunnel: wireguard::TunnelConfig {
                            private_key: wireguard::PrivateKey::from(private_key),
                            addresses: tunnel_addresses,
                        },
                        peer: wireguard::PeerConfig {
                            public_key: wireguard::PublicKey::from(public_key),
                            allowed_ips,
                            endpoint,
                            protocol: types::TransportProtocol::from_i32(peer.protocol)
                                .ok_or(Status::invalid_argument("invalid transport protocol"))?
                                .into(),
                        },
                        ipv4_gateway,
                        ipv6_gateway,
                    })
                }
            };

            Ok(RelaySettingsUpdate::CustomTunnelEndpoint(
                CustomTunnelEndpoint {
                    host: settings.host.clone(),
                    config,
                },
            ))
        }

        ProtoUpdateType::Normal(settings) => {
            // If `location` isn't provided, no changes are made.
            // If `location` is provided, but is an empty vector,
            // then the constraint is set to `Constraint::Any`.
            let location = settings.location.map(convert_proto_location);

            let tunnel_protocol = if let Some(update) = settings.tunnel_type {
                match update.tunnel_type {
                    Some(constraint) => match types::TunnelType::from_i32(constraint.tunnel_type) {
                        Some(types::TunnelType::Openvpn) => {
                            Some(Constraint::Only(TunnelType::OpenVpn))
                        }
                        Some(types::TunnelType::Wireguard) => {
                            Some(Constraint::Only(TunnelType::Wireguard))
                        }
                        None => return Err(Status::invalid_argument("invalid tunnel protocol")),
                    },
                    None => Some(Constraint::Any),
                }
            } else {
                None
            };

            let transport_protocol = if let Some(ref constraints) = settings.openvpn_constraints {
                match &constraints.protocol {
                    Some(constraint) => Some(
                        types::TransportProtocol::from_i32(constraint.protocol)
                            .ok_or(Status::invalid_argument("invalid transport protocol"))?
                            .into(),
                    ),
                    None => None,
                }
            } else {
                None
            };

            let providers = if let Some(ref provider_update) = settings.providers {
                if !provider_update.providers.is_empty() {
                    Some(Constraint::Only(
                        Providers::new(provider_update.providers.clone().into_iter()).map_err(
                            |_| Status::invalid_argument("must specify at least one provider"),
                        )?,
                    ))
                } else {
                    Some(Constraint::Any)
                }
            } else {
                None
            };
            let ip_version = if let Some(ref constraints) = settings.wireguard_constraints {
                match &constraints.ip_version {
                    Some(constraint) => match types::IpVersion::from_i32(constraint.protocol) {
                        Some(types::IpVersion::V4) => Some(IpVersion::V4),
                        Some(types::IpVersion::V6) => Some(IpVersion::V6),
                        None => {
                            return Err(Status::invalid_argument("invalid ip protocol version"))
                        }
                    },
                    None => None,
                }
            } else {
                None
            };

            Ok(RelaySettingsUpdate::Normal(RelayConstraintsUpdate {
                location,
                providers,
                tunnel_protocol,
                wireguard_constraints: settings.wireguard_constraints.map(|constraints| {
                    WireguardConstraints {
                        port: if constraints.port != 0 {
                            Constraint::Only(constraints.port as u16)
                        } else {
                            Constraint::Any
                        },
                        ip_version: Constraint::from(ip_version),
                    }
                }),
                openvpn_constraints: settings.openvpn_constraints.map(|constraints| {
                    OpenVpnConstraints {
                        port: if constraints.port != 0 {
                            Constraint::Only(constraints.port as u16)
                        } else {
                            Constraint::Any
                        },
                        protocol: Constraint::from(transport_protocol),
                    }
                }),
            }))
        }
    }
}

fn convert_wireguard_key_event(
    event: &mullvad_types::wireguard::KeygenEvent,
) -> types::KeygenEvent {
    use mullvad_types::wireguard::KeygenEvent::*;
    use types::keygen_event::KeygenEvent as ProtoEvent;

    types::KeygenEvent {
        event: match event {
            NewKey(_) => i32::from(ProtoEvent::NewKey),
            TooManyKeys => i32::from(ProtoEvent::TooManyKeys),
            GenerationFailure => i32::from(ProtoEvent::GenerationFailure),
        },
        new_key: if let NewKey(key) = event {
            Some(convert_public_key(&key))
        } else {
            None
        },
    }
}

fn convert_public_key(public_key: &wireguard::PublicKey) -> types::PublicKey {
    types::PublicKey {
        key: public_key.key.as_bytes().to_vec(),
        created: Some(types::Timestamp {
            seconds: public_key.created.timestamp(),
            nanos: 0,
        }),
    }
}

fn convert_relay_list_country(country: &RelayListCountry) -> types::RelayListCountry {
    let mut proto_country = types::RelayListCountry {
        name: country.name.clone(),
        code: country.code.clone(),
        cities: Vec::with_capacity(country.cities.len()),
    };

    for city in &country.cities {
        proto_country.cities.push(types::RelayListCity {
            name: city.name.clone(),
            code: city.code.clone(),
            latitude: city.latitude,
            longitude: city.longitude,
            relays: city
                .relays
                .iter()
                .map(|relay| convert_relay(relay))
                .collect(),
        });
    }

    proto_country
}

fn convert_relay(relay: &Relay) -> types::Relay {
    types::Relay {
        hostname: relay.hostname.clone(),
        ipv4_addr_in: relay.ipv4_addr_in.to_string(),
        ipv6_addr_in: relay
            .ipv6_addr_in
            .map(|addr| addr.to_string())
            .unwrap_or_default(),
        include_in_country: relay.include_in_country,
        active: relay.active,
        owned: relay.owned,
        provider: relay.provider.clone(),
        weight: relay.weight,
        tunnels: Some(types::RelayTunnels {
            openvpn: relay
                .tunnels
                .openvpn
                .iter()
                .map(|endpoint| types::OpenVpnEndpointData {
                    port: u32::from(endpoint.port),
                    protocol: i32::from(types::TransportProtocol::from(endpoint.protocol)),
                })
                .collect(),
            wireguard: relay
                .tunnels
                .wireguard
                .iter()
                .map(|endpoint| {
                    let port_ranges = endpoint
                        .port_ranges
                        .iter()
                        .map(|range| types::PortRange {
                            first: u32::from(range.0),
                            last: u32::from(range.1),
                        })
                        .collect();
                    types::WireguardEndpointData {
                        port_ranges,
                        ipv4_gateway: endpoint.ipv4_gateway.to_string(),
                        ipv6_gateway: endpoint.ipv6_gateway.to_string(),
                        public_key: endpoint.public_key.as_bytes().to_vec(),
                    }
                })
                .collect(),
        }),
        bridges: Some(types::RelayBridges {
            shadowsocks: relay
                .bridges
                .shadowsocks
                .iter()
                .map(|endpoint| types::ShadowsocksEndpointData {
                    port: u32::from(endpoint.port),
                    cipher: endpoint.cipher.clone(),
                    password: endpoint.password.clone(),
                    protocol: i32::from(types::TransportProtocol::from(endpoint.protocol)),
                })
                .collect(),
        }),
        location: relay.location.as_ref().map(|location| types::Location {
            country: location.country.clone(),
            country_code: location.country_code.clone(),
            city: location.city.clone(),
            city_code: location.city_code.clone(),
            latitude: location.latitude,
            longitude: location.longitude,
        }),
    }
}

fn convert_state(state: TunnelState) -> types::TunnelState {
    use talpid_types::tunnel::{
        ActionAfterDisconnect, ErrorStateCause, FirewallPolicyError, ParameterGenerationError,
    };
    use types::{
        error_state::{
            firewall_policy_error::ErrorType as PolicyErrorType, Cause as ProtoErrorCause,
            FirewallPolicyError as ProtoFirewallPolicyError,
            GenerationError as ProtoGenerationError,
        },
        tunnel_state::{self, State as ProtoState},
    };
    use TunnelState::*;

    let map_firewall_error = |firewall_error: &FirewallPolicyError| match firewall_error {
        FirewallPolicyError::Generic => ProtoFirewallPolicyError {
            r#type: i32::from(PolicyErrorType::Generic),
            ..Default::default()
        },
        #[cfg(windows)]
        FirewallPolicyError::Locked(blocking_app) => {
            let (lock_pid, lock_name) = match blocking_app {
                Some(app) => (app.pid, app.name.clone()),
                None => (0, "".to_string()),
            };

            ProtoFirewallPolicyError {
                r#type: i32::from(PolicyErrorType::Locked),
                lock_pid,
                lock_name,
            }
        }
    };

    let state = match state {
        Disconnected => ProtoState::Disconnected(tunnel_state::Disconnected {}),
        Connecting { endpoint, location } => ProtoState::Connecting(tunnel_state::Connecting {
            relay_info: Some(types::TunnelStateRelayInfo {
                tunnel_endpoint: Some(convert_endpoint(endpoint)),
                location: location.map(convert_geoip_location),
            }),
        }),
        Connected { endpoint, location } => ProtoState::Connected(tunnel_state::Connected {
            relay_info: Some(types::TunnelStateRelayInfo {
                tunnel_endpoint: Some(convert_endpoint(endpoint)),
                location: location.map(convert_geoip_location),
            }),
        }),
        Disconnecting(after_disconnect) => ProtoState::Disconnecting(tunnel_state::Disconnecting {
            after_disconnect: match after_disconnect {
                ActionAfterDisconnect::Nothing => i32::from(types::AfterDisconnect::Nothing),
                ActionAfterDisconnect::Block => i32::from(types::AfterDisconnect::Block),
                ActionAfterDisconnect::Reconnect => i32::from(types::AfterDisconnect::Reconnect),
            },
        }),
        Error(error_state) => ProtoState::Error(tunnel_state::Error {
            error_state: Some(types::ErrorState {
                cause: match error_state.cause() {
                    ErrorStateCause::AuthFailed(_) => i32::from(ProtoErrorCause::AuthFailed),
                    ErrorStateCause::Ipv6Unavailable => i32::from(ProtoErrorCause::Ipv6Unavailable),
                    ErrorStateCause::SetFirewallPolicyError(_) => {
                        i32::from(ProtoErrorCause::SetFirewallPolicyError)
                    }
                    ErrorStateCause::SetDnsError => i32::from(ProtoErrorCause::SetDnsError),
                    ErrorStateCause::StartTunnelError => {
                        i32::from(ProtoErrorCause::StartTunnelError)
                    }
                    ErrorStateCause::TunnelParameterError(_) => {
                        i32::from(ProtoErrorCause::TunnelParameterError)
                    }
                    ErrorStateCause::IsOffline => i32::from(ProtoErrorCause::IsOffline),
                    #[cfg(target_os = "android")]
                    ErrorStateCause::VpnPermissionDenied => {
                        i32::from(ProtoErrorCause::VpnPermissionDenied)
                    }
                },
                blocking_error: error_state.block_failure().map(map_firewall_error),
                auth_fail_reason: if let ErrorStateCause::AuthFailed(reason) = error_state.cause() {
                    reason.clone().unwrap_or_default()
                } else {
                    "".to_string()
                },
                parameter_error: if let ErrorStateCause::TunnelParameterError(reason) =
                    error_state.cause()
                {
                    match reason {
                        ParameterGenerationError::NoMatchingRelay => {
                            i32::from(ProtoGenerationError::NoMatchingRelay)
                        }
                        ParameterGenerationError::NoMatchingBridgeRelay => {
                            i32::from(ProtoGenerationError::NoMatchingBridgeRelay)
                        }
                        ParameterGenerationError::NoWireguardKey => {
                            i32::from(ProtoGenerationError::NoWireguardKey)
                        }
                        ParameterGenerationError::CustomTunnelHostResultionError => {
                            i32::from(ProtoGenerationError::CustomTunnelHostResolutionError)
                        }
                    }
                } else {
                    0
                },
                policy_error: if let ErrorStateCause::SetFirewallPolicyError(reason) =
                    error_state.cause()
                {
                    Some(map_firewall_error(reason))
                } else {
                    None
                },
            }),
        }),
    };

    types::TunnelState { state: Some(state) }
}

fn convert_endpoint(endpoint: talpid_types::net::TunnelEndpoint) -> types::TunnelEndpoint {
    use talpid_types::net;

    types::TunnelEndpoint {
        address: endpoint.endpoint.address.to_string(),
        protocol: i32::from(types::TransportProtocol::from(endpoint.endpoint.protocol)),
        tunnel_type: match endpoint.tunnel_type {
            net::TunnelType::Wireguard => i32::from(types::TunnelType::Wireguard),
            net::TunnelType::OpenVpn => i32::from(types::TunnelType::Openvpn),
        },
        proxy: endpoint.proxy.map(|proxy_ep| types::ProxyEndpoint {
            address: proxy_ep.endpoint.address.to_string(),
            protocol: i32::from(types::TransportProtocol::from(proxy_ep.endpoint.protocol)),
            proxy_type: match proxy_ep.proxy_type {
                net::proxy::ProxyType::Shadowsocks => i32::from(types::ProxyType::Shadowsocks),
                net::proxy::ProxyType::Custom => i32::from(types::ProxyType::Custom),
            },
        }),
    }
}

fn convert_geoip_location(geoip: GeoIpLocation) -> types::GeoIpLocation {
    types::GeoIpLocation {
        ipv4: geoip.ipv4.map(|ip| ip.to_string()).unwrap_or_default(),
        ipv6: geoip.ipv6.map(|ip| ip.to_string()).unwrap_or_default(),
        country: geoip.country,
        city: geoip.city.unwrap_or_default(),
        latitude: geoip.latitude,
        longitude: geoip.longitude,
        mullvad_exit_ip: geoip.mullvad_exit_ip,
        hostname: geoip.hostname.unwrap_or_default(),
        bridge_hostname: geoip.bridge_hostname.unwrap_or_default(),
    }
}

fn convert_proto_location(location: types::RelayLocation) -> Constraint<LocationConstraint> {
    if !location.hostname.is_empty() {
        Constraint::Only(LocationConstraint::Hostname(
            location.country,
            location.city,
            location.hostname,
        ))
    } else if !location.city.is_empty() {
        Constraint::Only(LocationConstraint::City(location.country, location.city))
    } else if !location.country.is_empty() {
        Constraint::Only(LocationConstraint::Country(location.country))
    } else {
        Constraint::Any
    }
}

pub struct ManagementInterfaceServer {
    subscriptions: Arc<RwLock<Vec<EventsListenerSender>>>,
    socket_path: String,
    server_abort_tx: triggered::Trigger,
    server_join_handle: Option<
        tokio::task::JoinHandle<std::result::Result<(), mullvad_management_interface::Error>>,
    >,
}

impl ManagementInterfaceServer {
    pub async fn start(tunnel_tx: DaemonCommandSender) -> Result<Self, Error> {
        let subscriptions = Arc::<RwLock<Vec<EventsListenerSender>>>::default();

        let socket_path = mullvad_paths::get_rpc_socket_path()
            .to_string_lossy()
            .to_string();

        let (server_abort_tx, server_abort_rx) = triggered::trigger();
        let (start_tx, start_rx) = mpsc::channel();
        let server = ManagementServiceImpl {
            daemon_tx: tunnel_tx,
            subscriptions: subscriptions.clone(),
        };
        let server_join_handle = tokio::spawn(mullvad_management_interface::spawn_rpc_server(
            server,
            start_tx,
            server_abort_rx,
        ));

        if let Err(_) = start_rx.recv() {
            return Err(server_join_handle
                .await
                .expect("Failed to resolve quit handle future")
                .map_err(Error::SetupError)
                .unwrap_err());
        }

        Ok(ManagementInterfaceServer {
            subscriptions,
            socket_path,
            server_abort_tx,
            server_join_handle: Some(server_join_handle),
        })
    }

    pub fn socket_path(&self) -> &str {
        &self.socket_path
    }

    pub fn event_broadcaster(&self) -> ManagementInterfaceEventBroadcaster {
        ManagementInterfaceEventBroadcaster {
            subscriptions: self.subscriptions.clone(),
            close_handle: self.server_abort_tx.clone(),
        }
    }

    /// Consumes the server and waits for it to finish.
    pub async fn run(self) {
        if let Some(server_join_handle) = self.server_join_handle {
            if let Err(error) = server_join_handle.await {
                log::error!("Management server panic: {:?}", error);
            }
            log::info!("Management interface shut down");
        }
    }
}

/// A handle that allows broadcasting messages to all subscribers of the management interface.
#[derive(Clone)]
pub struct ManagementInterfaceEventBroadcaster {
    subscriptions: Arc<RwLock<Vec<EventsListenerSender>>>,
    close_handle: triggered::Trigger,
}

impl EventListener for ManagementInterfaceEventBroadcaster {
    /// Sends a new state update to all `new_state` subscribers of the management interface.
    fn notify_new_state(&self, new_state: TunnelState) {
        self.notify(types::DaemonEvent {
            event: Some(daemon_event::Event::TunnelState(convert_state(new_state))),
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
        let mut new_list = types::RelayList {
            countries: Vec::new(),
        };
        new_list.countries.reserve(relay_list.countries.len());
        for country in &relay_list.countries {
            new_list.countries.push(convert_relay_list_country(country));
        }
        self.notify(types::DaemonEvent {
            event: Some(daemon_event::Event::RelayList(new_list)),
        })
    }

    fn notify_app_version(&self, app_version_info: version::AppVersionInfo) {
        log::debug!("Broadcasting new app version info");
        let new_info = types::AppVersionInfo::from(app_version_info);
        self.notify(types::DaemonEvent {
            event: Some(daemon_event::Event::VersionInfo(new_info)),
        })
    }

    fn notify_key_event(&self, key_event: mullvad_types::wireguard::KeygenEvent) {
        log::debug!("Broadcasting new wireguard key event");
        let new_event = convert_wireguard_key_event(&key_event);
        self.notify(types::DaemonEvent {
            event: Some(daemon_event::Event::KeyEvent(new_event)),
        })
    }
}

impl ManagementInterfaceEventBroadcaster {
    fn notify(&self, value: types::DaemonEvent) {
        let mut subscriptions = self.subscriptions.write();
        // TODO: using write-lock everywhere. use a mutex instead?
        subscriptions.retain(|tx| tx.send(Ok(value.clone())).is_ok());
    }
}

impl Drop for ManagementInterfaceEventBroadcaster {
    fn drop(&mut self) {
        self.close_handle.trigger();
    }
}

/// Converts [`mullvad_daemon::Error`] into a tonic status.
fn map_daemon_error(error: crate::Error) -> Status {
    use crate::Error as DaemonError;

    match error {
        DaemonError::RestError(error) => map_rest_error(error),
        DaemonError::SettingsError(error) => map_settings_error(error),
        DaemonError::AccountHistory(error) => map_account_history_error(error),
        DaemonError::NoAccountToken | DaemonError::NoAccountTokenHistory => {
            Status::unauthenticated(error.to_string())
        }
        error => Status::unknown(error.to_string()),
    }
}

/// Converts a REST API voucher error into a tonic status.
fn map_rest_voucher_error(error: RestError) -> Status {
    match error {
        RestError::ApiError(StatusCode::BAD_REQUEST, message) => match &message.as_str() {
            &mullvad_rpc::INVALID_VOUCHER => Status::new(Code::NotFound, INVALID_VOUCHER_MESSAGE),

            &mullvad_rpc::VOUCHER_USED => {
                Status::new(Code::ResourceExhausted, USED_VOUCHER_MESSAGE)
            }

            error => Status::unknown(format!("Voucher error: {}", error)),
        },
        error => map_rest_error(error),
    }
}

/// Converts a REST API error into a tonic status.
fn map_rest_error(error: RestError) -> Status {
    match error {
        RestError::ApiError(status, message)
            if status == StatusCode::UNAUTHORIZED || status == StatusCode::FORBIDDEN =>
        {
            Status::new(Code::Unauthenticated, message)
        }
        RestError::TimeoutError(_elapsed) => Status::deadline_exceeded("API request timed out"),
        RestError::HyperError(_) => Status::unavailable("Cannot reach the API"),
        error => Status::unknown(format!("REST error: {}", error)),
    }
}

/// Converts an instance of [`mullvad_daemon::settings::Error`] into a tonic status.
fn map_settings_error(error: settings::Error) -> Status {
    match error {
        settings::Error::DeleteError(..) | settings::Error::WriteError(..) => {
            Status::new(Code::FailedPrecondition, error.to_string())
        }
        settings::Error::SerializeError(..) => Status::new(Code::Internal, error.to_string()),
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

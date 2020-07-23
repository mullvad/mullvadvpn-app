use crate::{
    wireguard::DEFAULT_AUTOMATIC_KEY_ROTATION, DaemonCommand, DaemonCommandSender, EventListener,
};
use futures::compat::Future01CompatExt;
use futures01::{future, sync, Future};
use mullvad_paths;
use mullvad_rpc::{rest::Error as RestError, StatusCode};
use mullvad_types::{
    account::AccountToken,
    location::GeoIpLocation,
    relay_constraints::{
        BridgeConstraints, BridgeSettings, BridgeState, Constraint, LocationConstraint,
        OpenVpnConstraints, RelayConstraintsUpdate, RelaySettings, RelaySettingsUpdate,
        WireguardConstraints,
    },
    relay_list::{Relay, RelayList, RelayListCountry},
    settings::{Settings, TunnelOptions},
    states::{TargetState, TunnelState},
    version, wireguard, ConnectionConfig,
};
use parking_lot::RwLock;
use std::{
    io,
    sync::{mpsc, Arc},
};
use talpid_types::{
    net::{TransportProtocol, TunnelType},
    ErrorExt,
};

pub const INVALID_VOUCHER_CODE: i32 = -400;
pub const VOUCHER_USED_ALREADY_CODE: i32 = -401;
pub const INVALID_ACCOUNT_CODE: i32 = -200;


mod proto {
    tonic::include_proto!("mullvad_daemon.management_interface");
}

use proto::{
    daemon_event::Event as DaemonEventType,
    management_service_server::{ManagementService, ManagementServiceServer},
};

use tonic::{
    self,
    transport::{server::Connected, Server},
    Request, Response,
};

#[derive(err_derive::Error, Debug)]
#[error(no_from)]
pub enum Error {
    // Unable to start the management interface server
    #[error(display = "Unable to start management interface server")]
    SetupError(tonic::transport::Error),

    // Unable to set the permissions on the named pipe
    #[error(display = "Unable to set permissions for IPC endpoint")]
    PermissionsError(#[error(source)] io::Error),

    // Unable to start the tokio runtime
    #[error(display = "Failed to create the tokio runtime")]
    TokioRuntimeError(#[error(source)] tokio02::io::Error),
}

struct ManagementServiceImpl {
    daemon_tx: DaemonCommandSender,
    subscriptions: Arc<RwLock<Vec<EventsListenerSender>>>,
}

pub type ServiceResult<T> = std::result::Result<Response<T>, tonic::Status>;
type EventsListenerReceiver =
    tokio02::sync::mpsc::UnboundedReceiver<Result<proto::DaemonEvent, tonic::Status>>;
type EventsListenerSender =
    tokio02::sync::mpsc::UnboundedSender<Result<proto::DaemonEvent, tonic::Status>>;

#[tonic::async_trait]
impl ManagementService for ManagementServiceImpl {
    type GetRelayLocationsStream =
        tokio02::sync::mpsc::Receiver<Result<proto::RelayListCountry, tonic::Status>>;
    type GetSplitTunnelProcessesStream =
        tokio02::sync::mpsc::UnboundedReceiver<Result<i32, tonic::Status>>;
    type EventsListenStream = EventsListenerReceiver;

    // Control and get the tunnel state
    //

    async fn connect_tunnel(&self, _: Request<()>) -> ServiceResult<()> {
        log::debug!("connect_tunnel");

        let (tx, rx) = sync::oneshot::channel();
        self.send_command_to_daemon(DaemonCommand::SetTargetState(tx, TargetState::Secured))
            .and_then(|_| rx.map_err(|_| tonic::Status::internal("internal error")))
            .and_then(|result| match result {
                Ok(()) => Ok(Response::new(())),
                Err(()) => Err(tonic::Status::new(
                    tonic::Code::from(-900),
                    "No account token configured",
                )),
            })
            .compat()
            .await
    }

    async fn disconnect_tunnel(&self, _: Request<()>) -> ServiceResult<()> {
        log::debug!("disconnect_tunnel");

        let (tx, _) = sync::oneshot::channel();
        self.send_command_to_daemon(DaemonCommand::SetTargetState(tx, TargetState::Unsecured))
            .then(|_| Ok(Response::new(())))
            .compat()
            .await
    }

    async fn reconnect_tunnel(&self, _: Request<()>) -> ServiceResult<()> {
        log::debug!("reconnect_tunnel");
        self.send_command_to_daemon(DaemonCommand::Reconnect)
            .map(Response::new)
            .compat()
            .await
    }

    async fn get_tunnel_state(&self, _: Request<()>) -> ServiceResult<proto::TunnelState> {
        log::debug!("get_tunnel_state");
        let (tx, rx) = sync::oneshot::channel();
        self.send_command_to_daemon(DaemonCommand::GetState(tx))
            .and_then(|_| rx.map_err(|_| tonic::Status::internal("internal error")))
            .and_then(|state| Ok(Response::new(convert_state(state))))
            .compat()
            .await
    }

    // Control the daemon and receive events
    //

    async fn events_listen(&self, _: Request<()>) -> ServiceResult<Self::EventsListenStream> {
        let (tx, rx) = tokio02::sync::mpsc::unbounded_channel();

        let mut subscriptions = self.subscriptions.write();
        subscriptions.push(tx);

        Ok(Response::new(rx))
    }

    async fn prepare_restart(&self, _: Request<()>) -> ServiceResult<()> {
        log::debug!("prepare_restart");
        self.send_command_to_daemon(DaemonCommand::PrepareRestart)
            .map(Response::new)
            .compat()
            .await
    }

    async fn shutdown(&self, _: Request<()>) -> ServiceResult<()> {
        log::debug!("shutdown");
        self.send_command_to_daemon(DaemonCommand::Shutdown)
            .map(Response::new)
            .compat()
            .await
    }

    async fn factory_reset(&self, _: Request<()>) -> ServiceResult<()> {
        #[cfg(not(target_os = "android"))]
        {
            log::debug!("factory_reset");
            let (tx, rx) = sync::oneshot::channel();
            self.send_command_to_daemon(DaemonCommand::FactoryReset(tx))
                .and_then(|_| rx.map_err(|_| tonic::Status::internal("internal error")))
                .map(Response::new)
                .compat()
                .await
        }
        #[cfg(target_os = "android")]
        {
            Response::new(())
        }
    }

    async fn get_current_version(&self, _: Request<()>) -> ServiceResult<String> {
        log::debug!("get_current_version");
        let (tx, rx) = sync::oneshot::channel();
        self.send_command_to_daemon(DaemonCommand::GetCurrentVersion(tx))
            .and_then(|_| rx.map_err(|_| tonic::Status::internal("internal error")))
            .map(Response::new)
            .compat()
            .await
    }

    async fn get_version_info(&self, _: Request<()>) -> ServiceResult<proto::AppVersionInfo> {
        log::debug!("get_version_info");

        let (tx, rx) = sync::oneshot::channel();
        let app_version_info = self
            .send_command_to_daemon(DaemonCommand::GetVersionInfo(tx))
            .and_then(|_| rx.map_err(|_| tonic::Status::internal("internal error")))
            .compat()
            .await?;

        Ok(Response::new(convert_version_info(&app_version_info)))
    }

    // Relays and tunnel constraints
    //

    async fn update_relay_locations(&self, _: Request<()>) -> ServiceResult<()> {
        log::debug!("update_relay_locations");
        self.send_command_to_daemon(DaemonCommand::UpdateRelayLocations)
            .compat()
            .await
            .map(Response::new)
    }

    async fn update_relay_settings(
        &self,
        request: Request<proto::RelaySettingsUpdate>,
    ) -> ServiceResult<()> {
        log::debug!("update_relay_settings");
        let (tx, rx) = sync::oneshot::channel();
        let constraints_update = convert_relay_settings_update(&request.into_inner())?;

        let message = DaemonCommand::UpdateRelaySettings(tx, constraints_update);
        self.send_command_to_daemon(message)
            .and_then(|_| rx.map_err(|_| tonic::Status::internal("internal error")))
            .map(Response::new)
            .compat()
            .await
    }

    async fn get_relay_locations(
        &self,
        _: Request<()>,
    ) -> ServiceResult<Self::GetRelayLocationsStream> {
        log::debug!("get_relay_locations");

        let (tx, rx) = sync::oneshot::channel();
        let locations = self
            .send_command_to_daemon(DaemonCommand::GetRelayLocations(tx))
            .and_then(|_| rx.map_err(|_| tonic::Status::internal("internal error")))
            .compat()
            .await?;

        let (mut stream_tx, stream_rx) = tokio02::sync::mpsc::channel(locations.countries.len());

        tokio02::spawn(async move {
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

    async fn get_current_location(&self, _: Request<()>) -> ServiceResult<proto::GeoIpLocation> {
        log::debug!("get_current_location");
        let (tx, rx) = sync::oneshot::channel();
        self.send_command_to_daemon(DaemonCommand::GetCurrentLocation(tx))
            .and_then(|_| rx.map_err(|_| tonic::Status::internal("internal error")))
            .and_then(|geoip| {
                if let Some(geoip) = geoip {
                    Ok(Response::new(convert_geoip_location(geoip)))
                } else {
                    Err(tonic::Status::not_found("no location was found"))
                }
            })
            .compat()
            .await
    }

    async fn set_bridge_settings(
        &self,
        request: Request<proto::BridgeSettings>,
    ) -> ServiceResult<()> {
        use proto::bridge_settings::Type as BridgeSettingType;
        use talpid_types::net;

        let settings = request
            .into_inner()
            .r#type
            .ok_or(tonic::Status::invalid_argument("no settings provided"))?;

        let settings = match settings {
            BridgeSettingType::Normal(constraints) => {
                let constraint = match constraints.location {
                    None => Constraint::Any,
                    Some(location) => {
                        let hostname = location.hostname;
                        match hostname.len() {
                            0 => Constraint::Any,
                            1 => Constraint::Only(LocationConstraint::Country(hostname[0].clone())),
                            2 => Constraint::Only(LocationConstraint::City(
                                hostname[0].clone(),
                                hostname[1].clone(),
                            )),
                            3 => Constraint::Only(LocationConstraint::Hostname(
                                hostname[0].clone(),
                                hostname[1].clone(),
                                hostname[2].clone(),
                            )),
                            _ => {
                                return Err(tonic::Status::invalid_argument(
                                    "expected 1-3 elements",
                                ))
                            }
                        }
                    }
                };

                BridgeSettings::Normal(BridgeConstraints {
                    location: constraint,
                })
            }
            BridgeSettingType::Local(proxy_settings) => {
                let peer = proxy_settings
                    .peer
                    .parse()
                    .map_err(|_| tonic::Status::invalid_argument("failed to parse peer address"))?;
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
                    .map_err(|_| tonic::Status::invalid_argument("failed to parse IP address"))?;
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
                    .map_err(|_| tonic::Status::invalid_argument("failed to parse peer address"))?;
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

        let (tx, rx) = sync::oneshot::channel();
        self.send_command_to_daemon(DaemonCommand::SetBridgeSettings(tx, settings))
            .and_then(|_| rx.map_err(|_| tonic::Status::internal("internal error")))
            .and_then(|settings_result| {
                settings_result.map_err(|_| tonic::Status::internal("internal error"))
            })
            .map(Response::new)
            .compat()
            .await
    }

    async fn set_bridge_state(&self, request: Request<proto::BridgeState>) -> ServiceResult<()> {
        use proto::bridge_state::State;

        let bridge_state = match State::from_i32(request.into_inner().state) {
            Some(State::Auto) => BridgeState::Auto,
            Some(State::On) => BridgeState::On,
            Some(State::Off) => BridgeState::Off,
            None => return Err(tonic::Status::invalid_argument("unknown bridge state")),
        };

        log::debug!("set_bridge_state({:?})", bridge_state);
        let (tx, rx) = sync::oneshot::channel();
        self.send_command_to_daemon(DaemonCommand::SetBridgeState(tx, bridge_state))
            .and_then(|_| rx.map_err(|_| tonic::Status::internal("internal error")))
            .and_then(|settings_result| {
                settings_result.map_err(|_| tonic::Status::internal("internal error"))
            })
            .map(Response::new)
            .compat()
            .await
    }

    // Settings
    //

    async fn get_settings(&self, _: Request<()>) -> ServiceResult<proto::Settings> {
        log::debug!("get_settings");
        let (tx, rx) = sync::oneshot::channel();
        self.send_command_to_daemon(DaemonCommand::GetSettings(tx))
            .and_then(|_| rx.map_err(|_| tonic::Status::internal("internal error")))
            .map(|settings| Response::new(convert_settings(&settings)))
            .compat()
            .await
    }

    async fn set_allow_lan(&self, request: Request<bool>) -> ServiceResult<()> {
        let allow_lan = request.into_inner();
        log::debug!("set_allow_lan({})", allow_lan);
        let (tx, rx) = sync::oneshot::channel();
        self.send_command_to_daemon(DaemonCommand::SetAllowLan(tx, allow_lan))
            .and_then(|_| rx.map_err(|_| tonic::Status::internal("internal error")))
            .map(Response::new)
            .compat()
            .await
    }

    async fn set_show_beta_releases(&self, request: Request<bool>) -> ServiceResult<()> {
        let enabled = request.into_inner();
        log::debug!("set_show_beta_releases({})", enabled);
        let (tx, rx) = sync::oneshot::channel();
        self.send_command_to_daemon(DaemonCommand::SetShowBetaReleases(tx, enabled))
            .and_then(|_| rx.map_err(|_| tonic::Status::internal("internal error")))
            .map(Response::new)
            .compat()
            .await
    }

    async fn set_block_when_disconnected(&self, request: Request<bool>) -> ServiceResult<()> {
        let block_when_disconnected = request.into_inner();
        log::debug!("set_block_when_disconnected({})", block_when_disconnected);
        let (tx, rx) = sync::oneshot::channel();
        self.send_command_to_daemon(DaemonCommand::SetBlockWhenDisconnected(
            tx,
            block_when_disconnected,
        ))
        .and_then(|_| rx.map_err(|_| tonic::Status::internal("internal error")))
        .map(Response::new)
        .compat()
        .await
    }

    async fn set_auto_connect(&self, request: Request<bool>) -> ServiceResult<()> {
        let auto_connect = request.into_inner();
        log::debug!("set_auto_connect({})", auto_connect);
        let (tx, rx) = sync::oneshot::channel();
        self.send_command_to_daemon(DaemonCommand::SetAutoConnect(tx, auto_connect))
            .and_then(|_| rx.map_err(|_| tonic::Status::internal("internal error")))
            .map(Response::new)
            .compat()
            .await
    }

    async fn set_openvpn_mssfix(&self, request: Request<u32>) -> ServiceResult<()> {
        let mssfix = request.into_inner();
        let mssfix = if mssfix != 0 {
            Some(mssfix as u16)
        } else {
            None
        };
        log::debug!("set_openvpn_mssfix({:?})", mssfix);
        let (tx, rx) = sync::oneshot::channel();
        self.send_command_to_daemon(DaemonCommand::SetOpenVpnMssfix(tx, mssfix))
            .and_then(|_| rx.map_err(|_| tonic::Status::internal("internal error")))
            .map(Response::new)
            .compat()
            .await
    }

    async fn set_wireguard_mtu(&self, request: Request<u32>) -> ServiceResult<()> {
        let mtu = request.into_inner();
        let mtu = if mtu != 0 { Some(mtu as u16) } else { None };
        log::debug!("set_wireguard_mtu({:?})", mtu);
        let (tx, rx) = sync::oneshot::channel();
        self.send_command_to_daemon(DaemonCommand::SetWireguardMtu(tx, mtu))
            .and_then(|_| rx.map_err(|_| tonic::Status::internal("internal error")))
            .map(Response::new)
            .compat()
            .await
    }

    async fn set_enable_ipv6(&self, request: Request<bool>) -> ServiceResult<()> {
        let enable_ipv6 = request.into_inner();
        log::debug!("set_enable_ipv6({})", enable_ipv6);
        let (tx, rx) = sync::oneshot::channel();
        self.send_command_to_daemon(DaemonCommand::SetEnableIpv6(tx, enable_ipv6))
            .and_then(|_| rx.map_err(|_| tonic::Status::internal("internal error")))
            .map(Response::new)
            .compat()
            .await
    }

    // Account management
    //

    async fn create_new_account(&self, _: Request<()>) -> ServiceResult<String> {
        let (tx, rx) = sync::oneshot::channel();
        self.send_command_to_daemon(DaemonCommand::CreateNewAccount(tx))
            .and_then(|_| rx.map_err(|_| tonic::Status::internal("internal error")))
            .and_then(|result| match result {
                Ok(account_token) => Ok(Response::new(account_token)),
                Err(_) => Err(tonic::Status::internal("internal error")),
            })
            .compat()
            .await
    }

    async fn set_account(&self, request: Request<AccountToken>) -> ServiceResult<()> {
        log::debug!("set_account");
        let account_token = request.into_inner();
        let account_token = if account_token == "" {
            None
        } else {
            Some(account_token)
        };
        let (tx, rx) = sync::oneshot::channel();
        self.send_command_to_daemon(DaemonCommand::SetAccount(tx, account_token))
            .and_then(|_| {
                rx.map(Response::new)
                    .map_err(|_| tonic::Status::internal("internal error"))
            })
            .compat()
            .await
    }

    async fn get_account_data(
        &self,
        request: Request<AccountToken>,
    ) -> ServiceResult<proto::AccountData> {
        log::debug!("get_account_data");
        let account_token = request.into_inner();
        let (tx, rx) = sync::oneshot::channel();
        self.send_command_to_daemon(DaemonCommand::GetAccountData(tx, account_token))
            .and_then(|_| rx.map_err(|_| tonic::Status::internal("internal error")))
            .and_then(|rpc_future| {
                rpc_future
                    .map(|account_data| {
                        Response::new(proto::AccountData {
                            expiry: Some(prost_types::Timestamp {
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
                        map_rest_account_error(error)
                    })
            })
            .compat()
            .await
    }

    async fn get_account_history(&self, _: Request<()>) -> ServiceResult<proto::AccountHistory> {
        // TODO: this might be a stream
        log::debug!("get_account_history");
        let (tx, rx) = sync::oneshot::channel();
        self.send_command_to_daemon(DaemonCommand::GetAccountHistory(tx))
            .and_then(|_| rx.map_err(|_| tonic::Status::internal("internal error")))
            .map(|history| Response::new(proto::AccountHistory { token: history }))
            .compat()
            .await
    }

    async fn remove_account_from_history(
        &self,
        request: Request<AccountToken>,
    ) -> ServiceResult<()> {
        log::debug!("remove_account_from_history");
        let account_token = request.into_inner();
        let (tx, rx) = sync::oneshot::channel();
        self.send_command_to_daemon(DaemonCommand::RemoveAccountFromHistory(tx, account_token))
            .and_then(|_| rx.map_err(|_| tonic::Status::internal("internal error")))
            .map(Response::new)
            .compat()
            .await
    }

    async fn clear_account_history(&self, _: Request<()>) -> ServiceResult<()> {
        log::debug!("clear_account_history");
        let (tx, rx) = sync::oneshot::channel();
        self.send_command_to_daemon(DaemonCommand::ClearAccountHistory(tx))
            .and_then(|_| rx.map_err(|_| tonic::Status::internal("internal error")))
            .map(Response::new)
            .compat()
            .await
    }

    async fn get_www_auth_token(&self, _: Request<()>) -> ServiceResult<String> {
        log::debug!("get_www_auth_token");
        let (tx, rx) = sync::oneshot::channel();
        self.send_command_to_daemon(DaemonCommand::GetWwwAuthToken(tx))
            .and_then(|_| rx.map_err(|_| tonic::Status::internal("internal error")))
            .and_then(|rpc_future| {
                rpc_future
                    .map(Response::new)
                    .map_err(|error: mullvad_rpc::rest::Error| {
                        log::error!(
                            "Unable to get account data from API: {}",
                            error.display_chain()
                        );
                        map_rest_account_error(error)
                    })
            })
            .compat()
            .await
    }

    async fn submit_voucher(
        &self,
        request: Request<String>,
    ) -> ServiceResult<proto::VoucherSubmission> {
        log::debug!("submit_voucher");
        let voucher = request.into_inner();
        let (tx, rx) = sync::oneshot::channel();
        self.send_command_to_daemon(DaemonCommand::SubmitVoucher(tx, voucher))
            .and_then(|_| rx.map_err(|_| tonic::Status::internal("internal error")))
            .and_then(|f| {
                f.map(|submission| {
                    Response::new(proto::VoucherSubmission {
                        seconds_added: submission.time_added,
                        new_expiry: Some(prost_types::Timestamp {
                            seconds: submission.new_expiry.timestamp(),
                            nanos: 0,
                        }),
                    })
                })
                .map_err(|e| match e {
                    RestError::ApiError(StatusCode::BAD_REQUEST, message) => {
                        match &message.as_str() {
                            &mullvad_rpc::INVALID_VOUCHER => tonic::Status::new(
                                tonic::Code::from_i32(INVALID_VOUCHER_CODE),
                                message,
                            ),

                            &mullvad_rpc::VOUCHER_USED => tonic::Status::new(
                                tonic::Code::from_i32(VOUCHER_USED_ALREADY_CODE),
                                message,
                            ),

                            _ => tonic::Status::internal("internal error"),
                        }
                    }
                    _ => tonic::Status::internal("internal error"),
                })
            })
            .compat()
            .await
    }

    // WireGuard key management
    //

    async fn set_wireguard_rotation_interval(&self, request: Request<u32>) -> ServiceResult<()> {
        let interval = request.into_inner();

        log::debug!("set_wireguard_rotation_interval({:?})", interval);
        let (tx, rx) = sync::oneshot::channel();
        self.send_command_to_daemon(DaemonCommand::SetWireguardRotationInterval(
            tx,
            Some(interval),
        ))
        .and_then(|_| rx.map_err(|_| tonic::Status::internal("internal error")))
        .map(Response::new)
        .compat()
        .await
    }

    async fn reset_wireguard_rotation_interval(&self, _: Request<()>) -> ServiceResult<()> {
        log::debug!("reset_wireguard_rotation_interval");
        let (tx, rx) = sync::oneshot::channel();
        self.send_command_to_daemon(DaemonCommand::SetWireguardRotationInterval(tx, None))
            .and_then(|_| rx.map_err(|_| tonic::Status::internal("internal error")))
            .map(Response::new)
            .compat()
            .await
    }

    async fn generate_wireguard_key(&self, _: Request<()>) -> ServiceResult<proto::KeygenEvent> {
        // TODO: return error for TooManyKeys, GenerationFailure
        // on success, simply return the new key or nil
        log::debug!("generate_wireguard_key");
        let (tx, rx) = sync::oneshot::channel();
        self.send_command_to_daemon(DaemonCommand::GenerateWireguardKey(tx))
            .and_then(|_| rx.map_err(|_| tonic::Status::internal("internal error")))
            .map(|event| Response::new(convert_wireguard_key_event(&event)))
            .compat()
            .await
    }

    async fn get_wireguard_key(&self, _: Request<()>) -> ServiceResult<proto::PublicKey> {
        log::debug!("get_wireguard_key");
        let (tx, rx) = sync::oneshot::channel();
        self.send_command_to_daemon(DaemonCommand::GetWireguardKey(tx))
            .and_then(|_| rx.map_err(|_| tonic::Status::internal("internal error")))
            .then(|response| match response {
                Ok(Some(key)) => Ok(Response::new(convert_public_key(&key))),
                Ok(None) => Err(tonic::Status::not_found("no WireGuard key was found")),
                Err(e) => Err(e),
            })
            .compat()
            .await
    }

    async fn verify_wireguard_key(&self, _: Request<()>) -> ServiceResult<bool> {
        log::debug!("verify_wireguard_key");
        let (tx, rx) = sync::oneshot::channel();
        self.send_command_to_daemon(DaemonCommand::VerifyWireguardKey(tx))
            .and_then(|_| rx.map_err(|_| tonic::Status::internal("internal error")))
            .map(Response::new)
            .compat()
            .await
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
            let (tx, rx) = sync::oneshot::channel();
            let pids = self
                .send_command_to_daemon(DaemonCommand::GetSplitTunnelProcesses(tx))
                .and_then(|_| rx.map_err(|_| tonic::Status::internal("internal error")))
                .compat()
                .await?;

            let (tx, rx) = tokio02::sync::mpsc::unbounded_channel();
            tokio02::spawn(async move {
                for pid in pids {
                    let _ = tx.send(Ok(pid));
                }
            });

            Ok(Response::new(rx))
        }
        #[cfg(not(target_os = "linux"))]
        {
            let (_, rx) = tokio02::sync::mpsc::unbounded_channel();
            Ok(Response::new(rx))
        }
    }

    #[cfg(target_os = "linux")]
    async fn add_split_tunnel_process(&self, request: Request<i32>) -> ServiceResult<()> {
        let pid = request.into_inner();
        log::debug!("add_split_tunnel_process");
        let (tx, rx) = sync::oneshot::channel();
        self.send_command_to_daemon(DaemonCommand::AddSplitTunnelProcess(tx, pid))
            .and_then(|_| rx.map_err(|_| tonic::Status::internal("internal error")))
            .map(Response::new)
            .compat()
            .await
    }
    #[cfg(not(target_os = "linux"))]
    async fn add_split_tunnel_process(&self, _: Request<i32>) -> ServiceResult<()> {
        Ok(Response::new(()))
    }

    #[cfg(target_os = "linux")]
    async fn remove_split_tunnel_process(&self, request: Request<i32>) -> ServiceResult<()> {
        let pid = request.into_inner();
        log::debug!("remove_split_tunnel_process");
        let (tx, rx) = sync::oneshot::channel();
        self.send_command_to_daemon(DaemonCommand::RemoveSplitTunnelProcess(tx, pid))
            .and_then(|_| rx.map_err(|_| tonic::Status::internal("internal error")))
            .map(Response::new)
            .compat()
            .await
    }
    #[cfg(not(target_os = "linux"))]
    async fn remove_split_tunnel_process(&self, _: Request<i32>) -> ServiceResult<()> {
        Ok(Response::new(()))
    }

    async fn clear_split_tunnel_processes(&self, _: Request<()>) -> ServiceResult<()> {
        #[cfg(target_os = "linux")]
        {
            log::debug!("clear_split_tunnel_processes");
            let (tx, rx) = sync::oneshot::channel();
            self.send_command_to_daemon(DaemonCommand::ClearSplitTunnelProcesses(tx))
                .and_then(|_| rx.map_err(|_| tonic::Status::internal("internal error")))
                .map(Response::new)
                .compat()
                .await
        }
        #[cfg(not(target_os = "linux"))]
        {
            Ok(Response::new(()))
        }
    }
}

impl ManagementServiceImpl {
    /// Sends a command to the daemon and maps the error to an RPC error.
    fn send_command_to_daemon(
        &self,
        command: DaemonCommand,
    ) -> impl Future<Item = (), Error = tonic::Status> {
        future::result(
            self.daemon_tx
                .send(command)
                .map_err(|_| tonic::Status::internal("internal error")),
        )
    }
}

fn convert_settings(settings: &Settings) -> proto::Settings {
    proto::Settings {
        account_token: settings.get_account_token().unwrap_or_default(),
        relay_settings: Some(convert_relay_settings(&settings.get_relay_settings())),
        bridge_settings: Some(convert_bridge_settings(&settings.bridge_settings)),
        bridge_state: Some(convert_bridge_state(settings.get_bridge_state())),
        allow_lan: settings.allow_lan,
        block_when_disconnected: settings.block_when_disconnected,
        auto_connect: settings.auto_connect,
        tunnel_options: Some(convert_tunnel_options(&settings.tunnel_options)),
        show_beta_releases: settings.show_beta_releases,
    }
}

fn convert_relay_settings_update(
    settings: &proto::RelaySettingsUpdate,
) -> Result<RelaySettingsUpdate, tonic::Status> {
    use mullvad_types::CustomTunnelEndpoint;
    use proto::{
        connection_config::Config as ProtoConnectionConfig,
        relay_settings_update::Type as ProtoUpdateType,
    };
    use talpid_types::net::{self, openvpn, wireguard};

    let update_value = settings
        .r#type
        .clone()
        .ok_or(tonic::Status::invalid_argument("missing relay settings"))?;

    match update_value {
        ProtoUpdateType::Custom(settings) => {
            let config = settings
                .config
                .ok_or(tonic::Status::invalid_argument("missing relay settings"))?;
            let config = config
                .config
                .ok_or(tonic::Status::invalid_argument("missing relay settings"))?;
            let config = match config {
                ProtoConnectionConfig::Openvpn(config) => {
                    let address = match config.address.parse() {
                        Ok(address) => address,
                        Err(_) => return Err(tonic::Status::invalid_argument("invalid address")),
                    };

                    ConnectionConfig::OpenVpn(openvpn::ConnectionConfig {
                        endpoint: net::Endpoint {
                            address,
                            protocol: match proto::TransportProtocol::from_i32(config.protocol) {
                                Some(proto::TransportProtocol::Udp) => TransportProtocol::Udp,
                                Some(proto::TransportProtocol::Tcp) => TransportProtocol::Tcp,
                                None | Some(proto::TransportProtocol::AnyProtocol) => {
                                    return Err(tonic::Status::invalid_argument(
                                        "unknown transport protocol",
                                    ))
                                }
                            },
                        },
                        username: config.username.clone(),
                        password: config.password.clone(),
                    })
                }
                ProtoConnectionConfig::Wireguard(config) => {
                    let tunnel = config
                        .tunnel
                        .ok_or(tonic::Status::invalid_argument("missing tunnel config"))?;

                    // Copy the private key to an array
                    if tunnel.private_key.len() != 32 {
                        return Err(tonic::Status::invalid_argument("invalid private key"));
                    }

                    let mut private_key = [0; 32];
                    let buffer = &tunnel.private_key[..private_key.len()];
                    private_key.copy_from_slice(buffer);

                    let peer = config
                        .peer
                        .ok_or(tonic::Status::invalid_argument("missing peer config"))?;

                    // Copy the public key to an array
                    if peer.public_key.len() != 32 {
                        return Err(tonic::Status::invalid_argument("invalid public key"));
                    }

                    let mut public_key = [0; 32];
                    let buffer = &peer.public_key[..public_key.len()];
                    public_key.copy_from_slice(buffer);

                    let ipv4_gateway = match config.ipv4_gateway.parse() {
                        Ok(address) => address,
                        Err(_) => {
                            return Err(tonic::Status::invalid_argument("invalid IPv4 gateway"))
                        }
                    };
                    let ipv6_gateway = if !config.ipv6_gateway.is_empty() {
                        let address = match config.ipv6_gateway.parse() {
                            Ok(address) => address,
                            Err(_) => {
                                return Err(tonic::Status::invalid_argument("invalid IPv6 gateway"))
                            }
                        };
                        Some(address)
                    } else {
                        None
                    };

                    let endpoint = match peer.endpoint.parse() {
                        Ok(address) => address,
                        Err(_) => {
                            return Err(tonic::Status::invalid_argument("invalid peer address"))
                        }
                    };

                    let mut tunnel_addresses = Vec::new();
                    for address in tunnel.addresses {
                        let address = address
                            .parse()
                            .map_err(|_| tonic::Status::invalid_argument("invalid address"))?;
                        tunnel_addresses.push(address);
                    }

                    let mut allowed_ips = Vec::new();
                    for address in peer.allowed_ips {
                        let address = address
                            .parse()
                            .map_err(|_| tonic::Status::invalid_argument("invalid address"))?;
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
            let location = match settings.location {
                Some(location) => {
                    let location = location.hostname;
                    match location.len() {
                        0 => Some(Constraint::Any),
                        1 => Some(Constraint::Only(LocationConstraint::Country(
                            location[0].clone(),
                        ))),
                        2 => Some(Constraint::Only(LocationConstraint::City(
                            location[0].clone(),
                            location[1].clone(),
                        ))),
                        3 => Some(Constraint::Only(LocationConstraint::Hostname(
                            location[0].clone(),
                            location[1].clone(),
                            location[2].clone(),
                        ))),
                        _ => return Err(tonic::Status::invalid_argument("expected 0-3 elements")),
                    }
                }
                None => None,
            };

            let tunnel_protocol = if let Some(update) = settings.tunnel_type {
                match proto::TunnelType::from_i32(update.tunnel_type) {
                    Some(proto::TunnelType::AnyTunnel) => Some(Constraint::Any),
                    Some(proto::TunnelType::Openvpn) => Some(Constraint::Only(TunnelType::OpenVpn)),
                    Some(proto::TunnelType::Wireguard) => {
                        Some(Constraint::Only(TunnelType::Wireguard))
                    }
                    None => return Err(tonic::Status::invalid_argument("unknown tunnel protocol")),
                }
            } else {
                None
            };

            Ok(RelaySettingsUpdate::Normal(RelayConstraintsUpdate {
                location,
                tunnel_protocol,
                wireguard_constraints: settings.wireguard_constraints.map(|constraints| {
                    WireguardConstraints {
                        port: if constraints.port != 0 {
                            Constraint::Only(constraints.port as u16)
                        } else {
                            Constraint::Any
                        },
                    }
                }),
                openvpn_constraints: settings.openvpn_constraints.map(|constraints| {
                    OpenVpnConstraints {
                        port: if constraints.port != 0 {
                            Constraint::Only(constraints.port as u16)
                        } else {
                            Constraint::Any
                        },
                        protocol: match proto::TransportProtocol::from_i32(constraints.protocol) {
                            Some(proto::TransportProtocol::Udp) => {
                                Constraint::Only(TransportProtocol::Udp)
                            }
                            Some(proto::TransportProtocol::Tcp) => {
                                Constraint::Only(TransportProtocol::Tcp)
                            }
                            _ => Constraint::Any,
                        },
                    }
                }),
            }))
        }
    }
}

fn convert_relay_settings(settings: &RelaySettings) -> proto::RelaySettings {
    use proto::relay_settings;

    let endpoint = match settings {
        RelaySettings::CustomTunnelEndpoint(endpoint) => {
            relay_settings::Endpoint::Custom(proto::CustomRelaySettings {
                host: endpoint.host.clone(),
                config: Some(convert_connection_config(&endpoint.config)),
            })
        }
        RelaySettings::Normal(constraints) => {
            relay_settings::Endpoint::Normal(proto::NormalRelaySettings {
                location: convert_location_constraint(&constraints.location),
                tunnel_type: match constraints.tunnel_protocol {
                    Constraint::Any => proto::TunnelType::AnyTunnel as i32,
                    Constraint::Only(TunnelType::Wireguard) => proto::TunnelType::Wireguard as i32,
                    Constraint::Only(TunnelType::OpenVpn) => proto::TunnelType::Openvpn as i32,
                },

                wireguard_constraints: Some(proto::WireguardConstraints {
                    port: constraints.wireguard_constraints.port.unwrap_or(0) as u32,
                }),

                openvpn_constraints: Some(proto::OpenvpnConstraints {
                    port: constraints.openvpn_constraints.port.unwrap_or(0) as u32,
                    protocol: constraints
                        .openvpn_constraints
                        .protocol
                        .map(|protocol| match protocol {
                            TransportProtocol::Tcp => proto::TransportProtocol::Tcp,
                            TransportProtocol::Udp => proto::TransportProtocol::Udp,
                        })
                        .unwrap_or(proto::TransportProtocol::AnyProtocol)
                        as i32,
                }),
            })
        }
    };

    proto::RelaySettings {
        endpoint: Some(endpoint),
    }
}

fn convert_connection_config(config: &ConnectionConfig) -> proto::ConnectionConfig {
    use proto::connection_config;

    proto::ConnectionConfig {
        config: Some(match config {
            ConnectionConfig::OpenVpn(config) => {
                connection_config::Config::Openvpn(connection_config::OpenvpnConfig {
                    address: config.endpoint.address.to_string(),
                    protocol: match config.endpoint.protocol {
                        TransportProtocol::Tcp => proto::TransportProtocol::Tcp as i32,
                        TransportProtocol::Udp => proto::TransportProtocol::Udp as i32,
                    },
                    username: config.username.clone(),
                    password: config.password.clone(),
                })
            }
            ConnectionConfig::Wireguard(config) => {
                connection_config::Config::Wireguard(connection_config::WireguardConfig {
                    tunnel: Some(connection_config::wireguard_config::TunnelConfig {
                        private_key: config.tunnel.private_key.to_bytes().to_vec(),
                        addresses: config
                            .tunnel
                            .addresses
                            .iter()
                            .map(|address| address.to_string())
                            .collect(),
                    }),
                    peer: Some(connection_config::wireguard_config::PeerConfig {
                        public_key: config.peer.public_key.as_bytes().to_vec(),
                        allowed_ips: config
                            .peer
                            .allowed_ips
                            .iter()
                            .map(|address| address.to_string())
                            .collect(),
                        endpoint: config.peer.endpoint.to_string(),
                    }),
                    ipv4_gateway: config.ipv4_gateway.to_string(),
                    ipv6_gateway: config
                        .ipv6_gateway
                        .as_ref()
                        .map(|address| address.to_string())
                        .unwrap_or_default(),
                })
            }
        }),
    }
}

fn convert_bridge_settings(settings: &BridgeSettings) -> proto::BridgeSettings {
    use proto::bridge_settings::{self, Type as BridgeSettingType};
    use talpid_types::net;

    let settings = match settings {
        BridgeSettings::Normal(constraints) => {
            BridgeSettingType::Normal(proto::bridge_settings::BridgeConstraints {
                location: convert_location_constraint(&constraints.location),
            })
        }
        BridgeSettings::Custom(proxy_settings) => match proxy_settings {
            net::openvpn::ProxySettings::Local(proxy_settings) => {
                BridgeSettingType::Local(bridge_settings::LocalProxySettings {
                    port: proxy_settings.port as u32,
                    peer: proxy_settings.peer.to_string(),
                })
            }
            net::openvpn::ProxySettings::Remote(proxy_settings) => {
                BridgeSettingType::Remote(bridge_settings::RemoteProxySettings {
                    address: proxy_settings.address.to_string(),
                    auth: proxy_settings.auth.as_ref().map(|auth| {
                        bridge_settings::RemoteProxyAuth {
                            username: auth.username.clone(),
                            password: auth.password.clone(),
                        }
                    }),
                })
            }
            net::openvpn::ProxySettings::Shadowsocks(proxy_settings) => {
                BridgeSettingType::Shadowsocks(bridge_settings::ShadowsocksProxySettings {
                    peer: proxy_settings.peer.to_string(),
                    password: proxy_settings.password.clone(),
                    cipher: proxy_settings.cipher.clone(),
                })
            }
        },
    };

    proto::BridgeSettings {
        r#type: Some(settings),
    }
}

fn convert_wireguard_key_event(
    event: &mullvad_types::wireguard::KeygenEvent,
) -> proto::KeygenEvent {
    use mullvad_types::wireguard::KeygenEvent::*;
    use proto::keygen_event::KeygenEvent as ProtoEvent;

    proto::KeygenEvent {
        event: match event {
            NewKey(_) => ProtoEvent::NewKey as i32,
            TooManyKeys => ProtoEvent::TooManyKeys as i32,
            GenerationFailure => ProtoEvent::GenerationFailure as i32,
        },
        new_key: if let NewKey(key) = event {
            Some(convert_public_key(&key))
        } else {
            None
        },
    }
}

fn convert_public_key(public_key: &wireguard::PublicKey) -> proto::PublicKey {
    proto::PublicKey {
        key: public_key.key.as_bytes().to_vec(),
        created: Some(prost_types::Timestamp {
            seconds: public_key.created.timestamp(),
            nanos: 0,
        }),
    }
}

fn convert_location_constraint(
    location: &Constraint<LocationConstraint>,
) -> Option<proto::RelayLocation> {
    let location = match location {
        Constraint::Any => None,
        Constraint::Only(location) => Some(match location {
            LocationConstraint::Country(country) => [country.clone()].to_vec(),
            LocationConstraint::City(country, city) => [country.clone(), city.clone()].to_vec(),
            LocationConstraint::Hostname(country, city, host) => {
                [country.clone(), city.clone(), host.clone()].to_vec()
            }
        }),
    };
    location.map(|location| proto::RelayLocation { hostname: location })
}

fn convert_bridge_state(state: &BridgeState) -> proto::BridgeState {
    let state = match state {
        BridgeState::Auto => proto::bridge_state::State::Auto,
        BridgeState::On => proto::bridge_state::State::On,
        BridgeState::Off => proto::bridge_state::State::Off,
    };
    proto::BridgeState {
        state: state as i32,
    }
}

fn convert_tunnel_options(options: &TunnelOptions) -> proto::TunnelOptions {
    proto::TunnelOptions {
        openvpn: Some(proto::tunnel_options::OpenvpnOptions {
            mssfix: options.openvpn.mssfix.unwrap_or_default() as u32,
        }),
        wireguard: Some(proto::tunnel_options::WireguardOptions {
            mtu: options.wireguard.mtu.unwrap_or_default() as u32,
            automatic_rotation: options
                .wireguard
                .automatic_rotation
                .unwrap_or((DEFAULT_AUTOMATIC_KEY_ROTATION.as_secs() / 60u64 / 60u64) as u32),
        }),
        generic: Some(proto::tunnel_options::GenericOptions {
            enable_ipv6: options.generic.enable_ipv6,
        }),
    }
}

fn convert_relay_list_country(country: &RelayListCountry) -> proto::RelayListCountry {
    let mut proto_country = proto::RelayListCountry {
        name: country.name.clone(),
        code: country.code.clone(),
        cities: Vec::with_capacity(country.cities.len()),
    };

    for city in &country.cities {
        proto_country.cities.push(proto::RelayListCity {
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

fn convert_relay(relay: &Relay) -> proto::Relay {
    proto::Relay {
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
        tunnels: Some(proto::RelayTunnels {
            openvpn: relay
                .tunnels
                .openvpn
                .iter()
                .map(|endpoint| {
                    let protocol = match endpoint.protocol {
                        TransportProtocol::Udp => proto::TransportProtocol::Udp,
                        TransportProtocol::Tcp => proto::TransportProtocol::Tcp,
                    };
                    proto::OpenVpnEndpointData {
                        port: endpoint.port as u32,
                        protocol: protocol as i32,
                    }
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
                        .map(|range| proto::PortRange {
                            first: range.0 as u32,
                            last: range.1 as u32,
                        })
                        .collect();
                    proto::WireguardEndpointData {
                        port_ranges,
                        ipv4_gateway: endpoint.ipv4_gateway.to_string(),
                        ipv6_gateway: endpoint.ipv6_gateway.to_string(),
                        public_key: endpoint.public_key.as_bytes().to_vec(),
                    }
                })
                .collect(),
        }),
        bridges: Some(proto::RelayBridges {
            shadowsocks: relay
                .bridges
                .shadowsocks
                .iter()
                .map(|endpoint| {
                    let protocol = match endpoint.protocol {
                        TransportProtocol::Udp => proto::TransportProtocol::Udp,
                        TransportProtocol::Tcp => proto::TransportProtocol::Tcp,
                    };
                    proto::ShadowsocksEndpointData {
                        port: endpoint.port as u32,
                        cipher: endpoint.cipher.clone(),
                        password: endpoint.password.clone(),
                        protocol: protocol as i32,
                    }
                })
                .collect(),
        }),
        location: relay.location.as_ref().map(|location| proto::Location {
            country: location.country.clone(),
            country_code: location.country_code.clone(),
            city: location.city.clone(),
            city_code: location.city_code.clone(),
            latitude: location.latitude,
            longitude: location.longitude,
        }),
    }
}

fn convert_state(state: TunnelState) -> proto::TunnelState {
    use proto::{
        error_state::{Cause as ProtoErrorCause, GenerationError as ProtoGenerationError},
        tunnel_state::{self, State as ProtoState},
    };
    use talpid_types::tunnel::{ActionAfterDisconnect, ErrorStateCause, ParameterGenerationError};
    use TunnelState::*;

    let state = match state {
        Disconnected => ProtoState::Disconnected(tunnel_state::Disconnected {}),
        Connecting { endpoint, location } => ProtoState::Connecting(tunnel_state::Connecting {
            relay_info: Some(proto::TunnelStateRelayInfo {
                tunnel_endpoint: Some(convert_endpoint(endpoint)),
                location: location.map(convert_geoip_location),
            }),
        }),
        Connected { endpoint, location } => ProtoState::Connected(tunnel_state::Connected {
            relay_info: Some(proto::TunnelStateRelayInfo {
                tunnel_endpoint: Some(convert_endpoint(endpoint)),
                location: location.map(convert_geoip_location),
            }),
        }),
        Disconnecting(after_disconnect) => ProtoState::Disconnecting(tunnel_state::Disconnecting {
            after_disconnect: match after_disconnect {
                ActionAfterDisconnect::Nothing => proto::AfterDisconnect::Nothing as i32,
                ActionAfterDisconnect::Block => proto::AfterDisconnect::Block as i32,
                ActionAfterDisconnect::Reconnect => proto::AfterDisconnect::Reconnect as i32,
            },
        }),
        Error(error_state) => ProtoState::Error(tunnel_state::Error {
            error_state: Some(proto::ErrorState {
                cause: match error_state.cause() {
                    ErrorStateCause::AuthFailed(_) => ProtoErrorCause::AuthFailed as i32,
                    ErrorStateCause::Ipv6Unavailable => ProtoErrorCause::Ipv6Unavailable as i32,
                    ErrorStateCause::SetFirewallPolicyError => {
                        ProtoErrorCause::SetFirewallPolicyError as i32
                    }
                    ErrorStateCause::SetDnsError => ProtoErrorCause::SetDnsError as i32,
                    ErrorStateCause::StartTunnelError => ProtoErrorCause::StartTunnelError as i32,
                    ErrorStateCause::TunnelParameterError(_) => {
                        ProtoErrorCause::TunnelParameterError as i32
                    }
                    ErrorStateCause::IsOffline => ProtoErrorCause::IsOffline as i32,
                    ErrorStateCause::TapAdapterProblem => ProtoErrorCause::TapAdapterProblem as i32,
                    #[cfg(target_os = "android")]
                    ErrorStateCause::VpnPermissionDenied => {
                        ProtoErrorCause::VpnPermissionDenied as i32
                    }
                },
                is_blocking: error_state.is_blocking(),
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
                            ProtoGenerationError::NoMatchingRelay as i32
                        }
                        ParameterGenerationError::NoMatchingBridgeRelay => {
                            ProtoGenerationError::NoMatchingBridgeRelay as i32
                        }
                        ParameterGenerationError::NoWireguardKey => {
                            ProtoGenerationError::NoWireguardKey as i32
                        }
                        ParameterGenerationError::CustomTunnelHostResultionError => {
                            ProtoGenerationError::CustomTunnelHostResolutionError as i32
                        }
                    }
                } else {
                    0
                },
            }),
        }),
    };

    proto::TunnelState { state: Some(state) }
}

fn convert_endpoint(endpoint: talpid_types::net::TunnelEndpoint) -> proto::TunnelEndpoint {
    use talpid_types::net;

    proto::TunnelEndpoint {
        address: endpoint.endpoint.address.to_string(),
        protocol: match endpoint.endpoint.protocol {
            TransportProtocol::Tcp => proto::TransportProtocol::Tcp as i32,
            TransportProtocol::Udp => proto::TransportProtocol::Udp as i32,
        },
        tunnel_type: match endpoint.tunnel_type {
            net::TunnelType::Wireguard => proto::TunnelType::Wireguard as i32,
            net::TunnelType::OpenVpn => proto::TunnelType::Openvpn as i32,
        },
        proxy: endpoint.proxy.map(|proxy_ep| proto::ProxyEndpoint {
            address: proxy_ep.endpoint.address.to_string(),
            protocol: match proxy_ep.endpoint.protocol {
                TransportProtocol::Tcp => proto::TransportProtocol::Tcp as i32,
                TransportProtocol::Udp => proto::TransportProtocol::Udp as i32,
            },
            proxy_type: match proxy_ep.proxy_type {
                net::proxy::ProxyType::Shadowsocks => proto::ProxyType::Shadowsocks as i32,
                net::proxy::ProxyType::Custom => proto::ProxyType::Custom as i32,
            },
        }),
    }
}

fn convert_geoip_location(geoip: GeoIpLocation) -> proto::GeoIpLocation {
    proto::GeoIpLocation {
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

fn convert_version_info(version_info: &version::AppVersionInfo) -> proto::AppVersionInfo {
    proto::AppVersionInfo {
        supported: version_info.supported,
        latest_stable: version_info.latest_stable.clone(),
        latest_beta: version_info.latest_beta.clone(),
        suggested_upgrade: version_info.suggested_upgrade.clone().unwrap_or_default(),
    }
}

pub struct ManagementInterfaceServer {
    subscriptions: Arc<RwLock<Vec<EventsListenerSender>>>,
    socket_path: String,
    runtime: tokio02::runtime::Runtime,
    server_abort_tx: triggered::Trigger,
    server_join_handle:
        Option<tokio02::task::JoinHandle<std::result::Result<(), tonic::transport::Error>>>,
}

impl ManagementInterfaceServer {
    async fn start_server(
        socket_path: String,
        daemon_tx: DaemonCommandSender,
        server_start_tx: std::sync::mpsc::Sender<()>,
        abort_rx: triggered::Listener,
        subscriptions: Arc<RwLock<Vec<EventsListenerSender>>>,
    ) -> std::result::Result<(), tonic::transport::Error> {
        use futures::stream::TryStreamExt;
        use parity_tokio_ipc::{Endpoint as IpcEndpoint, SecurityAttributes};

        let mut endpoint = IpcEndpoint::new(socket_path);
        endpoint.set_security_attributes(
            SecurityAttributes::allow_everyone_create()
                .unwrap()
                .set_mode(777)
                .unwrap(),
        );
        let incoming = endpoint.incoming().unwrap();
        let _ = server_start_tx.send(());

        let server = ManagementServiceImpl {
            daemon_tx,
            subscriptions,
        };

        Server::builder()
            .add_service(ManagementServiceServer::new(server))
            .serve_with_incoming_shutdown(incoming.map_ok(StreamBox), abort_rx)
            .await
    }

    pub fn start(tunnel_tx: DaemonCommandSender) -> Result<Self, Error> {
        // TODO: don't spawn a tokio runtime here; make this function async
        let mut runtime = tokio02::runtime::Builder::new()
            .threaded_scheduler()
            .core_threads(1)
            .enable_all()
            .build()
            .map_err(Error::TokioRuntimeError)?;

        let subscriptions = Arc::<RwLock<Vec<EventsListenerSender>>>::default();

        let socket_path = mullvad_paths::get_rpc_socket_path()
            .to_string_lossy()
            .to_string();

        let (server_abort_tx, server_abort_rx) = triggered::trigger();
        let (start_tx, start_rx) = mpsc::channel();
        let server_join_handle = runtime.spawn(Self::start_server(
            socket_path.clone(),
            tunnel_tx,
            start_tx,
            server_abort_rx,
            subscriptions.clone(),
        ));

        if let Err(_) = start_rx.recv() {
            return Err(runtime
                .block_on(server_join_handle)
                .expect("Failed to resolve quit handle future")
                .map_err(Error::SetupError)
                .unwrap_err());
        }

        #[cfg(unix)]
        {
            use std::{fs, os::unix::fs::PermissionsExt};
            fs::set_permissions(&socket_path, PermissionsExt::from_mode(0o766))
                .map_err(Error::PermissionsError)?;
        }
        #[cfg(windows)]
        crate::windows_permissions::deny_network_access(&socket_path)
            .map_err(Error::PermissionsError)?;

        Ok(ManagementInterfaceServer {
            subscriptions,
            socket_path: socket_path.to_string(),
            runtime,
            server_abort_tx,
            server_join_handle: Some(server_join_handle),
        })
    }

    pub fn socket_path(&self) -> &str {
        &self.socket_path
    }

    pub fn event_broadcaster(&self) -> ManagementInterfaceEventBroadcaster {
        ManagementInterfaceEventBroadcaster {
            runtime: self.runtime.handle().clone(),
            subscriptions: self.subscriptions.clone(),
            close_handle: self.server_abort_tx.clone(),
        }
    }

    /// Consumes the server and waits for it to finish. Returns an error if the server exited
    /// due to an error.
    pub fn wait(mut self) {
        if let Some(server_join_handle) = self.server_join_handle {
            if let Err(error) = self.runtime.block_on(server_join_handle) {
                log::error!("Management server panic: {:?}", error);
            }
        }
    }
}

/// A handle that allows broadcasting messages to all subscribers of the management interface.
#[derive(Clone)]
pub struct ManagementInterfaceEventBroadcaster {
    runtime: tokio02::runtime::Handle,
    subscriptions: Arc<RwLock<Vec<EventsListenerSender>>>,
    close_handle: triggered::Trigger,
}

impl EventListener for ManagementInterfaceEventBroadcaster {
    /// Sends a new state update to all `new_state` subscribers of the management interface.
    fn notify_new_state(&self, new_state: TunnelState) {
        self.notify(proto::DaemonEvent {
            event: Some(DaemonEventType::TunnelState(convert_state(new_state))),
        })
    }

    /// Sends settings to all `settings` subscribers of the management interface.
    fn notify_settings(&self, settings: Settings) {
        log::debug!("Broadcasting new settings");
        self.notify(proto::DaemonEvent {
            event: Some(DaemonEventType::Settings(convert_settings(&settings))),
        })
    }

    /// Sends relays to all subscribers of the management interface.
    fn notify_relay_list(&self, relay_list: RelayList) {
        log::debug!("Broadcasting new relay list");
        let mut new_list = proto::RelayList {
            countries: Vec::new(),
        };
        new_list.countries.reserve(relay_list.countries.len());
        for country in &relay_list.countries {
            new_list.countries.push(convert_relay_list_country(country));
        }
        self.notify(proto::DaemonEvent {
            event: Some(DaemonEventType::RelayList(new_list)),
        })
    }

    fn notify_app_version(&self, app_version_info: version::AppVersionInfo) {
        log::debug!("Broadcasting new app version info");
        let new_info = convert_version_info(&app_version_info);
        self.notify(proto::DaemonEvent {
            event: Some(DaemonEventType::VersionInfo(new_info)),
        })
    }

    fn notify_key_event(&self, key_event: mullvad_types::wireguard::KeygenEvent) {
        log::debug!("Broadcasting new wireguard key event");
        let new_event = convert_wireguard_key_event(&key_event);
        self.notify(proto::DaemonEvent {
            event: Some(DaemonEventType::KeyEvent(new_event)),
        })
    }
}

impl ManagementInterfaceEventBroadcaster {
    fn notify(&self, value: proto::DaemonEvent) {
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

// Converts a REST API error for an account into a JSONRPC error for the JSONRPC client.
fn map_rest_account_error(error: RestError) -> tonic::Status {
    match error {
        RestError::ApiError(status, message)
            if status == StatusCode::UNAUTHORIZED || status == StatusCode::FORBIDDEN =>
        {
            tonic::Status::new(tonic::Code::from_i32(INVALID_ACCOUNT_CODE), message)
        }
        _ => tonic::Status::internal("internal error"),
    }
}


// FIXME
use std::{
    pin::Pin,
    task::{Context, Poll},
};
use tokio02::io::{AsyncRead, AsyncWrite};

#[derive(Debug)]
pub struct StreamBox<T: AsyncRead + AsyncWrite>(pub T);
impl<T: AsyncRead + AsyncWrite> Connected for StreamBox<T> {}
impl<T: AsyncRead + AsyncWrite + Unpin> AsyncRead for StreamBox<T> {
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut [u8],
    ) -> Poll<std::io::Result<usize>> {
        Pin::new(&mut self.0).poll_read(cx, buf)
    }
}
impl<T: AsyncRead + AsyncWrite + Unpin> AsyncWrite for StreamBox<T> {
    fn poll_write(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<std::io::Result<usize>> {
        Pin::new(&mut self.0).poll_write(cx, buf)
    }

    fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<std::io::Result<()>> {
        Pin::new(&mut self.0).poll_flush(cx)
    }

    fn poll_shutdown(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<std::io::Result<()>> {
        Pin::new(&mut self.0).poll_shutdown(cx)
    }
}

use crate::{BoxFuture, DaemonCommand, DaemonCommandSender, EventListener};
use jsonrpc_core::{
    futures::{future, sync, Future},
    Error, ErrorCode, MetaIoHandler, Metadata,
};
use jsonrpc_ipc_server;
use jsonrpc_macros::{build_rpc_trait, metadata, pubsub};
use jsonrpc_pubsub::{PubSubHandler, PubSubMetadata, Session, SubscriptionId};
use mullvad_paths;
use mullvad_rpc::{rest::Error as RestError, StatusCode};
use mullvad_types::{
    account::{AccountData, AccountToken, VoucherSubmission},
    location::GeoIpLocation,
    relay_constraints::{BridgeSettings, BridgeConstraints, BridgeState, Constraint, RelayConstraintsUpdate, RelaySettings, RelaySettingsUpdate},
    relay_list::{RelayList, RelayListCountry},
    settings::{Settings, TunnelOptions},
    states::{TargetState, TunnelState},
    version, wireguard, DaemonEvent,
};
use parking_lot::RwLock;
use std::{
    collections::{hash_map::Entry, HashMap},
    str::FromStr,
    sync::{Arc, mpsc},
};
use talpid_ipc;
use talpid_types::ErrorExt;
use uuid;
use futures::compat::Future01CompatExt;

pub const INVALID_VOUCHER_CODE: i64 = -400;
pub const VOUCHER_USED_ALREADY_CODE: i64 = -401;
pub const INVALID_ACCOUNT_CODE: i64 = -200;


mod proto {
    tonic::include_proto!("mullvad_daemon.management_interface");
}

use proto::management_service_server::{ManagementService, ManagementServiceServer};

use tonic::{
    self,
    transport::{server::Connected, Server},
    Request, Response,
};

struct ManagementServiceImpl {
    daemon_tx: DaemonCommandSender,
}

pub type ServiceResult<T> = std::result::Result<Response<T>, tonic::Status>;

#[tonic::async_trait]
impl ManagementService for ManagementServiceImpl {
    type GetRelayLocationsStream = tokio02::sync::mpsc::Receiver<Result<proto::RelayListCountry, tonic::Status>>;
    type GetSplitTunnelProcessesStream = tokio02::sync::mpsc::UnboundedReceiver<Result<i32, tonic::Status>>;

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

    async fn get_account_data(
        &self,
        request: Request<AccountToken>
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

                    // FIXME: map this to the correct error code
                    // see `map_rest_account_error`
                    tonic::Status::internal("internal error")
                })
            })
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
                    // FIXME: map this to the correct error code
                    // see `map_rest_account_error`
                    tonic::Status::internal("internal error")
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
                f
                .map(|submission| {
                    Response::new(proto::VoucherSubmission {
                        time_added: submission.time_added,
                        new_expiry: Some(prost_types::Timestamp {
                            seconds: submission.new_expiry.timestamp(),
                            nanos: 0,
                        }),
                    })
                })
                .map_err(|e| match e {
                    RestError::ApiError(StatusCode::BAD_REQUEST, message) => {
                        match &message.as_str() {

                            // FIXME: return other error codes
                            /*
                            &mullvad_rpc::INVALID_VOUCHER => Error {
                                code: ErrorCode::from(INVALID_VOUCHER_CODE),
                                message,
                                data: None,
                            },

                            &mullvad_rpc::VOUCHER_USED => Error {
                                code: ErrorCode::from(VOUCHER_USED_ALREADY_CODE),
                                message,
                                data: None,
                            },
                            */

                            _ => tonic::Status::internal("internal error"),
                        }
                    }
                    _ => tonic::Status::internal("internal error"),
                })
            })
            .compat()
            .await
    }

    async fn get_relay_locations(&self, _: Request<()>) -> ServiceResult<Self::GetRelayLocationsStream> {
        log::debug!("get_relay_locations");

        let (tx, rx) = sync::oneshot::channel();
        let locations = self.send_command_to_daemon(DaemonCommand::GetRelayLocations(tx))
            .and_then(|_| rx.map_err(|_| tonic::Status::internal("internal error")))
            .compat()
            .await?;

        let (mut stream_tx, stream_rx) = tokio02::sync::mpsc::channel(locations.countries.len());

        tokio02::spawn(async move {
            for country in &locations.countries {
                stream_tx.send(Ok(convert_relay_list_country(country))).await.unwrap();
            }
        });

        Ok(Response::new(stream_rx))
    }

    async fn update_relay_locations(&self, _: Request<()>) -> ServiceResult<()> {
        log::debug!("update_relay_locations");
        self.send_command_to_daemon(DaemonCommand::UpdateRelayLocations).compat().await.map(Response::new)
    }

    async fn set_account(
        &self,
        request: Request<AccountToken>,
    ) -> ServiceResult<()> {
        log::debug!("set_account");
        let account_token = request.into_inner();
        let account_token = if account_token == "" {
            None
        } else {
            Some(account_token)
        };
        let (tx, rx) = sync::oneshot::channel();
        self.send_command_to_daemon(DaemonCommand::SetAccount(tx, account_token))
            .and_then(|_| rx.map(Response::new).map_err(|_| tonic::Status::internal("internal error")))
            .compat()
            .await
    }

    async fn update_relay_settings(
        &self,
        request: Request<proto::RelaySettingsUpdate>,
    ) -> ServiceResult<()> {
        log::debug!("update_relay_settings");
        let (tx, rx) = sync::oneshot::channel();
        let constraints_update = convert_relay_settings_update(&request.into_inner());

        let message = DaemonCommand::UpdateRelaySettings(tx, constraints_update);
        self.send_command_to_daemon(message)
            .and_then(|_| rx.map_err(|_| tonic::Status::internal("internal error")))
            .map(Response::new)
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

    async fn set_block_when_disconnected(
        &self,
        request: Request<bool>,
    ) -> ServiceResult<()> {
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

    async fn connect_daemon(&self, _: Request<()>) -> ServiceResult<()> {
        log::debug!("connect_daemon");

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

    async fn disconnect_daemon(&self, _: Request<()>) -> ServiceResult<()> {
        log::debug!("disconnect_daemon");

        let (tx, _) = sync::oneshot::channel();
        self.send_command_to_daemon(DaemonCommand::SetTargetState(tx, TargetState::Unsecured))
            .then(|_| Ok(Response::new(())))
            .compat()
            .await
    }

    async fn reconnect(&self, _: Request<()>) -> ServiceResult<()> {
        log::debug!("reconnect");
        self.send_command_to_daemon(DaemonCommand::Reconnect).map(Response::new).compat().await
    }

    async fn get_state(&self, _: Request<()>) -> ServiceResult<proto::TunnelState> {
        use TunnelState::*;
        use proto::tunnel_state;

        log::debug!("get_state");
        let (tx, rx) = sync::oneshot::channel();
        self.send_command_to_daemon(DaemonCommand::GetState(tx))
            .and_then(|_| rx.map_err(|_| tonic::Status::internal("internal error")))
            .and_then(|state| {
                // FIXME: return correct state

                let state = match state {
                    Disconnected => tunnel_state::Disconnected {},
                    Connecting { .. } => tunnel_state::Disconnected {},
                    Connected { .. } => tunnel_state::Disconnected {},
                    Disconnecting(..) => tunnel_state::Disconnected {},
                    Error(..) => tunnel_state::Disconnected {},
                };

                Ok(Response::new(proto::TunnelState {
                    state: Some(tunnel_state::State::Disconnected(state))
                }))
            })
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

    async fn get_current_location(&self, _: Request<()>) -> ServiceResult<proto::GeoIpLocation> {
        // NOTE: optional result
        log::debug!("get_current_location");
        let (tx, rx) = sync::oneshot::channel();
        self.send_command_to_daemon(DaemonCommand::GetCurrentLocation(tx))
            .and_then(|_| rx.map_err(|_| tonic::Status::internal("internal error")))
            .and_then(|geoip| if let Some(geoip) = geoip {
                Ok(Response::new(proto::GeoIpLocation {
                    ipv4: geoip.ipv4.map(|ip| ip.to_string()).unwrap_or_default(),
                    ipv6: geoip.ipv6.map(|ip| ip.to_string()).unwrap_or_default(),
                    country: geoip.country,
                    city: geoip.city.unwrap_or_default(),
                    latitude: geoip.latitude,
                    longitude: geoip.longitude,
                    mullvad_exit_ip: geoip.mullvad_exit_ip,
                    hostname: geoip.hostname.unwrap_or_default(),
                    bridge_hostname: geoip.bridge_hostname.unwrap_or_default(),
                }))
            } else {
                // FIXME: handle error properly
                Err(tonic::Status::internal("internal error"))
            })
            .compat()
            .await
    }

    async fn shutdown(&self, _: Request<()>) -> ServiceResult<()> {
        log::debug!("shutdown");
        self.send_command_to_daemon(DaemonCommand::Shutdown).map(Response::new).compat().await
    }

    async fn prepare_restart(&self, _: Request<()>) -> ServiceResult<()> {
        log::debug!("prepare_restart");
        self.send_command_to_daemon(DaemonCommand::PrepareRestart).map(Response::new).compat().await
    }
    
    async fn get_account_history(&self, _: Request<()>) -> ServiceResult<proto::AccountHistory> {
        // TODO: this might be a stream
        log::debug!("get_account_history");
        let (tx, rx) = sync::oneshot::channel();
        self.send_command_to_daemon(DaemonCommand::GetAccountHistory(tx))
            .and_then(|_| rx.map_err(|_| tonic::Status::internal("internal error")))
            .map(|history| Response::new(proto::AccountHistory {
                token: history
            }))
            .compat()
            .await
    }

    async fn remove_account_from_history(
        &self,
        request: Request<AccountToken>
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
    
    async fn set_bridge_settings(&self, request: Request<proto::BridgeSettings>) -> ServiceResult<()> {
        use mullvad_types::relay_constraints::LocationConstraint;
        use talpid_types::net;
        use proto::bridge_settings::Type as BridgeSettingType;

        let settings = request.into_inner().r#type.ok_or(tonic::Status::invalid_argument("no settings provided"))?;

        // FIXME: use correct settings
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
                            _ => return Err(tonic::Status::invalid_argument("expected 1-3 elements")),
                        }
                    }
                };

                BridgeSettings::Normal(BridgeConstraints {
                    location: constraint
                })
            }
            BridgeSettingType::Local(proxy_settings) => {
                let proxy_settings = net::openvpn::ProxySettings::Local(net::openvpn::LocalProxySettings {
                    port: proxy_settings.port as u16,
                    peer: proxy_settings.peer.parse().map_err(|_| {
                        tonic::Status::invalid_argument("failed to parse peer address")
                    })?,
                });
                BridgeSettings::Custom(proxy_settings)
            }
            BridgeSettingType::Remote(_) => BridgeSettings::Normal(BridgeConstraints {
                // TODO
                location: Constraint::Any
            }),
            BridgeSettingType::Shadowsocks(_) => BridgeSettings::Normal(BridgeConstraints {
                // TODO
                location: Constraint::Any
            }),
        };

        log::debug!("set_bridge_settings({:?})", settings);

        let (tx, rx) = sync::oneshot::channel();
        self.send_command_to_daemon(DaemonCommand::SetBridgeSettings(tx, settings))
            .and_then(|_| rx.map_err(|_| tonic::Status::internal("internal error")))
            .and_then(|settings_result| settings_result.map_err(|_| tonic::Status::internal("internal error")))
            .map(Response::new)
            .compat()
            .await
    }
    
    async fn set_bridge_state(
        &self,
        request: Request<proto::BridgeState>,
    ) -> ServiceResult<()> {
        use proto::bridge_state::State;

        let bridge_state = match request.into_inner().state {
            x if x == State::Auto as i32 => BridgeState::Auto,
            x if x == State::On as i32 => BridgeState::On,
            x if x == State::Off as i32 => BridgeState::Off,
            _ => return Err(tonic::Status::invalid_argument("unknown bridge state")),
        };

        log::debug!("set_bridge_state({:?})", bridge_state);
        let (tx, rx) = sync::oneshot::channel();
        self.send_command_to_daemon(DaemonCommand::SetBridgeState(tx, bridge_state))
            .and_then(|_| rx.map_err(|_| tonic::Status::internal("internal error")))
            .and_then(|settings_result| settings_result.map_err(|_| tonic::Status::internal("internal error")))
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
    
    async fn set_wireguard_mtu(&self, request: Request<u32>) -> ServiceResult<()> {
        let mtu = request.into_inner();
        let mtu = if mtu != 0 {
            Some(mtu as u16)
        } else {
            None
        };
        log::debug!("set_wireguard_mtu({:?})", mtu);
        let (tx, rx) = sync::oneshot::channel();
        self.send_command_to_daemon(DaemonCommand::SetWireguardMtu(tx, mtu))
            .and_then(|_| rx.map_err(|_| tonic::Status::internal("internal error")))
            .map(Response::new)
            .compat()
            .await
    }

    async fn set_wireguard_rotation_interval(&self, request: Request<u32>) -> ServiceResult<()> {
        // FIXME: distinguish between disabled and unset
        let interval = request.into_inner();
        let interval = if interval != 0 {
            Some(interval)
        } else {
            None
        };
        
        log::debug!("set_wireguard_rotation_interval({:?})", interval);
        let (tx, rx) = sync::oneshot::channel();
        self.send_command_to_daemon(DaemonCommand::SetWireguardRotationInterval(tx, interval))
            .and_then(|_| rx.map_err(|_| tonic::Status::internal("internal error")))
            .map(Response::new)
            .compat()
            .await
    }

    async fn get_settings(&self, _: Request<()>) -> ServiceResult<proto::Settings> {
        log::debug!("get_settings");
        let (tx, rx) = sync::oneshot::channel();
        self.send_command_to_daemon(DaemonCommand::GetSettings(tx))
            .and_then(|_| rx.map_err(|_| tonic::Status::internal("internal error")))
            .map(|settings| {
                Response::new(proto::Settings {
                    account_token: settings.get_account_token().unwrap_or_default(),
                    relay_settings: Some(convert_relay_settings(&settings.get_relay_settings())),
                    bridge_settings: Some(convert_bridge_settings(&settings.bridge_settings)),
                    bridge_state: Some(convert_bridge_state(settings.get_bridge_state())),
                    allow_lan: settings.allow_lan,
                    block_when_disconnected: settings.block_when_disconnected,
                    auto_connect: settings.auto_connect,
                    tunnel_options: Some(convert_tunnel_options(&settings.tunnel_options)),
                    show_beta_releases: settings.show_beta_releases,
                })
            })
            .compat()
            .await
    }

    async fn generate_wireguard_key(
        &self,
        _: Request<()>,
    ) -> ServiceResult<proto::KeygenEvent> {
        // TODO: return error for TooManyKeys, GenerationFailure
        // on success, simply return the new key or nil
        log::debug!("generate_wireguard_key");
        let (tx, rx) = sync::oneshot::channel();
        self.send_command_to_daemon(DaemonCommand::GenerateWireguardKey(tx))
            .and_then(|_| rx.map_err(|_| tonic::Status::internal("internal error")))
            .map(|event| {
                use mullvad_types::wireguard::KeygenEvent as MullvadEvent;
                use proto::keygen_event::KeygenEvent as Event;
                Response::new(proto::KeygenEvent {
                    event: match event {
                        MullvadEvent::NewKey(_) => Event::NewKey as i32,
                        MullvadEvent::TooManyKeys => Event::TooManyKeys as i32,
                        MullvadEvent::GenerationFailure => Event::GenerationFailure as i32,
                    },
                    new_key: if let MullvadEvent::NewKey(key) = event {
                        Some(Self::convert_public_key(&key))
                    } else {
                        None
                    },
                })
            })
            .compat()
            .await
    }

    async fn get_wireguard_key(
        &self,
        _: Request<()>,
    ) -> ServiceResult<proto::PublicKey> {
        // FIXME: optional return
        log::debug!("get_wireguard_key");
        let (tx, rx) = sync::oneshot::channel();
        self.send_command_to_daemon(DaemonCommand::GetWireguardKey(tx))
            .and_then(|_| rx.map_err(|_| tonic::Status::internal("internal error")))
            .map(|public_key| Response::new(Self::convert_public_key(&public_key.unwrap())))
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
        let app_version_info = self.send_command_to_daemon(DaemonCommand::GetVersionInfo(tx))
            .and_then(|_| rx.map_err(|_| tonic::Status::internal("internal error")))
            .compat()
            .await?;

        Ok(Response::new(proto::AppVersionInfo {
            supported: app_version_info.supported,
            latest_stable: app_version_info.latest_stable,
            latest_beta: app_version_info.latest_beta,
            suggested_upgrade: app_version_info.suggested_upgrade.unwrap_or_default(),
        }))
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

    async fn get_split_tunnel_processes(&self, _: Request<()>) -> ServiceResult<Self::GetSplitTunnelProcessesStream> {
        #[cfg(target_os = "linux")]
        {
            log::debug!("get_split_tunnel_processes");
            let (tx, rx) = tokio02::sync::mpsc::unbounded_channel();
            // TODO
            Ok(Response::new(rx))
            /*
            log::debug!("get_split_tunnel_processes");
            let (tx, rx) = sync::oneshot::channel();
            self.send_command_to_daemon(DaemonCommand::GetSplitTunnelProcesses(tx))
                .and_then(|_| rx.map_err(|_| tonic::Status::internal("internal error")))
                .map(Response::new)
                .compat()
                .await
            */
        }
        #[cfg(not(target_os = "linux"))]
        {
            let (_, rx) = tokio02::sync::mpsc::unbounded_channel();
            Ok(Response::new(rx))
        }
    }

    /*
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
    */

    // TODO: daemon event subscriptions
}

impl ManagementServiceImpl {
    /// Sends a command to the daemon and maps the error to an RPC error.
    fn send_command_to_daemon(
        &self,
        command: DaemonCommand,
    ) -> impl Future<Item = (), Error = tonic::Status> {
        future::result(self.daemon_tx.send(command).map_err(|_| tonic::Status::internal("internal error")))
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
}

fn convert_relay_settings_update(settings: &proto::RelaySettingsUpdate) -> RelaySettingsUpdate {
    use proto::relay_settings::Endpoint;

    // FIXME: handle missing field correctly
    let endpoint = settings.r#type.clone().unwrap();

    // FIXME: custom
    // FIXME: parse input

    RelaySettingsUpdate::Normal(RelayConstraintsUpdate {
        location: None,
        tunnel_protocol: None,
        wireguard_constraints: None,
        openvpn_constraints: None,
    })
}

fn convert_relay_settings(settings: &RelaySettings) -> proto::RelaySettings {
    /*
    let endpoint = match settings {
        RelaySettings::CustomTunnelEndpoint(endpoint) => {
            // TODO
            //CustomRelaySettings

            proto::relay_settings::Endpoint::Custom(x)
        }
        RelaySettings::Normal(constraints) => {
            // TODO
            //NormalRelaySettings
            // NOTE: optional
            
            //#[prost(uint32, tag="1")]
            //pub port: u32,
            // NOTE: (optional) OpenVPN only
            //#[prost(enumeration="TransportProtocol", tag="2")]
            //pub protocol: i32,

            p

            
        }
    };
    */

    // FIXME: don't ignore input
    // FIXME: custom endpoint
    let constraints = proto::NormalRelaySettings {
        location: Some(proto::RelayLocation {
            hostname: vec![],
        }),
        tunnel_type: proto::TunnelType::Openvpn as i32,
        // FIXME: optional
        port: 0,
        // FIXME: optional, openvpn only
        protocol: proto::TransportProtocol::Tcp as i32,
    };
    let endpoint = proto::relay_settings::Endpoint::Normal(constraints);

    proto::RelaySettings {
        endpoint: Some(endpoint),
    }
}

fn convert_bridge_settings(settings: &BridgeSettings) -> proto::BridgeSettings {
    // TODO
    proto::BridgeSettings::default()
}

fn convert_bridge_state(settings: &BridgeState) -> proto::BridgeState {
    // TODO
    proto::BridgeState::default()
}

fn convert_tunnel_options(settings: &TunnelOptions) -> proto::TunnelOptions {
    // TODO
    proto::TunnelOptions::default()
}

fn convert_relay_list_country(country: &RelayListCountry) -> proto::RelayListCountry {
    // TODO
    proto::RelayListCountry::default()
}

build_rpc_trait! {
    pub trait ManagementInterfaceApi {
        type Metadata;

        /// Returns available countries.
        #[rpc(meta, name = "get_relay_locations")]
        fn get_relay_locations(&self, Self::Metadata) -> BoxFuture<RelayList, Error>;

        /// Update constraints put on the type of tunnel connection to use
        #[rpc(meta, name = "update_relay_settings")]
        fn update_relay_settings(
            &self,
            Self::Metadata, RelaySettingsUpdate
            ) -> BoxFuture<(), Error>;

        /*
        /// Try to connect if disconnected, or do nothing if already connecting/connected.
        #[rpc(meta, name = "connect")]
        fn connect(&self, Self::Metadata) -> BoxFuture<(), Error>;

        /// Disconnect the VPN tunnel if it is connecting/connected. Does nothing if already
        /// disconnected.
        #[rpc(meta, name = "disconnect")]
        fn disconnect(&self, Self::Metadata) -> BoxFuture<(), Error>;
        */

        /*
        /// Returns the current state of the Mullvad client. Changes to this state will
        /// be announced to subscribers of `new_state`.
        #[rpc(meta, name = "get_state")]
        fn get_state(&self, Self::Metadata) -> BoxFuture<TunnelState, Error>;
        */

        /*
        /// Performs a geoIP lookup and returns the current location as perceived by the public
        /// internet.
        #[rpc(meta, name = "get_current_location")]
        fn get_current_location(&self, Self::Metadata) -> BoxFuture<Option<GeoIpLocation>, Error>;
        */

        /// Removes all accounts from history, removing any associated keys in the process
        #[rpc(meta, name = "clear_account_history")]
        fn clear_account_history(&self, Self::Metadata) -> BoxFuture<(), Error>;

        /*
        /// Sets proxy details for OpenVPN
        #[rpc(meta, name = "set_bridge_settings")]
        fn set_bridge_settings(&self, Self::Metadata, BridgeSettings) -> BoxFuture<(), Error>;
        */

        /// Returns the current daemon settings
        #[rpc(meta, name = "get_settings")]
        fn get_settings(&self, Self::Metadata) -> BoxFuture<Settings, Error>;

        /*
        /// Retrieve information about the currently running and latest versions of the app
        #[rpc(meta, name = "get_version_info")]
        fn get_version_info(&self, Self::Metadata) -> BoxFuture<version::AppVersionInfo, Error>;
        */

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

    runtime: tokio02::runtime::Runtime,
    server_abort_tx: triggered::Trigger,
    server_join_handle: Option<tokio02::task::JoinHandle<std::result::Result<(), tonic::transport::Error>>>,
}

impl ManagementInterfaceServer {
    async fn start_server_crap(
        daemon_tx: DaemonCommandSender,
        server_start_tx: std::sync::mpsc::Sender<()>,
        abort_rx: triggered::Listener,
    ) -> std::result::Result<(), tonic::transport::Error>
    {
        use parity_tokio_ipc::{Endpoint as IpcEndpoint, SecurityAttributes};
        use futures::stream::TryStreamExt;

        //let ipc_path = "//./pipe/Mullvad VPNQWE";
        let ipc_path = std::path::PathBuf::from("/var/run/mullvad-vpnQWE");

        //let mut endpoint = IpcEndpoint::new(ipc_path.to_string());
        let mut endpoint = IpcEndpoint::new(ipc_path.to_string_lossy().to_string());
        endpoint.set_security_attributes(SecurityAttributes::allow_everyone_create().unwrap().set_mode(777).unwrap());
        let incoming = endpoint.incoming().unwrap(); // FIXME: do not unwrap
        let _ = server_start_tx.send(());

        let server = ManagementServiceImpl {
            daemon_tx
        };

        Server::builder()
            .add_service(ManagementServiceServer::new(server))
            .serve_with_incoming_shutdown(incoming.map_ok(StreamBox), abort_rx)
            .await
    }

    pub fn start(tunnel_tx: DaemonCommandSender) -> Result<Self, talpid_ipc::Error> {
        // TODO: don't spawn a tokio runtime here; make this function async
        let runtime = tokio02::runtime::Builder::new()
            .threaded_scheduler()
            .core_threads(1)
            .enable_all()
            .build()
            .unwrap(); // FIXME: do not unwrap here

        let (server_abort_tx, server_abort_rx) = triggered::trigger();
        let (start_tx, start_rx) = mpsc::channel();
        let server_join_handle = runtime.spawn(Self::start_server_crap(
            tunnel_tx.clone(),
            start_tx,
            server_abort_rx,
        ));
        if let Err(_) = start_rx.recv() {
            // FIXME: do not unwrap here
            /*
            return Err(runtime
                .block_on(server_join_handle)
                .expect("Failed to resolve quit handle future")
                .map_err(Error::EventDispatcherError)
                .unwrap_err());

            */
            // FIXME
            panic!("start_rx failed");
        }

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
            
            runtime,
            server_abort_tx,
            server_join_handle: Some(server_join_handle),
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

    /// Converts a REST API error for an account into a JSONRPC error for the JSONRPC client.
    fn map_rest_account_error(error: RestError) -> Error {
        match error {
            RestError::ApiError(status, message)
                if status == StatusCode::UNAUTHORIZED || status == StatusCode::FORBIDDEN =>
            {
                Error {
                    code: ErrorCode::from(INVALID_ACCOUNT_CODE),
                    message,
                    data: None,
                }
            }
            _ => Error::internal_error(),
        }
    }
}

impl ManagementInterfaceApi for ManagementInterface {
    type Metadata = Meta;

    fn get_relay_locations(&self, _: Self::Metadata) -> BoxFuture<RelayList, Error> {
        log::debug!("get_relay_locations");
        let (tx, rx) = sync::oneshot::channel();
        let future = self
            .send_command_to_daemon(DaemonCommand::GetRelayLocations(tx))
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

    /*
    fn get_state(&self, _: Self::Metadata) -> BoxFuture<TunnelState, Error> {
        log::debug!("get_state");
        let (state_tx, state_rx) = sync::oneshot::channel();
        let future = self
            .send_command_to_daemon(DaemonCommand::GetState(state_tx))
            .and_then(|_| state_rx.map_err(|_| Error::internal_error()));
        Box::new(future)
    }
    */

    fn clear_account_history(&self, _: Self::Metadata) -> BoxFuture<(), Error> {
        log::debug!("clear_account_history");
        let (tx, rx) = sync::oneshot::channel();
        let future = self
            .send_command_to_daemon(DaemonCommand::ClearAccountHistory(tx))
            .and_then(|_| rx.map_err(|_| Error::internal_error()));
        Box::new(future)
    }

    /*
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
            .and_then(|settings_result| settings_result.map_err(|_| Error::internal_error()));

        Box::new(future)
    }
    */

    fn get_settings(&self, _: Self::Metadata) -> BoxFuture<Settings, Error> {
        log::debug!("get_settings");
        let (tx, rx) = sync::oneshot::channel();
        let future = self
            .send_command_to_daemon(DaemonCommand::GetSettings(tx))
            .and_then(|_| rx.map_err(|_| Error::internal_error()));
        Box::new(future)
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

    fn poll_shutdown(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<std::io::Result<()>> {
        Pin::new(&mut self.0).poll_shutdown(cx)
    }
}

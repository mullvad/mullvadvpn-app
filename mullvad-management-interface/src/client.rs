//! Client that returns and takes mullvad types as arguments instead of prost-generated types
use crate::types;
#[cfg(not(target_os = "android"))]
use futures::{Stream, StreamExt};
use mullvad_types::{
    access_method::AccessMethodSetting,
    relay_list::RelayList,
    settings::Settings,
    states::TunnelState,
    version::AppVersionInfo,
    wireguard::WireguardIdentityInfo,
};
#[cfg(not(target_os = "android"))]
use mullvad_types::{
    access_method::{self, AccessMethod},
    custom_list::{CustomList, Id},
    features::FeatureIndicators,
    relay_constraints::{AllowedIps, ObfuscationSettings, RelayOverride, RelaySettings},
    relay_list::BridgeList,
    settings::{DnsOptions, SettingsKeyList},
    wireguard::QuantumResistantState,
};
use std::net::IpAddr;
#[cfg(not(target_os = "android"))]
use std::{path::Path, str::FromStr};
#[cfg(target_os = "windows")]
use talpid_types::split_tunnel::ExcludedProcess;
#[cfg(not(target_os = "android"))]
use tonic::{Code, Status};

type Error = super::Error;

pub type Result<T> = std::result::Result<T, super::Error>;

#[cfg(not(target_os = "android"))]
#[derive(Debug, Clone)]
pub struct MullvadProxyClient(crate::ManagementServiceClient);

/// All events that can happen in the daemon which clients should react to.
#[derive(Debug)]
pub enum DaemonEvent {
    TunnelState(TunnelState),
    Settings(Settings),
    RelayList(RelayList),
    AppVersionInfo(AppVersionInfo),
    /// The WireGuard identity was imported, replaced or cleared.
    WireguardIdentity(Option<WireguardIdentityInfo>),
    NewAccessMethod(AccessMethodSetting),
    LeakDetected(LeakInfo),
}

impl TryFrom<types::daemon_event::Event> for DaemonEvent {
    type Error = Error;

    fn try_from(value: types::daemon_event::Event) -> Result<Self> {
        match value {
            types::daemon_event::Event::TunnelState(state) => TunnelState::try_from(state)
                .map(DaemonEvent::TunnelState)
                .map_err(Error::InvalidResponse),
            types::daemon_event::Event::Settings(settings) => Settings::try_from(settings)
                .map(DaemonEvent::Settings)
                .map_err(Error::InvalidResponse),
            types::daemon_event::Event::RelayList(list) => RelayList::try_from(list)
                .map(DaemonEvent::RelayList)
                .map_err(Error::InvalidResponse),
            types::daemon_event::Event::VersionInfo(info) => AppVersionInfo::try_from(info)
                .map(DaemonEvent::AppVersionInfo)
                .map_err(Error::InvalidResponse),
            types::daemon_event::Event::WireguardIdentity(state) => {
                types::identity_state_from_proto(state)
                    .map(DaemonEvent::WireguardIdentity)
                    .map_err(Error::InvalidResponse)
            }
            types::daemon_event::Event::NewAccessMethod(event) => {
                AccessMethodSetting::try_from(event)
                    .map(DaemonEvent::NewAccessMethod)
                    .map_err(Error::InvalidResponse)
            }
            types::daemon_event::Event::LeakInfo(leak) => {
                LeakInfo::try_from(leak).map(DaemonEvent::LeakDetected)
            }
        }
    }
}

#[cfg(not(target_os = "android"))]
impl MullvadProxyClient {
    pub async fn new() -> Result<Self> {
        #[expect(deprecated)]
        crate::new_management_service_client().await.map(Self)
    }

    pub fn from_rpc_client(client: crate::ManagementServiceClient) -> Self {
        Self(client)
    }

    pub async fn connect_tunnel(&mut self) -> Result<bool> {
        Ok(self.0.connect_tunnel(()).await?.into_inner())
    }

    pub async fn disconnect_tunnel(&mut self, source: &str) -> Result<bool> {
        Ok(self
            .0
            .disconnect_tunnel(source.to_owned())
            .await?
            .into_inner())
    }

    pub async fn reconnect_tunnel(&mut self) -> Result<bool> {
        Ok(self.0.reconnect_tunnel(()).await?.into_inner())
    }

    pub async fn get_tunnel_state(&mut self) -> Result<TunnelState> {
        let state = self.0.get_tunnel_state(()).await?.into_inner();
        TunnelState::try_from(state).map_err(Error::InvalidResponse)
    }

    pub async fn events_listen<'a>(
        &mut self,
    ) -> Result<impl Stream<Item = Result<DaemonEvent>> + 'a> {
        let listener = self.0.events_listen(()).await?.into_inner();

        Ok(listener.map(|item| {
            let event = item?.event.ok_or(Error::MissingDaemonEvent)?;
            DaemonEvent::try_from(event)
        }))
    }

    /// DEPRECATED: Prefer to use `prepare_restart_v2`.
    pub async fn prepare_restart(&mut self) -> Result<()> {
        self.0.prepare_restart(()).await?;
        Ok(())
    }

    /// Tell the daemon to get ready for a restart by securing a user, i.e. putting firewall rules
    /// in place.
    ///
    /// - `shutdown`: Whether the daemon should shutdown immediately after its prepare-for-restart
    ///   routine.
    pub async fn prepare_restart_v2(&mut self, shutdown: bool) -> Result<()> {
        self.0.prepare_restart_v2(shutdown).await?;
        Ok(())
    }

    pub async fn factory_reset(&mut self) -> Result<()> {
        self.0.factory_reset(()).await?;
        Ok(())
    }

    pub async fn get_current_version(&mut self) -> Result<String> {
        Ok(self.0.get_current_version(()).await?.into_inner())
    }

    pub async fn get_version_info(&mut self) -> Result<AppVersionInfo> {
        let version_info = self.0.get_version_info(()).await?.into_inner();
        AppVersionInfo::try_from(version_info).map_err(Error::InvalidResponse)
    }

    pub async fn get_relay_locations(&mut self) -> Result<RelayList> {
        let list = self.0.get_relay_locations(()).await?.into_inner();
        mullvad_types::relay_list::RelayList::try_from(list).map_err(Error::InvalidResponse)
    }

    pub async fn get_bridges(&mut self) -> Result<BridgeList> {
        let list = self.0.get_bridges(()).await?.into_inner();
        mullvad_types::relay_list::BridgeList::try_from(list).map_err(Error::InvalidResponse)
    }

    pub async fn get_api_access_methods(&mut self) -> Result<Vec<AccessMethodSetting>> {
        let access_method_settings = self
            .0
            .get_settings(())
            .await?
            .into_inner()
            .api_access_methods
            .ok_or(Error::ApiAccessMethodSettingsNotFound)
            .and_then(|access_method_settings| {
                access_method::Settings::try_from(access_method_settings)
                    .map_err(Error::InvalidResponse)
            })?;

        Ok(access_method_settings.iter().cloned().collect())
    }

    pub async fn get_api_access_method(
        &mut self,
        id: &access_method::Id,
    ) -> Result<AccessMethodSetting> {
        self.get_api_access_methods()
            .await?
            .into_iter()
            .find(|api_access_method| api_access_method.get_id() == *id)
            .ok_or(Error::ApiAccessMethodNotFound)
    }

    pub async fn get_current_api_access_method(&mut self) -> Result<AccessMethodSetting> {
        self.0
            .get_current_api_access_method(())
            .await
            .map_err(Error::from)
            .map(tonic::Response::into_inner)
            .and_then(|access_method| {
                AccessMethodSetting::try_from(access_method).map_err(Error::InvalidResponse)
            })
    }

    pub async fn test_api_access_method(&mut self, id: access_method::Id) -> Result<bool> {
        let result = self
            .0
            .test_api_access_method_by_id(types::Uuid::from(id))
            .await?;
        Ok(result.into_inner())
    }

    pub async fn test_custom_api_access_method(
        &mut self,
        config: talpid_types::net::proxy::CustomProxy,
    ) -> Result<bool> {
        let result = self
            .0
            .test_custom_api_access_method(types::CustomProxy::from(config))
            .await?;
        Ok(result.into_inner())
    }

    pub async fn update_relay_locations(&mut self) -> Result<()> {
        self.0.update_relay_locations(()).await?;
        Ok(())
    }

    pub async fn set_relay_settings(&mut self, update: RelaySettings) -> Result<()> {
        let update = types::RelaySettings::from(update);
        self.0.set_relay_settings(update).await?;
        Ok(())
    }

    pub async fn set_obfuscation_settings(&mut self, settings: ObfuscationSettings) -> Result<()> {
        let settings = types::ObfuscationSettings::from(&settings);
        self.0.set_obfuscation_settings(settings).await?;
        Ok(())
    }

    pub async fn get_settings(&mut self) -> Result<Settings> {
        let settings = self.0.get_settings(()).await?.into_inner();
        Settings::try_from(settings).map_err(Error::InvalidResponse)
    }

    pub async fn reset_settings(&mut self, preserved: SettingsKeyList) -> Result<()> {
        let preserved = types::SettingsKeyList::from(preserved);
        self.0.reset_settings(preserved).await?;
        Ok(())
    }

    pub async fn set_allow_lan(&mut self, state: bool) -> Result<()> {
        self.0.set_allow_lan(state).await?;
        Ok(())
    }

    pub async fn set_show_beta_releases(&mut self, state: bool) -> Result<()> {
        self.0.set_show_beta_releases(state).await?;
        Ok(())
    }

    pub async fn set_lockdown_mode(&mut self, state: bool) -> Result<()> {
        self.0.set_lockdown_mode(state).await?;
        Ok(())
    }

    pub async fn set_auto_connect(&mut self, state: bool) -> Result<()> {
        self.0.set_auto_connect(state).await?;
        Ok(())
    }

    pub async fn set_wireguard_mtu(&mut self, mtu: Option<u16>) -> Result<()> {
        self.0
            .set_wireguard_mtu(mtu.map(u32::from).unwrap_or(0))
            .await?;
        Ok(())
    }

    pub async fn set_enable_ipv6(&mut self, state: bool) -> Result<()> {
        self.0.set_enable_ipv6(state).await?;
        Ok(())
    }

    pub async fn set_userspace_wireguard(&mut self, state: bool) -> Result<()> {
        self.0.set_userspace_wireguard(state).await?;
        Ok(())
    }

    pub async fn set_quantum_resistant_tunnel(
        &mut self,
        state: QuantumResistantState,
    ) -> Result<()> {
        let state = types::QuantumResistantState::from(state);
        self.0.set_quantum_resistant_tunnel(state).await?;
        Ok(())
    }

    pub async fn set_enable_daita(&mut self, value: bool) -> Result<()> {
        self.0.set_enable_daita(value).await?;
        Ok(())
    }

    pub async fn set_daita_settings(&mut self, enable: bool) -> Result<()> {
        let settings = types::DaitaSettings::from(enable);
        self.0.set_daita_settings(settings).await?;
        Ok(())
    }

    pub async fn set_dns_options(&mut self, options: DnsOptions) -> Result<()> {
        let options = types::DnsOptions::from(&options);
        self.0.set_dns_options(options).await?;
        Ok(())
    }

    pub async fn set_relay_override(&mut self, relay_override: RelayOverride) -> Result<()> {
        let r#override = types::RelayOverride::from(relay_override);
        self.0.set_relay_override(r#override).await?;
        Ok(())
    }

    pub async fn clear_all_relay_overrides(&mut self) -> Result<()> {
        self.0.clear_all_relay_overrides(()).await?;
        Ok(())
    }

    /// Import the `[Interface]` section of a WireGuard config file generated at mullvad.net.
    /// This replaces any existing identity.
    pub async fn import_wireguard_config(
        &mut self,
        config: String,
    ) -> Result<WireguardIdentityInfo> {
        let identity = self
            .0
            .import_wireguard_config(config)
            .await
            .map_err(|status| match status.code() {
                Code::InvalidArgument => Error::InvalidWireguardConfig(status.message().to_owned()),
                _other => Error::Rpc(Box::new(status)),
            })?
            .into_inner();
        WireguardIdentityInfo::try_from(identity).map_err(Error::InvalidResponse)
    }

    /// Forget the imported identity. This also disconnects the tunnel.
    pub async fn clear_wireguard_identity(&mut self) -> Result<()> {
        self.0.clear_wireguard_identity(()).await?;
        Ok(())
    }

    /// Get the public details of the imported identity, if there is one.
    pub async fn get_wireguard_identity(&mut self) -> Result<Option<WireguardIdentityInfo>> {
        let state = self.0.get_wireguard_identity(()).await?.into_inner();
        types::identity_state_from_proto(state).map_err(Error::InvalidResponse)
    }

    pub async fn create_custom_list(&mut self, name: String) -> Result<Id> {
        let request = types::NewCustomList {
            name,
            locations: Vec::new(),
        };
        let id = self
            .0
            .create_custom_list(request)
            .await
            .map_err(map_custom_list_error)?
            .into_inner();
        Id::from_str(&id).map_err(|_| Error::CustomListListNotFound)
    }

    pub async fn delete_custom_list(&mut self, id: Id) -> Result<()> {
        self.0
            .delete_custom_list(id.to_string())
            .await
            .map_err(map_custom_list_error)?;
        Ok(())
    }

    pub async fn update_custom_list(&mut self, custom_list: CustomList) -> Result<()> {
        self.0
            .update_custom_list(types::CustomList::from(custom_list))
            .await
            .map_err(map_custom_list_error)?;
        Ok(())
    }

    /// Remove all custom lists.
    pub async fn clear_custom_lists(&mut self) -> Result<()> {
        self.0
            .clear_custom_lists(())
            .await
            .map_err(map_custom_list_error)?;
        Ok(())
    }

    pub async fn add_access_method(
        &mut self,
        name: String,
        enabled: bool,
        access_method: AccessMethod,
    ) -> Result<()> {
        let request = types::NewAccessMethodSetting {
            name,
            enabled,
            access_method: Some(types::AccessMethod::from(access_method)),
        };
        self.0
            .add_api_access_method(request)
            .await
            .map_err(map_api_access_method_error)?;
        Ok(())
    }

    pub async fn remove_access_method(
        &mut self,
        api_access_method: access_method::Id,
    ) -> Result<()> {
        self.0
            .remove_api_access_method(types::Uuid::from(api_access_method))
            .await?;
        Ok(())
    }

    pub async fn update_access_method(
        &mut self,
        access_method_update: AccessMethodSetting,
    ) -> Result<()> {
        self.0
            .update_api_access_method(types::AccessMethodSetting::from(access_method_update))
            .await
            .map_err(map_api_access_method_error)?;
        Ok(())
    }

    /// Remove all custom API access methods.
    pub async fn clear_custom_access_methods(&mut self) -> Result<()> {
        self.0.clear_custom_api_access_methods(()).await?;
        Ok(())
    }

    /// Set the [`AccessMethod`] which `AccessModeSelector` should pick.
    pub async fn set_access_method(&mut self, api_access_method: access_method::Id) -> Result<()> {
        self.0
            .set_api_access_method(types::Uuid::from(api_access_method))
            .await?;
        Ok(())
    }

    pub async fn get_split_tunnel_processes(&mut self) -> Result<Vec<i32>> {
        use futures::TryStreamExt;

        let procs = self.0.get_split_tunnel_processes(()).await?.into_inner();
        procs.try_collect().await.map_err(Error::from)
    }

    pub async fn add_split_tunnel_process(&mut self, pid: i32) -> Result<()> {
        self.0.add_split_tunnel_process(pid).await?;
        Ok(())
    }

    pub async fn remove_split_tunnel_process(&mut self, pid: i32) -> Result<()> {
        self.0.remove_split_tunnel_process(pid).await?;
        Ok(())
    }

    pub async fn clear_split_tunnel_processes(&mut self) -> Result<()> {
        self.0.clear_split_tunnel_processes(()).await?;
        Ok(())
    }

    pub async fn add_split_tunnel_app<P: AsRef<Path>>(&mut self, path: P) -> Result<()> {
        let path = path.as_ref().to_str().ok_or(Error::PathMustBeUtf8)?;
        self.0.add_split_tunnel_app(path.to_owned()).await?;
        Ok(())
    }

    pub async fn remove_split_tunnel_app<P: AsRef<Path>>(&mut self, path: P) -> Result<()> {
        let path = path.as_ref().to_str().ok_or(Error::PathMustBeUtf8)?;
        self.0.remove_split_tunnel_app(path.to_owned()).await?;
        Ok(())
    }

    pub async fn clear_split_tunnel_apps(&mut self) -> Result<()> {
        self.0.clear_split_tunnel_apps(()).await?;
        Ok(())
    }

    /// Toggle split tunneling on (`state: true`) or off (`state: false`).
    pub async fn set_split_tunnel_state(&mut self, state: bool) -> Result<()> {
        self.0.set_split_tunnel_state(state).await?;
        Ok(())
    }

    #[cfg(target_os = "windows")]
    pub async fn get_excluded_processes(&mut self) -> Result<Vec<ExcludedProcess>> {
        let procs = self.0.get_excluded_processes(()).await?.into_inner();
        Ok(procs
            .processes
            .into_iter()
            .map(ExcludedProcess::from)
            .collect::<Vec<_>>())
    }

    // check_volumes

    pub async fn apply_json_settings(&mut self, blob: String) -> Result<()> {
        self.0.apply_json_settings(blob).await?;
        Ok(())
    }

    pub async fn export_json_settings(&mut self) -> Result<String> {
        let blob = self.0.export_json_settings(()).await?;
        Ok(blob.into_inner())
    }

    pub async fn get_feature_indicators(&mut self) -> Result<FeatureIndicators> {
        Ok(FeatureIndicators::from(
            self.0.get_feature_indicators(()).await?.into_inner(),
        ))
    }

    // Debug features
    pub async fn disable_relay(&mut self, relay: String) -> Result<()> {
        self.0.disable_relay(relay).await?;
        Ok(())
    }

    pub async fn enable_relay(&mut self, relay: String) -> Result<()> {
        self.0.enable_relay(relay).await?;
        Ok(())
    }

    pub async fn get_rollout_threshold(&mut self) -> Result<f32> {
        let rollout = self.0.get_rollout_threshold(()).await?;
        let threshold = rollout.into_inner().threshold;
        Ok(threshold)
    }

    pub async fn generate_new_rollout_threshold(&mut self) -> Result<f32> {
        let rollout = self.0.regenerate_rollout_threshold(()).await?;
        let threshold = rollout.into_inner().threshold;
        Ok(threshold)
    }

    pub async fn set_new_rollout_threshold_seed(&mut self, seed: u32) -> Result<()> {
        self.0
            .set_rollout_threshold_seed(types::Seed { seed })
            .await?;
        Ok(())
    }

    pub async fn set_wireguard_allowed_ips(&mut self, allowed_ips: AllowedIps) -> Result<()> {
        self.0
            .set_wireguard_allowed_ips(types::AllowedIpsList {
                values: allowed_ips
                    .networks()
                    .iter()
                    .map(ToString::to_string)
                    .collect(),
            })
            .await?;
        Ok(())
    }

    pub async fn set_log_level(&mut self, level: impl Into<types::LogLevel>) -> Result<()> {
        self.0.set_log_level(level.into()).await?;
        Ok(())
    }

    pub async fn set_rust_log_env_filter(&mut self, filter: String) -> Result<()> {
        self.0
            .set_rust_log_env_filter(types::LogFilter::from(filter))
            .await?;
        Ok(())
    }

    pub async fn log_listen(&mut self) -> Result<impl Stream<Item = Result<String>>> {
        let listener = self.0.log_listen(()).await?.into_inner();

        Ok(listener.map(|item| Ok(item?.message)))
    }

    #[cfg(feature = "custom-vpn")]
    pub async fn set_custom_vpn_config(
        &mut self,
        config: talpid_types::net::wireguard::CustomVpnConfig,
    ) -> Result<String> {
        let response = self
            .0
            .set_custom_vpn_config(types::CustomVpnConfig::from(config))
            .await?;
        Ok(response.into_inner().error)
    }

    #[cfg(feature = "custom-vpn")]
    pub async fn clear_custom_vpn_config(&mut self) -> Result<String> {
        let response = self.0.clear_custom_vpn_config(()).await?;
        Ok(response.into_inner().error)
    }

    /// Import a custom VPN configuration from the contents of a wg-quick file.
    #[cfg(feature = "custom-vpn")]
    pub async fn import_custom_vpn_config(&mut self, body: String) -> Result<String> {
        let response = self.0.import_custom_vpn_config(body).await?;
        Ok(response.into_inner().error)
    }
}

#[cfg(not(target_os = "android"))]
fn map_custom_list_error(status: Status) -> Error {
    match (status.code(), status.details()) {
        (Code::NotFound, crate::CUSTOM_LIST_LIST_NOT_FOUND_DETAILS) => {
            Error::CustomListListNotFound
        }
        (Code::AlreadyExists, crate::CUSTOM_LIST_LIST_EXISTS_DETAILS) => Error::CustomListExists,
        _other => Error::Rpc(Box::new(status)),
    }
}

#[cfg(not(target_os = "android"))]
fn map_api_access_method_error(status: Status) -> Error {
    match (status.code(), status.details()) {
        (Code::AlreadyExists, crate::API_ACCESS_METHOD_EXISTS_DETAILS) => {
            Error::ApiAccessMethodExists
        }
        _other => Error::Rpc(Box::new(status)),
    }
}

// Types that are only defined in the protobuf interface (as opposed to *-types crates).

/// Details about how a leak happened.
#[derive(Debug)]
pub struct LeakInfo {
    /// On what interface the leaky traffic was detected.
    pub interface: String,
    /// What network nodes that was reached.
    pub reachable_nodes: Vec<IpAddr>,
}

impl TryFrom<types::LeakInfo> for LeakInfo {
    type Error = Error;

    fn try_from(leak: types::LeakInfo) -> Result<Self> {
        let reachable_nodes = leak
            .ip_addrs
            .into_iter()
            .map(|ip| ip.parse().map_err(Error::IpAddr))
            .collect::<Result<_>>()?;
        Ok(LeakInfo {
            interface: leak.interface,
            reachable_nodes,
        })
    }
}

#[cfg(not(target_os = "android"))]
pub struct RelaySelectorClient(crate::RelaySelectorServiceClient);

#[cfg(not(target_os = "android"))]
impl RelaySelectorClient {
    pub async fn new() -> Result<Self> {
        let channel = crate::grpc_transport_channel().await?;
        let client = crate::RelaySelectorServiceClient::new(channel);
        Ok(Self(client))
    }

    pub async fn partition_relays(
        &mut self,
        predicate: crate::types::relay_selector::Predicate,
    ) -> Result<crate::types::relay_selector::RelayPartitions> {
        let result = self.0.partition_relays(predicate).await?.into_inner();
        Ok(result)
    }
}

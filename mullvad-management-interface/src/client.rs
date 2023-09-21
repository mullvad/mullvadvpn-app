//! Client that returns and takes mullvad types as arguments instead of prost-generated types

use crate::types;
use futures::{Stream, StreamExt};
use mullvad_types::{
    access_method::{
        daemon::{ApiAccessMethodReplace, ApiAccessMethodToggle},
        AccessMethod,
    },
    account::{AccountData, AccountToken, VoucherSubmission},
    custom_list::{CustomList, CustomListLocationUpdate},
    device::{Device, DeviceEvent, DeviceId, DeviceState, RemoveDeviceEvent},
    location::GeoIpLocation,
    relay_constraints::{BridgeSettings, BridgeState, ObfuscationSettings, RelaySettingsUpdate},
    relay_list::RelayList,
    settings::{DnsOptions, Settings},
    states::TunnelState,
    version::AppVersionInfo,
    wireguard::{PublicKey, QuantumResistantState, RotationInterval},
};
#[cfg(target_os = "windows")]
use std::path::Path;
#[cfg(target_os = "windows")]
use talpid_types::split_tunnel::ExcludedProcess;
use tonic::{Code, Status};

type Error = super::Error;

pub type Result<T> = std::result::Result<T, super::Error>;

#[derive(Debug, Clone)]
pub struct MullvadProxyClient(crate::ManagementServiceClient);

pub enum DaemonEvent {
    TunnelState(TunnelState),
    Settings(Settings),
    RelayList(RelayList),
    AppVersionInfo(AppVersionInfo),
    Device(DeviceEvent),
    RemoveDevice(RemoveDeviceEvent),
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
            types::daemon_event::Event::VersionInfo(info) => {
                Ok(DaemonEvent::AppVersionInfo(AppVersionInfo::from(info)))
            }
            types::daemon_event::Event::Device(event) => DeviceEvent::try_from(event)
                .map(DaemonEvent::Device)
                .map_err(Error::InvalidResponse),
            types::daemon_event::Event::RemoveDevice(event) => RemoveDeviceEvent::try_from(event)
                .map(DaemonEvent::RemoveDevice)
                .map_err(Error::InvalidResponse),
        }
    }
}

impl MullvadProxyClient {
    pub async fn new() -> Result<Self> {
        #[allow(deprecated)]
        super::new_rpc_client().await.map(Self)
    }

    pub async fn connect_tunnel(&mut self) -> Result<bool> {
        Ok(self
            .0
            .connect_tunnel(())
            .await
            .map_err(Error::Rpc)?
            .into_inner())
    }

    pub async fn disconnect_tunnel(&mut self) -> Result<bool> {
        Ok(self
            .0
            .disconnect_tunnel(())
            .await
            .map_err(Error::Rpc)?
            .into_inner())
    }

    pub async fn reconnect_tunnel(&mut self) -> Result<bool> {
        Ok(self
            .0
            .reconnect_tunnel(())
            .await
            .map_err(Error::Rpc)?
            .into_inner())
    }

    pub async fn get_tunnel_state(&mut self) -> Result<TunnelState> {
        let state = self
            .0
            .get_tunnel_state(())
            .await
            .map_err(Error::Rpc)?
            .into_inner();
        TunnelState::try_from(state).map_err(Error::InvalidResponse)
    }

    pub async fn events_listen(&mut self) -> Result<impl Stream<Item = Result<DaemonEvent>>> {
        let listener = self
            .0
            .events_listen(())
            .await
            .map_err(Error::Rpc)?
            .into_inner();

        Ok(listener.map(|item| {
            let event = item
                .map_err(Error::Rpc)?
                .event
                .ok_or(Error::MissingDaemonEvent)?;
            DaemonEvent::try_from(event)
        }))
    }

    pub async fn prepare_restart(&mut self) -> Result<()> {
        self.0.prepare_restart(()).await.map_err(Error::Rpc)?;
        Ok(())
    }

    pub async fn factory_reset(&mut self) -> Result<()> {
        self.0.factory_reset(()).await.map_err(Error::Rpc)?;
        Ok(())
    }

    pub async fn get_current_version(&mut self) -> Result<String> {
        Ok(self
            .0
            .get_current_version(())
            .await
            .map_err(Error::Rpc)?
            .into_inner())
    }

    pub async fn get_version_info(&mut self) -> Result<AppVersionInfo> {
        let version_info = self
            .0
            .get_version_info(())
            .await
            .map_err(Error::Rpc)?
            .into_inner();
        Ok(AppVersionInfo::from(version_info))
    }

    pub async fn get_relay_locations(&mut self) -> Result<RelayList> {
        let list = self
            .0
            .get_relay_locations(())
            .await
            .map_err(Error::Rpc)?
            .into_inner();
        mullvad_types::relay_list::RelayList::try_from(list).map_err(Error::InvalidResponse)
    }

    pub async fn get_api_access_methods(&mut self) -> Result<Vec<AccessMethod>> {
        self.0
            .get_api_access_methods(())
            .await
            .map_err(Error::Rpc)?
            .into_inner()
            .api_access_methods
            .into_iter()
            .map(|access_method| {
                AccessMethod::try_from(access_method).map_err(Error::InvalidResponse)
            })
            .collect()
    }

    pub async fn test_api(&mut self) -> Result<()> {
        self.0.get_api_addressess(()).await.map_err(Error::Rpc)?;
        Ok(())
    }

    pub async fn update_relay_locations(&mut self) -> Result<()> {
        self.0
            .update_relay_locations(())
            .await
            .map_err(Error::Rpc)?;
        Ok(())
    }

    pub async fn update_relay_settings(&mut self, update: RelaySettingsUpdate) -> Result<()> {
        let update = types::RelaySettingsUpdate::from(update);
        self.0
            .update_relay_settings(update)
            .await
            .map_err(Error::Rpc)?;
        Ok(())
    }

    pub async fn get_current_location(&mut self) -> Result<GeoIpLocation> {
        let location = self
            .0
            .get_current_location(())
            .await
            .map_err(map_location_error)?
            .into_inner();
        GeoIpLocation::try_from(location).map_err(Error::InvalidResponse)
    }

    pub async fn set_bridge_settings(&mut self, settings: BridgeSettings) -> Result<()> {
        let settings = types::BridgeSettings::from(settings);
        self.0
            .set_bridge_settings(settings)
            .await
            .map_err(Error::Rpc)?;
        Ok(())
    }

    pub async fn set_bridge_state(&mut self, state: BridgeState) -> Result<()> {
        let state = types::BridgeState::from(state);
        self.0.set_bridge_state(state).await.map_err(Error::Rpc)?;
        Ok(())
    }

    pub async fn set_obfuscation_settings(&mut self, settings: ObfuscationSettings) -> Result<()> {
        let settings = types::ObfuscationSettings::from(&settings);
        self.0
            .set_obfuscation_settings(settings)
            .await
            .map_err(Error::Rpc)?;
        Ok(())
    }

    pub async fn get_settings(&mut self) -> Result<Settings> {
        let settings = self
            .0
            .get_settings(())
            .await
            .map_err(Error::Rpc)?
            .into_inner();
        Settings::try_from(settings).map_err(Error::InvalidResponse)
    }

    pub async fn set_allow_lan(&mut self, state: bool) -> Result<()> {
        self.0.set_allow_lan(state).await.map_err(Error::Rpc)?;
        Ok(())
    }

    pub async fn set_show_beta_releases(&mut self, state: bool) -> Result<()> {
        self.0
            .set_show_beta_releases(state)
            .await
            .map_err(Error::Rpc)?;
        Ok(())
    }

    pub async fn set_block_when_disconnected(&mut self, state: bool) -> Result<()> {
        self.0
            .set_block_when_disconnected(state)
            .await
            .map_err(Error::Rpc)?;
        Ok(())
    }

    pub async fn set_auto_connect(&mut self, state: bool) -> Result<()> {
        self.0.set_auto_connect(state).await.map_err(Error::Rpc)?;
        Ok(())
    }

    pub async fn set_openvpn_mssfix(&mut self, mssfix: Option<u16>) -> Result<()> {
        self.0
            .set_openvpn_mssfix(mssfix.map(u32::from).unwrap_or(0))
            .await
            .map_err(Error::Rpc)?;
        Ok(())
    }

    pub async fn set_wireguard_mtu(&mut self, mtu: Option<u16>) -> Result<()> {
        self.0
            .set_wireguard_mtu(mtu.map(u32::from).unwrap_or(0))
            .await
            .map_err(Error::Rpc)?;
        Ok(())
    }

    pub async fn set_enable_ipv6(&mut self, state: bool) -> Result<()> {
        self.0.set_enable_ipv6(state).await.map_err(Error::Rpc)?;
        Ok(())
    }

    pub async fn set_quantum_resistant_tunnel(
        &mut self,
        state: QuantumResistantState,
    ) -> Result<()> {
        let state = types::QuantumResistantState::from(state);
        self.0
            .set_quantum_resistant_tunnel(state)
            .await
            .map_err(Error::Rpc)?;
        Ok(())
    }

    pub async fn set_dns_options(&mut self, options: DnsOptions) -> Result<()> {
        let options = types::DnsOptions::from(&options);
        self.0.set_dns_options(options).await.map_err(Error::Rpc)?;
        Ok(())
    }

    pub async fn create_new_account(&mut self) -> Result<AccountToken> {
        Ok(self
            .0
            .create_new_account(())
            .await
            .map_err(map_device_error)?
            .into_inner())
    }

    pub async fn login_account(&mut self, account: AccountToken) -> Result<()> {
        self.0
            .login_account(account)
            .await
            .map_err(map_device_error)?;
        Ok(())
    }

    pub async fn logout_account(&mut self) -> Result<()> {
        self.0.logout_account(()).await.map_err(Error::Rpc)?;
        Ok(())
    }

    pub async fn get_account_data(&mut self, account: AccountToken) -> Result<AccountData> {
        let data = self
            .0
            .get_account_data(account)
            .await
            .map_err(Error::Rpc)?
            .into_inner();
        AccountData::try_from(data).map_err(Error::InvalidResponse)
    }

    pub async fn get_account_history(&mut self) -> Result<Option<AccountToken>> {
        let history = self
            .0
            .get_account_history(())
            .await
            .map_err(Error::Rpc)?
            .into_inner();
        Ok(history.token)
    }

    pub async fn clear_account_history(&mut self) -> Result<()> {
        self.0.clear_account_history(()).await.map_err(Error::Rpc)?;
        Ok(())
    }

    // get_www_auth_token

    pub async fn submit_voucher(&mut self, voucher: String) -> Result<VoucherSubmission> {
        let result = self
            .0
            .submit_voucher(voucher)
            .await
            .map_err(|error| match error.code() {
                Code::NotFound => Error::InvalidVoucher,
                Code::ResourceExhausted => Error::UsedVoucher,
                _other => Error::Rpc(error),
            })?
            .into_inner();
        VoucherSubmission::try_from(result).map_err(Error::InvalidResponse)
    }

    pub async fn get_device(&mut self) -> Result<DeviceState> {
        let state = self
            .0
            .get_device(())
            .await
            .map_err(map_device_error)?
            .into_inner();
        DeviceState::try_from(state).map_err(Error::InvalidResponse)
    }

    pub async fn update_device(&mut self) -> Result<()> {
        self.0.update_device(()).await.map_err(map_device_error)?;
        Ok(())
    }

    pub async fn list_devices(&mut self, account: AccountToken) -> Result<Vec<Device>> {
        let list = self
            .0
            .list_devices(account)
            .await
            .map_err(map_device_error)?
            .into_inner();
        list.devices
            .into_iter()
            .map(|d| Device::try_from(d).map_err(Error::InvalidResponse))
            .collect::<Result<_>>()
    }

    pub async fn remove_device(
        &mut self,
        account: AccountToken,
        device_id: DeviceId,
    ) -> Result<()> {
        self.0
            .remove_device(types::DeviceRemoval {
                account_token: account,
                device_id,
            })
            .await
            .map_err(map_device_error)?;
        Ok(())
    }

    pub async fn set_wireguard_rotation_interval(
        &mut self,
        interval: RotationInterval,
    ) -> Result<()> {
        let duration = types::Duration::try_from(*interval.as_duration())
            .map_err(|_| Error::DurationTooLarge)?;
        self.0
            .set_wireguard_rotation_interval(duration)
            .await
            .map_err(Error::Rpc)?;
        Ok(())
    }

    pub async fn reset_wireguard_rotation_interval(&mut self) -> Result<()> {
        self.0
            .reset_wireguard_rotation_interval(())
            .await
            .map_err(Error::Rpc)?;
        Ok(())
    }

    pub async fn rotate_wireguard_key(&mut self) -> Result<()> {
        self.0.rotate_wireguard_key(()).await.map_err(Error::Rpc)?;
        Ok(())
    }

    pub async fn get_wireguard_key(&mut self) -> Result<PublicKey> {
        let key = self
            .0
            .get_wireguard_key(())
            .await
            .map_err(Error::Rpc)?
            .into_inner();
        PublicKey::try_from(key).map_err(Error::InvalidResponse)
    }

    pub async fn list_custom_lists(&mut self) -> Result<Vec<CustomList>> {
        let result = self
            .0
            .list_custom_lists(())
            .await
            .map_err(map_custom_list_error)?
            .into_inner()
            .try_into()
            .map_err(Error::InvalidResponse)?;
        Ok(result)
    }

    pub async fn get_custom_list(&mut self, name: String) -> Result<CustomList> {
        let result = self
            .0
            .get_custom_list(name)
            .await
            .map_err(map_custom_list_error)?
            .into_inner()
            .try_into()
            .map_err(Error::InvalidResponse)?;
        Ok(result)
    }

    pub async fn create_custom_list(&mut self, name: String) -> Result<()> {
        self.0
            .create_custom_list(name)
            .await
            .map_err(map_custom_list_error)?;
        Ok(())
    }

    pub async fn delete_custom_list(&mut self, name: String) -> Result<()> {
        self.0
            .delete_custom_list(name)
            .await
            .map_err(map_custom_list_error)?;
        Ok(())
    }

    pub async fn update_custom_list_location(
        &mut self,
        custom_list_update: CustomListLocationUpdate,
    ) -> Result<()> {
        self.0
            .update_custom_list_location(types::CustomListLocationUpdate::from(custom_list_update))
            .await
            .map_err(map_custom_list_error)?;
        Ok(())
    }

    pub async fn rename_custom_list(&mut self, name: String, new_name: String) -> Result<()> {
        self.0
            .rename_custom_list(types::CustomListRename::from((name, new_name)))
            .await
            .map_err(map_custom_list_error)?;
        Ok(())
    }

    pub async fn add_access_method(&mut self, access_method: AccessMethod) -> Result<()> {
        self.0
            .add_api_access_method(types::ApiAccessMethod::from(access_method))
            .await
            .map_err(Error::Rpc)
            .map(drop)
    }

    pub async fn toggle_access_method(
        &mut self,
        access_method_toggle: ApiAccessMethodToggle,
    ) -> Result<()> {
        self.0
            .toggle_api_access_method(types::ApiAccessMethodToggle::from(access_method_toggle))
            .await
            .map_err(Error::Rpc)
            .map(drop)
    }

    pub async fn remove_access_method(&mut self, access_method: AccessMethod) -> Result<()> {
        self.0
            .remove_api_access_method(types::ApiAccessMethod::from(access_method))
            .await
            .map_err(Error::Rpc)
            .map(drop)
    }

    pub async fn replace_access_method(
        &mut self,
        access_method_replace: ApiAccessMethodReplace,
    ) -> Result<()> {
        self.0
            .replace_api_access_method(types::ApiAccessMethodReplace::from(access_method_replace))
            .await
            .map_err(Error::Rpc)
            .map(drop)
    }

    /// Set the [`AccessMethod`] which [`ApiConnectionModeProvider`] should
    /// pick.
    ///
    /// - `access_method`: If `Some(access_method)`, [`ApiConnectionModeProvider`] will skip
    ///     ahead and return `access_method` when asked for a new access method.
    ///     If `None`, [`ApiConnectionModeProvider`] will pick the next access
    ///     method "randomly"
    ///
    /// [`ApiConnectionModeProvider`]: mullvad_daemon::api::ApiConnectionModeProvider
    pub async fn set_access_method(&mut self, access_method: AccessMethod) -> Result<()> {
        self.0
            .set_api_access_method(types::ApiAccessMethod::from(access_method))
            .await
            .map_err(Error::Rpc)
            .map(drop)
    }

    #[cfg(target_os = "linux")]
    pub async fn get_split_tunnel_processes(&mut self) -> Result<Vec<i32>> {
        use futures::TryStreamExt;

        let procs = self
            .0
            .get_split_tunnel_processes(())
            .await
            .map_err(Error::Rpc)?
            .into_inner();
        procs.try_collect().await.map_err(Error::Rpc)
    }

    #[cfg(target_os = "linux")]
    pub async fn add_split_tunnel_process(&mut self, pid: i32) -> Result<()> {
        self.0
            .add_split_tunnel_process(pid)
            .await
            .map_err(Error::Rpc)?;
        Ok(())
    }

    #[cfg(target_os = "linux")]
    pub async fn remove_split_tunnel_process(&mut self, pid: i32) -> Result<()> {
        self.0
            .remove_split_tunnel_process(pid)
            .await
            .map_err(Error::Rpc)?;
        Ok(())
    }

    #[cfg(target_os = "linux")]
    pub async fn clear_split_tunnel_processes(&mut self) -> Result<()> {
        self.0
            .clear_split_tunnel_processes(())
            .await
            .map_err(Error::Rpc)?;
        Ok(())
    }

    #[cfg(target_os = "windows")]
    pub async fn add_split_tunnel_app<P: AsRef<Path>>(&mut self, path: P) -> Result<()> {
        let path = path.as_ref().to_str().ok_or(Error::PathMustBeUtf8)?;
        self.0
            .add_split_tunnel_app(path.to_owned())
            .await
            .map_err(Error::Rpc)?;
        Ok(())
    }

    #[cfg(target_os = "windows")]
    pub async fn remove_split_tunnel_app<P: AsRef<Path>>(&mut self, path: P) -> Result<()> {
        let path = path.as_ref().to_str().ok_or(Error::PathMustBeUtf8)?;
        self.0
            .remove_split_tunnel_app(path.to_owned())
            .await
            .map_err(Error::Rpc)?;
        Ok(())
    }

    #[cfg(target_os = "windows")]
    pub async fn clear_split_tunnel_apps(&mut self) -> Result<()> {
        self.0
            .clear_split_tunnel_apps(())
            .await
            .map_err(Error::Rpc)?;
        Ok(())
    }

    #[cfg(target_os = "windows")]
    pub async fn set_split_tunnel_state(&mut self, state: bool) -> Result<()> {
        self.0
            .set_split_tunnel_state(state)
            .await
            .map_err(Error::Rpc)?;
        Ok(())
    }

    #[cfg(target_os = "windows")]
    pub async fn get_excluded_processes(&mut self) -> Result<Vec<ExcludedProcess>> {
        let procs = self
            .0
            .get_excluded_processes(())
            .await
            .map_err(Error::Rpc)?
            .into_inner();
        Ok(procs
            .processes
            .into_iter()
            .map(ExcludedProcess::from)
            .collect::<Vec<_>>())
    }

    // check_volumes
}

fn map_device_error(status: Status) -> Error {
    match status.code() {
        Code::ResourceExhausted => Error::TooManyDevices,
        Code::Unauthenticated => Error::InvalidAccount,
        Code::AlreadyExists => Error::AlreadyLoggedIn,
        Code::NotFound => Error::DeviceNotFound,
        _other => Error::Rpc(status),
    }
}

fn map_location_error(status: Status) -> Error {
    match status.code() {
        Code::NotFound => Error::NoLocationData,
        _other => Error::Rpc(status),
    }
}

fn map_custom_list_error(status: Status) -> Error {
    match status.code() {
        Code::NotFound => {
            let details = status.details();
            if details == crate::CUSTOM_LIST_LOCATION_NOT_FOUND_DETAILS {
                Error::LocationNotFoundInCustomlist
            } else if details == crate::CUSTOM_LIST_LIST_NOT_FOUND_DETAILS {
                Error::CustomListListNotFound
            } else {
                Error::Rpc(status)
            }
        }
        Code::AlreadyExists => {
            let details = status.details();
            if details == crate::CUSTOM_LIST_LOCATION_EXISTS_DETAILS {
                Error::LocationExistsInCustomList
            } else if details == crate::CUSTOM_LIST_LIST_EXISTS_DETAILS {
                Error::CustomListExists
            } else {
                Error::Rpc(status)
            }
        }
        Code::InvalidArgument => Error::CustomListCannotAddOrRemoveAny,
        _other => Error::Rpc(status),
    }
}

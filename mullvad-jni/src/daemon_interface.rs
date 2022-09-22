use futures::{channel::oneshot, executor::block_on};
use mullvad_daemon::{device, DaemonCommand, DaemonCommandSender};
use mullvad_types::{
    account::{AccountData, AccountToken, VoucherSubmission},
    device::{Device, DeviceState},
    location::GeoIpLocation,
    relay_constraints::RelaySettingsUpdate,
    relay_list::RelayList,
    settings::{DnsOptions, Settings},
    states::{TargetState, TunnelState},
    version::AppVersionInfo,
    wireguard,
};

#[derive(Debug, err_derive::Error)]
#[error(no_from)]
pub enum Error {
    #[error(display = "Can't send command to daemon because it is not running")]
    NoDaemon(#[error(source)] mullvad_daemon::Error),

    #[error(display = "No response received from daemon")]
    NoResponse,

    #[error(display = "Attempt to use daemon command sender before it was configured")]
    NoSender,

    #[error(display = "Error performing RPC with the remote API")]
    RpcError(#[error(source)] mullvad_api::rest::Error),

    #[error(display = "Failed to update settings")]
    SettingsError,

    #[error(display = "Daemon returned an error")]
    OtherError(#[error(source)] mullvad_daemon::Error),
}

impl From<mullvad_daemon::Error> for Error {
    fn from(error: mullvad_daemon::Error) -> Error {
        match error {
            mullvad_daemon::Error::RestError(error) => Error::RpcError(error),
            mullvad_daemon::Error::LoginError(device::Error::OtherRestError(error)) => {
                Error::RpcError(error)
            }
            mullvad_daemon::Error::ListDevicesError(device::Error::OtherRestError(error)) => {
                Error::RpcError(error)
            }
            error => Error::OtherError(error),
        }
    }
}

type Result<T> = std::result::Result<T, Error>;

pub struct DaemonInterface {
    command_sender: DaemonCommandSender,
}

impl DaemonInterface {
    pub fn new(command_sender: DaemonCommandSender) -> Self {
        DaemonInterface { command_sender }
    }

    pub fn connect(&self) -> Result<()> {
        let (tx, rx) = oneshot::channel();

        self.send_command(DaemonCommand::SetTargetState(tx, TargetState::Secured))?;

        block_on(rx).map(|_| ()).map_err(|_| Error::NoResponse)
    }

    pub fn create_new_account(&self) -> Result<String> {
        let (tx, rx) = oneshot::channel();

        self.send_command(DaemonCommand::CreateNewAccount(tx))?;

        block_on(rx)
            .map_err(|_| Error::NoResponse)?
            .map_err(Error::from)
    }

    pub fn disconnect(&self) -> Result<()> {
        let (tx, rx) = oneshot::channel();

        self.send_command(DaemonCommand::SetTargetState(tx, TargetState::Unsecured))?;

        block_on(rx).map(|_| ()).map_err(|_| Error::NoResponse)
    }

    pub fn get_account_data(&self, account_token: String) -> Result<AccountData> {
        let (tx, rx) = oneshot::channel();

        self.send_command(DaemonCommand::GetAccountData(tx, account_token))?;

        block_on(rx)
            .map_err(|_| Error::NoResponse)?
            .map_err(Error::RpcError)
    }

    pub fn get_account_history(&self) -> Result<Option<AccountToken>> {
        let (tx, rx) = oneshot::channel();

        self.send_command(DaemonCommand::GetAccountHistory(tx))?;

        block_on(rx).map_err(|_| Error::NoResponse)
    }

    pub fn get_www_auth_token(&self) -> Result<String> {
        let (tx, rx) = oneshot::channel();

        self.send_command(DaemonCommand::GetWwwAuthToken(tx))?;

        block_on(rx)
            .map_err(|_| Error::NoResponse)?
            .map_err(Error::from)
    }

    pub fn get_current_location(&self) -> Result<Option<GeoIpLocation>> {
        let (tx, rx) = oneshot::channel();

        self.send_command(DaemonCommand::GetCurrentLocation(tx))?;

        Ok(block_on(rx).map_err(|_| Error::NoResponse)?)
    }

    pub fn get_current_version(&self) -> Result<String> {
        let (tx, rx) = oneshot::channel();

        self.send_command(DaemonCommand::GetCurrentVersion(tx))?;

        Ok(block_on(rx).map_err(|_| Error::NoResponse)?)
    }

    pub fn get_relay_locations(&self) -> Result<RelayList> {
        let (tx, rx) = oneshot::channel();

        self.send_command(DaemonCommand::GetRelayLocations(tx))?;

        Ok(block_on(rx).map_err(|_| Error::NoResponse)?)
    }

    pub fn get_settings(&self) -> Result<Settings> {
        let (tx, rx) = oneshot::channel();

        self.send_command(DaemonCommand::GetSettings(tx))?;

        Ok(block_on(rx).map_err(|_| Error::NoResponse)?)
    }

    pub fn get_state(&self) -> Result<TunnelState> {
        let (tx, rx) = oneshot::channel();

        self.send_command(DaemonCommand::GetState(tx))?;

        Ok(block_on(rx).map_err(|_| Error::NoResponse)?)
    }

    pub fn get_version_info(&self) -> Result<AppVersionInfo> {
        let (tx, rx) = oneshot::channel();

        self.send_command(DaemonCommand::GetVersionInfo(tx))?;

        block_on(rx)
            .map_err(|_| Error::NoResponse)?
            .ok_or(Error::NoResponse)
    }

    pub fn reconnect(&self) -> Result<()> {
        let (tx, _) = oneshot::channel();

        self.send_command(DaemonCommand::Reconnect(tx))?;

        Ok(())
    }

    pub fn clear_account_history(&self) -> Result<()> {
        let (tx, rx) = oneshot::channel();

        self.send_command(DaemonCommand::ClearAccountHistory(tx))?;

        block_on(rx)
            .map_err(|_| Error::NoResponse)?
            .map_err(Error::from)
    }

    pub fn get_wireguard_key(&self) -> Result<Option<wireguard::PublicKey>> {
        let (tx, rx) = oneshot::channel();

        self.send_command(DaemonCommand::GetWireguardKey(tx))?;

        block_on(rx)
            .map_err(|_| Error::NoResponse)?
            .map_err(Error::from)
    }

    pub fn login_account(&self, account_token: String) -> Result<()> {
        let (tx, rx) = oneshot::channel();

        self.send_command(DaemonCommand::LoginAccount(tx, account_token))?;

        block_on(rx)
            .map_err(|_| Error::NoResponse)?
            .map_err(Error::from)
    }

    pub fn logout_account(&self) -> Result<()> {
        let (tx, rx) = oneshot::channel();

        self.send_command(DaemonCommand::LogoutAccount(tx))?;

        block_on(rx)
            .map_err(|_| Error::NoResponse)?
            .map_err(Error::from)
    }

    pub fn get_device(&self) -> Result<DeviceState> {
        let (tx, rx) = oneshot::channel();

        self.send_command(DaemonCommand::GetDevice(tx))?;

        block_on(rx)
            .map_err(|_| Error::NoResponse)?
            .map_err(Error::from)
    }

    pub fn update_device(&self) -> Result<()> {
        let (tx, rx) = oneshot::channel();

        self.send_command(DaemonCommand::UpdateDevice(tx))?;

        block_on(rx)
            .map_err(|_| Error::NoResponse)?
            .map_err(Error::from)
    }

    pub fn list_devices(&self, account_token: String) -> Result<Vec<Device>> {
        let (tx, rx) = oneshot::channel();

        self.send_command(DaemonCommand::ListDevices(tx, account_token))?;

        block_on(rx)
            .map_err(|_| Error::NoResponse)?
            .map_err(Error::from)
    }

    pub fn remove_device(&self, account_token: String, device_id: String) -> Result<()> {
        let (tx, rx) = oneshot::channel();

        self.send_command(DaemonCommand::RemoveDevice(tx, account_token, device_id))?;

        block_on(rx)
            .map_err(|_| Error::NoResponse)?
            .map_err(Error::from)
    }

    pub fn set_allow_lan(&self, allow_lan: bool) -> Result<()> {
        let (tx, rx) = oneshot::channel();

        self.send_command(DaemonCommand::SetAllowLan(tx, allow_lan))?;

        block_on(rx)
            .map_err(|_| Error::NoResponse)?
            .map_err(|_| Error::SettingsError)
    }

    pub fn set_auto_connect(&self, auto_connect: bool) -> Result<()> {
        let (tx, rx) = oneshot::channel();

        self.send_command(DaemonCommand::SetAutoConnect(tx, auto_connect))?;

        block_on(rx)
            .map_err(|_| Error::NoResponse)?
            .map_err(|_| Error::SettingsError)
    }

    pub fn set_dns_options(&self, dns_options: DnsOptions) -> Result<()> {
        let (tx, rx) = oneshot::channel();

        self.send_command(DaemonCommand::SetDnsOptions(tx, dns_options))?;

        block_on(rx)
            .map_err(|_| Error::NoResponse)?
            .map_err(|_| Error::SettingsError)
    }

    pub fn set_wireguard_mtu(&self, wireguard_mtu: Option<u16>) -> Result<()> {
        let (tx, rx) = oneshot::channel();

        self.send_command(DaemonCommand::SetWireguardMtu(tx, wireguard_mtu))?;

        block_on(rx)
            .map_err(|_| Error::NoResponse)?
            .map_err(|_| Error::SettingsError)
    }

    pub fn shutdown(&self) -> Result<()> {
        self.command_sender.shutdown().map_err(Error::NoDaemon)
    }

    pub fn submit_voucher(&self, voucher: String) -> Result<VoucherSubmission> {
        let (tx, rx) = oneshot::channel();

        self.send_command(DaemonCommand::SubmitVoucher(tx, voucher))?;

        block_on(rx)
            .map_err(|_| Error::NoResponse)?
            .map_err(Error::from)
    }

    pub fn update_relay_settings(&self, update: RelaySettingsUpdate) -> Result<()> {
        let (tx, rx) = oneshot::channel();

        self.send_command(DaemonCommand::UpdateRelaySettings(tx, update))?;

        block_on(rx)
            .map_err(|_| Error::NoResponse)?
            .map_err(|_| Error::SettingsError)
    }

    fn send_command(&self, command: DaemonCommand) -> Result<()> {
        self.command_sender.send(command).map_err(Error::NoDaemon)
    }
}

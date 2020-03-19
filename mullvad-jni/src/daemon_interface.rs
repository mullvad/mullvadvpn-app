use futures::{sync::oneshot, Future};
use mullvad_daemon::{DaemonCommand, DaemonCommandSender};
use mullvad_types::{
    account::AccountData,
    location::GeoIpLocation,
    relay_constraints::RelaySettingsUpdate,
    relay_list::RelayList,
    settings::Settings,
    states::{TargetState, TunnelState},
    version::AppVersionInfo,
    wireguard::{self, KeygenEvent},
};

#[derive(Debug, err_derive::Error)]
pub enum Error {
    #[error(display = "Can't send command to daemon because it is not running")]
    NoDaemon(#[error(source)] mullvad_daemon::Error),

    #[error(display = "No response received from daemon")]
    NoResponse,

    #[error(display = "Attempt to use daemon command sender before it was configured")]
    NoSender,

    #[error(display = "Error performing RPC with the remote API")]
    RpcError(#[error(source)] jsonrpc_client_core::Error),
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

        rx.wait().map_err(|_| Error::NoResponse)?.unwrap();

        Ok(())
    }

    pub fn disconnect(&self) -> Result<()> {
        let (tx, rx) = oneshot::channel();

        self.send_command(DaemonCommand::SetTargetState(tx, TargetState::Unsecured))?;

        rx.wait().map_err(|_| Error::NoResponse)?.unwrap();

        Ok(())
    }

    pub fn generate_wireguard_key(&self) -> Result<KeygenEvent> {
        let (tx, rx) = oneshot::channel();

        self.send_command(DaemonCommand::GenerateWireguardKey(tx))?;

        rx.wait().map_err(|_| Error::NoResponse)
    }

    pub fn get_account_data(&self, account_token: String) -> Result<AccountData> {
        let (tx, rx) = oneshot::channel();

        self.send_command(DaemonCommand::GetAccountData(tx, account_token))?;

        rx.wait()
            .map_err(|_| Error::NoResponse)?
            .wait()
            .map_err(Error::RpcError)
    }

    pub fn get_account_history(&self) -> Result<Vec<String>> {
        let (tx, rx) = oneshot::channel();

        self.send_command(DaemonCommand::GetAccountHistory(tx))?;

        rx.wait().map_err(|_| Error::NoResponse)
    }

    pub fn get_www_auth_token(&self) -> Result<String> {
        let (tx, rx) = oneshot::channel();

        self.send_command(DaemonCommand::GetWwwAuthToken(tx))?;

        rx.wait()
            .map_err(|_| Error::NoResponse)?
            .wait()
            .map_err(Error::RpcError)
    }

    pub fn get_current_location(&self) -> Result<Option<GeoIpLocation>> {
        let (tx, rx) = oneshot::channel();

        self.send_command(DaemonCommand::GetCurrentLocation(tx))?;

        Ok(rx.wait().map_err(|_| Error::NoResponse)?)
    }

    pub fn get_current_version(&self) -> Result<String> {
        let (tx, rx) = oneshot::channel();

        self.send_command(DaemonCommand::GetCurrentVersion(tx))?;

        Ok(rx.wait().map_err(|_| Error::NoResponse)?)
    }

    pub fn get_relay_locations(&self) -> Result<RelayList> {
        let (tx, rx) = oneshot::channel();

        self.send_command(DaemonCommand::GetRelayLocations(tx))?;

        Ok(rx.wait().map_err(|_| Error::NoResponse)?)
    }

    pub fn get_settings(&self) -> Result<Settings> {
        let (tx, rx) = oneshot::channel();

        self.send_command(DaemonCommand::GetSettings(tx))?;

        Ok(rx.wait().map_err(|_| Error::NoResponse)?)
    }

    pub fn get_state(&self) -> Result<TunnelState> {
        let (tx, rx) = oneshot::channel();

        self.send_command(DaemonCommand::GetState(tx))?;

        Ok(rx.wait().map_err(|_| Error::NoResponse)?)
    }

    pub fn get_version_info(&self) -> Result<AppVersionInfo> {
        let (tx, rx) = oneshot::channel();

        self.send_command(DaemonCommand::GetVersionInfo(tx))?;

        rx.wait().map_err(|_| Error::NoResponse)
    }

    pub fn reconnect(&self) -> Result<()> {
        self.send_command(DaemonCommand::Reconnect)?;

        Ok(())
    }

    pub fn get_wireguard_key(&self) -> Result<Option<wireguard::PublicKey>> {
        let (tx, rx) = oneshot::channel();

        self.send_command(DaemonCommand::GetWireguardKey(tx))?;

        rx.wait().map_err(|_| Error::NoResponse)
    }

    pub fn verify_wireguard_key(&self) -> Result<bool> {
        let (tx, rx) = oneshot::channel();

        self.send_command(DaemonCommand::VerifyWireguardKey(tx))?;
        rx.wait().map_err(|_| Error::NoResponse)
    }

    pub fn set_account(&self, account_token: Option<String>) -> Result<()> {
        let (tx, rx) = oneshot::channel();

        self.send_command(DaemonCommand::SetAccount(tx, account_token))?;

        rx.wait().map_err(|_| Error::NoResponse)
    }

    pub fn set_allow_lan(&self, allow_lan: bool) -> Result<()> {
        let (tx, rx) = oneshot::channel();

        self.send_command(DaemonCommand::SetAllowLan(tx, allow_lan))?;

        rx.wait().map_err(|_| Error::NoResponse)
    }

    pub fn set_auto_connect(&self, auto_connect: bool) -> Result<()> {
        let (tx, rx) = oneshot::channel();

        self.send_command(DaemonCommand::SetAutoConnect(tx, auto_connect))?;

        rx.wait().map_err(|_| Error::NoResponse)
    }

    pub fn set_enable_ipv6(&self, enable_ipv6: bool) -> Result<()> {
        let (tx, rx) = oneshot::channel();

        self.send_command(DaemonCommand::SetEnableIpv6(tx, enable_ipv6))?;

        rx.wait().map_err(|_| Error::NoResponse)
    }

    pub fn shutdown(&self) -> Result<()> {
        self.send_command(DaemonCommand::Shutdown)
    }

    pub fn update_relay_settings(&self, update: RelaySettingsUpdate) -> Result<()> {
        let (tx, rx) = oneshot::channel();

        self.send_command(DaemonCommand::UpdateRelaySettings(tx, update))?;

        rx.wait().map_err(|_| Error::NoResponse)
    }

    fn send_command(&self, command: DaemonCommand) -> Result<()> {
        self.command_sender.send(command).map_err(Error::NoDaemon)
    }
}

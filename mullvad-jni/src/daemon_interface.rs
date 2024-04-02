use futures::{channel::oneshot, executor::block_on};
use mullvad_daemon::{device, DaemonCommand, DaemonCommandSender};
use mullvad_types::{
    account::{AccountData, AccountToken, PlayPurchase, VoucherSubmission},
    custom_list::CustomList,
    device::{Device, DeviceState},
    relay_constraints::{ObfuscationSettings, RelaySettings},
    relay_list::RelayList,
    settings::{DnsOptions, Settings},
    states::{TargetState, TunnelState},
    version::AppVersionInfo,
    wireguard,
    wireguard::QuantumResistantState,
};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Can't send command to daemon because it is not running")]
    NoDaemon(#[source] mullvad_daemon::Error),

    #[error("No response received from daemon")]
    NoResponse,

    #[error("Attempt to use daemon command sender before it was configured")]
    NoSender,

    #[error("Error performing RPC with the remote API")]
    Api(#[source] mullvad_api::rest::Error),

    #[error("Failed to update settings")]
    UpdateSettings,

    #[error("Daemon returned an error")]
    Other(#[source] mullvad_daemon::Error),
}

impl From<mullvad_daemon::Error> for Error {
    fn from(error: mullvad_daemon::Error) -> Error {
        match error {
            mullvad_daemon::Error::RestError(error) => Error::Api(error),
            mullvad_daemon::Error::LoginError(device::Error::OtherRestError(error)) => {
                Error::Api(error)
            }
            mullvad_daemon::Error::ListDevicesError(device::Error::OtherRestError(error)) => {
                Error::Api(error)
            }
            error => Error::Other(error),
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

    pub fn shutdown(&self) -> Result<()> {
        self.command_sender.shutdown().map_err(Error::NoDaemon)
    }

    fn send_command(&self, command: DaemonCommand) -> Result<()> {
        self.command_sender.send(command).map_err(Error::NoDaemon)
    }
}

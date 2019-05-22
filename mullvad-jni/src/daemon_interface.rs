use futures::{sync::oneshot, Future};
use mullvad_daemon::{DaemonCommandSender, ManagementCommand};
use mullvad_types::account::AccountData;

#[derive(Debug, err_derive::Error)]
pub enum Error {
    #[error(display = "Can't send command to daemon because it is not running")]
    NoDaemon(#[error(cause)] mullvad_daemon::Error),

    #[error(display = "No response received from daemon")]
    NoResponse,

    #[error(display = "Attempt to use daemon command sender before it was configured")]
    NoSender,

    #[error(display = "Error performing RPC with the remote API")]
    RpcError(#[error(cause)] jsonrpc_client_core::Error),
}

type Result<T> = std::result::Result<T, Error>;

pub struct DaemonInterface {
    command_sender: Option<DaemonCommandSender>,
}

impl DaemonInterface {
    pub fn new() -> Self {
        DaemonInterface {
            command_sender: None,
        }
    }

    pub fn set_command_sender(&mut self, sender: DaemonCommandSender) {
        self.command_sender = Some(sender);
    }

    pub fn get_account_data(&self, account_token: String) -> Result<AccountData> {
        let (tx, rx) = oneshot::channel();

        self.send_command(ManagementCommand::GetAccountData(tx, account_token))?;

        rx.wait()
            .map_err(|_| Error::NoResponse)?
            .wait()
            .map_err(Error::RpcError)
    }

    fn send_command(&self, command: ManagementCommand) -> Result<()> {
        let sender = self.command_sender.as_ref().ok_or(Error::NoSender)?;

        sender.send(command).map_err(Error::NoDaemon)
    }
}

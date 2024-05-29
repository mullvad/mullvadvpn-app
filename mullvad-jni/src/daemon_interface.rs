use mullvad_daemon::{DaemonCommandSender, Error};

pub struct DaemonInterface(DaemonCommandSender);

impl DaemonInterface {
    pub fn new(command_sender: DaemonCommandSender) -> Self {
        DaemonInterface(command_sender)
    }

    pub fn shutdown(&self) -> Result<(), Error> {
        self.0.shutdown()
    }
}

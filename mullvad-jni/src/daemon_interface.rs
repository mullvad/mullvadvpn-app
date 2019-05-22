use mullvad_daemon::DaemonCommandSender;

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
}

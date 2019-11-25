use crate::tunnel_state_machine::TunnelCommand;
use futures::sync::mpsc::UnboundedSender;
use std::sync::Weak;

#[derive(err_derive::Error, Debug)]
#[error(display = "Dummy offline check error")]
pub struct Error;

pub struct MonitorHandle;

impl MonitorHandle {
    pub fn is_offline(&self) -> bool {
        false
    }
}

pub fn spawn_monitor(
    _sender: Weak<UnboundedSender<TunnelCommand>>,
) -> Result<MonitorHandle, Error> {
    Ok(MonitorHandle)
}

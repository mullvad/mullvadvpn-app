use crate::tunnel_state_machine::TunnelCommand;
use futures::sync::mpsc::UnboundedSender;

#[derive(err_derive::Error, Debug)]
#[error(display = "Dummy offline check error")]
pub struct Error;

pub struct MonitorHandle;

pub fn spawn_monitor(_sender: UnboundedSender<TunnelCommand>) -> Result<MonitorHandle, Error> {
    Ok(MonitorHandle)
}

pub fn is_offline() -> bool {
    false
}

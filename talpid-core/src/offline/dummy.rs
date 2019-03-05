use crate::tunnel_state_machine::TunnelCommand;
use futures::sync::mpsc::UnboundedSender;

error_chain! {}

pub struct MonitorHandle;

pub fn spawn_monitor(_sender: UnboundedSender<TunnelCommand>) -> Result<MonitorHandle> {
    Ok(MonitorHandle)
}

pub fn is_offline() -> bool {
    false
}

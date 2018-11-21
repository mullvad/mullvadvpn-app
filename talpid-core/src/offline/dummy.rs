use futures::sync::mpsc::UnboundedSender;
use tunnel_state_machine::TunnelCommand;

error_chain! {}

pub struct MonitorHandle;

pub fn spawn_monitor(_sender: UnboundedSender<TunnelCommand>) -> Result<MonitorHandle> {
    Ok(MonitorHandle)
}

pub fn is_offline() -> bool {
    false
}

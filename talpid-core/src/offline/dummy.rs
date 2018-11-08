use futures::sync::mpsc::UnboundedSender;
use tunnel_state_machine::TunnelCommand;

error_chain!{}

pub fn spawn_monitor(_sender: UnboundedSender<TunnelCommand>) -> Result<()> {
    Ok(())
}

pub fn is_offline() -> bool {
    false
}

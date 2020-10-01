use crate::{format, Result};
use mullvad_management_interface::{
    types::{daemon_event::Event as EventType, tunnel_state::State},
    ManagementServiceClient,
};
use tokio::task::JoinHandle;

// Listens to state changes and prints each new state. To stop listening for changes, return false
// in continue_condition.
pub async fn state_listen<C: Fn(&State) -> Result<bool> + Send + 'static>(
    rpc: &mut ManagementServiceClient,
    continue_condition: C,
) -> Result<JoinHandle<Result<()>>> {
    let mut events = rpc.events_listen(()).await?.into_inner();
    let join_handle = tokio::spawn(async move {
        loop {
            if let Some(event) = events.message().await? {
                if let EventType::TunnelState(new_state) = event.event.unwrap() {
                    format::print_state(&new_state);
                    match continue_condition(&new_state.state.unwrap()) {
                        Ok(false) => break Ok(()),
                        Err(e) => break Err(e),
                        _ => {}
                    }
                }
            }
        }
    });

    Ok(join_handle)
}

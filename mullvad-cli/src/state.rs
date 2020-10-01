use crate::Result;
use mullvad_management_interface::{
    types::{daemon_event::Event as EventType, tunnel_state::State, TunnelState},
    ManagementServiceClient,
};
use tokio::task::JoinHandle;

pub async fn state_listen(
    rpc: &mut ManagementServiceClient,
    begin_condition: impl Fn(&State) -> bool,
    continue_condition: Box<dyn (Fn(&State) -> bool) + Send>,
    on_new_state: Box<dyn Fn(&TunnelState) + Send>,
) -> Result<Option<JoinHandle<()>>> {
    let state = rpc.get_tunnel_state(()).await?.into_inner();
    on_new_state(&state);

    let join_handle_option = match state.state {
        Some(inner_state) if begin_condition(&inner_state) => {
            let mut events = rpc.events_listen(()).await?.into_inner();
            let join_handle = tokio::spawn(async move {
                while let Some(event) = events.message().await.unwrap_or(None) {
                    if let EventType::TunnelState(new_state) = event.event.unwrap() {
                        on_new_state(&new_state);
                        if !continue_condition(&new_state.state.unwrap()) {
                            break;
                        }
                    }
                }
            });

            Some(join_handle)
        }
        _ => None,
    };

    Ok(join_handle_option)
}

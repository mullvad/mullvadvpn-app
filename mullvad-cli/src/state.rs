use futures::{
    channel::{mpsc, mpsc::Receiver},
    SinkExt,
};
use mullvad_management_interface::{
    types::{daemon_event::Event as EventType, TunnelState},
    ManagementServiceClient,
};

// Spawns a new task that listens for tunnel state changes and forwards it through the returned
// channel.
pub fn state_listen(mut rpc: ManagementServiceClient) -> Receiver<TunnelState> {
    let (mut sender, receiver) = mpsc::channel::<TunnelState>(100);
    tokio::spawn(async move {
        let mut events = rpc.events_listen(()).await.unwrap().into_inner();
        while let Ok(Some(event)) = events.message().await {
            if let EventType::TunnelState(new_state) = event.event.unwrap() {
                sender.send(new_state).await.unwrap();
            }
        }
    });

    receiver
}

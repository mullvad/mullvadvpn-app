use crate::{Error, Result};
use futures::{
    channel::{mpsc, mpsc::Receiver},
    SinkExt,
};
use mullvad_management_interface::{
    types::{daemon_event::Event as EventType, TunnelState},
    ManagementServiceClient,
};

// Spawns a new task that listens for tunnel state changes and forwards it through the returned
// channel. Panics if called from outside of the Tokio runtime.
pub fn state_listen(mut rpc: ManagementServiceClient) -> Receiver<Result<TunnelState>> {
    let (mut sender, receiver) = mpsc::channel::<Result<TunnelState>>(1);
    tokio::spawn(async move {
        match rpc.events_listen(()).await {
            Ok(events) => {
                let mut events = events.into_inner();
                loop {
                    let forward = match events.message().await {
                        Ok(Some(event)) => match event.event.unwrap() {
                            EventType::TunnelState(new_state) => Some(Ok(new_state)),
                            _ => None,
                        },
                        Ok(None) => break,
                        Err(status) => Some(Err(Error::GrpcClientError(status))),
                    };

                    if let Some(message) = forward {
                        if let Err(_) = sender.send(message).await {
                            break;
                        }
                    }
                }
            }
            Err(status) => {
                let _ = sender.send(Err(Error::GrpcClientError(status))).await;
            }
        }
    });

    receiver
}

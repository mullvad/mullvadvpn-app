use crate::{Error, Result};
use futures::{
    channel::{mpsc, mpsc::Receiver},
    SinkExt, StreamExt,
};
use mullvad_management_interface::{client::DaemonEvent, MullvadProxyClient};
use mullvad_types::states::TunnelState;

// Spawns a new task that listens for tunnel state changes and forwards it through the returned
// channel. Panics if called from outside of the Tokio runtime.
pub fn state_listen(mut rpc: MullvadProxyClient) -> Receiver<Result<TunnelState>> {
    let (mut sender, receiver) = mpsc::channel::<Result<TunnelState>>(1);
    tokio::spawn(async move {
        match rpc.events_listen().await {
            Ok(mut events) => {
                //let mut events = events.into_inner();
                loop {
                    let forward = match events.next().await {
                        Some(Ok(event)) => {
                            if let DaemonEvent::TunnelState(state) = event {
                                Ok(state)
                            } else {
                                continue;
                            }
                        }
                        Some(Err(error)) => Err(Error::ManagementInterfaceError(error)),
                        None => break,
                    };

                    if sender.send(forward).await.is_err() {
                        break;
                    }
                }
            }
            Err(error) => {
                let _ = sender
                    .send(Err(Error::ManagementInterfaceError(error)))
                    .await;
            }
        }
    });

    receiver
}

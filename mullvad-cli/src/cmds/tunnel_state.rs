use crate::format;
use anyhow::{anyhow, Result};
use futures::{Stream, StreamExt};
use mullvad_management_interface::{client::DaemonEvent, MullvadProxyClient};
use mullvad_types::states::TunnelState;

pub async fn connect(wait: bool) -> Result<()> {
    let mut rpc = MullvadProxyClient::new().await?;

    let listener = if wait {
        Some(rpc.events_listen().await?)
    } else {
        None
    };

    if rpc.connect_tunnel().await? {
        if let Some(receiver) = listener {
            wait_for_tunnel_state(receiver, |state| match state {
                TunnelState::Connected { .. } => Ok(true),
                TunnelState::Error(_) => Err(anyhow!("Failed to connect")),
                _ => Ok(false),
            })
            .await?;
        }
    }

    Ok(())
}

pub async fn disconnect(wait: bool) -> Result<()> {
    let mut rpc = MullvadProxyClient::new().await?;

    let listener = if wait {
        Some(rpc.events_listen().await?)
    } else {
        None
    };

    if rpc.disconnect_tunnel().await? {
        if let Some(receiver) = listener {
            wait_for_tunnel_state(receiver, |state| Ok(state.is_disconnected())).await?;
        }
    }

    Ok(())
}

pub async fn reconnect(wait: bool) -> Result<()> {
    let mut rpc = MullvadProxyClient::new().await?;

    let listener = if wait {
        Some(rpc.events_listen().await?)
    } else {
        None
    };

    if rpc.reconnect_tunnel().await? {
        if let Some(receiver) = listener {
            wait_for_tunnel_state(receiver, |state| match state {
                TunnelState::Connected { .. } => Ok(true),
                TunnelState::Error(_) => Err(anyhow!("Failed to reconnect")),
                _ => Ok(false),
            })
            .await?;
        }
    }

    Ok(())
}

async fn wait_for_tunnel_state(
    mut event_stream: impl Stream<Item = std::result::Result<DaemonEvent, mullvad_management_interface::Error>>
        + Unpin,
    matches_event: impl Fn(&TunnelState) -> Result<bool>,
) -> Result<()> {
    while let Some(state) = event_stream.next().await {
        if let DaemonEvent::TunnelState(new_state) = state? {
            format::print_state(&new_state, false);
            if matches_event(&new_state)? {
                return Ok(());
            }
        }
    }
    Err(anyhow!("Failed to wait for expected tunnel state"))
}

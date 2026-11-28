use crate::{BIN_NAME, format};
use anyhow::{Result, anyhow, bail};
use futures::{Stream, StreamExt};
use mullvad_management_interface::{MullvadProxyClient, client::DaemonEvent};
use mullvad_types::states::TunnelState;

pub async fn connect(wait: bool) -> Result<()> {
    let mut rpc = MullvadProxyClient::new().await?;

    if rpc.get_wireguard_identity().await?.is_none() {
        bail!(
            "No WireGuard config has been imported. Import one generated at mullvad.net with \
             'mullvad config import <file>'"
        );
    }

    let listener = if wait {
        Some(rpc.events_listen().await?)
    } else {
        None
    };

    if rpc.connect_tunnel().await?
        && let Some(receiver) = listener
    {
        wait_for_tunnel_state(receiver, |state| match state {
            TunnelState::Connected { .. } => Ok(true),
            TunnelState::Error(_) => Err(anyhow!("Failed to connect")),
            _ => Ok(false),
        })
        .await?;
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

    if rpc
        .disconnect_tunnel(&format!("{BIN_NAME} disconnect"))
        .await?
        && let Some(receiver) = listener
    {
        wait_for_tunnel_state(receiver, |state| Ok(state.is_disconnected())).await?;
    }

    Ok(())
}

pub async fn reconnect(wait: bool) -> Result<()> {
    let mut rpc = MullvadProxyClient::new().await?;

    if rpc.get_wireguard_identity().await?.is_none() {
        bail!(
            "No WireGuard config has been imported. Import one generated at mullvad.net with \
             'mullvad config import <file>'"
        );
    }

    let listener = if wait {
        Some(rpc.events_listen().await?)
    } else {
        None
    };

    let reconnecting = rpc.reconnect_tunnel().await?;
    if !reconnecting {
        bail!("Not reconnecting due to being in disconnected state")
    }
    if let Some(receiver) = listener {
        wait_for_tunnel_state(receiver, |state| match state {
            TunnelState::Connected { .. } => Ok(true),
            TunnelState::Error(_) => Err(anyhow!("Failed to reconnect")),
            _ => Ok(false),
        })
        .await?;
    }

    Ok(())
}

async fn wait_for_tunnel_state(
    mut event_stream: impl Stream<
        Item = std::result::Result<DaemonEvent, mullvad_management_interface::Error>,
    > + Unpin,
    matches_event: impl Fn(&TunnelState) -> Result<bool>,
) -> Result<()> {
    while let Some(state) = event_stream.next().await {
        if let DaemonEvent::TunnelState(new_state) = state? {
            format::print_state(&new_state, None, false);
            if matches_event(&new_state)? {
                return Ok(());
            }
        }
    }
    Err(anyhow!("Failed to wait for expected tunnel state"))
}

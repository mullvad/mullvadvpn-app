use crate::format;
use anyhow::{anyhow, Result};
use futures::{Stream, StreamExt};
use mullvad_management_interface::{client::DaemonEvent, MullvadProxyClient};
use mullvad_types::{device::DeviceState, states::TunnelState};

pub async fn connect(wait: bool) -> Result<()> {
    let mut rpc = MullvadProxyClient::new().await?;

    let device_state = rpc.get_device().await?;
    print_account_loggedout(&device_state);

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

    let device_state = rpc.get_device().await?;
    print_account_loggedout(&device_state);

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

/// Checks the if the user is logged in. If not, we print a warning to get their
/// attention.
///
/// When using the CLI, the user could potentially end up in a situation where
/// they try to connect to a Mullvad relay without having successfully logged in
/// to their account. In this case, we at least want to issue a warning to guide
/// the user when they inevitably will go troubleshooting.
fn print_account_loggedout(state: &DeviceState) {
    match state {
        DeviceState::LoggedOut => println!("Warning: You are not logged in to an account."),
        DeviceState::Revoked => println!("Warning: This device has been revoked"),
        DeviceState::LoggedIn(_) => return, // Normal case, do nothing.
    };

    println!(
        "Mullvad is blocking all network traffic until you perform one of the following actions:

1. Login to a Mullvad account with available time/credits.
2. Disconnect from Mullvad VPN. This can either be done from the CLI or the Mullvad App.

For more information, try 'mullvad account -h' or 'mullvad disconnect -h'"
    );
}

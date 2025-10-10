use crate::format;
use anyhow::{Result, anyhow};
use futures::{Stream, StreamExt};
use mullvad_management_interface::{MullvadProxyClient, client::DaemonEvent};
use mullvad_types::{device::DeviceState, states::TunnelState};

/// If `wait` is true, `connect` will return once an out-IP has been assigned.
pub async fn connect(wait: bool) -> Result<()> {
    let mut rpc = MullvadProxyClient::new().await?;

    let device_state = rpc.get_device().await?;
    print_account_loggedout(&device_state);

    let listener = if wait {
        Some(rpc.events_listen().await?)
    } else {
        None
    };

    if rpc.connect_tunnel().await?
        && let Some(receiver) = listener
    {
        wait_for_out_ip(receiver).await?;
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

    if rpc.disconnect_tunnel().await?
        && let Some(receiver) = listener
    {
        wait_for_tunnel_state(receiver, |state| Ok(state.is_disconnected())).await?;
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

    if rpc.reconnect_tunnel().await?
        && let Some(receiver) = listener
    {
        wait_for_out_ip(receiver).await?;
    }

    Ok(())
}

/// Wait until a Connected event containing an out-IP is seen.
async fn wait_for_out_ip(
    event_stream: impl Stream<
        Item = std::result::Result<DaemonEvent, mullvad_management_interface::Error>,
    > + Unpin,
) -> Result<()> {
    wait_for_tunnel_state(event_stream, |state| match state {
        TunnelState::Connected {
            location: Some(location),
            ..
        } if location.ipv4.is_some() || location.ipv6.is_some() => Ok(true),
        TunnelState::Error(_) => Err(anyhow!("Failed to connect")),
        _ => Ok(false),
    })
    .await
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

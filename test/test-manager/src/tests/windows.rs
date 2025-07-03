//! Windows-specific tests.

use anyhow::{ensure, Context};
use mullvad_management_interface::MullvadProxyClient;
use test_rpc::ServiceClient;

use crate::tests::helpers;

use super::TestContext;

/// TODO: Explain me
#[test_function(target_os = "windows")]
async fn test_clearing_blocked_state_on_failed_upgrade(
    _: TestContext,
    rpc: ServiceClient,
    mullvad_client: MullvadProxyClient,
) -> anyhow::Result<()> {
    // Enter a secured state by connecting to a tunnel
    // This is necessary to ensure that the firewall rules are applied
    // Note that we do not need to wait for the tunnel to be fully connected
    mullvad_client
        .connect_tunnel()
        .await
        .expect("failed to begin connecting");
    let new_state = wait_for_tunnel_state(mullvad_client.clone(), |state| {
        matches!(
            state,
            TunnelState::Connecting { .. } | TunnelState::Error(..)
        )
    })
    .await?;
    // Simulate a failed upgrade
    // Prepare the daemon for restarting
    rpc.prepare_restart().await?;
    // Simulate that the daemon has been removed
    // Do this by disabling the system service (the important part is that it does not restart
    // automatically on reboot)
    rpc.disable_system_service_startup().await?;
    rpc.stop_app().await?;
    // Make sure that blocking firewall rules are active - there should be no leaks (yet) 💦❌
    let blocked = helpers::conncheck(&rpc).await.is_err();
    ensure!(
        blocked,
        "Device is leaking - blocking rules have not applied properly"
    );
    // Reboot - we expect desperate users to take this measure
    rpc.reboot().await?;
    // The conn check should now fail - the firewall filters should have been removed at this point 💦💦💦
    let mullvad_exit_ip = helpers::conncheck(&rpc)
        .await
        .context("Device is offline")?;
    ensure!(!mullvad_exit_ip, "Should *not* be a Mullvad Exit IP");
    // Make sure system service is enabled in the test runner.

    // Do the same thing again, but with lockdown mode enabled?
    // assert that the firewall rules are still applied

    // TODO: Move this to clean up routine
    rpc.enable_system_service_startup().await?;
    Ok(())
}

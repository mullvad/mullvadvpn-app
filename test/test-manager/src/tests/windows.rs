//! Windows-specific tests.

use anyhow::{ensure, Context};
use mullvad_management_interface::MullvadProxyClient;
use mullvad_types::states::TunnelState;
use test_macro::test_function;
use test_rpc::ServiceClient;

use crate::tests::helpers::{geoip_lookup_with_retries, wait_for_tunnel_state};

use super::TestContext;

/// TODO: Explain me
#[test_function(target_os = "windows")]
async fn test_clearing_blocked_state_on_failed_upgrade(
    _: TestContext,
    mut rpc: ServiceClient,
    mut mullvad_client: MullvadProxyClient,
) -> anyhow::Result<()> {
    let settings = mullvad_client.get_settings().await?;
    assert!(
        !settings.block_when_disconnected,
        "Block when dissconnected should be disabled"
    );
    assert!(!settings.auto_connect, "Auto connect should be disabled");

    log::info!("Connecting to tunnel to enter secured state");
    // This is necessary to ensure that the firewall rules are applied
    // Note that we do not need to wait for the tunnel to be fully connected
    mullvad_client
        .connect_tunnel()
        .await
        .expect("failed to begin connecting");
    log::info!("Waiting for tunnel state to be Connecting or Error");
    wait_for_tunnel_state(mullvad_client.clone(), |state| {
        matches!(
            state,
            TunnelState::Connecting { .. } | TunnelState::Error(..)
        )
    })
    .await?;
    // Simulate a failed upgrade
    log::info!("Preparing daemon for restart (simulate failed upgrade)");
    // Prepare the daemon for restarting
    mullvad_client
        .prepare_restart_v2(false)
        .await
        .context("Failed to prepare restart")?;
    // Simulate that the daemon has been removed
    log::info!("Disabling Mullvad daemon system service");
    // Do this by disabling the system service (the important part is that it does not restart
    // automatically on reboot)
    rpc.disable_mullvad_daemon().await?;
    // Make sure that blocking firewall rules are active - there should be no leaks (yet) 💦❌
    log::info!("Checking that blocking firewall rules are active...");
    let blocked = geoip_lookup_with_retries(&rpc).await.is_err();
    ensure!(
        blocked,
        "Device is leaking - blocking rules have not applied properly"
    );
    // Reboot - we expect desperate users to take this measure
    log::info!("Rebooting device...");
    rpc.reboot().await?;
    // The conn check should now fail - the firewall filters should have been removed at this point 💦💦💦
    log::info!("Checking connectivity after reboot (should be offline)");
    let mullvad_exit_ip = geoip_lookup_with_retries(&rpc)
        .await
        .context("Device is offline after reboot")?
        .mullvad_exit_ip;
    ensure!(!mullvad_exit_ip, "Should *not* be a Mullvad Exit IP");
    // Make sure system service is enabled in the test runner.

    // Do the same thing again, but with lockdown mode enabled?
    // assert that the firewall rules are still applied

    // TODO: Move this to clean up routine
    log::info!("Re-enabling Mullvad daemon system service");
    rpc.enable_mullvad_daemon().await?;
    Ok(())
}

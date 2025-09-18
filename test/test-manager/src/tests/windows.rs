//! Windows-specific tests.

use anyhow::{Context, ensure};
use mullvad_management_interface::MullvadProxyClient;
use mullvad_types::states::TunnelState;
use test_macro::test_function;
use test_rpc::ServiceClient;

use crate::tests::helpers::{geoip_lookup_with_retries, wait_for_tunnel_state};

use super::TestContext;

/// Test that, on a failed upgrade, blocking firewall rules are cleared on a reboot.
#[test_function(target_os = "windows")]
async fn test_clearing_blocked_state_on_failed_upgrade(
    _: TestContext,
    mut rpc: ServiceClient,
    mut mullvad_client: MullvadProxyClient,
) -> anyhow::Result<()> {
    // Assert that the below settings are disabled. If they are not,
    // then blocking firewall rules *will* persist after a reboot.
    {
        let settings = mullvad_client.get_settings().await?;
        ensure!(
            !settings.lockdown_mode,
            "Block when disconnected should be disabled"
        );
        ensure!(!settings.auto_connect, "Auto connect should be disabled");
    }

    log::info!("Connecting to tunnel to enter secured state");
    // This is necessary to ensure that the firewall rules are applied
    // Note that we do not need to wait for the tunnel to be fully connected
    mullvad_client
        .connect_tunnel()
        .await
        .context("failed to begin connecting")?;
    log::info!("Waiting for tunnel state to be Connected or Error");
    wait_for_tunnel_state(mullvad_client.clone(), |state| {
        matches!(
            state,
            TunnelState::Connected { .. } | TunnelState::Error(..)
        )
    })
    .await?;
    log::info!("Preparing daemon for restart (simulate failed upgrade)");
    mullvad_client
        .prepare_restart_v2(false)
        .await
        .context("Failed to prepare restart")?;

    // Simulate that the daemon has been uninstalled, by disabling the system service.
    // We cannot actually uninstall the daemon here, because it would remove the blocking firewall rules,
    // regardless of having called `prepare_restart_v2`.
    log::info!("Disabling Mullvad daemon system service");
    rpc.disable_mullvad_daemon().await?;
    rpc.stop_mullvad_daemon().await?;

    // Make sure that blocking firewall rules are active - there should be no leaks (yet) 💦❌
    log::info!("Checking that blocking firewall rules are active...");
    let geoip = geoip_lookup_with_retries(&rpc).await;
    ensure!(
        geoip.is_err(),
        "Device is leaking with geo IP '{:?}'- blocking rules have not applied properly",
        geoip.unwrap()
    );
    // Reboot - we expect desperate users to take this measure
    log::info!("Rebooting device...");
    rpc.reboot().await?;
    // The conn check should now fail - the firewall filters should have been removed at this point 💦💦💦
    log::info!("Checking connectivity after reboot (should be online, but not secured)");
    let mullvad_exit_ip = geoip_lookup_with_retries(&rpc)
        .await
        .context("Device is offline after reboot")?
        .mullvad_exit_ip;
    ensure!(!mullvad_exit_ip, "Should *not* be a Mullvad Exit IP");

    Ok(())
}

/// Test that, on a failed upgrade when `Auto-connect` is enabled, blocking firewall rules are *not* cleared on a reboot.
#[test_function(target_os = "windows")]
async fn test_not_clearing_blocked_state_on_failed_upgrade_with_lockdown_mode(
    _: TestContext,
    mut rpc: ServiceClient,
    mut mullvad_client: MullvadProxyClient,
) -> anyhow::Result<()> {
    // Make sure that lockdown mode is enabled.
    // If it is not, then blocking firewall rules *will not* persist after a reboot.
    {
        mullvad_client.set_lockdown_mode(true).await?;
        let settings = mullvad_client.get_settings().await?;
        ensure!(
            settings.lockdown_mode,
            "Block when disconnected should be enabled"
        );
        ensure!(!settings.auto_connect, "Auto connect should be disabled");
    }

    log::info!("Waiting for tunnel state to be Disconnected with lockdown enabled");
    wait_for_tunnel_state(mullvad_client.clone(), |state| {
        matches!(
            state,
            TunnelState::Disconnected { locked_down, .. }  if *locked_down
        )
    })
    .await?;
    log::info!("Preparing daemon for restart (simulate failed upgrade)");
    mullvad_client
        .prepare_restart_v2(false)
        .await
        .context("Failed to prepare restart")?;

    // Simulate that the daemon has been uninstalled, by disabling the system service.
    // We cannot actually uninstall the daemon here, because it would remove the blocking firewall rules,
    // regardless of having called `prepare_restart_v2`.
    log::info!("Disabling Mullvad daemon system service");
    rpc.disable_mullvad_daemon().await?;
    rpc.stop_mullvad_daemon().await?;

    // Make sure that blocking firewall rules are active - there should be no leaks 💦❌
    log::info!("Checking that blocking firewall rules are active...");
    let blocked = geoip_lookup_with_retries(&rpc).await.is_err();
    ensure!(
        blocked,
        "Device is leaking - blocking rules have not applied properly"
    );
    // Reboot - we expect desperate users to take this measure
    log::info!("Rebooting device...");
    rpc.reboot().await?;

    // The conn check should now fail - the firewall filters should *not* have been removed at this point 💦❌
    log::info!("Checking connectivity after reboot (should be blocked)");
    let blocked = geoip_lookup_with_retries(&rpc).await.is_err();
    ensure!(
        blocked,
        "Device is leaking - blocking rules have not applied properly"
    );

    Ok(())
}

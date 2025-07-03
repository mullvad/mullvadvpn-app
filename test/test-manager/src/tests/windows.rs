//! Windows-specific tests.

use anyhow::Context;
use mullvad_management_interface::MullvadProxyClient;
use test_rpc::ServiceClient;

use crate::tests::helpers;

use super::TestContext;

/// TODO: Explain me
#[test_function(target_os = "windows")]
async fn test_clearing_blocked_state_on_failed_upgrade(
    _: TestContext,
    rpc: ServiceClient,
    _: MullvadProxyClient,
) -> anyhow::Result<()> {
    // Simulate a failed upgrade
    // Prepare the daemon for restarting
    // TODO: How?

    // Simulate that the daemon has been removed
    // Do this by disabling the system service (the important part is that it does not restart
    // automatically on reboot)

    // The conn check should now fail - the firewall filters should have been removed at this point
    let mullvad_exit_ip = helpers::conncheck(&rpc)
        .await
        .context("Device is offline")?;
    assert!(!mullvad_exit_ip, "Should *not* be a Mullvad Exit IP");
    Ok(())
}

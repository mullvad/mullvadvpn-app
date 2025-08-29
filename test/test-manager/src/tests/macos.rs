//! macOS-specific tests.

use anyhow::{Context, bail, ensure};
use mullvad_management_interface::MullvadProxyClient;
use std::net::{Ipv4Addr, SocketAddr};
use test_macro::test_function;
use test_rpc::ServiceClient;

use super::TestContext;

/// Test that we can add and remove IP "aliases" to network interfaces.
///
/// This is effectively testing that macOS behaves as expected, and that future versions of it
/// don't break this functionality.
#[test_function(target_os = "macos")]
async fn test_ifconfig_add_alias(
    _: TestContext,
    rpc: ServiceClient,
    _: MullvadProxyClient,
) -> anyhow::Result<()> {
    let alias = Ipv4Addr::new(127, 123, 123, 123);
    let interface = "lo0";

    log::info!("Will try to assign alias {alias} to interface {interface}");

    // Sanity-check that alias does not exist before we add it.
    ensure!(
        !alias_exists(&rpc, interface, alias).await?,
        "Alias shouldn't exist before it's created. Was it left over from a previous test?"
    );

    // Add alias and assert that it exists.
    rpc.ifconfig_alias_add(interface, alias).await?;
    ensure!(
        alias_exists(&rpc, interface, alias).await?,
        "Alias should have been created!"
    );

    // Ensure that we clean up the alias after the test, even if it fails
    let rpc2 = rpc.clone();
    let _cleanup_guard = scopeguard::guard((), |()| {
        log::info!("Cleaning up after test_ifconfig_add_alias");

        let Ok(runtime_handle) = tokio::runtime::Handle::try_current() else {
            log::error!("Missing tokio runtime");
            return;
        };

        runtime_handle.spawn(async move {
            // Ensure that the alias is removed even if the test fails.
            if let Err(e) = rpc2.ifconfig_alias_remove(interface, alias).await {
                log::error!("Failed to remove alias {alias} from interface {interface}: {e}");
            }
        });
    });

    // Assert that we can bind to the alias.
    rpc.send_udp(
        None,
        SocketAddr::from((alias, 0)),
        SocketAddr::from((Ipv4Addr::LOCALHOST, 1234)),
    )
    .await
    .context("Failed to bind to alias")?;

    // Remove alias and assert that it doesn't exist.
    rpc.ifconfig_alias_remove(interface, alias).await?;
    ensure!(
        !alias_exists(&rpc, interface, alias).await?,
        "Alias should have been removed!"
    );

    Ok(())
}

/// Check if an IP alias exists for `interface`.
async fn alias_exists(
    rpc: &ServiceClient,
    interface: &str,
    alias: Ipv4Addr,
) -> anyhow::Result<bool> {
    let alias = alias.to_string();
    let result = rpc.exec("ifconfig", [interface]).await?;

    let stdout = String::from_utf8(result.stdout)?;
    let stderr = String::from_utf8(result.stderr)?;

    if result.code != Some(0) {
        log::error!("ifconfig stdout:\n{stdout}");
        log::error!("ifconfig stderr:\n{stderr}");
        bail!("`ifconfig` exited with code {:?}", result.code);
    }

    Ok(stdout.contains(&alias))
}

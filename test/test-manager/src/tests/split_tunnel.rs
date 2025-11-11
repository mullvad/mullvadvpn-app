use anyhow::Context;
use mullvad_management_interface::MullvadProxyClient;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use test_macro::test_function;
use test_rpc::{ServiceClient, meta::OsVersion};

use super::{
    TestContext,
    helpers::{self, ConnChecker},
    ui,
};

const LEAK_DESTINATION: SocketAddr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(1, 1, 1, 1)), 1337);

/// Test that split tunneling works by asserting the following:
/// - Splitting a process shouldn't do anything if tunnel is not connected.
/// - A split process should never push traffic through the tunnel.
/// - Splitting/unsplitting should work regardless if process is running.
#[test_function]
pub async fn test_split_tunnel(
    _ctx: TestContext,
    rpc: ServiceClient,
    mut mullvad_client: MullvadProxyClient,
) -> anyhow::Result<()> {
    // Skip test on macOS 12, since the feature is unsupported
    if is_macos_12_or_lower(&rpc).await? {
        return Ok(());
    }

    let mut checker = ConnChecker::new(rpc.clone(), mullvad_client.clone(), LEAK_DESTINATION);

    // Test that program is behaving when we are disconnected
    (checker.spawn().await?.assert_insecure().await)
        .with_context(|| "Test disconnected and unsplit")?;
    checker.split().await?;
    (checker.spawn().await?.assert_insecure().await)
        .with_context(|| "Test disconnected and split")?;
    checker.unsplit().await?;

    // Test that program is behaving being split/unsplit while running and we are disconnected
    let mut handle = checker.spawn().await?;
    handle.split().await?;
    (handle.assert_insecure().await)
        .with_context(|| "Test disconnected and being split while running")?;
    handle.unsplit().await?;
    (handle.assert_insecure().await)
        .with_context(|| "Test disconnected and being unsplit while running")?;
    drop(handle);

    helpers::connect_and_wait(&mut mullvad_client).await?;

    // Test running an unsplit program
    checker
        .spawn()
        .await?
        .assert_secure()
        .await
        .with_context(|| "Test connected and unsplit")?;

    // Test running a split program
    checker.split().await?;

    checker
        .spawn()
        .await?
        .assert_insecure()
        .await
        .with_context(|| "Test connected and split")?;

    checker.unsplit().await?;

    // Test splitting and unsplitting a program while it's running
    let mut handle = checker.spawn().await?;
    (handle.assert_secure().await).with_context(|| "Test connected and unsplit (again)")?;
    handle.split().await?;
    (handle.assert_insecure().await)
        .with_context(|| "Test connected and being split while running")?;
    handle.unsplit().await?;
    (handle.assert_secure().await)
        .with_context(|| "Test connected and being unsplit while running")?;

    Ok(())
}

/// Test that split tunneling works by asserting the following:
/// - Splitting a process with the split tunneling (ST) feature enabled and an active tunnel
///   allow the split process to leak.
/// - Disabling ST forces the split program to route its traffic through the active tunnel.
/// - Enabling ST allows the split program to leak again.
/// The property we're testing for here is that toggling ST respects the list of split apps, and
/// vice-versa.
///
/// NOTE: This will not work with Linux split tunneling, since there is no persistant list of split
/// apps(yet!).
#[test_function(target_os = "macos", target_os = "windows")]
pub async fn test_split_tunnel_toggle(
    _ctx: TestContext,
    rpc: ServiceClient,
    mut mullvad_client: MullvadProxyClient,
) -> anyhow::Result<()> {
    // I'm a gamer, so I want to split steam for maximum performance.
    let mut steam = ConnChecker::new(rpc.clone(), mullvad_client.clone(), LEAK_DESTINATION);
    // Enable the split tunneling feature in the daemon.
    // No apps are split at this point.
    //
    // Note: ConnChecker::split already does this, but being explicit with the state the we expect
    // the daemon to be in at any stage of the test is not harmful.
    mullvad_client.set_split_tunnel_state(true).await?; // <- Split tunneling: on
    // Connect.
    helpers::connect_and_wait(&mut mullvad_client).await?;
    // Assert that steam does not leak yet. We are yet to add it as a split app.
    let mut steam = steam.spawn().await?;
    steam.assert_secure().await?;
    // Assert that splitting the process does indeed leak.
    steam.split().await?; // <- SPLIT
    steam.assert_insecure().await?;
    // Disabling split-tunneling at the settings-level should force all traffic through the tunnel.
    // HACK: ConnChecker::split tries to be clever and enables ST at the settings-level.
    // Therefore we have to explicitly disable it *after* calling split.
    mullvad_client.set_split_tunnel_state(false).await?; // <- Split tunneling: off
    // Steam should now be forced to route traffic through the tunnel again.
    steam.assert_secure().await?;
    // Re-enabling split-tunneling will once again make the split program leak.
    mullvad_client.set_split_tunnel_state(true).await?; // <- Split tunneling: on
    steam.assert_insecure().await?;
    Ok(())
}

/// Test that split tunneling works by asserting the following:
/// - Splitting a process shouldn't do anything if tunnel is not connected.
/// - A split process should never push traffic through the tunnel.
/// - Splitting/unsplitting should work regardless if process is running.
#[test_function(target_os = "macos", target_os = "windows")]
pub async fn test_split_tunnel_ui(
    _ctx: TestContext,
    rpc: ServiceClient,
    _: MullvadProxyClient,
) -> anyhow::Result<()> {
    // Skip test on macOS 12 and on Linux, since the feature is unsupported
    if cfg!(target_os = "macos") && is_macos_12_or_lower(&rpc).await? {
        return Ok(());
    }

    let ui_result = ui::run_test(&rpc, &["split-tunneling.spec"]).await.unwrap();
    assert!(ui_result.success());

    Ok(())
}

async fn is_macos_12_or_lower(rpc: &ServiceClient) -> anyhow::Result<bool> {
    match rpc.get_os_version().await.context("Detect OS version")? {
        OsVersion::Macos(version) if version.major <= 12 => Ok(true),
        _ => Ok(false),
    }
}

use anyhow::Context;
use mullvad_management_interface::MullvadProxyClient;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use test_macro::test_function;
use test_rpc::{meta::OsVersion, ServiceClient};

use super::{
    helpers::{self, ConnChecker},
    ui, TestContext,
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
/// - Splitting a process shouldn't do anything if tunnel is not connected.
/// - A split process should never push traffic through the tunnel.
/// - Splitting/unsplitting should work regardless if process is running.
#[test_function(target_os = "macos")]
pub async fn test_split_tunnel_ui(
    _ctx: TestContext,
    rpc: ServiceClient,
    _: MullvadProxyClient,
) -> anyhow::Result<()> {
    // Skip test on macOS 12, since the feature is unsupported
    if is_macos_12_or_lower(&rpc).await? {
        return Ok(());
    }

    let ui_result = ui::run_test(&rpc, &["macos-split-tunneling.spec"])
        .await
        .unwrap();
    assert!(ui_result.success());

    Ok(())
}

async fn is_macos_12_or_lower(rpc: &ServiceClient) -> anyhow::Result<bool> {
    match rpc.get_os_version().await.context("Detect OS version")? {
        OsVersion::Macos(version) if version.major <= 12 => Ok(true),
        _ => Ok(false),
    }
}

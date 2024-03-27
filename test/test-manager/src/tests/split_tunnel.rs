use anyhow::Context;
use mullvad_management_interface::MullvadProxyClient;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use test_macro::test_function;
use test_rpc::ServiceClient;

use super::{
    helpers::{self, ConnChecker},
    TestContext,
};

const LEAK_DESTINATION: SocketAddr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(1, 1, 1, 1)), 1337);

/// Test that split tunneling works by asserting the following:
/// - Splitting a process shouldn't do anything if tunnel is not connected.
/// - A split process should never push traffic through the tunnel.
/// - Splitting/unsplitting should work regardless if process is running.
#[test_function(target_os = "linux", target_os = "windows")]
pub async fn test_split_tunnel(
    _ctx: TestContext,
    rpc: ServiceClient,
    mut mullvad_client: MullvadProxyClient,
) -> anyhow::Result<()> {
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

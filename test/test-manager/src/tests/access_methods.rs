//! Integration tests for API access methods.
//!
//! The tested access methods are:
//! * Shadowsocks
//! * SOCKS5 in remote mode
//!
//! These tests rely on working proxies to exist *somewhere* for all tested protocols.
//! If the proxies themselves are bad/not running, this test will fail due to issues
//! that are out of the test manager's control.
use anyhow::{Context, anyhow, ensure};

use mullvad_management_interface::MullvadProxyClient;
use mullvad_types::relay_list::RelayList;
use talpid_types::net::proxy::CustomProxy;
use test_macro::test_function;
use test_rpc::ServiceClient;

use crate::tests::config::TEST_CONFIG;

use super::TestContext;

/// Assert that API traffic can be proxied via a custom Shadowsocks proxy.
#[test_function]
async fn test_access_method_shadowsocks(
    _: TestContext,
    _rpc: ServiceClient,
    mut mullvad_client: MullvadProxyClient,
) -> anyhow::Result<()> {
    use mullvad_relay_selector::{RelaySelector, SelectorConfig};
    log::info!("Testing Shadowsocks access method");
    // Set up all the parameters needed to create a custom Shadowsocks access method.
    //
    // Since Mullvad's bridge servers host Shadowsocks relays, we can simply
    // select a bridge server to derive all the needed parameters.
    let bridge_list = mullvad_client.get_bridges().await.unwrap();
    let relay_selector =
        RelaySelector::from_list(SelectorConfig::default(), RelayList::default(), bridge_list);
    let access_method = relay_selector
        .get_bridge_forced()
        .context("`test_shadowsocks` needs at least one shadowsocks relay to execute. Found none in relay list.")?;
    log::info!("Selected shadowsocks bridge: {access_method:?}");
    assert_access_method_works(mullvad_client, access_method.clone())
        .await
        .with_context(|| anyhow!("Access method {access_method:?} did not work!"))?;
    Ok(())
}

/// Assert that API traffic can be proxied via a custom SOCKS5 proxy.
#[test_function]
async fn test_access_method_socks5_remote(
    _: TestContext,
    _rpc: ServiceClient,
    mullvad_client: MullvadProxyClient,
) -> anyhow::Result<()> {
    use crate::vm::network::SOCKS5_PORT;
    use std::net::SocketAddr;
    use talpid_types::net::proxy::Socks5Remote;
    log::info!("Testing SOCKS5 (Remote) access method");
    // Set up all the parameters needed to create a custom SOCKS5 access method.
    //
    // The remote SOCKS5 proxy is assumed to be running on the test manager. On
    // which port it listens to is defined as a constant in the `test-manager`
    // crate.
    let endpoint = SocketAddr::from((TEST_CONFIG.host_bridge_ip, SOCKS5_PORT));
    let access_method = Socks5Remote::new(endpoint);
    log::info!("Testing SOCKS5-proxy: {access_method:?}");
    assert_access_method_works(mullvad_client, access_method.clone())
        .await
        .with_context(|| anyhow!("Access method {access_method:?} did not work!"))?;
    Ok(())
}

async fn assert_access_method_works(
    mut mullvad_client: MullvadProxyClient,
    access_method: impl Into<CustomProxy> + std::fmt::Debug,
) -> anyhow::Result<()> {
    let successful = mullvad_client
        .test_custom_api_access_method(access_method.into())
        .await
        .context("Failed to test custom API access method")?;

    ensure!(successful, "Failed while testing access method");
    Ok(())
}

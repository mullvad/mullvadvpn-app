//! Integration tests for API access methods.
use super::{Error, TestContext};
use mullvad_management_interface::MullvadProxyClient;
use test_macro::test_function;
use test_rpc::ServiceClient;

/// Assert that custom access methods may be used to access the Mullvad API.
///
/// The tested access methods are:
/// * Shadowsocks
/// * Socks5 in remote mode
///
/// # Note
///
/// This tests assume that there exists working proxies *somewhere* for all
/// tested protocols. If the proxies themselves are bad/not running, this test
/// will fail due to issues that are out of the test manager's control.
#[test_function]
pub async fn test_custom_access_methods(
    _: TestContext,
    _rpc: ServiceClient,
    mullvad_client: MullvadProxyClient,
) -> Result<(), Error> {
    log::info!("Testing Shadowsocks access method");
    test_shadowsocks(mullvad_client.clone()).await?;
    log::info!("Testing SOCKS5 (Remote) access method");
    test_socks_remote(mullvad_client.clone()).await?;
    Ok(())
}

macro_rules! assert_access_method_works {
    ($mullvad_client:expr, $access_method:expr) => {
        let successful = $mullvad_client
            .test_custom_api_access_method($access_method.clone().into())
            .await
            .expect("Failed to test custom API access method");

        assert!(
            successful,
            "Failed while testing access method - {:?}",
            $access_method
        );
    };
}

async fn test_shadowsocks(mut mullvad_client: MullvadProxyClient) -> Result<(), Error> {
    use mullvad_relay_selector::{RelaySelector, SelectorConfig};
    // Set up all the parameters needed to create a custom Shadowsocks access method.
    //
    // Since Mullvad's bridge servers host Shadowsocks relays, we can simply
    // select a bridge server to derive all the needed parameters.
    let relay_list = mullvad_client.get_relay_locations().await.unwrap();
    let relay_selector = RelaySelector::from_list(SelectorConfig::default(), relay_list);
    let access_method = relay_selector
        .get_bridge_forced()
        .expect("`test_shadowsocks` needs at least one shadowsocks relay to execute. Found none in relay list.");
    assert_access_method_works!(mullvad_client, access_method);
    Ok(())
}

async fn test_socks_remote(mut mullvad_client: MullvadProxyClient) -> Result<(), Error> {
    use crate::vm::network::{NON_TUN_GATEWAY, SOCKS5_PORT};
    use std::net::SocketAddr;
    use talpid_types::net::proxy::{CustomProxy, Socks5Remote};
    // Set up all the parameters needed to create a custom SOCKS5 access method.
    //
    // The remote SOCKS5 proxy is assumed to be running on the test manager. On
    // which port it listens to is defined as a constant in the `test-manager`
    // crate.
    let endpoint = SocketAddr::from((NON_TUN_GATEWAY, SOCKS5_PORT));
    let access_method = CustomProxy::from(Socks5Remote::new(endpoint));
    assert_access_method_works!(mullvad_client, access_method);
    Ok(())
}

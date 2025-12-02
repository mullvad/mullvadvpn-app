use super::{
    Error, TestContext, helpers,
    helpers::{connect_and_wait, send_guest_probes},
};

use anyhow::{Context, ensure};
use mullvad_management_interface::MullvadProxyClient;
use mullvad_types::settings::{DefaultDnsOptions, DnsOptions, Settings};
use std::net::SocketAddr;
use test_macro::test_function;
use test_rpc::ServiceClient;

/// Verify that traffic to private IPs is blocked when
/// "local network sharing" is disabled, but not blocked
/// when it is enabled.
/// It only checks whether outgoing UDP, TCP, and ICMP is
/// blocked for a single arbitrary private IP and port.
#[test_function]
pub async fn test_lan(
    _: TestContext,
    rpc: ServiceClient,
    mut mullvad_client: MullvadProxyClient,
) -> Result<(), Error> {
    // Take care not to use some bogus IP in the guest's subnet, lest we just send ARP requests
    // These will fail if there's no actual host present
    let lan_destination = "10.1.2.3:1234".parse().unwrap();

    // Disable LAN sharing
    //

    log::info!("LAN sharing: disabled");

    mullvad_client
        .set_allow_lan(false)
        .await
        .expect("failed to disable LAN sharing");

    // Connect
    //

    connect_and_wait(&mut mullvad_client).await?;

    // Ensure LAN is not reachable
    //

    log::info!("Test whether outgoing LAN traffic is blocked");

    let default_interface = rpc.get_default_interface().await?;
    let detected_probes =
        send_guest_probes(rpc.clone(), default_interface.clone(), lan_destination).await;
    assert!(
        detected_probes.none(),
        "observed unexpected outgoing LAN packets: {detected_probes:?}"
    );

    // Enable LAN sharing
    //

    log::info!("LAN sharing: enabled");

    mullvad_client
        .set_allow_lan(true)
        .await
        .expect("failed to enable LAN sharing");

    // Ensure LAN is reachable
    //

    log::info!("Test whether outgoing LAN traffic is blocked");

    let detected_probes = send_guest_probes(rpc.clone(), default_interface, lan_destination).await;
    assert!(
        detected_probes.all(),
        "did not observe all outgoing LAN packets: {detected_probes:?}"
    );

    Ok(())
}

/// Enable lockdown mode. This test succeeds if:
///
/// * Disconnected state: Outgoing traffic leaks (UDP/TCP/ICMP) cannot be produced.
/// * Disconnected state: Outgoing traffic to a single private IP can be produced, if and only if
///   LAN sharing is enabled.
/// * Connected state: Outgoing traffic leaks (UDP/TCP/ICMP) cannot be produced.
///
/// # Limitations
///
/// These tests are performed on one single public IP address
/// and one private IP address. They detect basic leaks but
/// do not guarantee close conformity with the security
/// document.
#[test_function]
pub async fn test_lockdown(
    _: TestContext,
    rpc: ServiceClient,
    mut mullvad_client: MullvadProxyClient,
) -> Result<(), Error> {
    // Take care not to use some bogus IP in the guest's subnet, lest we just send ARP requests
    // These will fail if there's no actual host present
    let lan_destination = "10.1.2.3:1234".parse().unwrap();
    let inet_destination: SocketAddr = "1.1.1.1:1337".parse().unwrap();

    // Disable LAN sharing
    //

    log::info!("LAN sharing: disabled");

    mullvad_client
        .set_allow_lan(false)
        .await
        .expect("failed to disable LAN sharing");

    // Enable lockdown mode
    //
    mullvad_client
        .set_lockdown_mode(true)
        .await
        .expect("failed to enable lockdown mode");

    // Ensure all destinations are unreachable
    //

    let default_interface = rpc.get_default_interface().await?;

    let detected_probes =
        send_guest_probes(rpc.clone(), default_interface.clone(), lan_destination).await;
    assert!(
        detected_probes.none(),
        "observed outgoing packets to LAN: {detected_probes:?}"
    );

    let detected_probes =
        send_guest_probes(rpc.clone(), default_interface.clone(), inet_destination).await;
    assert!(
        detected_probes.none(),
        "observed outgoing packets to internet: {detected_probes:?}"
    );

    // Enable LAN sharing
    //

    log::info!("LAN sharing: enabled");

    mullvad_client
        .set_allow_lan(true)
        .await
        .expect("failed to enable LAN sharing");

    // Ensure private IPs are reachable, but not others
    //

    let detected_probes =
        send_guest_probes(rpc.clone(), default_interface.clone(), lan_destination).await;
    assert!(
        detected_probes.all(),
        "did not observe some outgoing packets: {detected_probes:?}"
    );

    let detected_probes =
        send_guest_probes(rpc.clone(), default_interface.clone(), inet_destination).await;
    assert!(
        detected_probes.none(),
        "observed outgoing packets to internet: {detected_probes:?}"
    );

    // Connect
    //

    connect_and_wait(&mut mullvad_client).await?;

    // Leak test
    //

    assert!(
        helpers::using_mullvad_exit(&rpc).await,
        "expected Mullvad exit IP"
    );

    // Send traffic outside the tunnel to sanity check that the internet is *not* reachable via non-
    // tunnel interfaces.
    let detected_probes = send_guest_probes(rpc.clone(), default_interface, inet_destination).await;
    assert!(
        detected_probes.none(),
        "observed outgoing packets to internet: {detected_probes:?}"
    );

    Ok(())
}

/// Reset settings to their default values.
///
/// The account should still be logged in after resetting settings.
#[test_function]
pub async fn test_reset_settings(
    _: TestContext,
    _: ServiceClient,
    mut mullvad_client: MullvadProxyClient,
) -> anyhow::Result<()> {
    // Change some settings to non-default values.
    {
        let err = "Failed to generate random settings";
        mullvad_client.set_allow_lan(true).await.context(err)?;
        mullvad_client
            .set_dns_options(DnsOptions {
                default_options: DefaultDnsOptions::new().block_ads().block_malware(),
                ..Default::default()
            })
            .await
            .context(err)?;
        mullvad_client.set_lockdown_mode(true).await.context(err)?;
    }
    // Note: We do not double-check that the daemon reports these settings changes. We trust other
    // E2E tests to cover this.
    // Soft-reset settings by invoking the `reset-settings` rpc.
    mullvad_client
        .reset_settings()
        .await
        .context("Failed to reset settings")?;
    // Check that all changed settings have indeed been reset.
    let daemon = mullvad_client
        .get_settings()
        .await
        .context("Failed to get settings")?;
    let default = Settings::default();
    // Santiy-check that data that shouldn't have been touched have indeed been kept intact.
    // Are we still logged in?
    // Are the custom lists intact?
    ensure!(
        mullvad_client.get_account_history().await?.is_some(),
        "Logged out after reset settings RPC"
    );
    // Note: We can not compare `daemon` with `default` directly because Settings::default is
    // non-deterministic ..
    ensure!(
        daemon.lockdown_mode == default.lockdown_mode,
        "Lockdown mode was not reset"
    );
    ensure!(
        daemon.allow_lan == default.allow_lan,
        "Lockdown mode was not reset"
    );
    ensure!(
        daemon.tunnel_options == default.tunnel_options,
        "Lockdown mode was not reset"
    );
    // TODO: check other data as well, such as log files. Do this after having written such a test
    // first.
    Ok(())
}

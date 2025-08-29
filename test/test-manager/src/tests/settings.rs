use super::{
    Error, TestContext, helpers,
    helpers::{connect_and_wait, send_guest_probes},
};

use mullvad_management_interface::MullvadProxyClient;
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
        .set_block_when_disconnected(true)
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

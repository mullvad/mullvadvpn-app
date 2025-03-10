//! Test mitigation for MUL-02-002-WP2
//!
//! Fail to leak traffic to verify that mitigation "Firewall allows deanonymization by eavesdropper" works.
//!
//! # Vulnerability
//! 1. Connect to a relay on port 443. Record this relay's IP address (the new gateway of the
//!    client)
//! 2. Start listening for unencrypted traffic on the outbound network interface
//!    (Choose some human-readable, identifiable payload to look for in the outgoing TCP packets)
//! 3. Start a rogue program which performs a GET request\* containing the payload defined in step 2
//! 4. The network snooper started in step 2 should now be able to observe the network request
//!    containing the identifiable payload being sent unencrypted over the wire
//!
//! \* or something similar, as long as it generates some traffic containing UDP and/or TCP packets
//!    with the correct payload.

use anyhow::{bail, ensure};
use mullvad_management_interface::MullvadProxyClient;
use mullvad_relay_selector::query::builder::{RelayQueryBuilder, TransportProtocol};
use mullvad_types::states::TunnelState;
use test_macro::test_function;
use test_rpc::ServiceClient;

use crate::network_monitor::{start_packet_monitor, MonitorOptions, ParsedPacket};
use crate::tests::helpers::{
    connect_and_wait, constrain_to_relay, disconnect_and_wait, ConnChecker,
};
use crate::tests::TestContext;

#[test_function]
pub async fn test_mul_02_002(
    _: TestContext,
    rpc: ServiceClient,
    mut mullvad_client: MullvadProxyClient,
) -> anyhow::Result<()> {
    // Step 1 - Choose a relay
    constrain_to_relay(
        &mut mullvad_client,
        RelayQueryBuilder::openvpn()
            .transport_protocol(TransportProtocol::Tcp)
            .port(443)
            .build(),
    )
    .await?;

    // Step 1.5 - Temporarily connect to the relay to get the target endpoint
    let tunnel_state = connect_and_wait(&mut mullvad_client).await?;
    let TunnelState::Connected { endpoint, .. } = tunnel_state else {
        bail!("Expected tunnel state to be `Connected` - instead it was {tunnel_state:?}");
    };
    disconnect_and_wait(&mut mullvad_client).await?;
    let target_endpoint = endpoint.endpoint.address;

    // Step 2 - Start a network monitor snooping the outbound network interface for some
    // identifiable payload
    let unique_identifier = "Hello there!";
    let identify_rogue_packet = move |packet: &ParsedPacket| {
        packet
            .payload
            .windows(unique_identifier.len())
            .any(|window| window == unique_identifier.as_bytes())
    };
    let rogue_packet_monitor =
        start_packet_monitor(identify_rogue_packet, MonitorOptions::default()).await;

    // Step 3 - Start the rogue program which will try to leak the unique identifier payload
    // to the chosen relay endpoint
    let mut checker = ConnChecker::new(rpc.clone(), mullvad_client.clone(), target_endpoint);
    checker.payload(unique_identifier);
    let mut conn_artist = checker.spawn().await?;
    // Before proceeding, assert that the method of detecting identifiable packets work.
    conn_artist.check_connection().await?;
    let monitor_result = rogue_packet_monitor.into_result().await?;

    log::info!("Checking that the identifiable payload was detectable without encryption");
    ensure!(
        !monitor_result.packets.is_empty(),
        "Did not observe rogue packets! The method seems to be broken"
    );
    log::info!("The identifiable payload was detected! (that's good)");

    // Step 4 - Finally, connect to a tunnel and assert that no outgoing traffic contains the
    // payload in plain text.
    connect_and_wait(&mut mullvad_client).await?;
    let rogue_packet_monitor =
        start_packet_monitor(identify_rogue_packet, MonitorOptions::default()).await;
    conn_artist.check_connection().await?;
    let monitor_result = rogue_packet_monitor.into_result().await?;

    log::info!("Checking that the identifiable payload was not detected");
    ensure!(
        monitor_result.packets.is_empty(),
        "Observed rogue packets! The tunnel seems to be leaking traffic"
    );

    Ok(())
}

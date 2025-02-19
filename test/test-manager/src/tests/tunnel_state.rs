use super::{
    helpers::{
        self, connect_and_wait, send_guest_probes, unreachable_wireguard_tunnel,
        wait_for_tunnel_state,
    },
    ui, Error, TestContext,
};
use crate::{assert_tunnel_state, tests::helpers::ping_sized_with_timeout};

use mullvad_management_interface::MullvadProxyClient;
use mullvad_relay_selector::query::builder::RelayQueryBuilder;
use mullvad_types::{
    constraints::Constraint,
    relay_constraints::{
        GeographicLocationConstraint, LocationConstraint, RelayConstraints, RelaySettings,
    },
    states::TunnelState,
    CustomTunnelEndpoint,
};
use std::{net::SocketAddr, time::Duration};
use talpid_types::net::{Endpoint, TransportProtocol, TunnelEndpoint, TunnelType};
use test_macro::test_function;
use test_rpc::ServiceClient;

/// Set up nftables rules to drop packets larger than `max_packet_size` on the host.
#[cfg(target_os = "linux")]
async fn setup_nftables_drop_pings_rule(
    max_packet_size: u16,
) -> scopeguard::ScopeGuard<(), impl FnOnce(())> {
    fn log_ruleset() {
        let output = std::process::Command::new("nft")
            .args(["list", "ruleset"])
            .output()
            .unwrap();

        log::debug!(
            "Set nftables ruleset to:\n{}",
            String::from_utf8(output.stdout).unwrap()
        );

        let exit_status = output.status;
        assert_eq!(exit_status.code(), Some(0));
    }
    // Set nftables ruleset
    crate::vm::network::linux::run_nft(
        &(format!(
            "table inet DropPings {{
                chain postrouting {{
                    type filter hook postrouting priority 0; policy accept;
                    ip length > {max_packet_size} drop;
                }}
            }}"
        )),
    )
    .await
    .unwrap();
    log_ruleset();

    scopeguard::guard((), |()| {
        let mut cmd = std::process::Command::new("nft");
        cmd.args(["delete", "table", "inet", "DropPings"]);
        let output = cmd.output().unwrap();
        if !output.status.success() {
            panic!("{}", std::str::from_utf8(&output.stderr).unwrap());
        }
        log_ruleset();
    })
}

#[test_function(target_os = "linux", target_os = "windows")]
async fn test_mtu_detection(
    _: TestContext,
    rpc: ServiceClient,
    mut mullvad_client: MullvadProxyClient,
) -> Result<(), Error> {
    const MAX_PACKET_SIZE: u16 = 800;
    const MARGIN: u16 = 200;
    let large_ping_size: usize = (MAX_PACKET_SIZE + MARGIN).into();

    log::info!("Verify tunnel state: disconnected");
    assert_tunnel_state!(&mut mullvad_client, TunnelState::Disconnected { .. });

    // mullvad.net address
    let inet_destination = "45.83.223.209".parse().unwrap();

    log::info!("Setting up nftables firewall rules");
    #[cfg(target_os = "linux")]
    let _nft_guard = setup_nftables_drop_pings_rule(MAX_PACKET_SIZE).await;

    // Test that the firewall rule works
    log::info!("Sending large ping outside tunnel");
    ping_sized_with_timeout(&rpc, inet_destination, None, large_ping_size)
        .await
        .expect_err("Ping larger than the filter should time out");

    connect_and_wait(&mut mullvad_client).await.unwrap();
    let tunnel_iface = helpers::get_tunnel_interface(&mut mullvad_client)
        .await
        .expect("failed to find tunnel interface");

    log::info!("Waiting for MTU detection");
    for _ in 0..10 {
        let mtu = rpc.get_interface_mtu(tunnel_iface.clone()).await?;
        if mtu < MAX_PACKET_SIZE {
            println!(
                "Tunnel MTU after dropping packets larger than {MAX_PACKET_SIZE} bytes: {mtu}"
            );
            return Ok(());
        }
        tokio::time::sleep(Duration::from_secs(1)).await;
    }
    panic!("MTU detection test failed")
}

/// Verify that outgoing TCP, UDP, and ICMP packets can be observed
/// in the disconnected state. The purpose is mostly to rule prevent
/// false negatives in other tests.
/// This also ensures that the disconnected view is shown in the Electron app.
#[test_function]
pub async fn test_disconnected_state(
    _: TestContext,
    rpc: ServiceClient,
    mut mullvad_client: MullvadProxyClient,
) -> Result<(), Error> {
    let inet_destination = "1.3.3.7:1337".parse().unwrap();

    log::info!("Verify tunnel state: disconnected");
    assert_tunnel_state!(&mut mullvad_client, TunnelState::Disconnected { .. });

    // Test whether outgoing packets can be observed
    //

    log::info!("Sending packets to {inet_destination}");

    let non_tunnel_interface = rpc
        .get_default_interface()
        .await
        .expect("failed to obtain non-tun interface");

    let detected_probes =
        send_guest_probes(rpc.clone(), non_tunnel_interface, inet_destination).await;
    assert!(
        detected_probes.all(),
        "did not see (all) outgoing packets to destination: {detected_probes:?}",
    );

    // Test UI view
    //

    log::info!("UI: Test disconnected state");
    let ui_result = ui::run_test(&rpc, &["disconnected.spec"]).await.unwrap();
    assert!(ui_result.success());

    Ok(())
}

/// Try to produce leaks in the connecting state by forcing
/// the app into the connecting state and trying to leak,
/// failing if any the following outbound traffic is
/// detected:
///
/// * TCP on port 53 and one other port
/// * UDP on port 53 and one other port
/// * ICMP (by pinging)
///
/// # Limitations
///
/// These tests are performed on one single public IP address
/// and one private IP address. They detect basic leaks but
/// do not guarantee close conformity with the security
/// document.
#[test_function]
pub async fn test_connecting_state(
    _: TestContext,
    rpc: ServiceClient,
    mut mullvad_client: MullvadProxyClient,
) -> Result<(), Error> {
    let inet_destination = "1.1.1.1:1337".parse().unwrap();
    // Take care not to use some bogus IP in the guest's subnet, lest we just send ARP requests
    // These will fail if there's no actual host present
    let lan_destination = "10.1.2.3:1234".parse().unwrap();
    let inet_dns = "1.1.1.1:53".parse().unwrap();
    let lan_dns = "10.1.2.3:53".parse().unwrap();

    log::info!("Verify tunnel state: disconnected");
    assert_tunnel_state!(&mut mullvad_client, TunnelState::Disconnected { .. });

    set_custom_endpoint(
        &mut mullvad_client,
        CustomTunnelEndpoint {
            host: "1.3.3.7".to_owned(),
            config: mullvad_types::ConnectionConfig::Wireguard(unreachable_wireguard_tunnel()),
        },
    )
    .await
    .expect("failed to update relay settings");

    mullvad_client
        .connect_tunnel()
        .await
        .expect("failed to begin connecting");
    let new_state = wait_for_tunnel_state(mullvad_client.clone(), |state| {
        matches!(
            state,
            TunnelState::Connecting { .. } | TunnelState::Error(..)
        )
    })
    .await?;

    assert!(
        matches!(new_state, TunnelState::Connecting { .. }),
        "failed to enter connecting state: {:?}",
        new_state
    );

    // Leak test
    //

    let non_tunnel_interface = rpc
        .get_default_interface()
        .await
        .expect("failed to obtain non-tun interface");

    assert!(
        send_guest_probes(rpc.clone(), non_tunnel_interface.clone(), inet_destination)
            .await
            .none(),
        "observed unexpected outgoing packets (inet)"
    );
    assert!(
        send_guest_probes(rpc.clone(), non_tunnel_interface.clone(), lan_destination)
            .await
            .none(),
        "observed unexpected outgoing packets (lan)"
    );
    assert!(
        send_guest_probes(rpc.clone(), non_tunnel_interface.clone(), inet_dns)
            .await
            .none(),
        "observed unexpected outgoing packets (DNS, inet)"
    );
    assert!(
        send_guest_probes(rpc.clone(), non_tunnel_interface, lan_dns)
            .await
            .none(),
        "observed unexpected outgoing packets (DNS, lan)"
    );

    assert_tunnel_state!(&mut mullvad_client, TunnelState::Connecting { .. });

    Ok(())
}

/// Try to produce leaks in the error state. Refer to the
/// `test_connecting_state` documentation for details.
#[test_function]
pub async fn test_error_state(
    _: TestContext,
    rpc: ServiceClient,
    mut mullvad_client: MullvadProxyClient,
) -> Result<(), Error> {
    let inet_destination = "1.1.1.1:1337".parse().unwrap();
    // Take care not to use some bogus IP in the guest's subnet, lest we just send ARP requests
    // These will fail if there's no actual host present
    let lan_destination = "10.1.2.3:1234".parse().unwrap();
    let inet_dns = "1.1.1.1:53".parse().unwrap();
    let lan_dns = "10.1.2.3:53".parse().unwrap();

    log::info!("Verify tunnel state: disconnected");
    assert_tunnel_state!(&mut mullvad_client, TunnelState::Disconnected { .. });

    // Connect to non-existent location
    //

    log::info!("Enter error state");

    let relay_settings = RelaySettings::Normal(RelayConstraints {
        location: Constraint::Only(LocationConstraint::from(
            GeographicLocationConstraint::country("xx"),
        )),
        ..Default::default()
    });

    mullvad_client
        .set_allow_lan(false)
        .await
        .expect("failed to disable LAN sharing");

    set_relay_settings(&mut mullvad_client, relay_settings)
        .await
        .expect("failed to update relay settings");

    let _ = connect_and_wait(&mut mullvad_client).await;
    assert_tunnel_state!(&mut mullvad_client, TunnelState::Error { .. });

    // Leak test
    //

    let default_interface = rpc
        .get_default_interface()
        .await
        .expect("failed to obtain non-tun interface");

    assert!(
        send_guest_probes(rpc.clone(), default_interface.clone(), inet_destination)
            .await
            .none(),
        "observed unexpected outgoing packets (inet)"
    );
    assert!(
        send_guest_probes(rpc.clone(), default_interface.clone(), lan_destination)
            .await
            .none(),
        "observed unexpected outgoing packets (lan)"
    );
    assert!(
        send_guest_probes(rpc.clone(), default_interface.clone(), inet_dns)
            .await
            .none(),
        "observed unexpected outgoing packets (DNS, inet)"
    );
    assert!(
        send_guest_probes(rpc.clone(), default_interface, lan_dns)
            .await
            .none(),
        "observed unexpected outgoing packets (DNS, lan)"
    );

    Ok(())
}

/// Connect to a single relay and verify that:
/// * Traffic can be sent and received in the tunnel. This is done by pinging a single public IP
///   address and failing if there is no response.
/// * The correct relay is used.
/// * Leaks outside the tunnel are blocked. Refer to the `test_connecting_state` documentation for
///   details.
#[test_function]
pub async fn test_connected_state(
    _: TestContext,
    rpc: ServiceClient,
    mut mullvad_client: MullvadProxyClient,
) -> anyhow::Result<()> {
    let inet_destination = "1.1.1.1:1337".parse().unwrap();

    // Set relay to use
    log::info!("Select relay");
    let relay = helpers::constrain_to_relay(
        &mut mullvad_client,
        RelayQueryBuilder::new().wireguard().build(),
    )
    .await?;

    // Connect
    connect_and_wait(&mut mullvad_client).await?;

    // Verify that endpoint was selected
    match mullvad_client.get_tunnel_state().await? {
        TunnelState::Connected {
            endpoint:
                TunnelEndpoint {
                    endpoint:
                        Endpoint {
                            address: SocketAddr::V4(addr),
                            protocol: TransportProtocol::Udp,
                        },
                    // TODO: Consider the type of `relay` / `relay_filter` instead
                    tunnel_type: TunnelType::Wireguard,
                    quantum_resistant: _,
                    proxy: None,
                    obfuscation: _,
                    entry_endpoint: None,
                    tunnel_interface: _,
                    daita: _,
                },
            ..
        } => {
            assert_eq!(*addr.ip(), relay.ipv4_addr_in);
        }
        actual => panic!("unexpected tunnel state: {:?}", actual),
    }

    // Ping outside of tunnel while connected
    //

    log::info!("Test whether outgoing non-tunnel traffic is blocked");

    let nontun_iface = rpc
        .get_default_interface()
        .await
        .expect("failed to find non-tun interface");

    let detected_probes = send_guest_probes(rpc.clone(), nontun_iface, inet_destination).await;
    assert!(
        detected_probes.none(),
        "observed unexpected outgoing packets: {detected_probes:?}"
    );

    assert!(
        helpers::using_mullvad_exit(&rpc).await,
        "expected Mullvad exit IP"
    );

    Ok(())
}

use super::{
    helpers::{
        self, connect_and_wait, send_guest_probes, set_relay_settings,
        unreachable_wireguard_tunnel, wait_for_tunnel_state,
    },
    ui, Error, TestContext,
};
use crate::assert_tunnel_state;
// use crate::tests::helpers::{disconnect_and_wait, ping_sized_with_timeout};
use crate::vm::network::DUMMY_LAN_INTERFACE_IP;

use mullvad_management_interface::MullvadProxyClient;
use mullvad_types::{
    relay_constraints::{
        Constraint, GeographicLocationConstraint, LocationConstraint, RelayConstraints,
        RelaySettings,
    },
    relay_list::{Relay, RelayEndpointData},
    states::TunnelState,
    CustomTunnelEndpoint,
};
use std::net::{IpAddr, SocketAddr};
use talpid_types::net::{Endpoint, TransportProtocol, TunnelEndpoint, TunnelType};
use test_macro::test_function;
use test_rpc::ServiceClient;

#[test_function]
pub async fn test_nft(
    _: TestContext,
    _rpc: ServiceClient,
    _mullvad_client: MullvadProxyClient,
) -> Result<(), Error> {
    // Plan:
    // Assert that we are in the disconnected state
    // Send ping or maximum size (1380?) to some external destination or the test manager
    // Connect
    // Send ping to the same destination and verify that it still works.
    // Disconnect
    // Apply firewall rule that drops pings data larger than, say 1000 bytes
    // Verify that (only) pings over 1000 - IPV4_HEADER_SIZE - ICMP_HEADER_SIZE are dropped
    // Connect
    // Verify that pings over 1000 - IPV4_HEADER_SIZE - ICMP_HEADER_SIZE are not dropped
    // Potentially disable the mtu detection by manually setting the mtu, then verify that it
    // stopped working

    // let large_ping_size = 1400;
    // let small_ping_size = 500;
    // let max_packet_size = 1200;
    //
    // log::info!("Verify tunnel state: disconnected");
    // assert_tunnel_state!(&mut mullvad_client, TunnelState::Disconnected { .. });
    //
    // let non_tunnel_interface = rpc
    //     .get_default_interface()
    //     .await
    //     .expect("failed to obtain non-tun interface");
    // let inet_destination = "1.3.3.7:1337".parse().unwrap();
    // ping_sized_with_timeout(
    //     &rpc,
    //     inet_destination,
    //     Some(non_tunnel_interface.clone()),
    //     large_ping_size,
    // )
    // .await?;
    // connect_and_wait(&mut mullvad_client).await?;
    // ping_sized_with_timeout(
    //     &rpc,
    //     inet_destination,
    //     Some(non_tunnel_interface.clone()),
    //     large_ping_size,
    // )
    // .await?;
    // disconnect_and_wait(&mut mullvad_client).await?;

    const RULES: &str = "table inet DropPings {
        chain output {
            type filter hook output priority 0; policy accept;
            ip length > 1000 drop;
          }
    }";
    use std::process::Stdio;
    use tokio::{io::AsyncWriteExt, process::Command};
    {
        let mut handle = Command::new("nft")
            .args(["-f", "-"])
            .stdin(Stdio::piped())
            .stdout(Stdio::null())
            .spawn()
            .unwrap();
        handle
            .stdin
            .take()
            .expect("Command should have a handle to stdin")
            .write_all(RULES.as_bytes())
            .await
            .unwrap();
        let exit_status = handle.wait().await.unwrap();
        assert_eq!(exit_status.code(), Some(0));
    }
    {
        let output = Command::new("nft")
            .args(["list", "ruleset"])
            .output()
            .await
            .unwrap();

        println!("{}", String::from_utf8(output.stdout).unwrap());

        let exit_status = output.status;
        assert_eq!(exit_status.code(), Some(0));
    }
    Ok(())
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
        send_guest_probes(rpc.clone(), non_tunnel_interface, inet_destination).await?;
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
    let lan_destination: SocketAddr = SocketAddr::new(IpAddr::V4(DUMMY_LAN_INTERFACE_IP), 1337);
    let inet_dns = "1.1.1.1:53".parse().unwrap();
    let lan_dns: SocketAddr = SocketAddr::new(IpAddr::V4(DUMMY_LAN_INTERFACE_IP), 53);

    log::info!("Verify tunnel state: disconnected");
    assert_tunnel_state!(&mut mullvad_client, TunnelState::Disconnected { .. });

    let relay_settings = RelaySettings::CustomTunnelEndpoint(CustomTunnelEndpoint {
        host: "1.3.3.7".to_owned(),
        config: mullvad_types::ConnectionConfig::Wireguard(unreachable_wireguard_tunnel()),
    });

    set_relay_settings(&mut mullvad_client, relay_settings)
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
            .await?
            .none(),
        "observed unexpected outgoing packets (inet)"
    );
    assert!(
        send_guest_probes(rpc.clone(), non_tunnel_interface.clone(), lan_destination)
            .await?
            .none(),
        "observed unexpected outgoing packets (lan)"
    );
    assert!(
        send_guest_probes(rpc.clone(), non_tunnel_interface.clone(), inet_dns)
            .await?
            .none(),
        "observed unexpected outgoing packets (DNS, inet)"
    );
    assert!(
        send_guest_probes(rpc.clone(), non_tunnel_interface, lan_dns)
            .await?
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
    let lan_destination: SocketAddr = SocketAddr::new(IpAddr::V4(DUMMY_LAN_INTERFACE_IP), 1337);
    let inet_dns = "1.1.1.1:53".parse().unwrap();
    let lan_dns: SocketAddr = SocketAddr::new(IpAddr::V4(DUMMY_LAN_INTERFACE_IP), 53);

    log::info!("Verify tunnel state: disconnected");
    assert_tunnel_state!(&mut mullvad_client, TunnelState::Disconnected { .. });

    // Connect to non-existent location
    //

    log::info!("Enter error state");

    let relay_settings = RelaySettings::Normal(RelayConstraints {
        location: Constraint::Only(LocationConstraint::Location(
            GeographicLocationConstraint::Country("xx".to_string()),
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
            .await?
            .none(),
        "observed unexpected outgoing packets (inet)"
    );
    assert!(
        send_guest_probes(rpc.clone(), default_interface.clone(), lan_destination)
            .await?
            .none(),
        "observed unexpected outgoing packets (lan)"
    );
    assert!(
        send_guest_probes(rpc.clone(), default_interface.clone(), inet_dns)
            .await?
            .none(),
        "observed unexpected outgoing packets (DNS, inet)"
    );
    assert!(
        send_guest_probes(rpc.clone(), default_interface, lan_dns)
            .await?
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
) -> Result<(), Error> {
    let inet_destination = "1.1.1.1:1337".parse().unwrap();

    // Set relay to use
    //

    log::info!("Select relay");

    let relay_filter = |relay: &Relay| {
        relay.active && matches!(relay.endpoint_data, RelayEndpointData::Wireguard(_))
    };

    let relay = helpers::filter_relays(&mut mullvad_client, relay_filter)
        .await?
        .pop()
        .unwrap();

    let relay_settings = RelaySettings::Normal(RelayConstraints {
        location: helpers::into_constraint(&relay),
        ..Default::default()
    });

    set_relay_settings(&mut mullvad_client, relay_settings)
        .await
        .expect("failed to update relay settings");

    // Connect
    //

    connect_and_wait(&mut mullvad_client).await?;

    // Verify that endpoint was selected
    //

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
                    obfuscation: None,
                    entry_endpoint: None,
                    tunnel_interface: _,
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

    let detected_probes = send_guest_probes(rpc.clone(), nontun_iface, inet_destination).await?;
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

/// Verify that the app defaults to the connecting state if it is started with a
/// corrupt state cache.
#[test_function]
pub async fn test_connecting_state_when_corrupted_state_cache(
    _: TestContext,
    rpc: ServiceClient,
    mullvad_client: MullvadProxyClient,
) -> Result<(), Error> {
    // The test should start in a disconnected state. Normally this would be
    // preserved when restarting the app, i.e. the user would still be
    // disconnected after a successfull restart. However, as we will
    // intentionally corrupt the state target cache the user should end up in
    // the connecting/connected state, *not in the disconnected state*, upon
    // restart.

    // Stopping the app should write to the state target cache.
    log::info!("Stopping the app");
    rpc.stop_mullvad_daemon().await?;

    // Intentionally corrupt the state cache. Note that we can not simply remove
    // the cache, as this will put the app in the default target state which is
    // 'unsecured'.
    log::info!("Figuring out where state cache resides on test runner ..");
    let state_cache = rpc
        .find_mullvad_app_cache_dir()
        .await?
        .join("target-start-state.json");
    log::info!(
        "Intentionally writing garbage to the state cache {file}",
        file = state_cache.display()
    );
    rpc.write_file(state_cache, "cookie was here".into())
        .await?;

    // Start the app & make sure that we start in the 'connecting state'. The
    // side-effect of this is that no network traffic is allowed to leak.
    log::info!("Starting the app back up again");
    rpc.start_mullvad_daemon().await?;
    wait_for_tunnel_state(mullvad_client.clone(), |state| !state.is_disconnected())
        .await
        .map_err(|err| {
            log::error!("App did not start in an expected state. \
                        App is not in either `Connecting` or `Connected` state after starting with corrupt state cache! \
                        There is a possibility of leaks during app startup ");
            err
        })?;
    log::info!("App successfully recovered from a corrupt tunnel state cache.");

    Ok(())
}

use super::{
    Error, TestContext,
    config::TEST_CONFIG,
    helpers::{self, apply_settings_from_relay_query, connect_and_wait, disconnect_and_wait},
};
use crate::{
    network_monitor::{MonitorOptions, start_packet_monitor},
    tests::helpers::{ConnChecker, geoip_lookup_with_retries, login_with_retries},
};

use anyhow::{Context, ensure};
use duplicate::duplicate_item;
use mullvad_management_interface::MullvadProxyClient;
use mullvad_relay_selector::query::builder::RelayQueryBuilder;
use mullvad_types::wireguard;
use std::{
    net::{IpAddr, Ipv4Addr, Ipv6Addr},
    str::FromStr,
};
use talpid_types::net::IpVersion;
use test_macro::test_function;
use test_rpc::{ServiceClient, meta::Os, mullvad_daemon::ServiceStatus};

use pnet_packet::ip::IpNextHeaderProtocols;

/// Set up a WireGuard tunnel.
/// This test fails if a working tunnel cannot be set up.
/// WARNING: This test will fail if host has something bound to port 53 such as a connected Mullvad
#[duplicate_item(
      VX     test_wireguard_tunnel_ipvx;
    [ V4 ] [ test_wireguard_tunnel_ipv4 ];
    [ V6 ] [ test_wireguard_tunnel_ipv6 ];
)]
#[test_function]
pub async fn test_wireguard_tunnel_ipvx(
    _: TestContext,
    rpc: ServiceClient,
    mut mullvad_client: MullvadProxyClient,
) -> Result<(), Error> {
    // TODO: observe UDP traffic on the expected destination/port (only)

    let ip_version = IpVersion::VX;
    const PORTS: [(u16, bool); 3] = [(53, true), (51820, true), (1, false)];

    for (port, should_succeed) in PORTS {
        log::info!("Connect to WireGuard endpoint on port {port}");

        let query = RelayQueryBuilder::wireguard()
            .port(port)
            .ip_version(ip_version)
            .build();

        apply_settings_from_relay_query(&mut mullvad_client, query)
            .await
            .unwrap();

        let connection_result = connect_and_wait(&mut mullvad_client).await;

        if should_succeed {
            let Ok(connection_result) = &connection_result else {
                panic!("connection must succeed for port {port}: {connection_result:?}");
            };

            let endpoint = connection_result.endpoint().expect("must have endpoint");
            let endpoint = endpoint.entry_endpoint.unwrap_or(endpoint.endpoint);
            assert!(matches!(endpoint.address.ip(), IpAddr::VX(..)));

            assert!(
                helpers::using_mullvad_exit(&rpc).await,
                "expected Mullvad exit IP"
            );
        } else {
            assert!(
                connection_result.is_err(),
                "connection must fail for port {port}: {connection_result:?}",
            );
        }

        disconnect_and_wait(&mut mullvad_client).await?;
    }

    Ok(())
}

/// Set up a WireGuard tunnel and check whether in-tunnel IPv6 works.
/// WARNING: This test will fail if host has something bound to port 53 such as a connected Mullvad
#[duplicate_item(
      VX     test_wireguard_ipv6_in_ipvx;
    [ V4 ] [ test_wireguard_ipv6_in_ipv4 ];
    [ V6 ] [ test_wireguard_ipv6_in_ipv6 ];
)]
#[test_function]
pub async fn test_wireguard_ipv6_in_ipvx(
    _: TestContext,
    rpc: ServiceClient,
    mut mullvad_client: MullvadProxyClient,
) -> Result<(), Error> {
    let ip_version = IpVersion::VX;

    let mut conn_checker_v4 = ConnChecker::new(
        rpc.clone(),
        mullvad_client.clone(),
        (Ipv4Addr::new(1, 1, 1, 1), 53),
    );

    let mut conn_checker_v6 = ConnChecker::new(
        rpc.clone(),
        mullvad_client.clone(),
        (Ipv6Addr::from_str("2606:4700:4700::1111").unwrap(), 53),
    );

    let mut conn_checker_v4 = conn_checker_v4.spawn().await?;
    let mut conn_checker_v6 = conn_checker_v6.spawn().await?;

    conn_checker_v4.assert_insecure().await?;
    conn_checker_v6.assert_insecure().await?;

    log::info!("Connect to WireGuard endpoint");

    let query = RelayQueryBuilder::wireguard()
        .ip_version(ip_version)
        .build();
    apply_settings_from_relay_query(&mut mullvad_client, query)
        .await
        .unwrap();

    // Test with in-tunnel IPv6 enabled
    mullvad_client.set_enable_ipv6(true).await?;
    let connection_result = connect_and_wait(&mut mullvad_client).await;
    assert!(connection_result.is_ok());
    conn_checker_v4.assert_secure().await?;
    conn_checker_v6.assert_secure().await?;

    // Test with in-tunnel IPv6 disabled
    mullvad_client.set_enable_ipv6(false).await?;
    let connection_result = connect_and_wait(&mut mullvad_client).await;
    assert!(connection_result.is_ok());
    conn_checker_v4.assert_secure().await?;
    conn_checker_v6.assert_blocked().await?; // ipv6 mustnt leak

    disconnect_and_wait(&mut mullvad_client).await?;

    Ok(())
}

/// Use udp2tcp obfuscation. This test connects to a WireGuard relay over TCP. It fails if no
/// outgoing TCP traffic to the relay is observed on the expected port.
#[test_function]
pub async fn test_udp2tcp_tunnel(
    _: TestContext,
    rpc: ServiceClient,
    mut mullvad_client: MullvadProxyClient,
) -> Result<(), Error> {
    let query = RelayQueryBuilder::wireguard().udp2tcp().build();

    apply_settings_from_relay_query(&mut mullvad_client, query).await?;

    log::info!("Connect to WireGuard via tcp2udp endpoint");

    connect_and_wait(&mut mullvad_client).await?;

    let endpoint = match mullvad_client.get_tunnel_state().await? {
        mullvad_types::states::TunnelState::Connected { endpoint, .. } => endpoint.endpoint,
        _ => panic!("unexpected tunnel state"),
    };

    // Set up packet monitor
    //

    let monitor = start_packet_monitor(
        move |packet| {
            packet.destination.ip() == endpoint.address.ip()
                && packet.protocol == IpNextHeaderProtocols::Tcp
        },
        MonitorOptions::default(),
    )
    .await;

    // Verify that we can reach stuff
    //

    assert!(
        helpers::using_mullvad_exit(&rpc).await,
        "expected Mullvad exit IP"
    );

    let monitor_result = monitor.into_result().await.unwrap();
    assert!(
        !monitor_result.packets.is_empty(),
        "detected no tcp traffic",
    );

    Ok(())
}

/// Use Shadowsocks obfuscation. This tests whether the daemon can establish a Shadowsocks tunnel.
/// Note that this doesn't verify that Shadowsocks is in fact being used.
#[test_function]
pub async fn test_wireguard_over_shadowsocks(
    _: TestContext,
    rpc: ServiceClient,
    mut mullvad_client: MullvadProxyClient,
) -> anyhow::Result<()> {
    let query = RelayQueryBuilder::wireguard().shadowsocks().build();

    apply_settings_from_relay_query(&mut mullvad_client, query).await?;

    log::info!("Connect to WireGuard via shadowsocks endpoint");

    connect_and_wait(&mut mullvad_client).await?;

    // Verify that we have a Mullvad exit IP
    //

    assert!(
        helpers::using_mullvad_exit(&rpc).await,
        "expected Mullvad exit IP"
    );

    Ok(())
}

/// Use QUIC obfuscation. This tests whether the daemon can establish a QUIC connection.
/// Note that this doesn't verify that the outgoing traffic looks like http traffic (even though it
/// doesn't sound too difficult to do?).
#[duplicate_item(
      VX     test_wireguard_over_quic_ipvx;
    [ V4 ] [ test_wireguard_over_quic_ipv4 ];
    [ V6 ] [ test_wireguard_over_quic_ipv6 ];
)]
#[test_function]
pub async fn test_wireguard_over_quic_ipvx(
    _: TestContext,
    rpc: ServiceClient,
    mut mullvad_client: MullvadProxyClient,
) -> anyhow::Result<()> {
    let ip_version = IpVersion::VX;

    log::info!("Enable QUIC as obfuscation method");
    let query = RelayQueryBuilder::wireguard()
        .ip_version(ip_version)
        .quic()
        .build();
    apply_settings_from_relay_query(&mut mullvad_client, query).await?;

    log::info!("Connect to WireGuard via QUIC endpoint");
    connect_and_wait(&mut mullvad_client).await?;

    // Verify that the device has a Mullvad exit IP
    let conncheck = geoip_lookup_with_retries(&rpc).await;
    let mullvad_exit_ip = conncheck
        .as_ref()
        .is_ok_and(|am_i_mullvad| am_i_mullvad.mullvad_exit_ip);
    ensure!(
        mullvad_exit_ip,
        "Device is either blocked ❌ or leaking 💦 - {:?}",
        conncheck,
    );

    Ok(())
}

/// Use LWO obfuscation. This tests whether the daemon can connect using LWO.
/// Note that this doesn't verify that the outgoing traffic does not look like WG
#[duplicate_item(
      VX     test_wireguard_over_lwo_ipvx;
    [ V4 ] [ test_wireguard_over_lwo_ipv4 ];
    [ V6 ] [ test_wireguard_over_lwo_ipv6 ];
)]
#[test_function]
pub async fn test_wireguard_over_lwo_ipvx(
    _: TestContext,
    rpc: ServiceClient,
    mut mullvad_client: MullvadProxyClient,
) -> anyhow::Result<()> {
    let ip_version = IpVersion::VX;

    log::info!("Enable LWO as obfuscation method");
    let query = RelayQueryBuilder::wireguard()
        .ip_version(ip_version)
        .lwo()
        .build();
    apply_settings_from_relay_query(&mut mullvad_client, query).await?;

    log::info!("Connect to WireGuard via LWO endpoint");
    connect_and_wait(&mut mullvad_client).await?;

    // Verify that the device has a Mullvad exit IP
    let conncheck = geoip_lookup_with_retries(&rpc).await;
    let mullvad_exit_ip = conncheck
        .as_ref()
        .is_ok_and(|am_i_mullvad| am_i_mullvad.mullvad_exit_ip);
    ensure!(
        mullvad_exit_ip,
        "Device is either blocked ❌ or leaking 💦 - {:?}",
        conncheck,
    );

    Ok(())
}

/// Test whether WireGuard multihop works. This fails if:
/// * No outgoing traffic to the entry relay is observed from the SUT.
/// * The conncheck reports an unexpected exit relay.
#[test_function]
pub async fn test_multihop(
    _: TestContext,
    rpc: ServiceClient,
    mut mullvad_client: MullvadProxyClient,
) -> Result<(), Error> {
    let query = RelayQueryBuilder::wireguard().multihop().build();

    apply_settings_from_relay_query(&mut mullvad_client, query).await?;

    // Connect
    //

    log::info!("Connect using WG multihop");

    connect_and_wait(&mut mullvad_client).await?;

    let (entry, exit) = match mullvad_client.get_tunnel_state().await? {
        mullvad_types::states::TunnelState::Connected { endpoint, .. } => {
            (endpoint.entry_endpoint.unwrap(), endpoint.endpoint)
        }
        actual => {
            panic!("unexpected tunnel state. Expected `TunnelState::Connected` but got {actual:?}")
        }
    };

    log::info!(
        "Selected entry {entry_addr} & exit relay {exit_addr}",
        entry_addr = entry.address,
        exit_addr = exit.address
    );

    // Record outgoing packets to the entry relay
    //

    let monitor = start_packet_monitor(
        move |packet| packet.destination.ip() == entry.address.ip(),
        MonitorOptions::default(),
    )
    .await;

    // Verify exit IP
    //

    assert!(
        helpers::using_mullvad_exit(&rpc).await,
        "expected Mullvad exit IP"
    );

    // Verify entry IP
    //

    log::info!("Verifying entry server");

    let monitor_result = monitor.into_result().await.unwrap();
    assert!(!monitor_result.packets.is_empty(), "no matching packets",);

    Ok(())
}

/// Test whether the daemon automatically connects on reboot when using
/// WireGuard.
///
/// # Limitations
///
/// This test does not guarantee that nothing leaks during boot or shutdown.
#[test_function]
pub async fn test_wireguard_autoconnect(
    _: TestContext,
    mut rpc: ServiceClient,
    mut mullvad_client: MullvadProxyClient,
) -> Result<(), Error> {
    mullvad_client
        .set_auto_connect(true)
        .await
        .expect("failed to enable auto-connect");

    helpers::reboot(&mut rpc).await?;
    rpc.mullvad_daemon_wait_for_state(|state| state == ServiceStatus::Running)
        .await?;

    log::info!("Waiting for daemon to connect");

    helpers::wait_for_tunnel_state(mullvad_client, |state| {
        matches!(state, mullvad_types::states::TunnelState::Connected { .. })
    })
    .await?;

    Ok(())
}

/// Test whether quantum-resistant tunnels can be set up.
///
/// # Limitations
///
/// This only checks whether we have a working tunnel and a PSK. It does not determine whether the
/// exchange part is correct.
///
/// We only check whether there is a PSK on Linux.
#[test_function]
pub async fn test_quantum_resistant_tunnel(
    _: TestContext,
    rpc: ServiceClient,
    mut mullvad_client: MullvadProxyClient,
) -> anyhow::Result<()> {
    mullvad_client
        .set_quantum_resistant_tunnel(wireguard::QuantumResistantState::Off)
        .await
        .expect("Failed to disable PQ tunnels");

    // PQ disabled: Find no "preshared key"
    //

    connect_and_wait(&mut mullvad_client).await?;
    check_tunnel_psk(&rpc, &mullvad_client, false).await;

    let query = RelayQueryBuilder::wireguard().build();

    apply_settings_from_relay_query(&mut mullvad_client, query).await?;

    mullvad_client
        .set_quantum_resistant_tunnel(wireguard::QuantumResistantState::On)
        .await
        .expect("Failed to enable PQ tunnels");

    // PQ enabled: Find "preshared key"
    //

    connect_and_wait(&mut mullvad_client).await?;
    check_tunnel_psk(&rpc, &mullvad_client, true).await;

    assert!(
        helpers::using_mullvad_exit(&rpc).await,
        "expected Mullvad exit IP"
    );

    Ok(())
}

async fn check_tunnel_psk(
    rpc: &ServiceClient,
    mullvad_client: &MullvadProxyClient,
    should_have_psk: bool,
) {
    match TEST_CONFIG.os {
        Os::Linux => {
            let name = helpers::get_tunnel_interface(&mut mullvad_client.clone())
                .await
                .expect("failed to get tun name");
            let output = rpc
                .exec("wg", vec!["show", &name].into_iter())
                .await
                .expect("failed to run wg");
            let parsed_output = std::str::from_utf8(&output.stdout).expect("non-utf8 output");
            assert!(
                parsed_output.contains("preshared key: ") == should_have_psk,
                "expected to NOT find preshared key"
            );
        }
        os => {
            log::warn!("Not checking if there is a PSK on {os}");
        }
    }
}

/// Test whether a PQ tunnel can be set up with multihop and UDP-over-TCP enabled.
///
/// # Limitations
///
/// This is not testing any of the individual components, just whether the daemon can connect when
/// all of these features are combined.
#[test_function]
pub async fn test_quantum_resistant_multihop_udp2tcp_tunnel(
    _: TestContext,
    rpc: ServiceClient,
    mut mullvad_client: MullvadProxyClient,
) -> Result<(), Error> {
    mullvad_client
        .set_quantum_resistant_tunnel(wireguard::QuantumResistantState::On)
        .await
        .expect("Failed to enable PQ tunnels");

    let query = RelayQueryBuilder::wireguard().multihop().udp2tcp().build();

    apply_settings_from_relay_query(&mut mullvad_client, query).await?;

    connect_and_wait(&mut mullvad_client).await?;

    assert!(
        helpers::using_mullvad_exit(&rpc).await,
        "expected Mullvad exit IP"
    );

    Ok(())
}

/// Test Shadowsocks, PQ, and WireGuard combined.
///
/// # Limitations
///
/// This is not testing any of the individual components, just whether the daemon can connect when
/// all of these features are combined.
#[duplicate_item(
      VX     test_quantum_resistant_multihop_shadowsocks_tunnel_ipvx;
    [ V4 ] [ test_quantum_resistant_multihop_shadowsocks_tunnel_ipv4 ];
    [ V6 ] [ test_quantum_resistant_multihop_shadowsocks_tunnel_ipv6 ];
)]
#[test_function]
pub async fn test_quantum_resistant_multihop_shadowsocks_tunnel_ipvx(
    _: TestContext,
    rpc: ServiceClient,
    mut mullvad_client: MullvadProxyClient,
) -> anyhow::Result<()> {
    let ip_version = IpVersion::VX;

    mullvad_client
        .set_quantum_resistant_tunnel(wireguard::QuantumResistantState::On)
        .await
        .context("Failed to enable PQ tunnels")?;

    let query = RelayQueryBuilder::wireguard()
        .ip_version(ip_version)
        .multihop()
        .shadowsocks()
        .build();

    apply_settings_from_relay_query(&mut mullvad_client, query).await?;

    connect_and_wait(&mut mullvad_client).await?;

    assert!(
        helpers::using_mullvad_exit(&rpc).await,
        "Expected Mullvad exit IP"
    );

    Ok(())
}

/// Test QUIC, PQ, and WireGuard combined.
///
/// # Limitations
///
/// This is not testing any of the individual components, just whether the daemon can connect when
/// all of these features are combined.
#[duplicate_item(
      VX     test_quantum_resistant_multihop_quic_tunnel_ipvx;
    [ V4 ] [ test_quantum_resistant_multihop_quic_tunnel_ipv4 ];
    [ V6 ] [ test_quantum_resistant_multihop_quic_tunnel_ipv6 ];
)]
#[test_function]
pub async fn test_quantum_resistant_multihop_quic_tunnel_ipvx(
    _: TestContext,
    rpc: ServiceClient,
    mut mullvad_client: MullvadProxyClient,
) -> anyhow::Result<()> {
    let ip_version = IpVersion::VX;

    mullvad_client
        // TODO: Why is this needed, exactly?
        .set_quantum_resistant_tunnel(wireguard::QuantumResistantState::On)
        .await
        .context("Failed to enable PQ tunnels")?;

    let query = RelayQueryBuilder::wireguard()
        .ip_version(ip_version)
        .quantum_resistant()
        .multihop()
        .quic()
        .build();

    apply_settings_from_relay_query(&mut mullvad_client, query).await?;

    connect_and_wait(&mut mullvad_client).await?;

    assert!(
        helpers::using_mullvad_exit(&rpc).await,
        "Expected Mullvad exit IP"
    );

    Ok(())
}

/// Verify that the app can connect to a VPN server and get working internet when the API is down.
/// As long as the user has managed to log in to the app, establishing a tunnel should work even if
/// the API is down (This includes actually being down, not just censored).
///
/// The test procedure is as follows:
///     1. The app is logged in
///     2. The app is killed
///     3. The API is "removed" (override API IP/host to something bogus)
///     4. The app is restarted
///     5. Verify that it starts as intended and a tunnel can be established
#[test_function]
pub async fn test_establish_tunnel_without_api(
    ctx: TestContext,
    rpc: ServiceClient,
    mut mullvad_client: MullvadProxyClient,
) -> anyhow::Result<()> {
    // 1
    login_with_retries(&mut mullvad_client).await?;
    // 2
    rpc.stop_mullvad_daemon().await?;
    // 3
    let borked_env = [("MULLVAD_API_ADDR", "1.3.3.7:421")];
    // 4
    log::debug!("Restarting the daemon with the following overrides: {borked_env:?}");
    let mut mullvad_client =
        helpers::restart_daemon_with(&rpc, &ctx, mullvad_client, borked_env).await?;
    // 5
    connect_and_wait(&mut mullvad_client).await?;
    // Profit
    Ok(())
}

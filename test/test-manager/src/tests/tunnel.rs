use super::helpers::{
    self, connect_and_wait, disconnect_and_wait, set_bridge_settings, set_relay_settings,
};
use super::{config::TEST_CONFIG, Error, TestContext};
use crate::network_monitor::{start_packet_monitor, MonitorOptions};
use crate::tests::helpers::{login_with_retries, ConnChecker};

use mullvad_management_interface::MullvadProxyClient;
use mullvad_relay_selector::query::builder::RelayQueryBuilder;
use mullvad_types::{
    constraints::Constraint,
    relay_constraints::{
        self, BridgeConstraints, BridgeSettings, BridgeType, OpenVpnConstraints, RelayConstraints,
        RelaySettings, SelectedObfuscation, TransportPort, Udp2TcpObfuscationSettings,
        WireguardConstraints,
    },
    wireguard,
};
use std::net::SocketAddr;
use talpid_types::net::{
    proxy::{CustomProxy, Socks5Local, Socks5Remote},
    TransportProtocol, TunnelType,
};
use test_macro::test_function;
use test_rpc::meta::Os;
use test_rpc::mullvad_daemon::ServiceStatus;
use test_rpc::ServiceClient;

use pnet_packet::ip::IpNextHeaderProtocols;

/// Set up an OpenVPN tunnel, UDP as well as TCP.
/// This test fails if a working tunnel cannot be set up.
#[test_function]
pub async fn test_openvpn_tunnel(
    _: TestContext,
    rpc: ServiceClient,
    mut mullvad_client: MullvadProxyClient,
) -> Result<(), Error> {
    // TODO: observe traffic on the expected destination/port (only)

    const CONSTRAINTS: [(&str, Constraint<TransportPort>); 3] = [
        ("any", Constraint::Any),
        (
            "UDP",
            Constraint::Only(TransportPort {
                protocol: TransportProtocol::Udp,
                port: Constraint::Any,
            }),
        ),
        (
            "TCP",
            Constraint::Only(TransportPort {
                protocol: TransportProtocol::Tcp,
                port: Constraint::Any,
            }),
        ),
    ];

    for (protocol, constraint) in CONSTRAINTS {
        log::info!("Connect to {protocol} OpenVPN endpoint");

        let relay_settings = RelaySettings::Normal(RelayConstraints {
            tunnel_protocol: Constraint::Only(TunnelType::OpenVpn),
            openvpn_constraints: OpenVpnConstraints { port: constraint },
            ..Default::default()
        });

        set_relay_settings(&mut mullvad_client, relay_settings)
            .await
            .expect("failed to update relay settings");

        connect_and_wait(&mut mullvad_client).await?;

        assert!(
            helpers::using_mullvad_exit(&rpc).await,
            "expected Mullvad exit IP"
        );

        disconnect_and_wait(&mut mullvad_client).await?;
    }

    Ok(())
}

/// Set up a WireGuard tunnel.
/// This test fails if a working tunnel cannot be set up.
/// WARNING: This test will fail if host has something bound to port 53 such as a connected Mullvad
#[test_function]
pub async fn test_wireguard_tunnel(
    _: TestContext,
    rpc: ServiceClient,
    mut mullvad_client: MullvadProxyClient,
) -> Result<(), Error> {
    // TODO: observe UDP traffic on the expected destination/port (only)
    // TODO: IPv6

    const PORTS: [(u16, bool); 3] = [(53, true), (51820, true), (1, false)];

    for (port, should_succeed) in PORTS {
        log::info!("Connect to WireGuard endpoint on port {port}");

        let relay_settings = RelaySettings::Normal(RelayConstraints {
            tunnel_protocol: Constraint::Only(TunnelType::Wireguard),
            wireguard_constraints: WireguardConstraints {
                port: Constraint::Only(port),
                ..Default::default()
            },
            ..Default::default()
        });

        set_relay_settings(&mut mullvad_client, relay_settings)
            .await
            .expect("failed to update relay settings");

        let connection_result = connect_and_wait(&mut mullvad_client).await;
        assert_eq!(
            connection_result.is_ok(),
            should_succeed,
            "unexpected result for port {port}: {connection_result:?}",
        );

        if should_succeed {
            assert!(
                helpers::using_mullvad_exit(&rpc).await,
                "expected Mullvad exit IP"
            );
        }

        disconnect_and_wait(&mut mullvad_client).await?;
    }

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
    mullvad_client
        .set_obfuscation_settings(relay_constraints::ObfuscationSettings {
            selected_obfuscation: SelectedObfuscation::Udp2Tcp,
            udp2tcp: Udp2TcpObfuscationSettings {
                port: Constraint::Any,
            },
        })
        .await
        .expect("failed to enable udp2tcp");

    let relay_settings = RelaySettings::Normal(RelayConstraints {
        tunnel_protocol: Constraint::Only(TunnelType::Wireguard),
        ..Default::default()
    });

    set_relay_settings(&mut mullvad_client, relay_settings)
        .await
        .expect("failed to update relay settings");

    log::info!("Connect to WireGuard via tcp2udp endpoint");

    connect_and_wait(&mut mullvad_client).await?;

    let endpoint = match mullvad_client.get_tunnel_state().await? {
        mullvad_types::states::TunnelState::Connected { endpoint, .. } => endpoint.endpoint,
        _ => panic!("unexpected tunnel state"),
    };

    //
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

    //
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

/// Test whether bridge mode works. This fails if:
/// * No outgoing traffic to the bridge/entry relay is
///   observed from the SUT.
/// * The conncheck reports an unexpected exit relay.
#[test_function]
pub async fn test_bridge(
    _: TestContext,
    rpc: ServiceClient,
    mut mullvad_client: MullvadProxyClient,
) -> Result<(), Error> {
    //
    // Enable bridge mode
    //
    log::info!("Updating bridge settings");

    mullvad_client
        .set_bridge_state(relay_constraints::BridgeState::On)
        .await
        .expect("failed to enable bridge mode");

    set_bridge_settings(&mut mullvad_client, BridgeSettings::default())
        .await
        .expect("failed to update bridge settings");

    set_relay_settings(
        &mut mullvad_client,
        RelaySettings::Normal(RelayConstraints {
            tunnel_protocol: Constraint::Only(TunnelType::OpenVpn),
            ..Default::default()
        }),
    )
    .await
    .expect("failed to update relay settings");

    //
    // Connect to VPN
    //

    log::info!("Connect to OpenVPN relay via bridge");

    connect_and_wait(&mut mullvad_client).await?;

    let (entry, exit) = match mullvad_client.get_tunnel_state().await? {
        mullvad_types::states::TunnelState::Connected { endpoint, .. } => {
            (endpoint.proxy.unwrap().endpoint, endpoint.endpoint)
        }
        actual => {
            panic!("unexpected tunnel state. Expected `TunnelState::Connected` but got {actual:?}")
        }
    };

    log::info!(
        "Selected entry bridge {entry_addr} & exit relay {exit_addr}",
        entry_addr = entry.address,
        exit_addr = exit.address
    );

    // Start recording outgoing packets. Their destination will be verified
    // against the bridge's IP address later.
    let monitor = start_packet_monitor(
        move |packet| packet.destination.ip() == entry.address.ip(),
        MonitorOptions::default(),
    )
    .await;

    //
    // Verify exit IP
    //

    log::info!("Verifying exit server");

    assert!(
        helpers::using_mullvad_exit(&rpc).await,
        "expected Mullvad exit IP"
    );

    //
    // Verify entry IP
    //

    log::info!("Verifying entry server");

    let monitor_result = monitor.into_result().await.unwrap();
    assert!(
        !monitor_result.packets.is_empty(),
        "detected no traffic to entry server",
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
    let relay_constraints = RelayQueryBuilder::new()
        .wireguard()
        .multihop()
        .into_constraint();

    set_relay_settings(
        &mut mullvad_client,
        RelaySettings::Normal(relay_constraints),
    )
    .await
    .expect("failed to update relay settings");

    //
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

    //
    // Record outgoing packets to the entry relay
    //

    let monitor = start_packet_monitor(
        move |packet| packet.destination.ip() == entry.address.ip(),
        MonitorOptions::default(),
    )
    .await;

    //
    // Verify exit IP
    //

    assert!(
        helpers::using_mullvad_exit(&rpc).await,
        "expected Mullvad exit IP"
    );

    //
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
    log::info!("Setting tunnel protocol to WireGuard");

    let relay_settings = RelaySettings::Normal(RelayConstraints {
        tunnel_protocol: Constraint::Only(TunnelType::Wireguard),
        ..Default::default()
    });

    set_relay_settings(&mut mullvad_client, relay_settings)
        .await
        .expect("failed to update relay settings");

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

/// Test whether the daemon automatically connects on reboot when using
/// OpenVPN.
///
/// # Limitations
///
/// This test does not guarantee that nothing leaks during boot or shutdown.
#[test_function]
pub async fn test_openvpn_autoconnect(
    _: TestContext,
    mut rpc: ServiceClient,
    mut mullvad_client: MullvadProxyClient,
) -> Result<(), Error> {
    log::info!("Setting tunnel protocol to OpenVPN");

    let relay_settings = RelaySettings::Normal(RelayConstraints {
        tunnel_protocol: Constraint::Only(TunnelType::OpenVpn),
        ..Default::default()
    });

    set_relay_settings(&mut mullvad_client, relay_settings)
        .await
        .expect("failed to update relay settings");

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
) -> Result<(), Error> {
    mullvad_client
        .set_quantum_resistant_tunnel(wireguard::QuantumResistantState::Off)
        .await
        .expect("Failed to disable PQ tunnels");

    //
    // PQ disabled: Find no "preshared key"
    //

    connect_and_wait(&mut mullvad_client).await?;
    check_tunnel_psk(&rpc, &mullvad_client, false).await;

    log::info!("Setting tunnel protocol to WireGuard");

    let relay_settings = RelaySettings::Normal(RelayConstraints {
        tunnel_protocol: Constraint::Only(TunnelType::Wireguard),
        ..Default::default()
    });
    set_relay_settings(&mut mullvad_client, relay_settings)
        .await
        .expect("Failed to update relay settings");

    mullvad_client
        .set_quantum_resistant_tunnel(wireguard::QuantumResistantState::On)
        .await
        .expect("Failed to enable PQ tunnels");

    //
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

    mullvad_client
        .set_obfuscation_settings(relay_constraints::ObfuscationSettings {
            selected_obfuscation: SelectedObfuscation::Udp2Tcp,
            udp2tcp: Udp2TcpObfuscationSettings {
                port: Constraint::Any,
            },
        })
        .await
        .expect("Failed to enable obfuscation");

    let relay_constraints = RelayQueryBuilder::new()
        .wireguard()
        .multihop()
        .into_constraint();

    mullvad_client
        .set_relay_settings(RelaySettings::Normal(relay_constraints))
        .await
        .expect("Failed to update relay settings");

    connect_and_wait(&mut mullvad_client).await?;

    assert!(
        helpers::using_mullvad_exit(&rpc).await,
        "expected Mullvad exit IP"
    );

    Ok(())
}

/// Try to connect to an OpenVPN relay via a remote, passwordless SOCKS5 server.
/// * No outgoing traffic to the bridge/entry relay is observed from the SUT.
/// * The conncheck reports an unexpected exit relay.
#[test_function]
pub async fn test_remote_socks_bridge(
    _: TestContext,
    rpc: ServiceClient,
    mut mullvad_client: MullvadProxyClient,
) -> Result<(), Error> {
    mullvad_client
        .set_bridge_state(relay_constraints::BridgeState::On)
        .await
        .expect("failed to enable bridge mode");

    mullvad_client
        .set_bridge_settings(BridgeSettings {
            bridge_type: BridgeType::Custom,
            normal: BridgeConstraints::default(),
            custom: Some(CustomProxy::Socks5Remote(Socks5Remote::new((
                crate::vm::network::NON_TUN_GATEWAY,
                crate::vm::network::SOCKS5_PORT,
            )))),
        })
        .await
        .expect("failed to update bridge settings");

    set_relay_settings(
        &mut mullvad_client,
        RelaySettings::Normal(RelayConstraints {
            tunnel_protocol: Constraint::Only(TunnelType::OpenVpn),
            ..Default::default()
        }),
    )
    .await
    .expect("failed to update relay settings");

    //
    // Connect to VPN
    //

    connect_and_wait(&mut mullvad_client).await?;

    let (entry, exit) = match mullvad_client.get_tunnel_state().await? {
        mullvad_types::states::TunnelState::Connected { endpoint, .. } => {
            (endpoint.proxy.unwrap().endpoint, endpoint.endpoint)
        }
        actual => {
            panic!("unexpected tunnel state. Expected `TunnelState::Connected` but got {actual:?}")
        }
    };

    log::info!(
        "Selected entry bridge {entry_addr} & exit relay {exit_addr}",
        entry_addr = entry.address,
        exit_addr = exit.address
    );

    // Start recording outgoing packets. Their destination will be verified
    // against the bridge's IP address later.
    let monitor = start_packet_monitor(
        move |packet| packet.destination.ip() == entry.address.ip(),
        MonitorOptions::default(),
    )
    .await;

    //
    // Verify exit IP
    //

    log::info!("Verifying exit server");

    assert!(
        helpers::using_mullvad_exit(&rpc).await,
        "expected Mullvad exit IP"
    );

    //
    // Verify entry IP
    //

    log::info!("Verifying entry server");

    let monitor_result = monitor.into_result().await.unwrap();
    assert!(
        !monitor_result.packets.is_empty(),
        "detected no traffic to entry server",
    );

    Ok(())
}

/// Try to connect to an OpenVPN relay via a local, passwordless SOCKS5 server.
/// * No outgoing traffic to the bridge/entry relay is observed from the SUT.
/// * The conncheck reports an unexpected exit relay.
#[test_function]
pub async fn test_local_socks_bridge(
    _: TestContext,
    rpc: ServiceClient,
    mut mullvad_client: MullvadProxyClient,
) -> Result<(), Error> {
    let remote_addr = SocketAddr::from((
        crate::vm::network::NON_TUN_GATEWAY,
        crate::vm::network::SOCKS5_PORT,
    ));
    let socks_server = rpc
        .start_tcp_forward("127.0.0.1:0".parse().unwrap(), remote_addr)
        .await
        .expect("failed to start TCP forward");

    mullvad_client
        .set_bridge_state(relay_constraints::BridgeState::On)
        .await
        .expect("failed to enable bridge mode");

    mullvad_client
        .set_bridge_settings(BridgeSettings {
            bridge_type: BridgeType::Custom,
            normal: BridgeConstraints::default(),
            custom: Some(CustomProxy::Socks5Local(
                Socks5Local::new_with_transport_protocol(
                    remote_addr,
                    socks_server.bind_addr().port(),
                    TransportProtocol::Tcp,
                ),
            )),
        })
        .await
        .expect("failed to update bridge settings");

    set_relay_settings(
        &mut mullvad_client,
        RelaySettings::Normal(RelayConstraints {
            tunnel_protocol: Constraint::Only(TunnelType::OpenVpn),
            ..Default::default()
        }),
    )
    .await
    .expect("failed to update relay settings");

    //
    // Connect to VPN
    //

    connect_and_wait(&mut mullvad_client).await?;

    let (entry, exit) = match mullvad_client.get_tunnel_state().await? {
        mullvad_types::states::TunnelState::Connected { endpoint, .. } => {
            (endpoint.proxy.unwrap().endpoint, endpoint.endpoint)
        }
        actual => {
            panic!("unexpected tunnel state. Expected `TunnelState::Connected` but got {actual:?}")
        }
    };

    log::info!(
        "Selected entry bridge {entry_addr} & exit relay {exit_addr}",
        entry_addr = entry.address,
        exit_addr = exit.address
    );

    // Start recording outgoing packets. Their destination will be verified
    // against the bridge's IP address later.
    let monitor = start_packet_monitor(
        move |packet| packet.destination.ip() == entry.address.ip(),
        MonitorOptions::default(),
    )
    .await;

    //
    // Verify exit IP
    //

    log::info!("Verifying exit server");

    assert!(
        helpers::using_mullvad_exit(&rpc).await,
        "expected Mullvad exit IP"
    );

    //
    // Verify entry IP
    //

    log::info!("Verifying entry server");

    let monitor_result = monitor.into_result().await.unwrap();
    assert!(
        !monitor_result.packets.is_empty(),
        "detected no traffic to entry server",
    );

    Ok(())
}

/// Verify that the app can connect to a VPN server and get working internet when the API is down.
/// As long as the user has managed to log in to the app, establishing a tunnel should work even if the API is down (This includes actually being down, not just censored).
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

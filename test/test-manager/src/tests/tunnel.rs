use super::helpers::{
    self, connect_and_wait, disconnect_and_wait, geoip_lookup_with_retries, update_relay_settings,
};
use super::{Error, TestContext};
use std::net::IpAddr;

use crate::network_monitor::{start_packet_monitor, MonitorOptions};
use mullvad_management_interface::{types, ManagementServiceClient};
use mullvad_types::relay_constraints::{
    Constraint, LocationConstraint, OpenVpnConstraints, RelayConstraintsUpdate,
    RelaySettingsUpdate, WireguardConstraints,
};
use mullvad_types::relay_constraints::{GeographicLocationConstraint, TransportPort};
use pnet_packet::ip::IpNextHeaderProtocols;
use talpid_types::net::{TransportProtocol, TunnelType};
use test_macro::test_function;
use test_rpc::meta::Os;
use test_rpc::mullvad_daemon::ServiceStatus;
use test_rpc::{Interface, ServiceClient};

/// Set up an OpenVPN tunnel, UDP as well as TCP.
/// This test fails if a working tunnel cannot be set up.
#[test_function]
pub async fn test_openvpn_tunnel(
    _: TestContext,
    rpc: ServiceClient,
    mut mullvad_client: ManagementServiceClient,
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

        let relay_settings = RelaySettingsUpdate::Normal(RelayConstraintsUpdate {
            location: Some(Constraint::Only(LocationConstraint::Location(
                GeographicLocationConstraint::Country("se".to_string()),
            ))),
            tunnel_protocol: Some(Constraint::Only(TunnelType::OpenVpn)),
            openvpn_constraints: Some(OpenVpnConstraints { port: constraint }),
            ..Default::default()
        });

        update_relay_settings(&mut mullvad_client, relay_settings)
            .await
            .expect("failed to update relay settings");

        connect_and_wait(&mut mullvad_client).await?;

        // Send traffic through the tunnel to sanity check that the internet is reachable.
        log::info!("Test whether tunnel traffic works");
        let geoip_lookup = geoip_lookup_with_retries(&rpc).await.unwrap();
        assert!(geoip_lookup.mullvad_exit_ip, "Exit ip is not from Mullvad");

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
    mut mullvad_client: ManagementServiceClient,
) -> Result<(), Error> {
    // TODO: observe UDP traffic on the expected destination/port (only)
    // TODO: IPv6

    const PORTS: [(u16, bool); 3] = [(53, true), (51820, true), (1, false)];

    for (port, should_succeed) in PORTS {
        log::info!("Connect to WireGuard endpoint on port {port}");

        let relay_settings = RelaySettingsUpdate::Normal(RelayConstraintsUpdate {
            location: Some(Constraint::Only(LocationConstraint::Location(
                GeographicLocationConstraint::Country("se".to_string()),
            ))),
            tunnel_protocol: Some(Constraint::Only(TunnelType::Wireguard)),
            wireguard_constraints: Some(WireguardConstraints {
                port: Constraint::Only(port),
                ..Default::default()
            }),
            ..Default::default()
        });

        update_relay_settings(&mut mullvad_client, relay_settings)
            .await
            .expect("failed to update relay settings");

        let connection_result = connect_and_wait(&mut mullvad_client).await;
        assert_eq!(
            connection_result.is_ok(),
            should_succeed,
            "unexpected result for port {port}: {connection_result:?}",
        );

        if should_succeed {
            // Send traffic through the tunnel to sanity check that the internet is reachable.
            log::info!("Test whether tunnel traffic works");
            let geoip_lookup = geoip_lookup_with_retries(&rpc).await.unwrap();
            assert!(geoip_lookup.mullvad_exit_ip, "Exit ip is not from Mullvad");
        }

        disconnect_and_wait(&mut mullvad_client).await?;
    }

    Ok(())
}

/// Use udp2tcp obfuscation. This test connects to a
/// WireGuard relay over TCP. It fails if no outgoing TCP
/// traffic to the relay is observed on the expected port.
#[test_function]
pub async fn test_udp2tcp_tunnel(
    _: TestContext,
    rpc: ServiceClient,
    mut mullvad_client: ManagementServiceClient,
) -> Result<(), Error> {
    // TODO: check if src <-> target / tcp is observed (only)
    // TODO: ping a public IP on the fake network (not possible using real relay)

    mullvad_client
        .set_obfuscation_settings(types::ObfuscationSettings {
            selected_obfuscation: i32::from(
                types::obfuscation_settings::SelectedObfuscation::Udp2tcp,
            ),
            udp2tcp: Some(types::Udp2TcpObfuscationSettings { port: 0 }),
        })
        .await
        .expect("failed to enable udp2tcp");

    let relay_settings = RelaySettingsUpdate::Normal(RelayConstraintsUpdate {
        location: Some(Constraint::Only(LocationConstraint::Location(
            GeographicLocationConstraint::Country("se".to_string()),
        ))),
        tunnel_protocol: Some(Constraint::Only(TunnelType::Wireguard)),
        wireguard_constraints: Some(WireguardConstraints::default()),
        ..Default::default()
    });

    update_relay_settings(&mut mullvad_client, relay_settings)
        .await
        .expect("failed to update relay settings");

    log::info!("Connect to WireGuard via tcp2udp endpoint");

    connect_and_wait(&mut mullvad_client).await?;

    //
    // Set up packet monitor
    //

    let guest_ip = rpc
        .get_interface_ip(Interface::NonTunnel)
        .await
        .expect("failed to obtain inet interface IP");

    let monitor = start_packet_monitor(
        move |packet| {
            packet.source.ip() != guest_ip || (packet.protocol == IpNextHeaderProtocols::Tcp)
        },
        MonitorOptions::default(),
    )
    .await;

    //
    // Verify that we can reach stuff
    //

    // Send traffic through the tunnel to sanity check that the internet is reachable.
    log::info!("Test whether tunnel traffic works");
    let geoip_lookup = geoip_lookup_with_retries(&rpc).await.unwrap();
    assert!(geoip_lookup.mullvad_exit_ip, "Exit ip is not from Mullvad");

    let monitor_result = monitor.into_result().await.unwrap();
    assert_eq!(monitor_result.discarded_packets, 0);

    disconnect_and_wait(&mut mullvad_client).await?;

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
    mut mullvad_client: ManagementServiceClient,
) -> Result<(), Error> {
    log::info!("Select relay");
    let bridge_filter = |bridge: &types::Relay| {
        bridge.active && bridge.endpoint_type == i32::from(types::relay::RelayType::Bridge)
    };
    let ovpn_filter = |relay: &types::Relay| {
        relay.active && relay.endpoint_type == i32::from(types::relay::RelayType::Openvpn)
    };
    let entry = helpers::filter_relays(&mut mullvad_client, bridge_filter)
        .await?
        .pop()
        .unwrap();
    let exit = helpers::filter_relays(&mut mullvad_client, ovpn_filter)
        .await?
        .pop()
        .unwrap();

    //
    // Enable bridge mode
    //

    log::info!("Updating bridge settings");

    mullvad_client
        .set_bridge_state(types::BridgeState {
            state: i32::from(types::bridge_state::State::On),
        })
        .await
        .expect("failed to enable bridge mode");

    mullvad_client
        .set_bridge_settings(types::BridgeSettings {
            r#type: Some(types::bridge_settings::Type::Normal(
                types::bridge_settings::BridgeConstraints {
                    location: helpers::into_locationconstraint(&entry)
                        .map(types::LocationConstraint::from),
                    providers: vec![],
                    ownership: i32::from(types::Ownership::Any),
                },
            )),
        })
        .await
        .expect("failed to update bridge settings");

    let relay_settings = RelaySettingsUpdate::Normal(RelayConstraintsUpdate {
        location: helpers::into_constraint(&exit),
        tunnel_protocol: Some(Constraint::Only(TunnelType::OpenVpn)),
        ..Default::default()
    });

    update_relay_settings(&mut mullvad_client, relay_settings)
        .await
        .expect("failed to update relay settings");

    //
    // Connect to VPN
    //

    log::info!("Connect to OpenVPN relay via bridge");

    let monitor = start_packet_monitor(
        move |packet| packet.destination.ip() == entry.ipv4_addr_in.parse::<IpAddr>().unwrap(),
        MonitorOptions::default(),
    )
    .await;

    connect_and_wait(&mut mullvad_client)
        .await
        .expect("connect_and_wait");

    //
    // Verify entry IP
    //

    log::info!("Verifying entry server");

    let monitor_result = monitor.into_result().await.unwrap();
    assert!(
        !monitor_result.packets.is_empty(),
        "detected no traffic to entry server",
    );

    //
    // Verify exit IP
    //

    log::info!("Verifying exit server");

    let geoip = geoip_lookup_with_retries(&rpc).await?;
    assert_eq!(geoip.mullvad_exit_ip_hostname, exit.hostname);

    disconnect_and_wait(&mut mullvad_client).await?;

    Ok(())
}

/// Test whether WireGuard multihop works. This fails if:
/// * No outgoing traffic to the entry relay is
///   observed from the SUT.
/// * The conncheck reports an unexpected exit relay.
#[test_function]
pub async fn test_multihop(
    _: TestContext,
    rpc: ServiceClient,
    mut mullvad_client: ManagementServiceClient,
) -> Result<(), Error> {
    //
    // Set relays to use
    //

    log::info!("Select relay");
    let relay_filter = |relay: &types::Relay| {
        relay.active && relay.endpoint_type == i32::from(types::relay::RelayType::Wireguard)
    };
    let (entry, exit) = helpers::random_entry_and_exit(&mut mullvad_client, relay_filter).await?;
    let exit_constraint = helpers::into_constraint(&exit);
    let entry_constraint =
        helpers::into_constraint(&entry).map(|entry_location| WireguardConstraints {
            use_multihop: true,
            entry_location,
            ..Default::default()
        });

    let relay_settings = RelaySettingsUpdate::Normal(RelayConstraintsUpdate {
        location: exit_constraint,
        wireguard_constraints: entry_constraint,
        ..Default::default()
    });

    update_relay_settings(&mut mullvad_client, relay_settings)
        .await
        .expect("failed to update relay settings");

    //
    // Connect
    //

    let monitor = start_packet_monitor(
        move |packet| {
            packet.destination.ip() == entry.ipv4_addr_in.parse::<IpAddr>().unwrap()
                && packet.protocol == IpNextHeaderProtocols::Udp
        },
        MonitorOptions::default(),
    )
    .await;

    connect_and_wait(&mut mullvad_client).await?;

    //
    // Verify entry IP
    //

    log::info!("Verifying entry server");

    let monitor_result = monitor.into_result().await.unwrap();
    assert!(!monitor_result.packets.is_empty(), "no matching packets",);

    //
    // Verify exit IP
    //

    log::info!("Verifying exit server");

    let geoip = geoip_lookup_with_retries(&rpc).await?;
    assert_eq!(geoip.mullvad_exit_ip_hostname, exit.hostname);

    disconnect_and_wait(&mut mullvad_client).await?;

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
    mut mullvad_client: ManagementServiceClient,
) -> Result<(), Error> {
    log::info!("Setting tunnel protocol to WireGuard");

    let relay_settings = RelaySettingsUpdate::Normal(RelayConstraintsUpdate {
        location: Some(Constraint::Only(LocationConstraint::Location(
            GeographicLocationConstraint::Country("se".to_string()),
        ))),
        tunnel_protocol: Some(Constraint::Only(TunnelType::Wireguard)),
        ..Default::default()
    });

    update_relay_settings(&mut mullvad_client, relay_settings)
        .await
        .expect("failed to update relay settings");

    mullvad_client
        .set_auto_connect(true)
        .await
        .expect("failed to enable auto-connect");

    reboot(&mut rpc).await?;
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
    mut mullvad_client: ManagementServiceClient,
) -> Result<(), Error> {
    log::info!("Setting tunnel protocol to OpenVPN");

    let relay_settings = RelaySettingsUpdate::Normal(RelayConstraintsUpdate {
        location: Some(Constraint::Only(LocationConstraint::Location(
            GeographicLocationConstraint::Country("se".to_string()),
        ))),
        tunnel_protocol: Some(Constraint::Only(TunnelType::OpenVpn)),
        ..Default::default()
    });

    update_relay_settings(&mut mullvad_client, relay_settings)
        .await
        .expect("failed to update relay settings");

    mullvad_client
        .set_auto_connect(true)
        .await
        .expect("failed to enable auto-connect");

    reboot(&mut rpc).await?;
    rpc.mullvad_daemon_wait_for_state(|state| state == ServiceStatus::Running)
        .await?;

    log::info!("Waiting for daemon to connect");

    helpers::wait_for_tunnel_state(mullvad_client, |state| {
        matches!(state, mullvad_types::states::TunnelState::Connected { .. })
    })
    .await?;

    Ok(())
}

async fn reboot(rpc: &mut ServiceClient) -> Result<(), Error> {
    rpc.reboot().await?;

    // The tunnel must be reconfigured after the virtual machine is up,
    // or macOS refuses to assign an IP. The reasons for this are poorly understood.
    #[cfg(target_os = "macos")]
    crate::vm::network::macos::configure_tunnel()
        .await
        .map_err(|error| Error::Other(format!("Failed to recreate custom wg tun: {error}")))?;

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
    mut mullvad_client: ManagementServiceClient,
) -> Result<(), Error> {
    mullvad_client
        .set_quantum_resistant_tunnel(types::QuantumResistantState {
            state: i32::from(types::quantum_resistant_state::State::Off),
        })
        .await
        .expect("Failed to disable PQ tunnels");

    //
    // PQ disabled: Find no "preshared key"
    //

    connect_and_wait(&mut mullvad_client).await?;
    check_tunnel_psk(&rpc, false).await;

    log::info!("Setting tunnel protocol to WireGuard");

    let relay_settings = RelaySettingsUpdate::Normal(RelayConstraintsUpdate {
        location: Some(Constraint::Only(LocationConstraint::Location(
            GeographicLocationConstraint::Country("se".to_string()),
        ))),
        tunnel_protocol: Some(Constraint::Only(TunnelType::Wireguard)),
        ..Default::default()
    });

    update_relay_settings(&mut mullvad_client, relay_settings)
        .await
        .expect("Failed to update relay settings");

    mullvad_client
        .set_quantum_resistant_tunnel(types::QuantumResistantState {
            state: i32::from(types::quantum_resistant_state::State::On),
        })
        .await
        .expect("Failed to enable PQ tunnels");

    //
    // PQ enabled: Find "preshared key"
    //

    connect_and_wait(&mut mullvad_client).await?;
    check_tunnel_psk(&rpc, true).await;

    // Send traffic through the tunnel to sanity check that the internet is reachable.
    log::info!("Test whether tunnel traffic works");
    let geoip_lookup = geoip_lookup_with_retries(&rpc).await.unwrap();
    assert!(geoip_lookup.mullvad_exit_ip, "Exit ip is not from Mullvad");

    Ok(())
}

async fn check_tunnel_psk(rpc: &ServiceClient, should_have_psk: bool) {
    match rpc.get_os().await.expect("failed to get OS") {
        Os::Linux => {
            let name = rpc
                .get_interface_name(Interface::Tunnel)
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
    mut mullvad_client: ManagementServiceClient,
) -> Result<(), Error> {
    mullvad_client
        .set_quantum_resistant_tunnel(types::QuantumResistantState {
            state: i32::from(types::quantum_resistant_state::State::On),
        })
        .await
        .expect("Failed to enable PQ tunnels");

    mullvad_client
        .set_obfuscation_settings(types::ObfuscationSettings {
            selected_obfuscation: i32::from(
                types::obfuscation_settings::SelectedObfuscation::Udp2tcp,
            ),
            udp2tcp: Some(types::Udp2TcpObfuscationSettings { port: 0 }),
        })
        .await
        .expect("Failed to enable obfuscation");

    let relay_settings = RelaySettingsUpdate::Normal(RelayConstraintsUpdate {
        location: Some(Constraint::Only(LocationConstraint::Location(
            GeographicLocationConstraint::Country("se".to_string()),
        ))),
        wireguard_constraints: Some(WireguardConstraints {
            use_multihop: true,
            entry_location: Constraint::Only(LocationConstraint::Location(
                GeographicLocationConstraint::Country("se".to_string()),
            )),
            ..Default::default()
        }),
        ..Default::default()
    });

    update_relay_settings(&mut mullvad_client, relay_settings)
        .await
        .expect("Failed to update relay settings");

    connect_and_wait(&mut mullvad_client).await?;

    // Send traffic through the tunnel to sanity check that the internet is reachable.
    log::info!("Test whether tunnel traffic works");
    let geoip_lookup = geoip_lookup_with_retries(&rpc).await.unwrap();
    assert!(geoip_lookup.mullvad_exit_ip, "Exit ip is not from Mullvad");

    Ok(())
}

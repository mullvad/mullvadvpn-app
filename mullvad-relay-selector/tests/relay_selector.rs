//! Tests for verifying that the relay selector works as expected.

use once_cell::sync::Lazy;
use std::collections::HashSet;
use talpid_types::net::{
    obfuscation::ObfuscatorConfig,
    wireguard::PublicKey,
    Endpoint,
    TransportProtocol::{Tcp, Udp},
    TunnelType,
};

use mullvad_relay_selector::{
    query::{builder::RelayQueryBuilder, BridgeQuery, ObfuscationQuery, OpenVpnRelayQuery},
    Error, GetRelay, RelaySelector, RuntimeParameters, SelectorConfig, WireguardConfig,
    RETRY_ORDER,
};
use mullvad_types::{
    constraints::Constraint,
    endpoint::MullvadEndpoint,
    relay_constraints::{
        BridgeConstraints, BridgeState, GeographicLocationConstraint, Ownership, Providers,
        TransportPort,
    },
    relay_list::{
        BridgeEndpointData, OpenVpnEndpoint, OpenVpnEndpointData, Relay, RelayEndpointData,
        RelayList, RelayListCity, RelayListCountry, ShadowsocksEndpointData, WireguardEndpointData,
        WireguardRelayEndpointData,
    },
};

static RELAYS: Lazy<RelayList> = Lazy::new(|| RelayList {
    etag: None,
    countries: vec![RelayListCountry {
        name: "Sweden".to_string(),
        code: "se".to_string(),
        cities: vec![RelayListCity {
            name: "Gothenburg".to_string(),
            code: "got".to_string(),
            latitude: 57.70887,
            longitude: 11.97456,
            relays: vec![
                Relay {
                    hostname: "se9-wireguard".to_string(),
                    ipv4_addr_in: "185.213.154.68".parse().unwrap(),
                    ipv6_addr_in: Some("2a03:1b20:5:f011::a09f".parse().unwrap()),
                    include_in_country: true,
                    active: true,
                    owned: true,
                    provider: "provider0".to_string(),
                    weight: 1,
                    endpoint_data: RelayEndpointData::Wireguard(WireguardRelayEndpointData {
                        public_key: PublicKey::from_base64(
                            "BLNHNoGO88LjV/wDBa7CUUwUzPq/fO2UwcGLy56hKy4=",
                        )
                        .unwrap(),
                        daita: false,
                        shadowsocks_extra_addr_in: vec![],
                    }),
                    location: None,
                },
                Relay {
                    hostname: "se10-wireguard".to_string(),
                    ipv4_addr_in: "185.213.154.69".parse().unwrap(),
                    ipv6_addr_in: Some("2a03:1b20:5:f011::a10f".parse().unwrap()),
                    include_in_country: true,
                    active: true,
                    owned: false,
                    provider: "provider1".to_string(),
                    weight: 1,
                    endpoint_data: RelayEndpointData::Wireguard(WireguardRelayEndpointData {
                        public_key: PublicKey::from_base64(
                            "BLNHNoGO88LjV/wDBa7CUUwUzPq/fO2UwcGLy56hKy4=",
                        )
                        .unwrap(),
                        daita: false,
                        shadowsocks_extra_addr_in: vec![],
                    }),
                    location: None,
                },
                Relay {
                    hostname: "se-got-001".to_string(),
                    ipv4_addr_in: "185.213.154.131".parse().unwrap(),
                    ipv6_addr_in: None,
                    include_in_country: true,
                    active: true,
                    owned: true,
                    provider: "provider2".to_string(),
                    weight: 1,
                    endpoint_data: RelayEndpointData::Openvpn,
                    location: None,
                },
                Relay {
                    hostname: "se-got-002".to_string(),
                    ipv4_addr_in: "1.2.3.4".parse().unwrap(),
                    ipv6_addr_in: None,
                    include_in_country: true,
                    active: true,
                    owned: true,
                    provider: "provider0".to_string(),
                    weight: 1,
                    endpoint_data: RelayEndpointData::Openvpn,
                    location: None,
                },
                Relay {
                    hostname: "se-got-br-001".to_string(),
                    ipv4_addr_in: "1.3.3.7".parse().unwrap(),
                    ipv6_addr_in: None,
                    include_in_country: true,
                    active: true,
                    owned: true,
                    provider: "provider3".to_string(),
                    weight: 1,
                    endpoint_data: RelayEndpointData::Bridge,
                    location: None,
                },
            ],
        }],
    }],
    openvpn: OpenVpnEndpointData {
        ports: vec![
            OpenVpnEndpoint {
                port: 1194,
                protocol: Udp,
            },
            OpenVpnEndpoint {
                port: 443,
                protocol: Tcp,
            },
            OpenVpnEndpoint {
                port: 80,
                protocol: Tcp,
            },
        ],
    },
    bridge: BridgeEndpointData {
        shadowsocks: vec![
            ShadowsocksEndpointData {
                port: 443,
                cipher: "aes-256-gcm".to_string(),
                password: "mullvad".to_string(),
                protocol: Tcp,
            },
            ShadowsocksEndpointData {
                port: 1234,
                cipher: "aes-256-cfb".to_string(),
                password: "mullvad".to_string(),
                protocol: Udp,
            },
            ShadowsocksEndpointData {
                port: 1236,
                cipher: "aes-256-gcm".to_string(),
                password: "mullvad".to_string(),
                protocol: Udp,
            },
        ],
    },
    wireguard: WireguardEndpointData {
        port_ranges: vec![
            (53, 53),
            (443, 443),
            (4000, 33433),
            (33565, 51820),
            (52000, 60000),
        ],
        ipv4_gateway: "10.64.0.1".parse().unwrap(),
        ipv6_gateway: "fc00:bbbb:bbbb:bb01::1".parse().unwrap(),
        udp2tcp_ports: vec![],
        shadowsocks_port_ranges: vec![(100, 200), (1000, 2000)],
    },
});

// Helper functions
fn unwrap_relay(get_result: GetRelay) -> Relay {
    match get_result {
        GetRelay::Wireguard { inner, .. } => match inner {
            crate::WireguardConfig::Singlehop { exit } => exit,
            crate::WireguardConfig::Multihop { exit, .. } => exit,
        },
        GetRelay::OpenVpn { exit, .. } => exit,
        GetRelay::Custom(custom) => {
            panic!("Can not extract regular relay from custom relay: {custom}")
        }
    }
}

fn unwrap_entry_relay(get_result: GetRelay) -> Relay {
    match get_result {
        GetRelay::Wireguard { inner, .. } => match inner {
            crate::WireguardConfig::Singlehop { exit } => exit,
            crate::WireguardConfig::Multihop { entry, .. } => entry,
        },
        GetRelay::OpenVpn { exit, .. } => exit,
        GetRelay::Custom(custom) => {
            panic!("Can not extract regular relay from custom relay: {custom}")
        }
    }
}

fn unwrap_endpoint(get_result: GetRelay) -> MullvadEndpoint {
    match get_result {
        GetRelay::Wireguard { endpoint, .. } => MullvadEndpoint::Wireguard(endpoint),
        GetRelay::OpenVpn { endpoint, .. } => MullvadEndpoint::OpenVpn(endpoint),
        GetRelay::Custom(custom) => {
            panic!("Can not extract Mullvad endpoint from custom relay: {custom}")
        }
    }
}

fn tunnel_type(relay: &Relay) -> TunnelType {
    match relay.endpoint_data {
        RelayEndpointData::Openvpn | RelayEndpointData::Bridge => TunnelType::OpenVpn,
        RelayEndpointData::Wireguard(_) => TunnelType::Wireguard,
    }
}

fn default_relay_selector() -> RelaySelector {
    RelaySelector::from_list(SelectorConfig::default(), RELAYS.clone())
}

fn supports_daita(relay: &Relay) -> bool {
    match relay.endpoint_data {
        RelayEndpointData::Wireguard(WireguardRelayEndpointData { daita, .. }) => daita,
        _ => false,
    }
}

/// This is not an actual test. Rather, it serves as a reminder that if [`RETRY_ORDER`] is modified,
/// the programmer should be made aware to update all external documents which rely on the retry
/// order to be correct.
///
/// When all necessary changes have been made, feel free to update this test to mirror the new
/// [`RETRY_ORDER`].
#[test]
fn assert_retry_order() {
    use talpid_types::net::{IpVersion, TransportProtocol};
    let expected_retry_order = vec![
        // 1
        RelayQueryBuilder::new().build(),
        // 2
        RelayQueryBuilder::new().wireguard().port(443).build(),
        // 3
        RelayQueryBuilder::new()
            .wireguard()
            .ip_version(IpVersion::V6)
            .build(),
        // 4
        RelayQueryBuilder::new()
            .openvpn()
            .transport_protocol(TransportProtocol::Tcp)
            .port(443)
            .build(),
        // 5
        RelayQueryBuilder::new().wireguard().shadowsocks().build(),
        // 6
        RelayQueryBuilder::new().wireguard().udp2tcp().build(),
        // 7
        RelayQueryBuilder::new()
            .wireguard()
            .udp2tcp()
            .ip_version(IpVersion::V6)
            .build(),
        // 8
        RelayQueryBuilder::new()
            .openvpn()
            .transport_protocol(TransportProtocol::Tcp)
            .bridge()
            .build(),
    ];

    assert!(
        *RETRY_ORDER == expected_retry_order,
        "
    The relay selector's retry order has been modified!
    Make sure to update `docs/relay-selector.md` with these changes.
    Lastly, you may go ahead and fix this test to reflect the new retry order.
    "
    );
}

/// Test whether the relay selector seems to respect the order as defined by [`RETRY_ORDER`].
#[test]
fn test_retry_order() {
    // In order to for the relay queries defined by `RETRY_ORDER` to always take precedence,
    // the user settings need to be 'neutral' on the type of relay that it wants to connect to.
    // A default `SelectorConfig` *should* have this property, but a more robust way to guarantee
    // this would be to create a neutral relay query and supply it to the relay selector at every
    // call to the `get_relay` function.
    let relay_selector = default_relay_selector();
    for (retry_attempt, query) in RETRY_ORDER.iter().enumerate() {
        let relay = relay_selector
            .get_relay(retry_attempt, RuntimeParameters { ipv6: true })
            .unwrap_or_else(|_| panic!("Retry attempt {retry_attempt} did not yield any relay"));
        // For each relay, cross-check that the it has the expected tunnel protocol
        let tunnel_type = tunnel_type(&unwrap_relay(relay.clone()));
        assert_eq!(
            tunnel_type,
            query.tunnel_protocol.unwrap_or(TunnelType::Wireguard),
            "Retry attempt {retry_attempt} yielded an unexpected tunnel type"
        );
        // Then perform some protocol-specific probing as well.
        match relay {
            GetRelay::Wireguard {
                endpoint,
                obfuscator,
                ..
            } => {
                assert!(query
                    .wireguard_constraints
                    .ip_version
                    .matches_eq(&match endpoint.peer.endpoint.ip() {
                        std::net::IpAddr::V4(_) => talpid_types::net::IpVersion::V4,
                        std::net::IpAddr::V6(_) => talpid_types::net::IpVersion::V6,
                    }));
                assert!(query
                    .wireguard_constraints
                    .port
                    .matches_eq(&endpoint.peer.endpoint.port()));
                assert!(match &query.wireguard_constraints.obfuscation {
                    ObfuscationQuery::Auto => true,
                    ObfuscationQuery::Off => obfuscator.is_none(),
                    ObfuscationQuery::Udp2tcp(_) | ObfuscationQuery::Shadowsocks(_) =>
                        obfuscator.is_some(),
                });
            }
            GetRelay::OpenVpn {
                endpoint, bridge, ..
            } => {
                if BridgeQuery::should_use_bridge(&query.openvpn_constraints.bridge_settings) {
                    assert!(bridge.is_some(), "Relay selector should have selected a bridge for query {query:?}, but bridge was `None`");
                };
                assert!(query
                    .openvpn_constraints
                    .port
                    .map(|transport_port| transport_port.port.matches_eq(&endpoint.address.port()))
                    .unwrap_or(true),
                    "The query {query:?} defined a port to use, but the chosen relay endpoint did not match that port number.
                    Expected: {expected}
                    Actual: {actual}",
                    expected = query.openvpn_constraints.port.unwrap().port.unwrap(), actual = endpoint.address.port()
                );

                assert!(query.openvpn_constraints.port.map(|transport_port| transport_port.protocol == endpoint.protocol).unwrap_or(true),
                    "The query {query:?} defined a transport protocol to use, but the chosen relay endpoint did not match that transport protocol.
                    Expected: {expected}
                    Actual: {actual}",
                    expected = query.openvpn_constraints.port.unwrap().protocol, actual = endpoint.protocol
                );
            }
            GetRelay::Custom(_) => unreachable!(),
        }
    }
}

/// Verify that Wireguard is preferred if the tunnel type is set to auto.
#[test]
fn prefer_wireguard_when_auto() {
    // Turn on bridge state. This is only relevant when selecting OpenVPN relays, but turning
    // this configuration option should not prompt the relay selector to prefer OpenVPN.
    let config = SelectorConfig {
        bridge_state: BridgeState::On,
        ..SelectorConfig::default()
    };
    let relay_selector = RelaySelector::from_list(config, RELAYS.clone());
    for _ in 0..100 {
        let query = RelayQueryBuilder::new().build();
        let relay = relay_selector.get_relay_by_query(query).unwrap();
        let tunnel_type = tunnel_type(&unwrap_relay(relay));
        assert_eq!(tunnel_type, TunnelType::Wireguard);
    }
}

/// If a Wireguard relay is only specified by it's hostname (and not tunnel type), the relay
/// selector should still return a relay of the correct tunnel type (Wireguard).
#[test]
fn test_prefer_wireguard_if_location_supports_it() {
    let relay_selector = default_relay_selector();
    let query = RelayQueryBuilder::new()
        .location(GeographicLocationConstraint::hostname(
            "se",
            "got",
            "se9-wireguard",
        ))
        .build();

    for _ in 0..RETRY_ORDER.len() {
        let relay = relay_selector.get_relay_by_query(query.clone()).unwrap();
        let tunnel_typ = tunnel_type(&unwrap_relay(relay));
        assert_eq!(tunnel_typ, TunnelType::Wireguard);
    }
}

/// If an OpenVPN relay is only specified by it's hostname (and not tunnel type), the relay selector
/// should still return a relay of the correct tunnel type (OpenVPN).
#[test]
fn test_prefer_openvpn_if_location_supports_it() {
    let relay_selector = default_relay_selector();
    let query = RelayQueryBuilder::new()
        .location(GeographicLocationConstraint::hostname(
            "se",
            "got",
            "se-got-001",
        ))
        .build();

    for _ in 0..RETRY_ORDER.len() {
        let relay = relay_selector.get_relay_by_query(query.clone()).unwrap();
        let tunnel_typ = tunnel_type(&unwrap_relay(relay));
        assert_eq!(tunnel_typ, TunnelType::OpenVpn);
    }
}

/// Assert that the relay selector does *not* return a multihop configuration where the exit and
/// entry relay are the same, even if the constraints would allow for it. Also verify that the relay
/// selector is smart enough to pick either the entry or exit relay first depending on which one
/// ends up yielding a valid configuration.
#[test]
fn test_wireguard_entry() {
    // Define a relay list containing exactly two Wireguard relays in Gothenburg.
    let relays = RelayList {
        etag: None,
        countries: vec![RelayListCountry {
            name: "Sweden".to_string(),
            code: "se".to_string(),
            cities: vec![RelayListCity {
                name: "Gothenburg".to_string(),
                code: "got".to_string(),
                latitude: 57.70887,
                longitude: 11.97456,
                relays: vec![
                    Relay {
                        hostname: "se9-wireguard".to_string(),
                        ipv4_addr_in: "185.213.154.68".parse().unwrap(),
                        ipv6_addr_in: Some("2a03:1b20:5:f011::a09f".parse().unwrap()),
                        include_in_country: true,
                        active: true,
                        owned: true,
                        provider: "provider0".to_string(),
                        weight: 1,
                        endpoint_data: RelayEndpointData::Wireguard(WireguardRelayEndpointData {
                            public_key: PublicKey::from_base64(
                                "BLNHNoGO88LjV/wDBa7CUUwUzPq/fO2UwcGLy56hKy4=",
                            )
                            .unwrap(),
                            daita: false,
                            shadowsocks_extra_addr_in: vec![],
                        }),
                        location: None,
                    },
                    Relay {
                        hostname: "se10-wireguard".to_string(),
                        ipv4_addr_in: "185.213.154.69".parse().unwrap(),
                        ipv6_addr_in: Some("2a03:1b20:5:f011::a10f".parse().unwrap()),
                        include_in_country: true,
                        active: true,
                        owned: false,
                        provider: "provider1".to_string(),
                        weight: 1,
                        endpoint_data: RelayEndpointData::Wireguard(WireguardRelayEndpointData {
                            public_key: PublicKey::from_base64(
                                "BLNHNoGO88LjV/wDBa7CUUwUzPq/fO2UwcGLy56hKy4=",
                            )
                            .unwrap(),
                            daita: false,
                            shadowsocks_extra_addr_in: vec![],
                        }),
                        location: None,
                    },
                ],
            }],
        }],
        openvpn: OpenVpnEndpointData { ports: vec![] },
        bridge: BridgeEndpointData {
            shadowsocks: vec![],
        },
        wireguard: WireguardEndpointData {
            port_ranges: vec![
                (53, 53),
                (443, 443),
                (4000, 33433),
                (33565, 51820),
                (52000, 60000),
            ],
            ipv4_gateway: "10.64.0.1".parse().unwrap(),
            ipv6_gateway: "fc00:bbbb:bbbb:bb01::1".parse().unwrap(),
            udp2tcp_ports: vec![],
            shadowsocks_port_ranges: vec![(100, 200), (1000, 2000)],
        },
    };

    let relay_selector = RelaySelector::from_list(SelectorConfig::default(), relays);
    let specific_hostname = "se10-wireguard";
    let specific_location = GeographicLocationConstraint::hostname("se", "got", specific_hostname);
    let general_location = GeographicLocationConstraint::city("se", "got");

    // general_location candidates: [se-09-wireguard, se-10-wireguard]
    // specific_location candidates: [se-10-wireguard]
    for _ in 0..100 {
        // Because the entry location constraint is more specific than the exit loation constraint,
        // the entry location should always become `specific_location`
        let query = RelayQueryBuilder::new()
            .wireguard()
            .location(general_location.clone())
            .multihop()
            .entry(specific_location.clone())
            .build();

        let relay = relay_selector.get_relay_by_query(query).unwrap();
        match relay {
            GetRelay::Wireguard {
                inner: WireguardConfig::Multihop { exit, entry },
                ..
            } => {
                assert_eq!(entry.hostname, specific_hostname);
                assert_ne!(exit.hostname, entry.hostname);
                assert_ne!(exit.ipv4_addr_in, entry.ipv4_addr_in);
            }
            wrong_relay => panic!(
            "Relay selector should have picked a Wireguard relay, instead chose {wrong_relay:?}"
        ),
        }
    }

    // general_location candidates: [se-09-wireguard, se-10-wireguard]
    // specific_location candidates: [se-10-wireguard]
    for _ in 0..100 {
        // Because the exit location constraint is more specific than the entry loation constraint,
        // the exit location should always become `specific_location`
        let query = RelayQueryBuilder::new()
            .wireguard()
            .location(specific_location.clone())
            .multihop()
            .entry(general_location.clone())
            .build();

        let relay = relay_selector.get_relay_by_query(query).unwrap();
        match relay {
            GetRelay::Wireguard {
                inner: WireguardConfig::Multihop { exit, entry },
                ..
            } => {
                assert_eq!(exit.hostname, specific_hostname);
                assert_ne!(exit.hostname, entry.hostname);
                assert_ne!(exit.ipv4_addr_in, entry.ipv4_addr_in);
            }
            wrong_relay => panic!(
            "Relay selector should have picked a Wireguard relay, instead chose {wrong_relay:?}"
        ),
        }
    }
}

/// If a Wireguard multihop constraint has the same entry and exit relay, the relay selector
/// should fail to come up with a valid configuration.
///
/// If instead the entry and exit relay are distinct, and assuming that the relays exist, the relay
/// selector should instead always return a valid configuration.
#[test]
fn test_wireguard_entry_hostname_collision() {
    let relay_selector = default_relay_selector();
    // Define two distinct Wireguard relays.
    let host1 = GeographicLocationConstraint::hostname("se", "got", "se9-wireguard");
    let host2 = GeographicLocationConstraint::hostname("se", "got", "se10-wireguard");

    let invalid_multihop_query = RelayQueryBuilder::new().wireguard()
        // Here we set `host1` to be the exit relay
        .location(host1.clone())
        .multihop()
        // .. and here we set `host1` to also be the entry relay!
        .entry(host1.clone())
        .build();

    // Assert that the same host cannot be used for entry and exit
    assert!(relay_selector
        .get_relay_by_query(invalid_multihop_query)
        .is_err());

    let valid_multihop_query = RelayQueryBuilder::new().wireguard()
        .location(host1)
        .multihop()
        // We correct the erroneous query by setting `host2` as the entry relay
        .entry(host2)
        .build();

    // Assert that the new query succeeds when the entry and exit hosts differ
    assert!(relay_selector
        .get_relay_by_query(valid_multihop_query)
        .is_ok())
}

/// Test that the relay selector:
/// * returns an OpenVPN relay given a constraint of a valid transport protocol + port combo
/// * does *not* return an OpenVPN relay given a constraint of an *invalid* transport protocol +
///   port combo
#[test]
fn test_openvpn_constraints() {
    let relay_selector = default_relay_selector();
    const ACTUAL_TCP_PORT: u16 = 443;
    const ACTUAL_UDP_PORT: u16 = 1194;
    const NON_EXISTENT_PORT: u16 = 1337;

    // Test all combinations of constraints, and whether they should
    // match some relay
    let constraint_combinations = [
        (RelayQueryBuilder::new().openvpn().build(), true),
        (
            RelayQueryBuilder::new()
                .openvpn()
                .transport_protocol(Udp)
                .build(),
            true,
        ),
        (
            RelayQueryBuilder::new()
                .openvpn()
                .transport_protocol(Tcp)
                .build(),
            true,
        ),
        (
            RelayQueryBuilder::new()
                .openvpn()
                .transport_protocol(Udp)
                .port(ACTUAL_UDP_PORT)
                .build(),
            true,
        ),
        (
            RelayQueryBuilder::new()
                .openvpn()
                .transport_protocol(Udp)
                .port(NON_EXISTENT_PORT)
                .build(),
            false,
        ),
        (
            RelayQueryBuilder::new()
                .openvpn()
                .transport_protocol(Tcp)
                .port(ACTUAL_TCP_PORT)
                .build(),
            true,
        ),
        (
            RelayQueryBuilder::new()
                .openvpn()
                .transport_protocol(Tcp)
                .port(NON_EXISTENT_PORT)
                .build(),
            false,
        ),
    ];

    let matches_constraints =
        |endpoint: Endpoint, constraints: &OpenVpnRelayQuery| match constraints.port {
            Constraint::Any => (),
            Constraint::Only(TransportPort { protocol, port }) => {
                assert_eq!(endpoint.protocol, protocol);
                match port {
                    Constraint::Any => (),
                    Constraint::Only(port) => assert_eq!(port, endpoint.address.port()),
                }
            }
        };

    for (query, should_match) in constraint_combinations.into_iter() {
        for _ in 0..100 {
            let relay: Result<_, Error> = relay_selector.get_relay_by_query(query.clone());
            if !should_match {
                relay.expect_err("Unexpected relay");
            } else {
                match relay.expect("Expected to find a relay") {
                    GetRelay::OpenVpn { endpoint, .. } =>  {
                        matches_constraints(endpoint, &query.openvpn_constraints);
                    },
                    wrong_relay => panic!("Relay selector should have picked an OpenVPN relay, instead chose {wrong_relay:?}")
                };
            }
        }
    }
}

/// Construct a query for multihop configuration and assert that the relay selector picks an
/// accompanying entry relay.
#[test]
fn test_selecting_wireguard_location_will_consider_multihop() {
    let relay_selector = default_relay_selector();

    for _ in 0..100 {
        let query = RelayQueryBuilder::new().wireguard().multihop().build();
        let relay = relay_selector.get_relay_by_query(query.clone()).unwrap();
        assert!(matches!(
            relay,
            GetRelay::Wireguard {
                inner: WireguardConfig::Multihop { .. },
                ..
            }
        ))
    }
}

/// Construct a query for multihop configuration, but the tunnel protocol is forcefully set to Any.
/// If a Wireguard relay is chosen, the relay selector should also pick an accompanying entry relay.
#[test]
fn test_selecting_any_relay_will_consider_multihop() {
    let relay_selector = default_relay_selector();
    let mut query = RelayQueryBuilder::new().wireguard().multihop().build();
    query.tunnel_protocol = Constraint::Any;

    for _ in 0..100 {
        let relay = relay_selector.get_relay_by_query(query.clone()).unwrap();
        assert!(matches!(relay, GetRelay::Wireguard { inner: WireguardConfig::Multihop { .. }, .. }),
            "Relay selector should have picked a Wireguard relay with multihop, instead chose {relay:?}"
        );
    }
}

/// Construct a query for a Wireguard configuration where UDP2TCP obfuscation is selected and
/// multihop is explicitly turned off. Assert that the relay selector always return an obfuscator
/// configuration.
#[test]
fn test_selecting_wireguard_endpoint_with_udp2tcp_obfuscation() {
    let relay_selector = default_relay_selector();
    let mut query = RelayQueryBuilder::new().wireguard().udp2tcp().build();
    query.wireguard_constraints.use_multihop = Constraint::Only(false);

    let relay = relay_selector.get_relay_by_query(query).unwrap();
    match relay {
        GetRelay::Wireguard {
            obfuscator,
            inner: WireguardConfig::Singlehop { .. },
            ..
        } => {
            assert!(obfuscator.is_some_and(|obfuscator| matches!(
                obfuscator.config,
                ObfuscatorConfig::Udp2Tcp { .. }
            )))
        }
        wrong_relay => panic!(
            "Relay selector should have picked a Wireguard relay, instead chose {wrong_relay:?}"
        ),
    }
}

/// Construct a query for a Wireguard configuration where UDP2TCP obfuscation is set to "Auto" and
/// multihop is explicitly turned off. Assert that the relay selector does *not* return an
/// obfuscator config.
///
/// [`RelaySelector::get_relay`] may still enable obfuscation if it is present in [`RETRY_ORDER`].
#[test]
fn test_selecting_wireguard_endpoint_with_auto_obfuscation() {
    let relay_selector = default_relay_selector();

    let mut query = RelayQueryBuilder::new().wireguard().build();
    query.wireguard_constraints.obfuscation = ObfuscationQuery::Auto;

    for _ in 0..100 {
        let relay = relay_selector.get_relay_by_query(query.clone()).unwrap();
        match relay {
            GetRelay::Wireguard { obfuscator, .. } => {
                assert!(obfuscator.is_none());
            }
            wrong_relay => panic!(
            "Relay selector should have picked a Wireguard relay, instead chose {wrong_relay:?}"
        ),
        }
    }
}

/// Construct a query for a Wireguard configuration with UDP2TCP obfuscation, and make sure that
/// all configurations contain a valid port.
#[test]
fn test_selected_wireguard_endpoints_use_correct_port_ranges() {
    const TCP2UDP_PORTS: [u16; 3] = [80, 443, 5001];
    let relay_selector = default_relay_selector();
    // Note that we do *not* specify any port here!
    let query = RelayQueryBuilder::new().wireguard().udp2tcp().build();

    for _ in 0..1000 {
        let relay = relay_selector.get_relay_by_query(query.clone()).unwrap();
        match relay {
            GetRelay::Wireguard {
                obfuscator,
                inner: WireguardConfig::Singlehop { .. },
                ..
            } => {
                let Some(obfuscator) = obfuscator else {
                    panic!("Relay selector should have picked an obfuscator")
                };
                assert!(matches!(obfuscator.config,
                    ObfuscatorConfig::Udp2Tcp { endpoint } if
                        TCP2UDP_PORTS.contains(&endpoint.port()),
                ))
            }
            wrong_relay => panic!(
            "Relay selector should have picked a Wireguard relay, instead chose {wrong_relay:?}"
        ),
        };
    }
}

/// Verify that any query which sets an explicit [`Ownership`] is respected by the relay selector.
#[test]
fn test_ownership() {
    let relay_selector = default_relay_selector();

    for _ in 0..100 {
        // Construct an arbitrary query for owned relays.
        let query = RelayQueryBuilder::new()
            .ownership(Ownership::MullvadOwned)
            .build();
        let relay = relay_selector.get_relay_by_query(query).unwrap();
        // Check that the relay is owned by Mullvad.
        assert!(unwrap_relay(relay).owned);
    }

    for _ in 0..100 {
        // Construct an arbitrary query for rented relays.
        let query = RelayQueryBuilder::new()
            .ownership(Ownership::Rented)
            .build();
        let relay = relay_selector.get_relay_by_query(query).unwrap();
        // Check that the relay is rented.
        assert!(!unwrap_relay(relay).owned);
    }
}

/// Verify that server and port selection varies between retry attempts.
#[test]
fn test_load_balancing() {
    const ATTEMPTS: usize = 100;
    let relay_selector = default_relay_selector();
    let location = GeographicLocationConstraint::country("se");
    for query in [
        RelayQueryBuilder::new().location(location.clone()).build(),
        RelayQueryBuilder::new()
            .wireguard()
            .location(location.clone())
            .build(),
        RelayQueryBuilder::new()
            .openvpn()
            .location(location)
            .build(),
    ] {
        // Collect the range of unique relay ports and IP addresses over a large number of queries.
        let (ports, ips): (HashSet<u16>, HashSet<std::net::IpAddr>) = std::iter::repeat(query.clone())
            .take(ATTEMPTS)
            // Execute the query
            .map(|query| relay_selector.get_relay_by_query(query).unwrap())
            // Perform some plumbing ..
            .map(unwrap_endpoint)
            .map(|endpoint| endpoint.to_endpoint().address)
            // Extract the selected relay's port + IP address
            .map(|endpoint| (endpoint.port(), endpoint.ip()))
            .unzip();

        assert!(
            ports.len() > 1,
            "expected more than 1 port, got {ports:?}, for tunnel protocol {tunnel_protocol:?}",
            tunnel_protocol = query.tunnel_protocol,
        );
        assert!(
            ips.len() > 1,
            "expected more than 1 server, got {ips:?}, for tunnel protocol {tunnel_protocol:?}",
            tunnel_protocol = query.tunnel_protocol,
        );
    }
}

/// Construct a query for a relay with specific providers and verify that every chosen relay has
/// the correct associated provider.
#[test]
fn test_providers() {
    const EXPECTED_PROVIDERS: [&str; 2] = ["provider0", "provider2"];
    let providers = Providers::new(EXPECTED_PROVIDERS).unwrap();
    let relay_selector = default_relay_selector();

    for _attempt in 0..100 {
        let query = RelayQueryBuilder::new()
            .providers(providers.clone())
            .build();
        let relay = relay_selector.get_relay_by_query(query).unwrap();

        match &relay {
            GetRelay::Wireguard { .. } => {
                let exit = unwrap_relay(relay);
                assert!(
                    EXPECTED_PROVIDERS.contains(&exit.provider.as_str()),
                    "cannot find provider {provider} in {EXPECTED_PROVIDERS:?}",
                    provider = exit.provider
                )
            }
            wrong_relay => panic!(
            "Relay selector should have picked a Wireguard relay, instead chose {wrong_relay:?}"
        ),
        };
    }
}

/// Verify that bridges are automatically used when bridge mode is set
/// to automatic.
#[test]
fn test_openvpn_auto_bridge() {
    let relay_selector = default_relay_selector();
    let retry_order = [
        // This attempt should not use bridge
        RelayQueryBuilder::new().openvpn().build(),
        // This attempt should use a bridge
        RelayQueryBuilder::new().openvpn().bridge().build(),
    ];

    for (retry_attempt, query) in retry_order
        .iter()
        .cycle()
        .enumerate()
        .take(100 * retry_order.len())
    {
        let relay = relay_selector
            .get_relay_with_custom_params(retry_attempt, &retry_order, RuntimeParameters::default())
            .unwrap();
        match relay {
            GetRelay::OpenVpn { bridge, .. } => {
                if BridgeQuery::should_use_bridge(&query.openvpn_constraints.bridge_settings) {
                    assert!(bridge.is_some())
                } else {
                    assert!(bridge.is_none())
                }
            }
            wrong_relay => panic!(
                "Relay selector should have picked an OpenVPN relay, instead chose {wrong_relay:?}"
            ),
        }
    }
}

/// Ensure that `include_in_country` is ignored if all relays have it set to false (i.e., some
/// relay is returned). Also ensure that `include_in_country` is respected if some relays
/// have it set to true (i.e., that relay is never returned)
#[test]
fn test_include_in_country() {
    let mut relay_list = RelayList {
        etag: None,
        countries: vec![RelayListCountry {
            name: "Sweden".to_string(),
            code: "se".to_string(),
            cities: vec![RelayListCity {
                name: "Gothenburg".to_string(),
                code: "got".to_string(),
                latitude: 57.70887,
                longitude: 11.97456,
                relays: vec![
                    Relay {
                        hostname: "se9-wireguard".to_string(),
                        ipv4_addr_in: "185.213.154.68".parse().unwrap(),
                        ipv6_addr_in: Some("2a03:1b20:5:f011::a09f".parse().unwrap()),
                        include_in_country: false,
                        active: true,
                        owned: true,
                        provider: "31173".to_string(),
                        weight: 1,
                        endpoint_data: RelayEndpointData::Wireguard(WireguardRelayEndpointData {
                            public_key: PublicKey::from_base64(
                                "BLNHNoGO88LjV/wDBa7CUUwUzPq/fO2UwcGLy56hKy4=",
                            )
                            .unwrap(),
                            shadowsocks_extra_addr_in: vec![],
                            daita: false,
                        }),
                        location: None,
                    },
                    Relay {
                        hostname: "se10-wireguard".to_string(),
                        ipv4_addr_in: "185.213.154.69".parse().unwrap(),
                        ipv6_addr_in: Some("2a03:1b20:5:f011::a10f".parse().unwrap()),
                        include_in_country: false,
                        active: true,
                        owned: false,
                        provider: "31173".to_string(),
                        weight: 1,
                        endpoint_data: RelayEndpointData::Wireguard(WireguardRelayEndpointData {
                            public_key: PublicKey::from_base64(
                                "BLNHNoGO88LjV/wDBa7CUUwUzPq/fO2UwcGLy56hKy4=",
                            )
                            .unwrap(),
                            shadowsocks_extra_addr_in: vec![],
                            daita: false,
                        }),
                        location: None,
                    },
                ],
            }],
        }],
        openvpn: OpenVpnEndpointData {
            ports: vec![
                OpenVpnEndpoint {
                    port: 1194,
                    protocol: Udp,
                },
                OpenVpnEndpoint {
                    port: 443,
                    protocol: Tcp,
                },
                OpenVpnEndpoint {
                    port: 80,
                    protocol: Tcp,
                },
            ],
        },
        bridge: BridgeEndpointData {
            shadowsocks: vec![],
        },
        wireguard: WireguardEndpointData {
            port_ranges: vec![(53, 53), (4000, 33433), (33565, 51820), (52000, 60000)],
            ipv4_gateway: "10.64.0.1".parse().unwrap(),
            ipv6_gateway: "fc00:bbbb:bbbb:bb01::1".parse().unwrap(),
            udp2tcp_ports: vec![],
            shadowsocks_port_ranges: vec![],
        },
    };

    // If include_in_country is false for all relays, a relay must be selected anyway.
    let relay_selector = RelaySelector::from_list(SelectorConfig::default(), relay_list.clone());
    assert!(relay_selector
        .get_relay(0, RuntimeParameters::default())
        .is_ok());

    // If include_in_country is true for some relay, it must always be selected.
    relay_list.countries[0].cities[0].relays[0].include_in_country = true;
    let expected_hostname = relay_list.countries[0].cities[0].relays[0].hostname.clone();
    let relay_selector = RelaySelector::from_list(SelectorConfig::default(), relay_list);
    let relay = unwrap_relay(
        relay_selector
            .get_relay(0, RuntimeParameters::default())
            .expect("expected match"),
    );

    assert!(
        matches!(relay, Relay { ref hostname, .. } if hostname == &expected_hostname),
        "found {relay:?}, expected {expected_hostname:?}",
    )
}

/// Verify that the relay selector ignores bridge state when WireGuard should be used.
#[test]
fn ignore_bridge_state_when_wireguard_is_used() {
    // Note: The location implies a Wireguard relay.
    let location = GeographicLocationConstraint::hostname("se", "got", "se10-wireguard");
    // .. while the query otherwise does not.
    let query = RelayQueryBuilder::new().location(location).build();
    let config = SelectorConfig {
        bridge_state: BridgeState::On,
        ..SelectorConfig::default()
    };
    let relay_selector = RelaySelector::from_list(config, RELAYS.clone());
    for _ in 0..100 {
        let relay = relay_selector.get_relay_by_query(query.clone()).unwrap();
        let tunnel_type = tunnel_type(&unwrap_relay(relay));
        assert_eq!(tunnel_type, TunnelType::Wireguard);
    }
}

/// Handle bridge setting when falling back on OpenVPN
#[test]
fn openvpn_handle_bridge_settings() {
    // First, construct a query to choose an OpenVPN relay to talk to over UDP.
    let mut query = RelayQueryBuilder::new()
        .openvpn()
        .transport_protocol(Udp)
        .build();

    let config = SelectorConfig {
        bridge_state: BridgeState::On,
        ..SelectorConfig::default()
    };
    let relay_selector = RelaySelector::from_list(config, RELAYS.clone());
    let relay = relay_selector.get_relay_by_query(query.clone()).unwrap();
    // Assert that the resulting relay uses UDP.
    match relay {
        GetRelay::OpenVpn { endpoint, .. } => {
            assert_eq!(endpoint.protocol, Udp);
        }
        wrong_relay => panic!(
            "Relay selector should have picked an OpenVPN relay, instead chose {wrong_relay:?}"
        ),
    }
    // Tweaking the query just slightly to try to enable bridge mode, while sill using UDP,
    // should fail.
    query.openvpn_constraints.bridge_settings =
        Constraint::Only(BridgeQuery::Normal(BridgeConstraints::default()));
    let relay = relay_selector.get_relay_by_query(query.clone());
    assert!(relay.is_err());

    // Correcting the query to use TCP, the relay selector should yield a valid relay + bridge
    query.openvpn_constraints.port = Constraint::Only(TransportPort {
        protocol: Tcp,
        port: Constraint::default(),
    });
    let relay = relay_selector.get_relay_by_query(query.clone()).unwrap();
    match relay {
        GetRelay::OpenVpn {
            endpoint, bridge, ..
        } => {
            assert!(bridge.is_some());
            assert_eq!(endpoint.protocol, Tcp);
        }
        wrong_relay => panic!(
            "Relay selector should have picked an OpenVPN relay, instead chose {wrong_relay:?}"
        ),
    };
}

/// Verify that the relay selector correctly gives back an OpenVPN relay + bridge when the user's
/// settings indicate that bridge mode is on, but the transport protocol is set to auto. Note that
/// it is only valid to use TCP with bridges. Trying to use UDP over bridges is not allowed, and
/// the relay selector should fail to select a relay in these cases.
#[test]
fn openvpn_bridge_with_automatic_transport_protocol() {
    // Enable bridge mode.
    let config = SelectorConfig {
        bridge_state: BridgeState::On,
        ..SelectorConfig::default()
    };
    let relay_selector = RelaySelector::from_list(config, RELAYS.clone());

    // First, construct a query to choose an OpenVPN relay and bridge.
    let mut query = RelayQueryBuilder::new().openvpn().bridge().build();
    // Forcefully modify the transport protocol, as the builder will ensure that the transport
    // protocol is set to TCP.
    query.openvpn_constraints.port = Constraint::Any;

    for _ in 0..100 {
        let relay = relay_selector.get_relay_by_query(query.clone()).unwrap();
        // Assert that the relay selector is able to cope with the transport protocol being set to
        // auto.
        match relay {
            GetRelay::OpenVpn { endpoint, .. } => {
                assert_eq!(endpoint.protocol, Tcp);
            }
            wrong_relay => panic!(
                "Relay selector should have picked an OpenVPN relay, instead chose {wrong_relay:?}"
            ),
        }
    }

    // Modify the query slightly to forcefully use UDP. This should not be allowed!
    let query = RelayQueryBuilder::new()
        .openvpn()
        .bridge()
        .transport_protocol(Udp)
        .build();
    for _ in 0..100 {
        let relay = relay_selector.get_relay_by_query(query.clone());
        assert!(relay.is_err())
    }
}

/// Return only entry relays that support DAITA when DAITA filtering is enabled. All relays that
/// support DAITA also support NOT DAITA. Thus, disabling it should not cause any WireGuard relays
/// to be filtered out.
#[test]
fn test_daita() {
    let relay_list = RelayList {
        etag: None,
        countries: vec![RelayListCountry {
            name: "Sweden".to_string(),
            code: "se".to_string(),
            cities: vec![RelayListCity {
                name: "Gothenburg".to_string(),
                code: "got".to_string(),
                latitude: 57.70887,
                longitude: 11.97456,
                relays: vec![
                    Relay {
                        hostname: "se9-wireguard".to_string(),
                        ipv4_addr_in: "185.213.154.68".parse().unwrap(),
                        ipv6_addr_in: Some("2a03:1b20:5:f011::a09f".parse().unwrap()),
                        include_in_country: true,
                        active: true,
                        owned: true,
                        provider: "31173".to_string(),
                        weight: 1,
                        endpoint_data: RelayEndpointData::Wireguard(WireguardRelayEndpointData {
                            public_key: PublicKey::from_base64(
                                "BLNHNoGO88LjV/wDBa7CUUwUzPq/fO2UwcGLy56hKy4=",
                            )
                            .unwrap(),
                            shadowsocks_extra_addr_in: vec![],
                            daita: false,
                        }),
                        location: None,
                    },
                    Relay {
                        hostname: "se10-wireguard".to_string(),
                        ipv4_addr_in: "185.213.154.69".parse().unwrap(),
                        ipv6_addr_in: Some("2a03:1b20:5:f011::a10f".parse().unwrap()),
                        include_in_country: true,
                        active: true,
                        owned: false,
                        provider: "31173".to_string(),
                        weight: 1,
                        endpoint_data: RelayEndpointData::Wireguard(WireguardRelayEndpointData {
                            public_key: PublicKey::from_base64(
                                "BLNHNoGO88LjV/wDBa7CUUwUzPq/fO2UwcGLy56hKy4=",
                            )
                            .unwrap(),
                            shadowsocks_extra_addr_in: vec![],
                            daita: true,
                        }),
                        location: None,
                    },
                ],
            }],
        }],
        openvpn: OpenVpnEndpointData { ports: vec![] },
        bridge: BridgeEndpointData {
            shadowsocks: vec![],
        },
        wireguard: WireguardEndpointData {
            port_ranges: vec![(53, 53), (4000, 33433), (33565, 51820), (52000, 60000)],
            ipv4_gateway: "10.64.0.1".parse().unwrap(),
            ipv6_gateway: "fc00:bbbb:bbbb:bb01::1".parse().unwrap(),
            shadowsocks_port_ranges: vec![],
            udp2tcp_ports: vec![],
        },
    };

    let daita_supporting_relay =
        GeographicLocationConstraint::hostname("se", "got", "se10-wireguard");
    let nondaita_supporting_relay =
        GeographicLocationConstraint::hostname("se", "got", "se9-wireguard");

    let relay_selector = RelaySelector::from_list(SelectorConfig::default(), relay_list);

    // Only pick relays that support DAITA
    let query = RelayQueryBuilder::new().wireguard().daita().build();
    let relay = unwrap_entry_relay(relay_selector.get_relay_by_query(query).unwrap());
    assert!(
        supports_daita(&relay),
        "Selector supported relay that does not support DAITA: {relay:?}"
    );

    // Fail when only non-DAITA relays match constraints
    let query = RelayQueryBuilder::new()
        .wireguard()
        .daita()
        .location(nondaita_supporting_relay.clone())
        .build();
    relay_selector
        .get_relay_by_query(query)
        .expect_err("Expected to find no matching relay");

    // DAITA-supporting relays can be picked even when it is disabled
    let query = RelayQueryBuilder::new()
        .wireguard()
        .location(daita_supporting_relay.clone())
        .build();
    relay_selector
        .get_relay_by_query(query)
        .expect("Expected DAITA-supporting relay to work without DAITA");

    // Non DAITA-supporting relays can be picked when it is disabled
    let query = RelayQueryBuilder::new()
        .wireguard()
        .location(nondaita_supporting_relay.clone())
        .build();
    relay_selector
        .get_relay_by_query(query)
        .expect("Expected DAITA-supporting relay to work without DAITA");

    // Entry relay must support daita
    let query = RelayQueryBuilder::new()
        .wireguard()
        .daita()
        .multihop()
        .build();
    let relay = relay_selector.get_relay_by_query(query).unwrap();
    match relay {
        GetRelay::Wireguard {
            inner: WireguardConfig::Multihop { exit: _, entry },
            ..
        } => {
            assert!(supports_daita(&entry), "entry relay must support DAITA");
        }
        wrong_relay => panic!(
            "Relay selector should have picked a Wireguard relay, instead chose {wrong_relay:?}"
        ),
    }

    // Exit relay does not have to support daita
    let query = RelayQueryBuilder::new()
        .wireguard()
        .daita()
        .multihop()
        .location(nondaita_supporting_relay)
        .build();
    let relay = relay_selector.get_relay_by_query(query).unwrap();
    match relay {
        GetRelay::Wireguard {
            inner: WireguardConfig::Multihop { exit, entry: _ },
            ..
        } => {
            assert!(
                !supports_daita(&exit),
                "expected non DAITA-supporting exit relay, got {exit:?}"
            );
        }
        wrong_relay => panic!(
            "Relay selector should have picked a Wireguard relay, instead chose {wrong_relay:?}"
        ),
    }
}

/// Check that if  the original user query would yield a relay, the result of running the query
/// which is the intersection between the user query and any of the default queries shall never
/// fail.
#[test]
fn valid_user_setting_should_yield_relay() {
    // Make a valid user relay constraint
    let location = GeographicLocationConstraint::hostname("se", "got", "se9-wireguard");
    let user_query = RelayQueryBuilder::new().location(location.clone()).build();
    let user_constraints = RelayQueryBuilder::new()
        .location(location.clone())
        .into_constraint();

    let config = SelectorConfig {
        relay_settings: user_constraints.into(),
        ..SelectorConfig::default()
    };
    let relay_selector = RelaySelector::from_list(config, RELAYS.clone());
    let user_result = relay_selector.get_relay_by_query(user_query.clone());
    for retry_attempt in 0..RETRY_ORDER.len() {
        let post_unification_result =
            relay_selector.get_relay(retry_attempt, RuntimeParameters::default());
        if user_result.is_ok() {
            assert!(post_unification_result.is_ok(), "Expected Post-unification query to be valid because original query {:#?} yielded a connection configuration", user_query)
        }
    }
}

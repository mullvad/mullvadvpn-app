//! Tests for verifying that the relay selector works as expected.

use rand::{SeedableRng, rngs::SmallRng, seq::IndexedRandom};
use std::{
    collections::HashSet,
    net::{IpAddr, Ipv4Addr, Ipv6Addr},
    sync::LazyLock,
};
use talpid_types::net::{
    IpVersion,
    TransportProtocol::{Tcp, Udp},
    obfuscation::{ObfuscatorConfig, Obfuscators},
    wireguard::PublicKey,
};

use mullvad_relay_selector::{
    Error, GetRelay, RETRY_ORDER, RelaySelector, SelectedObfuscator, SelectorConfig,
    WireguardConfig,
    query::{ObfuscationQuery, builder::RelayQueryBuilder},
};
use mullvad_types::{
    endpoint::MullvadEndpoint,
    location::Location,
    relay_constraints::{GeographicLocationConstraint, Ownership, Providers, RelayOverride},
    relay_list::{
        BridgeEndpointData, EndpointData, Quic, Relay, RelayEndpointData, RelayList, RelayListCity,
        RelayListCountry, ShadowsocksEndpointData, WireguardRelayEndpointData,
    },
};
use vec1::vec1;

static DUMMY_LOCATION: LazyLock<Location> = LazyLock::new(|| Location {
    country: "Sweden".to_string(),
    country_code: "se".to_string(),
    city: "Gothenburg".to_string(),
    city_code: "got".to_string(),
    latitude: 57.71,
    longitude: 11.97,
});

static WIREGUARD_PUBKEY: LazyLock<PublicKey> = LazyLock::new(|| {
    PublicKey::from_base64("BLNHNoGO88LjV/wDBa7CUUwUzPq/fO2UwcGLy56hKy4=").unwrap()
});

static RELAYS: LazyLock<RelayList> = LazyLock::new(|| RelayList {
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
                    overridden_ipv4: false,
                    overridden_ipv6: false,
                    include_in_country: true,
                    active: true,
                    owned: true,
                    provider: "provider0".to_string(),
                    weight: 1,
                    endpoint_data: RelayEndpointData::Wireguard(
                        WireguardRelayEndpointData::new(WIREGUARD_PUBKEY.clone())
                            .set_daita(true)
                            .set_quic(Quic::new(
                                vec1![
                                    "185.213.154.68".parse().unwrap(),
                                    "2a03:1b20:5:f011::a09f".parse().unwrap(),
                                ],
                                "Bearer test".to_owned(),
                                "se9-wireguard.blockerad.eu".to_owned(),
                            ))
                            .set_lwo(true),
                    ),
                    location: DUMMY_LOCATION.clone(),
                },
                Relay {
                    hostname: "se10-wireguard".to_string(),
                    ipv4_addr_in: "185.213.154.69".parse().unwrap(),
                    ipv6_addr_in: Some("2a03:1b20:5:f011::a10f".parse().unwrap()),
                    overridden_ipv4: false,
                    overridden_ipv6: false,
                    include_in_country: true,
                    active: true,
                    owned: false,
                    provider: "provider1".to_string(),
                    weight: 1,
                    endpoint_data: RelayEndpointData::Wireguard(WireguardRelayEndpointData::new(
                        PublicKey::from_base64("BLNHNoGO88LjV/wDBa7CUUwUzPq/fO2UwcGLy56hKy4=")
                            .unwrap(),
                    )),
                    location: DUMMY_LOCATION.clone(),
                },
                Relay {
                    hostname: "se11-wireguard".to_string(),
                    ipv4_addr_in: "185.213.154.69".parse().unwrap(),
                    ipv6_addr_in: Some("2a03:1b20:5:f011::a11f".parse().unwrap()),
                    overridden_ipv4: false,
                    overridden_ipv6: false,
                    include_in_country: true,
                    active: true,
                    owned: false,
                    provider: "provider2".to_string(),
                    weight: 1,
                    endpoint_data: RelayEndpointData::Wireguard(
                        WireguardRelayEndpointData::new(
                            PublicKey::from_base64("BLNHNoGO88LjV/wDBa7CUUwUzPq/fO2UwcGLy56hKy4=")
                                .unwrap(),
                        )
                        .set_daita(true),
                    ),
                    location: DUMMY_LOCATION.clone(),
                },
                Relay {
                    hostname: "se-got-br-001".to_string(),
                    ipv4_addr_in: "1.3.3.7".parse().unwrap(),
                    ipv6_addr_in: None,
                    overridden_ipv4: false,
                    overridden_ipv6: false,
                    include_in_country: true,
                    active: true,
                    owned: true,
                    provider: "provider3".to_string(),
                    weight: 1,
                    endpoint_data: RelayEndpointData::Bridge,
                    location: DUMMY_LOCATION.clone(),
                },
                SHADOWSOCKS_RELAY.clone(),
            ],
        }],
    }],
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
    wireguard: EndpointData {
        port_ranges: vec![
            53..=53,
            443..=443,
            4000..=33433,
            33565..=51820,
            52000..=60000,
        ],
        ipv4_gateway: "10.64.0.1".parse().unwrap(),
        ipv6_gateway: "fc00:bbbb:bbbb:bb01::1".parse().unwrap(),
        udp2tcp_ports: vec![],
        shadowsocks_port_ranges: vec![100..=200, 1000..=2000],
    },
});

static DAITA_RELAY_LOCATION: LazyLock<GeographicLocationConstraint> =
    LazyLock::new(|| GeographicLocationConstraint::hostname("se", "got", "se9-wireguard"));
static NON_DAITA_RELAY_LOCATION: LazyLock<GeographicLocationConstraint> =
    LazyLock::new(|| GeographicLocationConstraint::hostname("se", "got", "se10-wireguard"));

/// A Shadowsocks relay with additional addresses
static SHADOWSOCKS_RELAY: LazyLock<Relay> = LazyLock::new(|| Relay {
    hostname: SHADOWSOCKS_RELAY_LOCATION
        .get_hostname()
        .unwrap()
        .to_owned(),
    ipv4_addr_in: SHADOWSOCKS_RELAY_IPV4,
    ipv6_addr_in: Some(SHADOWSOCKS_RELAY_IPV6),
    overridden_ipv4: false,
    overridden_ipv6: false,
    include_in_country: true,
    active: true,
    owned: true,
    provider: "provider0".to_string(),
    weight: 1,
    endpoint_data: RelayEndpointData::Wireguard(
        WireguardRelayEndpointData::new(
            PublicKey::from_base64("eaNHNoGO88LjV/wDBa7CUUwUzPq/fO2UwcGLy56hKy4=").unwrap(),
        )
        .add_shadowsocks_extra_in_addrs(SHADOWSOCKS_RELAY_EXTRA_ADDRS.iter().copied()),
    ),
    location: DUMMY_LOCATION.clone(),
});
const SHADOWSOCKS_RELAY_IPV4: Ipv4Addr = Ipv4Addr::new(123, 123, 123, 1);
const SHADOWSOCKS_RELAY_IPV6: Ipv6Addr = Ipv6Addr::new(0x123, 0, 0, 0, 0, 0, 0, 2);
const SHADOWSOCKS_RELAY_EXTRA_ADDRS: &[IpAddr; 2] = &[
    IpAddr::V4(Ipv4Addr::new(123, 123, 123, 2)),
    IpAddr::V6(Ipv6Addr::new(0x123, 0, 0, 0, 0, 0, 0, 2)),
];
static SHADOWSOCKS_RELAY_LOCATION: LazyLock<GeographicLocationConstraint> =
    LazyLock::new(|| GeographicLocationConstraint::hostname("se", "got", "se1337-wireguard"));

// Helper functions
fn unwrap_relay(get_result: GetRelay) -> Relay {
    match get_result {
        GetRelay::Mullvad { inner, .. } => match inner {
            crate::WireguardConfig::Singlehop { exit } => exit,
            crate::WireguardConfig::Multihop { exit, .. } => exit,
        },
        GetRelay::Custom(custom) => {
            panic!("Can not extract regular relay from custom relay: {custom}")
        }
    }
}

fn unwrap_entry_relay(get_result: GetRelay) -> Relay {
    match get_result {
        GetRelay::Mullvad { inner, .. } => match inner {
            crate::WireguardConfig::Singlehop { exit } => exit,
            crate::WireguardConfig::Multihop { entry, .. } => entry,
        },
        GetRelay::Custom(custom) => {
            panic!("Can not extract regular relay from custom relay: {custom}")
        }
    }
}

fn unwrap_multihop_entry_exit_relays(get_result: GetRelay) -> (Relay, Relay) {
    match get_result {
        GetRelay::Mullvad {
            inner: crate::WireguardConfig::Multihop { entry, exit },
            ..
        } => (entry, exit),
        relay => {
            panic!("Relay is not a Wireguard multihop relay: {relay:?}")
        }
    }
}

fn unwrap_endpoint(get_result: GetRelay) -> MullvadEndpoint {
    match get_result {
        GetRelay::Mullvad { endpoint, .. } => endpoint,
        GetRelay::Custom(custom) => {
            panic!("Can not extract Mullvad endpoint from custom relay: {custom}")
        }
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

/// This is not an actual test. Rather, it serves as a reminder that if [`RETRY_ORDER`] is
/// modified, the programmer should be made aware to update all external documents which rely on the
/// retry order to be correct.
///
/// When all necessary changes have been made, feel free to update this test to mirror the new
/// [`RETRY_ORDER`].
#[test]
fn assert_retry_order() {
    use talpid_types::net::IpVersion;
    let expected_retry_order = vec![
        // 1 (wireguard)
        RelayQueryBuilder::new().build(),
        // 2
        RelayQueryBuilder::new().ip_version(IpVersion::V6).build(),
        // 3
        RelayQueryBuilder::new().shadowsocks().build(),
        // 4
        RelayQueryBuilder::new().quic().build(),
        // 5
        RelayQueryBuilder::new().udp2tcp().build(),
        // 6
        RelayQueryBuilder::new()
            .udp2tcp()
            .ip_version(IpVersion::V6)
            .build(),
        // 7
        RelayQueryBuilder::new().lwo().build(),
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

/// Test whether the relay selector seems to respect the order as defined by
/// [`RETRY_ORDER`].
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
            .get_relay(
                retry_attempt,
                talpid_types::net::IpAvailability::Ipv4AndIpv6,
            )
            .unwrap_or_else(|_| panic!("Retry attempt {retry_attempt} did not yield any relay"));
        // Then perform some protocol-specific probing as well.
        match relay {
            GetRelay::Mullvad {
                endpoint,
                obfuscator,
                ..
            } => {
                assert!(query.wireguard_constraints().ip_version.matches_eq(
                    &match endpoint.peer.endpoint.ip() {
                        std::net::IpAddr::V4(_) => talpid_types::net::IpVersion::V4,
                        std::net::IpAddr::V6(_) => talpid_types::net::IpVersion::V6,
                    }
                ));
                assert!(
                    query
                        .wireguard_constraints()
                        .port
                        .matches_eq(&endpoint.peer.endpoint.port())
                );
                assert!(match &query.wireguard_constraints().obfuscation {
                    ObfuscationQuery::Auto => true,
                    ObfuscationQuery::Off | ObfuscationQuery::Port => obfuscator.is_none(),
                    ObfuscationQuery::Quic
                    | ObfuscationQuery::Udp2tcp(_)
                    | ObfuscationQuery::Shadowsocks(_)
                    | ObfuscationQuery::Lwo => obfuscator.is_some(),
                });
            }
            _ => unreachable!(),
        }
    }
}

/// Assert that the relay selector does *not* return a multihop configuration where the exit and
/// entry relay are the same, even if the constraints would allow for it. Also verify that the relay
/// selector is smart enough to pick either the entry or exit relay first depending on which one
/// ends up yielding a valid configuration.
#[test]
fn test_entry() {
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
                        overridden_ipv4: false,
                        overridden_ipv6: false,
                        include_in_country: true,
                        active: true,
                        owned: true,
                        provider: "provider0".to_string(),
                        weight: 1,
                        endpoint_data: RelayEndpointData::Wireguard(
                            WireguardRelayEndpointData::new(WIREGUARD_PUBKEY.clone()),
                        ),
                        location: DUMMY_LOCATION.clone(),
                    },
                    Relay {
                        hostname: "se10-wireguard".to_string(),
                        ipv4_addr_in: "185.213.154.69".parse().unwrap(),
                        ipv6_addr_in: Some("2a03:1b20:5:f011::a10f".parse().unwrap()),
                        overridden_ipv4: false,
                        overridden_ipv6: false,
                        include_in_country: true,
                        active: true,
                        owned: false,
                        provider: "provider1".to_string(),
                        weight: 1,
                        endpoint_data: RelayEndpointData::Wireguard(
                            WireguardRelayEndpointData::new(WIREGUARD_PUBKEY.clone()),
                        ),
                        location: DUMMY_LOCATION.clone(),
                    },
                ],
            }],
        }],
        bridge: BridgeEndpointData {
            shadowsocks: vec![],
        },
        wireguard: EndpointData {
            port_ranges: vec![
                53..=53,
                443..=443,
                4000..=33433,
                33565..=51820,
                52000..=60000,
            ],
            ipv4_gateway: "10.64.0.1".parse().unwrap(),
            ipv6_gateway: "fc00:bbbb:bbbb:bb01::1".parse().unwrap(),
            udp2tcp_ports: vec![],
            shadowsocks_port_ranges: vec![100..=200, 1000..=2000],
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
            .location(general_location.clone())
            .multihop()
            .entry(specific_location.clone())
            .build();

        let relay = relay_selector.get_relay_by_query(query).unwrap();
        match relay {
            GetRelay::Mullvad {
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
            .location(specific_location.clone())
            .multihop()
            .entry(general_location.clone())
            .build();

        let relay = relay_selector.get_relay_by_query(query).unwrap();
        match relay {
            GetRelay::Mullvad {
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

/// If a multihop constraint has the same entry and exit relay, the relay selector
/// should fail to come up with a valid configuration.
///
/// If instead the entry and exit relay are distinct, and assuming that the relays exist, the relay
/// selector should instead always return a valid configuration.
#[test]
fn test_entry_hostname_collision() {
    let relay_selector = default_relay_selector();
    // Define two distinct Wireguard relays.
    let host1 = GeographicLocationConstraint::hostname("se", "got", "se9-wireguard");
    let host2 = GeographicLocationConstraint::hostname("se", "got", "se10-wireguard");

    let invalid_multihop_query = RelayQueryBuilder::new()
        // Here we set `host1` to be the exit relay
        .location(host1.clone())
        .multihop()
        // .. and here we set `host1` to also be the entry relay!
        .entry(host1.clone())
        .build();

    // Assert that the same host cannot be used for entry and exit
    assert!(
        relay_selector
            .get_relay_by_query(invalid_multihop_query)
            .is_err()
    );

    let valid_multihop_query = RelayQueryBuilder::new()
        .location(host1)
        .multihop()
        // We correct the erroneous query by setting `host2` as the entry relay
        .entry(host2)
        .build();

    // Assert that the new query succeeds when the entry and exit hosts differ
    assert!(
        relay_selector
            .get_relay_by_query(valid_multihop_query)
            .is_ok()
    )
}

/// Construct a query for multihop configuration and assert that the relay selector picks an
/// accompanying entry relay.
#[test]
fn test_selecting_location_will_consider_multihop() {
    let relay_selector = default_relay_selector();

    for _ in 0..100 {
        let query = RelayQueryBuilder::new().multihop().build();
        let relay = relay_selector.get_relay_by_query(query.clone()).unwrap();
        assert!(matches!(
            relay,
            GetRelay::Mullvad {
                inner: WireguardConfig::Multihop { .. },
                ..
            }
        ))
    }
}

/// Test whether Shadowsocks is always selected as the obfuscation protocol when Shadowsocks is
/// selected.
#[test]
fn test_selecting_over_shadowsocks() {
    let relay_selector = RelaySelector::from_list(SelectorConfig::default(), RELAYS.clone());

    let query = RelayQueryBuilder::new().shadowsocks().build();
    assert!(!query.wireguard_constraints().multihop());

    let relay = relay_selector.get_relay_by_query(query).unwrap();
    match relay {
        GetRelay::Mullvad {
            obfuscator,
            inner: WireguardConfig::Singlehop { .. },
            ..
        } => {
            assert!(obfuscator.is_some_and(|obfuscator| matches!(
                obfuscator.config,
                Obfuscators::Single(ObfuscatorConfig::Shadowsocks { .. })
            )))
        }
        wrong_relay => panic!(
            "Relay selector should have picked a Wireguard relay with Shadowsocks, instead chose {wrong_relay:?}"
        ),
    }
}

/// Test whether extra Shadowsocks IPs are selected when available
#[test]
fn test_selecting_over_shadowsocks_extra_ips() {
    let relay_selector = RelaySelector::from_list(SelectorConfig::default(), RELAYS.clone());

    let query = RelayQueryBuilder::new()
        .location(SHADOWSOCKS_RELAY_LOCATION.clone())
        .shadowsocks()
        .build();
    assert!(!query.wireguard_constraints().multihop());

    let relay = relay_selector.get_relay_by_query(query).unwrap();
    match relay {
        GetRelay::Mullvad {
            obfuscator:
                Some(SelectedObfuscator {
                    config: Obfuscators::Single(ObfuscatorConfig::Shadowsocks { endpoint }),
                    ..
                }),
            inner: WireguardConfig::Singlehop { exit },
            ..
        } => {
            assert!(!exit.overridden_ipv4);
            assert!(!exit.overridden_ipv6);
            assert!(
                SHADOWSOCKS_RELAY_EXTRA_ADDRS.contains(&endpoint.ip()),
                "{endpoint} is not an additional IP"
            );
        }
        wrong_relay => panic!(
            "Relay selector should have picked a Mullvad relay with Shadowsocks, instead chose {wrong_relay:?}"
        ),
    }
}

/// Test whether Quic is always selected as the obfuscation protocol when Quic is selected.
#[test]
fn test_selecting_over_quic() {
    let relay_selector = RelaySelector::from_list(SelectorConfig::default(), RELAYS.clone());

    let query = RelayQueryBuilder::new().quic().build();
    assert!(!query.wireguard_constraints().multihop());

    let relay = relay_selector.get_relay_by_query(query).unwrap();
    match relay {
        GetRelay::Mullvad {
            obfuscator,
            inner: WireguardConfig::Singlehop { .. },
            ..
        } => {
            assert!(obfuscator.is_some_and(|obfuscator| matches!(
                obfuscator.config,
                Obfuscators::Single(ObfuscatorConfig::Quic { .. }),
            )))
        }
        wrong_relay => panic!(
            "Relay selector should have picked a Mullvad relay with Quic, instead chose {wrong_relay:?}"
        ),
    }
}

/// Test LWO relay selection
#[test]
fn test_selecting_over_lwo() {
    let relay_selector = RelaySelector::from_list(SelectorConfig::default(), RELAYS.clone());

    let query = RelayQueryBuilder::new().lwo().build();
    assert!(!query.wireguard_constraints().multihop());

    let relay = relay_selector.get_relay_by_query(query).unwrap();
    match relay {
        GetRelay::Mullvad {
            obfuscator,
            inner: WireguardConfig::Singlehop { .. },
            ..
        } => {
            assert!(obfuscator.is_some_and(|obfuscator| matches!(
                obfuscator.config,
                Obfuscators::Single(ObfuscatorConfig::Lwo { .. }),
            )))
        }
        wrong_relay => panic!(
            "Relay selector should have picked a Mullvad relay with LWO, instead chose {wrong_relay:?}"
        ),
    }
}

/// Ignore extra IPv4 addresses when overrides are set
#[test]
fn test_selecting_ignore_extra_ips_override_v4() {
    const OVERRIDE_IPV4: Ipv4Addr = Ipv4Addr::new(1, 3, 3, 7);

    let config = mullvad_relay_selector::SelectorConfig {
        relay_overrides: vec![RelayOverride {
            hostname: SHADOWSOCKS_RELAY_LOCATION.get_hostname().unwrap().clone(),
            ipv4_addr_in: Some(OVERRIDE_IPV4),
            ipv6_addr_in: None,
        }],
        ..Default::default()
    };

    let relay_selector = RelaySelector::from_list(config, RELAYS.clone());

    let query_v4 = RelayQueryBuilder::new()
        .location(SHADOWSOCKS_RELAY_LOCATION.clone())
        .ip_version(IpVersion::V4)
        .shadowsocks()
        .build();
    assert!(!query_v4.wireguard_constraints().multihop());

    let relay = relay_selector.get_relay_by_query(query_v4).unwrap();
    match relay {
        GetRelay::Mullvad {
            obfuscator:
                Some(SelectedObfuscator {
                    config: Obfuscators::Single(ObfuscatorConfig::Shadowsocks { endpoint }),
                    ..
                }),
            inner: WireguardConfig::Singlehop { exit },
            ..
        } => {
            assert!(exit.overridden_ipv4);
            assert!(!exit.overridden_ipv6);
            assert_eq!(endpoint.ip(), IpAddr::from(OVERRIDE_IPV4));
        }
        wrong_relay => panic!(
            "Relay selector should have picked a Mullvad relay with Shadowsocks, instead chose {wrong_relay:?}"
        ),
    }
}

/// Ignore extra IPv6 addresses when overrides are set
#[test]
fn test_selecting_ignore_extra_ips_override_v6() {
    const OVERRIDE_IPV6: Ipv6Addr = Ipv6Addr::new(1, 0, 0, 0, 0, 0, 10, 10);

    let config = SelectorConfig {
        relay_overrides: vec![RelayOverride {
            hostname: SHADOWSOCKS_RELAY_LOCATION.get_hostname().unwrap().clone(),
            ipv4_addr_in: None,
            ipv6_addr_in: Some(OVERRIDE_IPV6),
        }],
        ..Default::default()
    };

    let relay_selector = RelaySelector::from_list(config, RELAYS.clone());

    let query_v6 = RelayQueryBuilder::new()
        .location(SHADOWSOCKS_RELAY_LOCATION.clone())
        .ip_version(IpVersion::V6)
        .shadowsocks()
        .build();
    assert!(!query_v6.wireguard_constraints().multihop());

    let relay = relay_selector.get_relay_by_query(query_v6).unwrap();
    match relay {
        GetRelay::Mullvad {
            obfuscator:
                Some(SelectedObfuscator {
                    config: Obfuscators::Single(ObfuscatorConfig::Shadowsocks { endpoint }),
                    ..
                }),
            inner: WireguardConfig::Singlehop { exit },
            ..
        } => {
            assert!(exit.overridden_ipv6);
            assert!(!exit.overridden_ipv4);
            assert_eq!(endpoint.ip(), IpAddr::from(OVERRIDE_IPV6));
        }
        wrong_relay => panic!(
            "Relay selector should have picked a Mullvad relay with Shadowsocks, instead chose {wrong_relay:?}"
        ),
    }
}

/// Construct a query for a Wireguard relay with specific port choices.
#[test]
fn test_wg_port_selection() {
    let default_ports = [53, 51820];
    let relay_selector = default_relay_selector();
    let mut rng = SmallRng::seed_from_u64(1337);

    for _ in 0..100 {
        let port = *default_ports.choose(&mut rng).unwrap();
        let query = RelayQueryBuilder::new().port(port).build();

        let relay = relay_selector.get_relay_by_query(query).unwrap();
        match relay {
            GetRelay::Mullvad { endpoint, .. } => {
                assert_eq!(endpoint.peer.endpoint.port(), port);
            }
            wrong_relay => panic!(
                "Relay selector should have picked a Mullvad relay, instead chose {wrong_relay:?}"
            ),
        }
    }
}

/// Construct a query for a Wireguard configuration where UDP2TCP obfuscation is selected and
/// multihop is explicitly turned off. Assert that the relay selector always return an obfuscator
/// configuration.
#[test]
fn test_selecting_endpoint_with_udp2tcp_obfuscation() {
    let relay_selector = default_relay_selector();
    let query = RelayQueryBuilder::new().udp2tcp().build();
    assert!(!query.wireguard_constraints().multihop());

    let relay = relay_selector.get_relay_by_query(query).unwrap();
    match relay {
        GetRelay::Mullvad {
            obfuscator,
            inner: WireguardConfig::Singlehop { .. },
            ..
        } => {
            assert!(obfuscator.is_some_and(|obfuscator| matches!(
                obfuscator.config,
                Obfuscators::Single(ObfuscatorConfig::Udp2Tcp { .. })
            )))
        }
        wrong_relay => panic!(
            "Relay selector should have picked a Mullvad relay, instead chose {wrong_relay:?}"
        ),
    }
}

/// Construct a query for a Wireguard configuration where obfuscation is set to "Auto" and
/// multihop is explicitly turned off. Assert that the relay selector does *not* return an
/// obfuscator config.
///
/// [`RelaySelector::get_relay`] may still enable obfuscation if it is present in [`RETRY_ORDER`].
#[cfg(not(feature = "staggered-obfuscation"))]
#[test]
fn test_selecting_endpoint_with_auto_obfuscation() {
    let relay_selector = default_relay_selector();

    let query = RelayQueryBuilder::new().build();
    assert_eq!(
        query.wireguard_constraints().obfuscation,
        ObfuscationQuery::Auto
    );

    for _ in 0..100 {
        let relay = relay_selector.get_relay_by_query(query.clone()).unwrap();
        match relay {
            GetRelay::Mullvad { obfuscator, .. } => {
                assert!(obfuscator.is_none());
            }
            wrong_relay => panic!(
                "Relay selector should have picked a Mullvad relay, instead chose {wrong_relay:?}"
            ),
        }
    }
}

/// Construct a query for a configuration with UDP2TCP obfuscation, and make sure that
/// all configurations contain a valid port.
#[test]
fn test_udp2tcp_use_correct_port_ranges() {
    const TCP2UDP_PORTS: [u16; 3] = [80, 443, 5001];
    let relay_selector = default_relay_selector();
    // Note that we do *not* specify any port here!
    let query = RelayQueryBuilder::new().udp2tcp().build();

    for _ in 0..1000 {
        let relay = relay_selector.get_relay_by_query(query.clone()).unwrap();
        match relay {
            GetRelay::Mullvad {
                obfuscator,
                inner: WireguardConfig::Singlehop { .. },
                ..
            } => {
                let Some(obfuscator) = obfuscator else {
                    panic!("Relay selector should have picked an obfuscator")
                };
                assert!(matches!(obfuscator.config,
                    Obfuscators::Single(ObfuscatorConfig::Udp2Tcp { endpoint }) if
                        TCP2UDP_PORTS.contains(&endpoint.port()),
                ))
            }
            wrong_relay => panic!(
                "Relay selector should have picked a Mullvad relay, instead chose {wrong_relay:?}"
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

/// Verify that any query which sets an explicit [`Ownership`] is respected by the relay selector
/// and that it works to set separate entry and exit ownerships for a multihop.
#[test]
fn test_multihop_ownership() {
    let relay_selector = default_relay_selector();

    for _ in 0..100 {
        // Construct an arbitrary query for owned relays.
        let query = RelayQueryBuilder::new()
            .multihop()
            .ownership(Ownership::MullvadOwned)
            .entry_ownership(Ownership::Rented)
            .build();
        let relay = relay_selector.get_relay_by_query(query).unwrap();
        // Check that the _exit_ relay is owned by Mullvad.
        assert!(unwrap_relay(relay.clone()).owned);
        // Check that the _entry_ relay is rented.
        assert!(!unwrap_entry_relay(relay).owned);
    }

    for _ in 0..100 {
        // Construct an arbitrary query for rented relays.
        let query = RelayQueryBuilder::new()
            .multihop()
            .ownership(Ownership::Rented)
            .entry_ownership(Ownership::MullvadOwned)
            .build();
        let relay = relay_selector.get_relay_by_query(query).unwrap();
        // Check that the _exit_ relay is rented.
        assert!(!unwrap_relay(relay.clone()).owned);
        // Check that the _entry_ relay is owned by Mullvad.
        assert!(unwrap_entry_relay(relay).owned);
    }
}

/// Verify that server and port selection varies between retry attempts.
#[test]
fn test_load_balancing() {
    const ATTEMPTS: usize = 100;
    let relay_selector = default_relay_selector();
    let location = GeographicLocationConstraint::country("se");
    let query = RelayQueryBuilder::new().location(location.clone()).build();
    // Collect the range of unique relay ports and IP addresses over a large number of queries.
    let (ports, ips): (HashSet<u16>, HashSet<std::net::IpAddr>) = std::iter::repeat_n(query.clone(), ATTEMPTS)
        // Execute the query
        .map(|query| relay_selector.get_relay_by_query(query).unwrap())
        // Perform some plumbing ..
        .map(unwrap_endpoint)
        .map(|endpoint| endpoint.to_endpoint().address)
        // Extract the selected relay's port + IP address
        .map(|endpoint| (endpoint.port(), endpoint.ip()))
        .unzip();

    assert!(ports.len() > 1, "expected more than 1 port, got {ports:?}");
    assert!(ips.len() > 1, "expected more than 1 server, got {ips:?}");
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
            GetRelay::Mullvad { .. } => {
                let exit = unwrap_relay(relay);
                assert!(
                    EXPECTED_PROVIDERS.contains(&exit.provider.as_str()),
                    "cannot find provider {provider} in {EXPECTED_PROVIDERS:?}",
                    provider = exit.provider
                )
            }
            wrong_relay => panic!(
                "Relay selector should have picked a Mullvad relay, instead chose {wrong_relay:?}"
            ),
        };
    }
}

/// Construct a query for a relay with specific providers and verify that every chosen relay has
/// the correct associated provider and that it works to select a separate set of providers for
/// entry and exit relays when doing a multihop.
#[test]
fn test_multihop_providers() {
    const EXPECTED_PROVIDERS: [&str; 2] = ["provider0", "provider2"];
    const EXPECTED_ENTRY_PROVIDERS: [&str; 2] = ["provider1", "provider3"];
    let providers = Providers::new(EXPECTED_PROVIDERS).unwrap();
    let entry_providers = Providers::new(EXPECTED_ENTRY_PROVIDERS).unwrap();
    let relay_selector = default_relay_selector();

    for _attempt in 0..100 {
        let query = RelayQueryBuilder::new()
            .multihop()
            .providers(providers.clone())
            .entry_providers(entry_providers.clone())
            .build();
        let relay = relay_selector.get_relay_by_query(query).unwrap();

        let (entry, exit) = unwrap_multihop_entry_exit_relays(relay);

        assert!(
            EXPECTED_PROVIDERS.contains(&exit.provider.as_str()),
            "cannot find exit provider {provider} in {EXPECTED_PROVIDERS:?}",
            provider = exit.provider
        );
        assert!(
            EXPECTED_ENTRY_PROVIDERS.contains(&entry.provider.as_str()),
            "cannot find entry provider {provider} in {EXPECTED_ENTRY_PROVIDERS:?}",
            provider = entry.provider
        );
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
                        overridden_ipv4: false,
                        overridden_ipv6: false,
                        include_in_country: false,
                        active: true,
                        owned: true,
                        provider: "31173".to_string(),
                        weight: 1,
                        endpoint_data: RelayEndpointData::Wireguard(
                            WireguardRelayEndpointData::new(WIREGUARD_PUBKEY.clone()),
                        ),
                        location: DUMMY_LOCATION.clone(),
                    },
                    Relay {
                        hostname: "se10-wireguard".to_string(),
                        ipv4_addr_in: "185.213.154.69".parse().unwrap(),
                        ipv6_addr_in: Some("2a03:1b20:5:f011::a10f".parse().unwrap()),
                        overridden_ipv4: false,
                        overridden_ipv6: false,
                        include_in_country: false,
                        active: true,
                        owned: false,
                        provider: "31173".to_string(),
                        weight: 1,
                        endpoint_data: RelayEndpointData::Wireguard(
                            WireguardRelayEndpointData::new(WIREGUARD_PUBKEY.clone()),
                        ),
                        location: DUMMY_LOCATION.clone(),
                    },
                ],
            }],
        }],
        bridge: BridgeEndpointData {
            shadowsocks: vec![],
        },
        wireguard: EndpointData {
            port_ranges: vec![53..=53, 4000..=33433, 33565..=51820, 52000..=60000],
            ipv4_gateway: "10.64.0.1".parse().unwrap(),
            ipv6_gateway: "fc00:bbbb:bbbb:bb01::1".parse().unwrap(),
            udp2tcp_ports: vec![],
            shadowsocks_port_ranges: vec![],
        },
    };

    // If include_in_country is false for all relays, a relay must be selected anyway.
    let relay_selector = RelaySelector::from_list(SelectorConfig::default(), relay_list.clone());
    assert!(
        relay_selector
            .get_relay(0, talpid_types::net::IpAvailability::Ipv4)
            .is_ok()
    );

    // If include_in_country is true for some relay, it must always be selected.
    relay_list.countries[0].cities[0].relays[0].include_in_country = true;
    let expected_hostname = relay_list.countries[0].cities[0].relays[0].hostname.clone();
    let relay_selector = RelaySelector::from_list(SelectorConfig::default(), relay_list);
    let relay = unwrap_relay(
        relay_selector
            .get_relay(0, talpid_types::net::IpAvailability::Ipv4)
            .expect("expected match"),
    );

    assert!(
        matches!(relay, Relay { ref hostname, .. } if hostname == &expected_hostname),
        "found {relay:?}, expected {expected_hostname:?}",
    )
}

/// Always use smart routing to select a DAITA-enabled entry relay if both smart routing and
/// multihop is enabled. This applies even if the entry is set explicitly.
/// DAITA is a core privacy feature
#[test]
fn test_daita_smart_routing_overrides_multihop() {
    let relay_selector = RelaySelector::from_list(SelectorConfig::default(), RELAYS.clone());
    let query = RelayQueryBuilder::
        new()
        .daita()
        .daita_use_multihop_if_necessary(true)
        .multihop()
        // Set the entry to a relay that explicitly does *not* support DAITA.
        // Later, we check that the smart routing disregards this choice and selects a DAITA-enabled
        // relay instead.
        .entry(NON_DAITA_RELAY_LOCATION.clone())
        .build();

    for _ in 0..100 {
        // Make sure a DAITA-enabled relay is always selected due to smart routing.
        let relay = relay_selector
            .get_relay_by_query(query.clone())
            .expect("Expected to find a relay with daita_use_multihop_if_necessary");
        match relay {
            GetRelay::Mullvad {
                inner: WireguardConfig::Multihop { entry, exit: _ },
                ..
            } => {
                assert!(supports_daita(&entry), "entry relay must support DAITA");
            }
            wrong_relay => panic!(
                "Relay selector should have picked two Wireguard relays, instead chose {wrong_relay:?}"
            ),
        }
    }

    // Assert that disabling smart routing for this query will fail to generate a valid multihop
    // config, thus blocking the user.
    let query = RelayQueryBuilder::new()
        .daita()
        .daita_use_multihop_if_necessary(false)
        .multihop()
        .entry(NON_DAITA_RELAY_LOCATION.clone())
        .build();

    let relay = relay_selector.get_relay_by_query(query);

    assert!(
        relay.is_err(),
        "expected there to be no valid multihop configuration! Instead got {relay:#?}"
    );
}

/// Return only entry relays that support DAITA when DAITA filtering is enabled. All relays that
/// support DAITA also support NOT DAITA. Thus, disabling it should not cause any WireGuard relays
/// to be filtered out.
#[test]
fn test_daita() {
    let relay_selector = RelaySelector::from_list(SelectorConfig::default(), RELAYS.clone());

    // Only pick relays that support DAITA
    let query = RelayQueryBuilder::new()
        .daita()
        .daita_use_multihop_if_necessary(false)
        .build();
    let relay = unwrap_entry_relay(relay_selector.get_relay_by_query(query).unwrap());
    assert!(
        supports_daita(&relay),
        "Selector supported relay that does not support DAITA: {relay:?}"
    );

    // Fail when only non-DAITA relays match constraints
    let query = RelayQueryBuilder::new()
        .daita()
        .daita_use_multihop_if_necessary(false)
        .location(NON_DAITA_RELAY_LOCATION.clone())
        .build();
    relay_selector
        .get_relay_by_query(query)
        .expect_err("Expected to find no matching relay");

    // Should be able to connect to non-DAITA relay with use_multihop_if_necessary
    let query = RelayQueryBuilder::new()
        .daita()
        .daita_use_multihop_if_necessary(true)
        .location(NON_DAITA_RELAY_LOCATION.clone())
        .build();
    let relay = relay_selector
        .get_relay_by_query(query)
        .expect("Expected to find a relay with daita_use_multihop_if_necessary");
    match relay {
        GetRelay::Mullvad {
            inner: WireguardConfig::Multihop { exit, entry },
            ..
        } => {
            assert!(supports_daita(&entry), "entry relay must support DAITA");
            assert!(!supports_daita(&exit), "exit relay must not support DAITA");
        }
        wrong_relay => panic!(
            "Relay selector should have picked two Wireguard relays, instead chose {wrong_relay:?}"
        ),
    }

    // Should be able to connect to DAITA relay with use_multihop_if_necessary
    let query = RelayQueryBuilder::new()
        .daita()
        .daita_use_multihop_if_necessary(true)
        .location(DAITA_RELAY_LOCATION.clone())
        .build();
    let relay = relay_selector
        .get_relay_by_query(query)
        .expect("Expected to find a relay with daita_use_multihop_if_necessary");
    match relay {
        GetRelay::Mullvad {
            inner: WireguardConfig::Singlehop { exit },
            ..
        } => {
            assert!(supports_daita(&exit), "entry relay must support DAITA");
        }
        wrong_relay => panic!(
            "Relay selector should have picked a single Wireguard relay, instead chose {wrong_relay:?}"
        ),
    }

    // DAITA-supporting relays can be picked even when it is disabled
    let query = RelayQueryBuilder::new()
        .location(DAITA_RELAY_LOCATION.clone())
        .build();
    relay_selector
        .get_relay_by_query(query)
        .expect("Expected DAITA-supporting relay to work without DAITA");

    // Non DAITA-supporting relays can be picked when it is disabled
    let query = RelayQueryBuilder::new()
        .location(NON_DAITA_RELAY_LOCATION.clone())
        .build();
    relay_selector
        .get_relay_by_query(query)
        .expect("Expected DAITA-supporting relay to work without DAITA");

    // Entry relay must support daita
    let query = RelayQueryBuilder::new()
        .daita()
        .daita_use_multihop_if_necessary(false)
        .multihop()
        .build();
    let relay = relay_selector.get_relay_by_query(query).unwrap();
    match relay {
        GetRelay::Mullvad {
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
        .daita()
        .daita_use_multihop_if_necessary(false)
        .multihop()
        .location(NON_DAITA_RELAY_LOCATION.clone())
        .build();
    let relay = relay_selector.get_relay_by_query(query).unwrap();
    match relay {
        GetRelay::Mullvad {
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

/// Check that if the original user query would yield a relay, the result of running the query
/// which is the intersection between the user query and any of the default queries shall never
/// fail.
#[test]
fn valid_user_setting_should_yield_relay() {
    // Make a valid user relay constraint
    let location = GeographicLocationConstraint::hostname("se", "got", "se9-wireguard");
    let user_query = RelayQueryBuilder::new().location(location.clone()).build();
    let (user_constraints, ..) = RelayQueryBuilder::new()
        .location(location.clone())
        .build()
        .into_settings();

    let config = SelectorConfig {
        relay_settings: user_constraints.into(),
        ..SelectorConfig::default()
    };
    let relay_selector = RelaySelector::from_list(config, RELAYS.clone());
    let user_result = relay_selector.get_relay_by_query(user_query.clone());
    for retry_attempt in 0..RETRY_ORDER.len() {
        let post_unification_result =
            relay_selector.get_relay(retry_attempt, talpid_types::net::IpAvailability::Ipv4);
        if user_result.is_ok() {
            assert!(
                post_unification_result.is_ok(),
                "Expected Post-unification query to be valid because original query {user_query:#?} yielded a connection configuration"
            )
        }
    }
}

/// Check that if IPv4 is not available and shadowsocks obfuscation is requested
/// it should return a relay with IPv6 address.
#[test]
fn test_shadowsocks_runtime_ipv4_unavailable() {
    // Make a valid user relay constraint
    let (relay_constraints, obfs_settings) = RelayQueryBuilder::new()
        .shadowsocks()
        .build()
        .into_settings();

    let config = SelectorConfig {
        relay_settings: relay_constraints.into(),
        obfuscation_settings: obfs_settings,
        ..SelectorConfig::default()
    };
    let relay_selector = RelaySelector::from_list(config, RELAYS.clone());
    let runtime_parameters = talpid_types::net::IpAvailability::Ipv6;
    let user_result = relay_selector.get_relay(0, runtime_parameters).unwrap();
    assert!(
        matches!(user_result, GetRelay::Mullvad {
        obfuscator: Some(SelectedObfuscator {
            config: Obfuscators::Single(ObfuscatorConfig::Shadowsocks {
                endpoint,
                ..
            }),
            ..
        }),
        ..
    } if endpoint.is_ipv6()),
        "expected IPv6 endpoint for Shadowsocks, got {user_result:?}"
    );
}

/// Check that if IPv4 is not available, a relay with an IPv6 endpoint is returned.
#[test]
fn test_runtime_ipv4_unavailable() {
    // Make a valid user relay constraint
    let (relay_constraints, ..) = RelayQueryBuilder::new().build().into_settings();

    let config = SelectorConfig {
        relay_settings: relay_constraints.into(),
        ..SelectorConfig::default()
    };
    let relay_selector = RelaySelector::from_list(config, RELAYS.clone());
    let runtime_parameters = talpid_types::net::IpAvailability::Ipv6;
    let relay = relay_selector.get_relay(0, runtime_parameters).unwrap();
    match relay {
        GetRelay::Mullvad { endpoint, .. } => {
            assert!(
                endpoint.peer.endpoint.is_ipv6(),
                "expected IPv6 endpoint, got {endpoint:?}",
            );
        }
        wrong_relay => panic!(
            "Relay selector should have picked a Mullvad relay, instead chose {wrong_relay:?}"
        ),
    }
}

/// Check that the relay selector is able to disregard `include_in_country` flag if necessary.
///
/// This test case prevents regressions to the `include_in_country` filtering logic.
#[test]
fn include_in_country_with_few_relays() -> Result<(), Error> {
    let query = RelayQueryBuilder::new()
        .multihop()
        .location(GeographicLocationConstraint::country("se"))
        .entry(GeographicLocationConstraint::country("se"))
        .build();

    // The relay selector ought to resolve the query to any of the following configurations
    // {entry: se-sto-wg-009, exit: se-sto-wg-204}
    // {entry: se-sto-wg-204, exit: se-sto-wg-009}
    let relays = {
        let stockholm = Location {
            country: "Sweden".to_string(),
            country_code: "se".to_string(),
            city: "Stockholm".to_string(),
            city_code: "sto".to_string(),
            latitude: 59.3289,
            longitude: 18.0649,
        };
        let wireguard = EndpointData {
            port_ranges: vec![443..=443],
            ..Default::default()
        };
        RelayList {
            countries: vec![RelayListCountry {
                name: "Sweden".to_string(),
                code: "se".to_string(),
                cities: vec![RelayListCity {
                    name: "Stockholm".to_string(),
                    code: "sto".to_string(),
                    latitude: 59.3289,
                    longitude: 18.0649,
                    relays: vec![
                        Relay {
                            hostname: "se-sto-wg-009".to_string(),
                            ipv4_addr_in: "185.195.233.69".parse().unwrap(),
                            ipv6_addr_in: "2a03:1b20:4:f011::a09f".parse().ok(),
                            overridden_ipv4: false,
                            overridden_ipv6: false,
                            // This is the important part
                            include_in_country: false,
                            active: true,
                            owned: true,
                            provider: "31173".to_string(),
                            weight: 1,
                            location: stockholm.clone(),
                            endpoint_data: RelayEndpointData::Wireguard(
                                WireguardRelayEndpointData::new(
                                    PublicKey::from_base64(
                                        "t1XlQD7rER0JUPrmh3R5IpxjUP9YOqodJAwfRorNxl4=",
                                    )
                                    .unwrap(),
                                ),
                            ),
                        },
                        Relay {
                            hostname: "se-sto-wg-204".to_string(),
                            ipv4_addr_in: "89.37.63.190".parse().unwrap(),
                            ipv6_addr_in: "2a02:6ea0:1508:4::f001".parse().ok(),
                            overridden_ipv4: false,
                            overridden_ipv6: false,
                            // This is the important part
                            include_in_country: true,
                            active: true,
                            owned: false,
                            provider: "DataPacket".to_string(),
                            weight: 200,
                            location: stockholm,
                            endpoint_data: RelayEndpointData::Wireguard(
                                WireguardRelayEndpointData::new(
                                    PublicKey::from_base64(
                                        "cPhM7ShRWQmKiJtD9Wd1vDh0GwIlaMvFb/WPrP58FH8=",
                                    )
                                    .unwrap(),
                                ),
                            ),
                        },
                    ],
                }],
            }],
            wireguard,
            ..Default::default()
        }
    };
    let relay_selector = RelaySelector::from_list(SelectorConfig::default(), relays);

    relay_selector.get_relay_by_query(query)?;
    Ok(())
}

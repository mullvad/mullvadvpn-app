//! Tests for verifying that the relay selector works as expected.

use std::{
    collections::HashSet,
    net::{IpAddr, Ipv4Addr, Ipv6Addr},
    sync::LazyLock,
};
use talpid_types::net::{
    IpVersion,
    TransportProtocol::{Tcp, Udp},
    obfuscation::{ObfuscatorConfig, Obfuscators},
    proxy::ShadowsocksCipher,
    wireguard::PublicKey,
};

use mullvad_relay_selector::{
    Error, GetRelay, MultihopConstraints, Predicate, RETRY_ORDER, Reason, RelaySelector,
    WireguardConfig,
    query::{ObfuscationMode, builder::RelayQueryBuilder},
};
use mullvad_types::{
    constraints::Constraint,
    endpoint::MullvadEndpoint,
    location::Location,
    relay_constraints::{
        GeographicLocationConstraint, LwoSettings, Ownership, Providers, RelayOverride,
    },
    relay_list::{
        Bridge, BridgeEndpointData, BridgeList, EndpointData, Quic, Relay, RelayList,
        RelayListCity, RelayListCountry, ShadowsocksEndpointData, WireguardRelay,
        WireguardRelayEndpointData,
    },
    settings::Settings,
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
    countries: vec![RelayListCountry {
        name: "Sweden".to_string(),
        code: "se".to_string(),
        cities: vec![RelayListCity {
            name: "Gothenburg".to_string(),
            code: "got".to_string(),
            latitude: 57.70887,
            longitude: 11.97456,
            relays: vec![
                WireguardRelay {
                    overridden_ipv4: false,
                    overridden_ipv6: false,
                    include_in_country: true,
                    owned: true,
                    provider: "provider0".to_string(),
                    endpoint_data: WireguardRelayEndpointData::new(WIREGUARD_PUBKEY.clone())
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
                    inner: Relay {
                        location: DUMMY_LOCATION.clone(),
                        hostname: "se9-wireguard".to_string(),
                        ipv4_addr_in: "185.213.154.68".parse().unwrap(),
                        ipv6_addr_in: Some("2a03:1b20:5:f011::a09f".parse().unwrap()),
                        active: true,
                        weight: 1,
                    },
                },
                WireguardRelay {
                    overridden_ipv4: false,
                    overridden_ipv6: false,
                    include_in_country: true,
                    owned: false,
                    provider: "provider1".to_string(),
                    endpoint_data: WireguardRelayEndpointData::new(
                        PublicKey::from_base64("BLNHNoGO88LjV/wDBa7CUUwUzPq/fO2UwcGLy56hKy4=")
                            .unwrap(),
                    ),
                    inner: Relay {
                        hostname: "se10-wireguard".to_string(),
                        location: DUMMY_LOCATION.clone(),
                        weight: 1,
                        active: true,
                        ipv4_addr_in: "185.213.154.69".parse().unwrap(),
                        ipv6_addr_in: Some("2a03:1b20:5:f011::a10f".parse().unwrap()),
                    },
                },
                WireguardRelay {
                    overridden_ipv4: false,
                    overridden_ipv6: false,
                    include_in_country: true,
                    owned: false,
                    provider: "provider2".to_string(),
                    endpoint_data: WireguardRelayEndpointData::new(
                        PublicKey::from_base64("BLNHNoGO88LjV/wDBa7CUUwUzPq/fO2UwcGLy56hKy4=")
                            .unwrap(),
                    )
                    .set_daita(true),
                    inner: Relay {
                        location: DUMMY_LOCATION.clone(),
                        weight: 1,
                        active: true,
                        hostname: "se11-wireguard".to_string(),
                        ipv4_addr_in: "185.213.154.69".parse().unwrap(),
                        ipv6_addr_in: Some("2a03:1b20:5:f011::a11f".parse().unwrap()),
                    },
                },
                SHADOWSOCKS_RELAY.clone(),
            ],
        }],
    }],

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
        udp2tcp_ports: vec![80, 443, 5001],
        shadowsocks_port_ranges: vec![100..=200, 1000..=2000],
    },
});

static BRIDGES: LazyLock<BridgeList> = LazyLock::new(|| BridgeList {
    bridges: vec![Bridge(Relay {
        hostname: "se-got-br-001".to_string(),
        ipv4_addr_in: "1.3.3.7".parse().unwrap(),
        ipv6_addr_in: None,
        active: true,
        weight: 1,
        location: DUMMY_LOCATION.clone(),
    })],
    bridge_endpoint: BridgeEndpointData {
        shadowsocks: vec![
            ShadowsocksEndpointData {
                port: 443,
                cipher: ShadowsocksCipher::new("aes-256-gcm").unwrap(),
                password: "mullvad".to_string(),
                protocol: Tcp,
            },
            ShadowsocksEndpointData {
                port: 1234,
                cipher: ShadowsocksCipher::new("aes-256-cfb").unwrap(),
                password: "mullvad".to_string(),
                protocol: Udp,
            },
            ShadowsocksEndpointData {
                port: 1236,
                cipher: ShadowsocksCipher::new("aes-256-gcm").unwrap(),
                password: "mullvad".to_string(),
                protocol: Udp,
            },
        ],
    },
});

static DAITA_RELAY_LOCATION: LazyLock<GeographicLocationConstraint> =
    LazyLock::new(|| GeographicLocationConstraint::hostname("se", "got", "se9-wireguard"));
static NON_DAITA_RELAY_LOCATION: LazyLock<GeographicLocationConstraint> =
    LazyLock::new(|| GeographicLocationConstraint::hostname("se", "got", "se10-wireguard"));

/// A Shadowsocks relay with additional addresses
static SHADOWSOCKS_RELAY: LazyLock<WireguardRelay> = LazyLock::new(|| WireguardRelay {
    overridden_ipv4: false,
    overridden_ipv6: false,
    include_in_country: true,
    owned: true,
    provider: "provider0".to_string(),
    endpoint_data: WireguardRelayEndpointData::new(
        PublicKey::from_base64("eaNHNoGO88LjV/wDBa7CUUwUzPq/fO2UwcGLy56hKy4=").unwrap(),
    )
    .add_shadowsocks_extra_in_addrs(SHADOWSOCKS_RELAY_EXTRA_ADDRS.iter().copied()),

    inner: Relay {
        location: DUMMY_LOCATION.clone(),
        hostname: SHADOWSOCKS_RELAY_LOCATION
            .get_hostname()
            .unwrap()
            .to_owned(),
        ipv4_addr_in: SHADOWSOCKS_RELAY_IPV4,
        ipv6_addr_in: Some(SHADOWSOCKS_RELAY_IPV6),
        active: true,
        weight: 1,
    },
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
fn unwrap_relay(get_result: GetRelay) -> WireguardRelay {
    match get_result.inner {
        crate::WireguardConfig::Singlehop { exit } => exit,
        crate::WireguardConfig::Multihop { exit, .. } => exit,
    }
}

fn unwrap_entry_relay(get_result: GetRelay) -> WireguardRelay {
    match get_result.inner {
        crate::WireguardConfig::Singlehop { exit } => exit,
        crate::WireguardConfig::Multihop { entry, .. } => entry,
    }
}

fn unwrap_endpoint(get_result: GetRelay) -> MullvadEndpoint {
    get_result.endpoint
}

fn default_relay_selector() -> RelaySelector {
    RelaySelector::from_settings(&Settings::default(), RELAYS.clone(), BRIDGES.clone())
}

fn supports_daita(relay: &WireguardRelay) -> bool {
    relay.endpoint_data.daita
}

/// Tests that exercise the full relay-selection pipeline via
/// [`RelaySelector::get_relay`] and [`RelaySelector::get_relay_by_query`].
mod relay_selection {
    use super::*;

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
        // A default config *should* have this property, but a more robust way to guarantee
        // this would be to create a neutral relay query and supply it to the relay selector at every
        // call to the `get_relay` function.
        let relay_selector = default_relay_selector();
        for (retry_attempt, query) in RETRY_ORDER.iter().enumerate() {
            let relay = relay_selector
                .get_relay(
                    retry_attempt,
                    talpid_types::net::IpAvailability::Ipv4AndIpv6,
                )
                .unwrap_or_else(|_| {
                    panic!("Retry attempt {retry_attempt} did not yield any relay")
                });
            // Then perform some protocol-specific probing as well.
            let GetRelay {
                endpoint,
                obfuscator,
                ..
            } = relay;
            assert!(query.wireguard_constraints().ip_version.matches_eq(
                &match endpoint.peer.endpoint.ip() {
                    std::net::IpAddr::V4(_) => talpid_types::net::IpVersion::V4,
                    std::net::IpAddr::V6(_) => talpid_types::net::IpVersion::V6,
                }
            ));
            assert!(match &query.wireguard_constraints().obfuscation {
                Constraint::Any => true,
                Constraint::Only(ObfuscationMode::Off | ObfuscationMode::Port(_)) =>
                    obfuscator.is_none(),
                Constraint::Only(
                    ObfuscationMode::Quic
                    | ObfuscationMode::Udp2tcp(_)
                    | ObfuscationMode::Shadowsocks(_)
                    | ObfuscationMode::Lwo(_),
                ) => obfuscator.is_some(),
            });
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
            countries: vec![RelayListCountry {
                name: "Sweden".to_string(),
                code: "se".to_string(),
                cities: vec![RelayListCity {
                    name: "Gothenburg".to_string(),
                    code: "got".to_string(),
                    latitude: 57.70887,
                    longitude: 11.97456,
                    relays: vec![
                        WireguardRelay {
                            overridden_ipv4: false,
                            overridden_ipv6: false,
                            include_in_country: true,
                            owned: true,
                            provider: "provider0".to_string(),
                            endpoint_data: WireguardRelayEndpointData::new(
                                WIREGUARD_PUBKEY.clone(),
                            ),
                            inner: Relay {
                                hostname: "se9-wireguard".to_string(),
                                ipv4_addr_in: "185.213.154.68".parse().unwrap(),
                                ipv6_addr_in: Some("2a03:1b20:5:f011::a09f".parse().unwrap()),
                                active: true,
                                weight: 1,
                                location: DUMMY_LOCATION.clone(),
                            },
                        },
                        WireguardRelay {
                            overridden_ipv4: false,
                            overridden_ipv6: false,
                            include_in_country: true,
                            owned: false,
                            provider: "provider1".to_string(),
                            endpoint_data: WireguardRelayEndpointData::new(
                                WIREGUARD_PUBKEY.clone(),
                            ),
                            inner: Relay {
                                hostname: "se10-wireguard".to_string(),
                                ipv4_addr_in: "185.213.154.69".parse().unwrap(),
                                ipv6_addr_in: Some("2a03:1b20:5:f011::a10f".parse().unwrap()),
                                active: true,
                                location: DUMMY_LOCATION.clone(),
                                weight: 1,
                            },
                        },
                    ],
                }],
            }],
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

        let bridges = BridgeList::default();

        let relay_selector = RelaySelector::from_settings(&Settings::default(), relays, bridges);
        let specific_hostname = "se10-wireguard";
        let specific_location =
            GeographicLocationConstraint::hostname("se", "got", specific_hostname);
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
            let WireguardConfig::Multihop { exit, entry } = relay.inner else {
                panic!(
                    "Relay selector returned unexpected relay: {:?}",
                    relay.inner
                );
            };
            assert_eq!(entry.hostname, specific_hostname);
            assert_ne!(exit.hostname, entry.hostname);
            assert_ne!(exit.ipv4_addr_in, entry.ipv4_addr_in);
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
            let WireguardConfig::Multihop { exit, entry } = relay.inner else {
                panic!(
                    "Relay selector returned unexpected relay: {:?}",
                    relay.inner
                );
            };
            assert_eq!(exit.hostname, specific_hostname);
            assert_ne!(exit.hostname, entry.hostname);
            assert_ne!(exit.ipv4_addr_in, entry.ipv4_addr_in);
        }
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
                GetRelay {
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
        let relay_selector =
            RelaySelector::from_settings(&Settings::default(), RELAYS.clone(), BRIDGES.clone());

        let query = RelayQueryBuilder::new().shadowsocks().build();
        assert!(!query.wireguard_constraints().multihop());

        let relay = relay_selector.get_relay_by_query(query).unwrap();
        let WireguardConfig::Singlehop { .. } = relay.inner else {
            panic!(
                "Relay selector expected a singlehop relay with Shadowsocks obfuscation, got: {:?}",
                relay.inner
            )
        };
        assert!(relay.obfuscator.is_some_and(|obfuscator| matches!(
            obfuscator,
            Obfuscators::Single(ObfuscatorConfig::Shadowsocks { .. })
        )));
    }

    /// Test whether extra Shadowsocks IPs are selected when available
    #[test]
    fn test_selecting_over_shadowsocks_extra_ips() {
        let relay_selector =
            RelaySelector::from_settings(&Settings::default(), RELAYS.clone(), BRIDGES.clone());

        let query = RelayQueryBuilder::new()
            .location(SHADOWSOCKS_RELAY_LOCATION.clone())
            .shadowsocks()
            .build();
        assert!(!query.wireguard_constraints().multihop());

        let relay = relay_selector.get_relay_by_query(query).unwrap();
        let GetRelay {
            inner, obfuscator, ..
        } = relay;
        let WireguardConfig::Singlehop { exit } = inner else {
            panic!(
                "Relay selector expected a singlehop relay with Shadowsocks obfuscation, got: {inner:?}"
            )
        };
        let Some(Obfuscators::Single(ObfuscatorConfig::Shadowsocks { endpoint })) = obfuscator
        else {
            panic!("Relay selector expected Shadowsocks obfuscation, got: {obfuscator:?}")
        };
        assert!(!exit.overridden_ipv4);
        assert!(!exit.overridden_ipv6);
        assert!(
            SHADOWSOCKS_RELAY_EXTRA_ADDRS.contains(&endpoint.ip()),
            "{endpoint} is not an additional IP"
        );
    }

    /// Test whether Quic is always selected as the obfuscation protocol when Quic is selected.
    #[test]
    fn test_selecting_over_quic() {
        let relay_selector =
            RelaySelector::from_settings(&Settings::default(), RELAYS.clone(), BRIDGES.clone());

        let query = RelayQueryBuilder::new().quic().build();
        assert!(!query.wireguard_constraints().multihop());

        let relay = relay_selector.get_relay_by_query(query).unwrap();
        let WireguardConfig::Singlehop { .. } = relay.inner else {
            panic!(
                "Relay selector expected a singlehop relay with Quic obfuscation, got: {:?}",
                relay.inner
            )
        };
        assert!(relay.obfuscator.is_some_and(|obfuscator| matches!(
            obfuscator,
            Obfuscators::Single(ObfuscatorConfig::Quic { .. }),
        )));
    }

    /// Test LWO relay selection
    #[test]
    fn test_selecting_over_lwo() {
        let relay_selector =
            RelaySelector::from_settings(&Settings::default(), RELAYS.clone(), BRIDGES.clone());

        let query = RelayQueryBuilder::new().lwo().build();
        assert!(!query.wireguard_constraints().multihop());

        let relay = relay_selector.get_relay_by_query(query).unwrap();
        let WireguardConfig::Singlehop { .. } = relay.inner else {
            panic!(
                "Relay selector expected a singlehop relay with LWO obfuscation, got: {:?}",
                relay.inner
            )
        };
        assert!(relay.obfuscator.is_some_and(|obfuscator| matches!(
            obfuscator,
            Obfuscators::Single(ObfuscatorConfig::Lwo { .. }),
        )));
    }

    /// Ignore extra IPv4 addresses when overrides are set
    #[test]
    fn test_selecting_ignore_extra_ips_override_v4() {
        const OVERRIDE_IPV4: Ipv4Addr = Ipv4Addr::new(1, 3, 3, 7);

        let relay_list = RELAYS.clone().apply_overrides(vec![RelayOverride {
            hostname: SHADOWSOCKS_RELAY_LOCATION.get_hostname().unwrap().clone(),
            ipv4_addr_in: Some(OVERRIDE_IPV4),
            ipv6_addr_in: None,
        }]);

        let relay_selector =
            RelaySelector::from_settings(&Settings::default(), relay_list, BRIDGES.clone());

        let query_v4 = RelayQueryBuilder::new()
            .location(SHADOWSOCKS_RELAY_LOCATION.clone())
            .ip_version(IpVersion::V4)
            .shadowsocks()
            .build();
        assert!(!query_v4.wireguard_constraints().multihop());

        let relay = relay_selector.get_relay_by_query(query_v4).unwrap();
        let GetRelay {
            inner, obfuscator, ..
        } = relay;
        let WireguardConfig::Singlehop { exit } = inner else {
            panic!(
                "Relay selector expected a singlehop relay with Shadowsocks obfuscation, got: {inner:?}"
            )
        };
        let Some(Obfuscators::Single(ObfuscatorConfig::Shadowsocks { endpoint })) = obfuscator
        else {
            panic!("Relay selector expected Shadowsocks obfuscation, got: {obfuscator:?}")
        };
        assert!(exit.overridden_ipv4);
        assert!(!exit.overridden_ipv6);
        assert_eq!(endpoint.ip(), IpAddr::from(OVERRIDE_IPV4));
    }

    /// Ignore extra IPv6 addresses when overrides are set
    #[test]
    fn test_selecting_ignore_extra_ips_override_v6() {
        const OVERRIDE_IPV6: Ipv6Addr = Ipv6Addr::new(1, 0, 0, 0, 0, 0, 10, 10);

        let relay_list = RELAYS.clone().apply_overrides(vec![RelayOverride {
            hostname: SHADOWSOCKS_RELAY_LOCATION.get_hostname().unwrap().clone(),
            ipv4_addr_in: None,
            ipv6_addr_in: Some(OVERRIDE_IPV6),
        }]);

        let relay_selector =
            RelaySelector::from_settings(&Settings::default(), relay_list, BRIDGES.clone());

        let query_v6 = RelayQueryBuilder::new()
            .location(SHADOWSOCKS_RELAY_LOCATION.clone())
            .ip_version(IpVersion::V6)
            .shadowsocks()
            .build();
        assert!(!query_v6.wireguard_constraints().multihop());

        let relay = relay_selector.get_relay_by_query(query_v6).unwrap();
        let GetRelay {
            inner, obfuscator, ..
        } = relay;
        let WireguardConfig::Singlehop { exit } = inner else {
            panic!(
                "Relay selector expected a singlehop relay with Shadowsocks obfuscation, got: {inner:?}"
            )
        };
        let Some(Obfuscators::Single(ObfuscatorConfig::Shadowsocks { endpoint })) = obfuscator
        else {
            panic!("Relay selector expected Shadowsocks obfuscation, got: {obfuscator:?}")
        };
        assert!(exit.overridden_ipv6);
        assert!(!exit.overridden_ipv4);
        assert_eq!(endpoint.ip(), IpAddr::from(OVERRIDE_IPV6));
    }

    /// Construct a query for a Wireguard relay with specific port choices.
    #[test]
    fn test_wg_port_selection() {
        let relay_selector = default_relay_selector();
        for port in [53, 51820] {
            let query = RelayQueryBuilder::new().port(port).build();
            let relay = relay_selector.get_relay_by_query(query).unwrap();
            assert_eq!(relay.endpoint.peer.endpoint.port(), port);
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
        let WireguardConfig::Singlehop { .. } = relay.inner else {
            panic!(
                "Relay selector returned unexpected relay: {:?}",
                relay.inner
            )
        };
        assert!(relay.obfuscator.is_some_and(|obfuscator| matches!(
            obfuscator,
            Obfuscators::Single(ObfuscatorConfig::Udp2Tcp { .. })
        )));
    }

    /// Construct a query for a Wireguard configuration where obfuscation is set to "Auto" and
    /// multihop is explicitly turned off. Assert that the relay selector does *not* return an
    /// obfuscator config.
    ///
    /// [`RelaySelector::get_relay`] may still enable obfuscation if it is present in [`RETRY_ORDER`].
    #[cfg(not(feature = "staggered-obfuscation"))]
    #[test]
    fn test_selecting_endpoint_with_auto_obfuscation() {
        use mullvad_types::constraints::Constraint;

        let relay_selector = default_relay_selector();

        let query = RelayQueryBuilder::new().build();
        assert_eq!(query.wireguard_constraints().obfuscation, Constraint::Any);

        for _ in 0..100 {
            let relay = relay_selector.get_relay_by_query(query.clone()).unwrap();
            assert!(relay.obfuscator.is_none());
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
            let WireguardConfig::Singlehop { .. } = relay.inner else {
                panic!(
                    "Relay selector returned unexpected relay: {:?}",
                    relay.inner
                )
            };
            let Some(obfuscator) = relay.obfuscator else {
                panic!("Relay selector should have picked an obfuscator")
            };
            assert!(matches!(
                obfuscator,
                Obfuscators::Single(ObfuscatorConfig::Udp2Tcp { endpoint }) if
                    TCP2UDP_PORTS.contains(&endpoint.port()),
            ));
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

    /// Ensure that `include_in_country` is ignored if all relays have it set to false (i.e., some
    /// relay is returned). Also ensure that `include_in_country` is respected if some relays
    /// have it set to true (i.e., that relay is never returned)
    #[test]
    fn test_include_in_country() {
        let mut relay_list = RelayList {
            countries: vec![RelayListCountry {
                name: "Sweden".to_string(),
                code: "se".to_string(),
                cities: vec![RelayListCity {
                    name: "Gothenburg".to_string(),
                    code: "got".to_string(),
                    latitude: 57.70887,
                    longitude: 11.97456,
                    relays: vec![
                        WireguardRelay {
                            overridden_ipv4: false,
                            overridden_ipv6: false,
                            include_in_country: false,
                            owned: true,
                            provider: "31173".to_string(),
                            endpoint_data: WireguardRelayEndpointData::new(
                                WIREGUARD_PUBKEY.clone(),
                            ),
                            inner: Relay {
                                location: DUMMY_LOCATION.clone(),
                                weight: 1,
                                active: true,
                                hostname: "se9-wireguard".to_string(),
                                ipv4_addr_in: "185.213.154.68".parse().unwrap(),
                                ipv6_addr_in: Some("2a03:1b20:5:f011::a09f".parse().unwrap()),
                            },
                        },
                        WireguardRelay {
                            overridden_ipv4: false,
                            overridden_ipv6: false,
                            include_in_country: false,
                            owned: false,
                            provider: "31173".to_string(),
                            endpoint_data: WireguardRelayEndpointData::new(
                                WIREGUARD_PUBKEY.clone(),
                            ),
                            inner: Relay {
                                active: true,
                                location: DUMMY_LOCATION.clone(),
                                weight: 1,
                                hostname: "se10-wireguard".to_string(),
                                ipv4_addr_in: "185.213.154.69".parse().unwrap(),
                                ipv6_addr_in: Some("2a03:1b20:5:f011::a10f".parse().unwrap()),
                            },
                        },
                    ],
                }],
            }],
            wireguard: EndpointData {
                port_ranges: vec![53..=53, 4000..=33433, 33565..=51820, 52000..=60000],
                ipv4_gateway: "10.64.0.1".parse().unwrap(),
                ipv6_gateway: "fc00:bbbb:bbbb:bb01::1".parse().unwrap(),
                udp2tcp_ports: vec![],
                shadowsocks_port_ranges: vec![],
            },
        };

        // If include_in_country is false for all relays, a relay must be selected anyway.
        let relay_selector = RelaySelector::from_settings(
            &Settings::default(),
            relay_list.clone(),
            BridgeList::default(),
        );
        assert!(
            relay_selector
                .get_relay(0, talpid_types::net::IpAvailability::Ipv4)
                .is_ok()
        );

        // If include_in_country is true for some relay, it must always be selected.
        relay_list.countries[0].cities[0].relays[0].include_in_country = true;
        let expected_hostname = relay_list.countries[0].cities[0].relays[0].hostname.clone();
        let relay_selector =
            RelaySelector::from_settings(&Settings::default(), relay_list, BridgeList::default());
        let relay = unwrap_relay(
            relay_selector
                .get_relay(0, talpid_types::net::IpAvailability::Ipv4)
                .expect("expected match"),
        );

        assert!(
            matches!(relay.inner, Relay { ref hostname, .. } if hostname == &expected_hostname),
            "found {relay:?}, expected {expected_hostname:?}",
        )
    }

    /// Return only entry relays that support DAITA when DAITA filtering is enabled. All relays that
    /// support DAITA also support NOT DAITA. Thus, disabling it should not cause any WireGuard relays
    /// to be filtered out.
    #[test]
    fn test_daita() {
        let relay_selector =
            RelaySelector::from_settings(&Settings::default(), RELAYS.clone(), BRIDGES.clone());

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
            GetRelay {
                inner: WireguardConfig::Multihop { exit, entry },
                ..
            } => {
                assert!(supports_daita(&entry), "entry relay must support DAITA");
                assert!(!supports_daita(&exit), "exit relay must not support DAITA");
            }
            wrong_relay => panic!("Relay selector expected a multihop relay, got: {wrong_relay:?}"),
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
            GetRelay {
                inner: WireguardConfig::Singlehop { exit },
                ..
            } => {
                assert!(supports_daita(&exit), "entry relay must support DAITA");
            }
            wrong_relay => {
                panic!("Relay selector expected a singlehop relay, got: {wrong_relay:?}")
            }
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
            GetRelay {
                inner: WireguardConfig::Multihop { exit: _, entry },
                ..
            } => {
                assert!(supports_daita(&entry), "entry relay must support DAITA");
            }
            wrong_relay => panic!("Relay selector returned unexpected relay: {wrong_relay:?}"),
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
            GetRelay {
                inner: WireguardConfig::Multihop { exit, entry: _ },
                ..
            } => {
                assert!(
                    !supports_daita(&exit),
                    "expected non DAITA-supporting exit relay, got {exit:?}"
                );
            }
            wrong_relay => panic!("Relay selector returned unexpected relay: {wrong_relay:?}"),
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

        let relay_selector =
            RelaySelector::from_query(user_query.clone(), RELAYS.clone(), BRIDGES.clone());
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
                shadowsocks_port_ranges: vec![100..=200, 1000..=2000],
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
                            WireguardRelay {
                                overridden_ipv4: false,
                                overridden_ipv6: false,
                                // This is the important part
                                include_in_country: false,
                                owned: true,
                                provider: "31173".to_string(),
                                endpoint_data: WireguardRelayEndpointData::new(
                                    PublicKey::from_base64(
                                        "t1XlQD7rER0JUPrmh3R5IpxjUP9YOqodJAwfRorNxl4=",
                                    )
                                    .unwrap(),
                                ),
                                inner: Relay {
                                    hostname: "se-sto-wg-009".to_string(),
                                    ipv4_addr_in: "185.195.233.69".parse().unwrap(),
                                    ipv6_addr_in: "2a03:1b20:4:f011::a09f".parse().ok(),
                                    active: true,
                                    weight: 1,
                                    location: stockholm.clone(),
                                },
                            },
                            WireguardRelay {
                                overridden_ipv4: false,
                                overridden_ipv6: false,
                                // This is the important part
                                include_in_country: true,
                                owned: false,
                                provider: "DataPacket".to_string(),
                                endpoint_data: WireguardRelayEndpointData::new(
                                    PublicKey::from_base64(
                                        "cPhM7ShRWQmKiJtD9Wd1vDh0GwIlaMvFb/WPrP58FH8=",
                                    )
                                    .unwrap(),
                                ),
                                inner: Relay {
                                    location: stockholm,
                                    active: true,
                                    weight: 200,
                                    hostname: "se-sto-wg-204".to_string(),
                                    ipv4_addr_in: "89.37.63.190".parse().unwrap(),
                                    ipv6_addr_in: "2a02:6ea0:1508:4::f001".parse().ok(),
                                },
                            },
                        ],
                    }],
                }],
                wireguard,
            }
        };
        let relay_selector =
            RelaySelector::from_settings(&Settings::default(), relays, BridgeList::default());

        relay_selector.get_relay_by_query(query)?;
        Ok(())
    }
}

/// Tests covering the [RelaySelector::partition_relays] algorithm.
///
/// # Snapshot tests
/// The snapshot files live in `tests/snapshots/`. To review and accept changes after
/// intentionally modifying the algorithm, run:
///
/// ```text
/// cargo insta review -p mullvad-relay-selector
/// ```
mod partition_relays {
    use itertools::Itertools;
    use mullvad_relay_selector::{EntryConstraints, ExitConstraints, RelayPartitions};
    use mullvad_types::constraints::Constraint;
    use mullvad_types::relay_constraints::{LocationConstraint, ShadowsocksSettings};
    use std::collections::HashSet;

    use super::*;

    use super::relay_list_builder::RelayListBuilder;

    // An updated relay list can be fetched using
    // `cargo run -p  mullvad-api --bin relay_list -- --internal`
    static RELAYS: LazyLock<(RelayList, BridgeList)> = LazyLock::new(|| {
        let relays = include_bytes!("./relays.json");
        serde_json::from_slice(relays).unwrap()
    });

    /// Create a [`RelaySelector`] using [`RELAYS`] as a backing relay list.
    fn relay_selector() -> RelaySelector {
        static RELAY_SELECTOR: LazyLock<RelaySelector> = LazyLock::new(|| {
            let (relay_list, bridge_list) = &*RELAYS;
            RelaySelector::from_settings(
                &Settings::default(),
                relay_list.clone(),
                bridge_list.clone(),
            )
        });
        RELAY_SELECTOR.clone()
    }

    /// Get a set of unique `[Reason]`s from the discards of a relay partition.
    fn unique_reasons(RelayPartitions { discards, .. }: RelayPartitions) -> HashSet<Reason> {
        discards
            .into_iter()
            .flat_map(|(_relay, reasons)| reasons)
            .collect()
    }

    /// Verify that the results of constraining [`Ownership`] and/or [`Providers`], separately
    /// for the entry and exit in multihop case, is reflected in the chosen relay and in the
    /// "reasons" of the discarded relays.
    #[test]
    fn multihop_ownership_and_provider() {
        let exit_constraints = [
            ExitConstraints::default().ownership(Ownership::MullvadOwned),
            ExitConstraints::default().ownership(Ownership::Rented),
            ExitConstraints::default().providers(Providers::new(["100TB"]).unwrap()),
            ExitConstraints::default().providers(Providers::new(["100TB", "31173"]).unwrap()),
            ExitConstraints::default()
                .providers(Providers::new(["31173"]).unwrap())
                .ownership(Ownership::MullvadOwned),
        ];

        // Test all combinations of entry and exit constraints.
        for (entry_general, exit_constraints) in exit_constraints
            .iter()
            .cartesian_product(exit_constraints.iter())
        {
            let entry_constraints = EntryConstraints::default().general(entry_general.clone());
            let multihop_constraints = MultihopConstraints::default()
                .entry(entry_constraints.clone())
                .exit(exit_constraints.clone());

            for scenario in [
                Predicate::Singlehop(entry_constraints.clone()),
                Predicate::Autohop(entry_constraints),
                Predicate::Entry(multihop_constraints.clone()),
                Predicate::Exit(multihop_constraints),
            ] {
                // Select the constraints that corresponds to the returned relays of `partition_relays` for the given
                let expected_exit_constraints = match &scenario {
                    Predicate::Exit(MultihopConstraints { exit, .. }) => exit,
                    Predicate::Singlehop(entry)
                    | Predicate::Autohop(entry)
                    | Predicate::Entry(MultihopConstraints { entry, .. }) => &entry.general,
                };

                let relays = relay_selector().partition_relays(scenario.clone());
                assert!(!relays.matches.is_empty());

                for relay in &relays.matches {
                    if let Constraint::Only(ownership) = expected_exit_constraints.ownership {
                        assert_eq!(
                            relay.owned,
                            ownership.mullvad(),
                            "{scenario:#?} => {relay:#?}"
                        );
                    }
                    if let Constraint::Only(ref providers) = expected_exit_constraints.providers {
                        assert!(
                            providers.providers().contains(&relay.provider),
                            "cannot find exit provider {provider} in {providers:?}",
                            provider = relay.provider
                        );
                    }
                }

                let mut expected_reasons =
                    HashSet::from([Reason::Inactive, Reason::IncludeInCountry]);
                if expected_exit_constraints.ownership.is_only() {
                    expected_reasons.insert(Reason::Ownership);
                }
                if expected_exit_constraints.providers.is_only() {
                    expected_reasons.insert(Reason::Providers);
                }
                assert!(
                    unique_reasons(relays).is_subset(&expected_reasons),
                    "expected reasons to be a subset of expected_reasons"
                );
            }
        }
    }

    /// If a multihop constraint has the same entry and exit relay, the relay selector
    /// should fail to come up with a valid configuration.
    ///
    /// If instead the entry and exit relay are distinct, and assuming that the relays exist, the relay
    /// selector should instead always return a valid configuration.
    #[test]
    fn entry_hostname_collision() {
        let relay_selector = relay_selector();
        // Define two distinct Wireguard relays.
        let wg101 = LocationConstraint::from(GeographicLocationConstraint::hostname(
            "se",
            "got",
            "se-got-wg-101",
        ));
        let wg001 = LocationConstraint::from(GeographicLocationConstraint::hostname(
            "se",
            "got",
            "se-got-wg-001",
        ));

        let mut constraints = MultihopConstraints::default();
        constraints.entry.general.location = wg101.clone().into();
        constraints.exit.location = wg101.clone().into();

        let RelayPartitions {
            matches, discards, ..
        } = relay_selector.partition_relays(Predicate::Exit(constraints.clone()));
        // Assert that the same host cannot be used for entry and exit
        assert_eq!(matches.len(), 0);
        assert_eq!(
            discards
                .iter()
                .find_map(|(relay, reasons)| {
                    (relay.hostname.as_str() == "se-got-wg-101").then_some(reasons)
                })
                .unwrap(),
            &vec![Reason::Conflict]
        );

        // Correct the erroneous query by setting `wg001` as the entry relay
        let mut constraints = MultihopConstraints::default();
        constraints.exit.location = wg101.into();
        constraints.entry.general.location = wg001.into();
        let query = relay_selector.partition_relays(Predicate::Exit(constraints.clone()));
        // Assert that the new query succeeds when the entry and exit hosts differ
        assert!(!query.matches.is_empty());
    }

    /// Regression test for [`AutohopPartition::into_relay_partitions`].
    ///
    /// Set up a city with two relays: one DAITA-supporting (`only_daita`) and one not
    /// (`other`). Under autohop with DAITA enabled and the location pinned to that city:
    /// - singlehop matches only `only_daita`
    /// - multihop entries (location=Any after autohop conversion) is uniquely `only_daita`
    /// - multihop exits would match both, but `remove_conflicting_relay` moves
    ///   `only_daita` out of exits with [Conflict] because it's the unique entry
    ///
    /// Autohop must still report `only_daita` as a match — it's a valid singlehop
    /// configuration. Naively returning `multihop.exits` would put it in discards.
    #[test]
    fn autohop_singlehop_survives_multihop_conflict() {
        let mut relay_list = RelayListBuilder::new();
        relay_list.add_relay("only_daita").endpoint_data.daita = true;
        relay_list.add_relay("other");
        let relay_selector = RelaySelector::from(relay_list);

        let constraints = EntryConstraints::default()
            .daita(true)
            .general(ExitConstraints::default().city("tc", "tc"));

        let RelayPartitions { matches, discards } =
            relay_selector.partition_relays(Predicate::Autohop(constraints));

        let match_hostnames: HashSet<&str> = matches.iter().map(|r| r.hostname.as_str()).collect();
        assert_eq!(
            match_hostnames,
            HashSet::from(["only_daita", "other"]),
            "both relays should be autohop matches: \
             `only_daita` via singlehop (and as the multihop entry), \
             `other` via multihop exit",
        );
        assert!(
            !discards.iter().any(|(r, _)| r.hostname == "only_daita"),
            "`only_daita` must not be discarded with [Conflict] — \
             the conflict only blocks multihop, not singlehop. discards: {discards:?}",
        );
    }

    /// Test that filtering on obfuscation works.
    #[test]
    fn obfuscation() {
        // Setup relay selector
        let mut relay_list = RelayListBuilder::new();
        relay_list.add_relay("basic");
        relay_list.add_relay("basic_ipv6").ipv6_addr_in = Some(Ipv6Addr::UNSPECIFIED);
        relay_list.add_relay("lwo").endpoint_data.lwo = true;
        relay_list.add_relay("quic_ipv4").endpoint_data.quic = Some(Quic::new(
            vec1![Ipv4Addr::UNSPECIFIED.into()],
            String::new(),
            String::new(),
        ));
        relay_list.add_relay("quic_ipv6").endpoint_data.quic = Some(Quic::new(
            vec1![Ipv6Addr::UNSPECIFIED.into()],
            String::new(),
            String::new(),
        ));

        relay_list.inner.wireguard.shadowsocks_port_ranges = vec![100..=200];
        relay_list
            .add_relay("shadowsocks_extra_ipv6")
            .endpoint_data
            .shadowsocks_extra_addr_in = HashSet::from([Ipv6Addr::UNSPECIFIED.into()]);
        let relay_selector = RelaySelector::from(relay_list);

        // "Auto" matches all relays
        let constraints = EntryConstraints::default();
        let RelayPartitions {
            matches: _,
            discards,
            ..
        } = relay_selector.partition_relays(Predicate::Singlehop(constraints));
        assert!(discards.is_empty());

        // Quic with Ipv4 constraint
        let constraints = EntryConstraints::default()
            .obfuscation(ObfuscationMode::Quic)
            .ip_version(IpVersion::V4);
        let RelayPartitions {
            matches, discards, ..
        } = relay_selector.partition_relays(Predicate::Singlehop(constraints));
        assert_eq!(
            &matches.into_iter().exactly_one().unwrap().hostname,
            "quic_ipv4"
        );
        for (relay, reasons) in discards {
            if &relay.hostname == "quic_ipv6" {
                assert_eq!(reasons, vec![Reason::IpVersion]);
            } else {
                assert_eq!(reasons, vec![Reason::Obfuscation]);
            }
        }

        // Plain shadowsocks matches all relays
        let constraints = EntryConstraints::default()
            .obfuscation(ObfuscationMode::Shadowsocks(ShadowsocksSettings::default()));
        let RelayPartitions {
            matches: _,
            discards,
            ..
        } = relay_selector.partition_relays(Predicate::Singlehop(constraints));
        assert!(discards.is_empty(), "Plain shadowsocks matches all relays");

        // Shadowsocks with a port outside the configured port ranges (100..=200):
        let out_of_range_port = 999; // outside 100..=200
        let constraints = EntryConstraints::default().obfuscation(ObfuscationMode::Shadowsocks(
            ShadowsocksSettings {
                port: Constraint::Only(out_of_range_port),
            },
        ));
        let RelayPartitions {
            matches, discards, ..
        } = relay_selector.partition_relays(Predicate::Singlehop(constraints));
        // Only the relay with an extra IPv6 address should match (its extra addr satisfies
        // `ip_version=Any` → IpVersionMatch::Ok → AcceptObfuscationEndpoint).
        assert_eq!(
            &matches.into_iter().exactly_one().unwrap().hostname,
            "shadowsocks_extra_ipv6",
            "Only the relay with extra IPv6 addr should match when port is out of range"
        );
        // All other relays have no extra addresses (IpVersionMatch::None) and the port is
        // outside the WireGuard shadowsocks ranges, so they must be rejected with Reason::Port.
        for (relay, reasons) in &discards {
            assert_eq!(
                reasons,
                &vec![Reason::Port],
                "relay '{}' should be rejected with Reason::Port",
                relay.hostname
            );
        }

        // Shadowsocks with ip_version=V4 and a port outside the configured port ranges:
        let constraints = EntryConstraints::default()
            .obfuscation(ObfuscationMode::Shadowsocks(ShadowsocksSettings {
                port: Constraint::Only(out_of_range_port),
            }))
            .ip_version(IpVersion::V4);
        let RelayPartitions {
            matches, discards, ..
        } = relay_selector.partition_relays(Predicate::Singlehop(constraints));
        assert!(
            matches.is_empty(),
            "No relay should match shadowsocks+V4+out-of-range port"
        );
        for (relay, reasons) in &discards {
            if relay.hostname == "shadowsocks_extra_ipv6" {
                // Has extra addrs but only IPv6 → IpVersionMatch::Other → Reason::IpVersion.
                // Switching to IPv6 would unblock this relay.
                assert_eq!(
                    reasons,
                    &vec![Reason::IpVersion],
                    "relay '{}' should be rejected with Reason::IpVersion",
                    relay.hostname
                );
            } else {
                // Other relays have no extra ss addrs at all → IpVersionMatch::None → Reason::Port.
                // The port is the only thing blocking; the WireGuard endpoint would work with V4.
                assert_eq!(
                    reasons,
                    &vec![Reason::Port],
                    "relay '{}' should be rejected with Reason::Port",
                    relay.hostname
                );
            }
        }
    }

    /// Check that if IPv4 is not available, a relay with an IPv6 endpoint is returned.
    #[test]
    fn runtime_ipv4_unavailable() {
        let mut relay_list_builder = RelayListBuilder::new();
        let has_ipv6 = relay_list_builder.add_relay("has_ipv6");
        has_ipv6.inner.ipv6_addr_in = Some(Ipv6Addr::LOCALHOST);
        let has_ipv6_clone = has_ipv6.clone();
        let hasnt_ipv6_clone = relay_list_builder.add_relay("hasnt_ipv6").clone();

        let relay_selector = RelaySelector::from(relay_list_builder);
        let constraints = EntryConstraints::default().ip_version(IpVersion::V6);
        // Query for all DAITA relays.
        let RelayPartitions {
            matches, discards, ..
        } = relay_selector.partition_relays(Predicate::Singlehop(constraints));
        assert_eq!(matches, vec![has_ipv6_clone]);
        assert_eq!(discards, vec![(hasnt_ipv6_clone, vec![Reason::IpVersion])]);
    }

    /// Check that if IPv4 is not available and shadowsocks obfuscation is requested
    /// it should return a relay with IPv6 address.
    #[test]
    fn shadowsocks_runtime_ipv4_unavailable() {
        let constraints = EntryConstraints::default()
            .ip_version(IpVersion::V6)
            .obfuscation(ObfuscationMode::Shadowsocks(ShadowsocksSettings {
                port: Constraint::Only(1337), // Port only usable with dedicated shadowsocks IP
            }));
        let query = relay_selector().partition_relays(Predicate::Singlehop(constraints));
        assert!(!query.matches.is_empty());
        for relay in &query.matches {
            assert!(
                relay
                    .endpoint_data
                    .shadowsocks_extra_addr_in
                    .iter()
                    .any(|ip| ip.is_ipv6()),
                "{relay:#?}"
            );
        }

        assert!(relay_selector().relay_list(|r| {
            r.relays()
                .any(|r| r.endpoint_data.shadowsocks_extra_addr_in.is_empty())
        }));
        assert_eq!(
            unique_reasons(query),
            HashSet::from_iter([
                Reason::Port,
                Reason::IpVersion,
                Reason::Inactive,
                Reason::IncludeInCountry
            ])
        );
    }

    /// Test that filtering on DAITA works.
    #[test]
    fn daita() {
        let relay_selector = relay_selector();
        let constraints = EntryConstraints::default().daita(true);
        // Query for all DAITA relays.
        let RelayPartitions {
            matches, discards, ..
        } = relay_selector.partition_relays(Predicate::Singlehop(constraints));
        for relay in matches {
            assert!(relay.endpoint_data.daita)
        }
        // Not all relays were discarded because they do not have DAITA, but some were!
        // Use them as entry relays, and use smart routing to forcibly select alternate entry
        // routes.
        for relay in discards
            .into_iter()
            .filter_map(|(discard, reasons)| (reasons == vec![Reason::Daita]).then_some(discard))
        {
            // Force the entry relay to be a relay without DAITA.
            let constraints = EntryConstraints::default().daita(true).general(
                ExitConstraints::default().location(GeographicLocationConstraint::hostname(
                    relay.location.country_code.clone(),
                    relay.location.city_code.clone(),
                    relay.hostname.clone(),
                )),
            );
            // Demonstrate the difference between autohop / singlehop.
            let RelayPartitions { matches, .. } =
                relay_selector.partition_relays(Predicate::Autohop(constraints.clone()));
            assert_eq!(matches, vec![relay]);

            let RelayPartitions { matches, .. } =
                relay_selector.partition_relays(Predicate::Singlehop(constraints));
            assert!(matches.is_empty());
        }
    }

    /// Always use smart routing to select a DAITA-enabled entry relay if both smart routing and
    /// multihop is enabled. This applies even if the entry is set explicitly.
    #[test]
    fn daita_smart_routing_overrides_multihop() {
        let relay_selector = relay_selector();
        let daita_constraints = EntryConstraints::default().daita(true);
        for non_daita_relay in relay_selector
            .partition_relays(Predicate::Singlehop(daita_constraints.clone()))
            .discards
            .into_iter()
            .filter_map(|(discard, reasons)| {
                (reasons.contains(&Reason::Daita) && !reasons.contains(&Reason::Inactive))
                    .then_some(discard)
            })
        {
            let mut constraints = daita_constraints.clone();
            // Force the entry relay to be a relay without DAITA.
            constraints.general.location =
                LocationConstraint::from(GeographicLocationConstraint::hostname(
                    non_daita_relay.location.country.clone(),
                    non_daita_relay.location.city.clone(),
                    non_daita_relay.hostname.clone(),
                ))
                .into();

            // Make sure a DAITA-enabled relay is always selected due to smart routing.
            let query = relay_selector.partition_relays(Predicate::Autohop(constraints.clone()));
            for relay in query.matches {
                assert!(relay.endpoint_data.daita, "{relay:#?}");
            }
            // Note: We have already asserted that the same query without smart routing works at
            // this point.
        }
    }

    /// Test that when a relay is rejected because of a single setting, removing that setting from the
    /// constraints will unblock the relay.
    #[test]
    fn test_unblocking_based_on_reasons() {
        fn test_constraint(expected_reason: Reason, constraints: EntryConstraints) {
            let relay_selector = relay_selector();
            let expected_reasons = vec![expected_reason];
            let non_matching_relays = relay_selector
                .partition_relays(Predicate::Singlehop(constraints))
                .discards
                .into_iter()
                .filter(|(_, reasons)| reasons == &expected_reasons)
                .map(|(discard, _)| discard)
                .collect::<HashSet<_>>();
            assert!(
                !non_matching_relays.is_empty(),
                "Set should not be empty, else test is useless"
            );
            let RelayPartitions {
                matches,
                discards: _,
                ..
            } = relay_selector.partition_relays(Predicate::Singlehop(EntryConstraints::default()));
            non_matching_relays.is_subset(&HashSet::from_iter(matches));
        }

        let test_cases = [
            (Reason::Daita, EntryConstraints::default().daita(true)),
            (
                Reason::Obfuscation,
                EntryConstraints::default().obfuscation(ObfuscationMode::Quic),
            ),
            (
                Reason::Ownership,
                EntryConstraints::default().ownership(Ownership::MullvadOwned),
            ),
            (
                Reason::Providers,
                EntryConstraints::default().providers(Providers::new(["DataPacket"]).unwrap()),
            ),
            (
                Reason::Port,
                EntryConstraints::default().obfuscation(ObfuscationMode::Shadowsocks(
                    ShadowsocksSettings {
                        port: Constraint::Only(123),
                    },
                )),
            ),
        ];
        for (reason, constraint) in test_cases {
            test_constraint(reason, constraint);
        }
    }

    /// "Daita + No Direct only + Provider. Providers (currently) affects both entry and exit
    /// relays, while DAITA only affects entry relays.
    #[test]
    fn daita_no_direct_only_provider() {
        // "100TB" does not have any DAITA relays, and because filters should apply for both entry
        // and exit even if we're autohoppin', we should not show any matching relays.
        let providers = Providers::new(["100TB"]).unwrap();

        let constraints = EntryConstraints::default().providers(providers).daita(true);

        let query = relay_selector().partition_relays(Predicate::Autohop(constraints));
        assert_eq!(query.matches.len(), 0);

        // Assert that a relay was discarded because we could not select an entry because of
        // DAITA and provider constraints not making sense.
        //
        // Note that some relays will be discarded simply because they lack DAITA OR are operated
        // by the wrong provider.
        let reasons = unique_reasons(query);
        assert!(
            reasons.is_subset(&HashSet::from([
                Reason::Providers,
                Reason::Daita,
                Reason::Inactive,
                Reason::IncludeInCountry,
            ])),
            "{reasons:#?}"
        );
    }

    /// Autohopping through an alternate entry relay should only be done iff the settings force us
    /// to. I.e. all relays which may be connected to directly should of course not be discarded.
    #[test]
    fn autohop_no_need_for_alternate_entry() {
        // A slightly more contrived example than `daita_no_direct_only_provider`: perform a
        // pre-step pruning all DAITA relays from the relay list. As such, no other factor comes
        // into play (provider, ownership).
        let mut relay_list = RelayListBuilder::new();
        relay_list.add_relay("non-daita-relay");
        let relay_selector = RelaySelector::from(relay_list);

        let constraints = EntryConstraints::default();

        let RelayPartitions {
            matches, discards, ..
        } = relay_selector.partition_relays(Predicate::Autohop(constraints));
        // The single relay in the relay list ought to be matched in this instance.
        assert!(
            discards.is_empty(),
            "Discard should be empty: {discards:#?}"
        );
        assert_eq!(matches.len(), 1);
        assert_eq!(
            matches.first().map(|relay| relay.hostname.as_ref()),
            Some("non-daita-relay")
        );
    }

    /// Assert that a relay is accepted even if the location does not contain any eligible relays when Autohopping.
    #[test]
    fn no_entry_constraints_on_autohop_exit() {
        // Curate a relay list with exactly two relays; one with DAITA & one without.
        let mut relay_list = RelayListBuilder::new();
        // Without DAITA.
        relay_list.add_location("non-daita-land", "non-daita-city");
        relay_list.add_relay("non-daita-relay");
        relay_list.add_location("daita-land", "daita-city");
        // With DAITA.
        let daita_relay = relay_list.add_relay("daita-relay");
        daita_relay.endpoint_data.daita = true;
        let relay_selector = RelaySelector::from(relay_list);

        // Constraint with DAITA and a location that doesn't have it
        let constraints = EntryConstraints::default()
            .daita(true)
            .general(ExitConstraints::default().country("non-daita-land"));

        let RelayPartitions {
            mut matches,
            mut discards,
            ..
        } = relay_selector.partition_relays(Predicate::Autohop(constraints));

        // Should still show the relay from the non-daita-country.
        assert_eq!(matches.len(), 1);
        let matching = matches.pop().unwrap();
        assert_eq!(
            matching.hostname,
            "non-daita-relay".to_string(),
            "{matching:#?}"
        );
        // The daita-relay should be discarded due to incompatible location constraint.
        assert_eq!(discards.len(), 1);
        let (discarded, reasons) = discards.pop().unwrap();
        assert_eq!(
            discarded.hostname,
            "daita-relay".to_string(),
            "{discarded:#?}"
        );
        assert_eq!(reasons.as_slice(), &[Reason::Location]);
    }

    /// Test that the autohop predicate discards inactive relays.
    #[test]
    fn test_autohop_inactive() {
        let mut relay_list = RelayListBuilder::new();
        relay_list.add_location("country", "city");
        relay_list.add_relay("inactive").active = false;
        relay_list.add_relay("active");
        let relay_selector = RelaySelector::from(relay_list);
        let results = relay_selector.partition_relays(Predicate::Autohop(EntryConstraints {
            general: ExitConstraints {
                location: Constraint::Only(LocationConstraint::Location(
                    GeographicLocationConstraint::Hostname(
                        "country".into(),
                        "city".into(),
                        "inactive".into(),
                    ),
                )),
                ..Default::default()
            },
            ..Default::default()
        }));
        assert!(
            results
                .discards
                .iter()
                .find(|d| &d.0.hostname == "inactive")
                .unwrap()
                .1
                == vec![Reason::Inactive]
        );
        assert!(
            results
                .discards
                .iter()
                .find(|d| &d.0.hostname == "active")
                .unwrap()
                .1
                == vec![Reason::Location]
        );
    }

    /// Test some multihop scenarios:
    /// - First scenario:
    ///     - Selecting one entry city with exactly one mathcing relay will remove it from the list of exit relays.
    ///     - Selecting one exit city with exactly one matching relay will remove it from the list of entry relays.
    /// - Second scenario:
    ///     - Selecting one entry country with exactly one matching relay will remove it from the list of exit relays.
    ///     - Selecting one exit country with exactly one matching relay will remove it from the list of entry relays.
    #[test]
    fn multihop() {
        let relay_selector = {
            // Curate a relay list with exactly two relays in one country; one with DAITA & one without.
            let mut relay_list = RelayListBuilder::new();
            relay_list.add_location("albania", "tirana");
            // Without DAITA.
            relay_list.add_relay("non-daita-relay");
            // With DAITA.
            let daita_relay = relay_list.add_relay("daita-relay");
            daita_relay.endpoint_data.daita = true;
            RelaySelector::from(relay_list)
        };

        // Second scenario: The entry constraint is set rather loosely to "albania", but because there is
        // only one relay with DAITA the relay selector should reserve that relay when showing the list of
        // possible exit relays.
        //
        // Note: This is also a good description for the first scenario described in the doc-comment.
        let first = {
            let entry_constraints = EntryConstraints::default()
                .daita(true)
                .general(ExitConstraints::default().country("albania"));
            let exit_constraints = ExitConstraints::default().city("albania", "tirana");
            MultihopConstraints::default()
                .entry(entry_constraints)
                .exit(exit_constraints)
        };

        let second = {
            let entry_constraints = EntryConstraints::default()
                .daita(true)
                .general(ExitConstraints::default().country("albania"));
            let exit_constraints = ExitConstraints::default().country("albania");
            MultihopConstraints::default()
                .entry(entry_constraints)
                .exit(exit_constraints)
        };

        for scenario in [first, second] {
            let RelayPartitions {
                mut matches,
                mut discards,
                ..
            } = relay_selector.partition_relays(Predicate::Entry(scenario.clone()));
            // Should show the relay with DAITA, and discard the one without.
            let matched = matches.pop().unwrap();
            assert!(matched.daita(), "{matched:#?}");
            let discarded = discards.pop().unwrap();
            assert!(!discarded.0.daita(), "{discarded:#?}");
            // Assert that the exit relay list only contain the non-DAITA relay.
            let RelayPartitions {
                mut matches,
                mut discards,
                ..
            } = relay_selector.partition_relays(Predicate::Exit(scenario));
            // Should show the relay with DAITA, and discard the one without.
            let matched = matches.pop().unwrap();
            assert!(!matched.daita(), "{matched:#?}");
            let discarded = discards.pop().unwrap();
            assert!(discarded.0.daita(), "{discarded:#?}");
        }
    }

    mod snapshots {
        //! Snapshot tests for [`super::super::RelaySelector::partition_relays`].
        //!
        //! Each test covers one filtering scenario and produces a dedicated snapshot file in
        //! `tests/snapshots/`. The snapshots use the checked-in `relays.json` relay list so they
        //! reflect real production relay diversity: mixed ownership, multiple providers, relays
        //! with/without DAITA, QUIC, LWO, shadowsocks extra addresses, etc.
        //!
        //! The snapshots will be large, but that is intentional — any algorithmic change will
        //! produce a clear, reviewable diff across the full relay population.
        //!
        //! To review and accept changes after intentionally modifying the algorithm, run:
        //!
        //! ```text
        //! cargo insta review -p mullvad-relay-selector
        //! ```

        use std::collections::BTreeMap;

        use super::*;

        /// Converts [`RelayPartitions`] into a sorted `hostname -> status` map for snapshotting.
        ///
        /// Each relay is represented as a single line, e.g.:
        ///   `al-tia-wg-003: Discarded(Daita)`
        ///   `se-got-wg-101: Match`
        fn snapshot(predicate: Predicate) -> BTreeMap<String, String> {
            let RelayPartitions { matches, discards } =
                relay_selector().partition_relays(predicate);
            let mut map = BTreeMap::new();
            for relay in matches {
                map.insert(relay.inner.hostname, "Match".to_string());
            }
            for (relay, reasons) in discards {
                let reasons = reasons
                    .into_iter()
                    .map(|r| format!("{r:?}"))
                    .sorted()
                    .join(" ");
                map.insert(relay.inner.hostname, format!("Discarded({reasons})"));
            }
            map
        }

        // --- Singlehop ---

        #[test]
        fn singlehop_default() {
            insta::assert_yaml_snapshot!(snapshot(Predicate::Singlehop(
                EntryConstraints::default()
            )));
        }

        #[test]
        fn singlehop_daita() {
            insta::assert_yaml_snapshot!(snapshot(Predicate::Singlehop(
                EntryConstraints::default().daita(true)
            )));
        }

        #[test]
        fn singlehop_ipv6() {
            insta::assert_yaml_snapshot!(snapshot(Predicate::Singlehop(
                EntryConstraints::default().ip_version(IpVersion::V6)
            )));
        }

        #[test]
        fn singlehop_mullvad_owned() {
            insta::assert_yaml_snapshot!(snapshot(Predicate::Singlehop(
                EntryConstraints::default().ownership(Ownership::MullvadOwned)
            )));
        }

        #[test]
        fn singlehop_quic() {
            insta::assert_yaml_snapshot!(snapshot(Predicate::Singlehop(
                EntryConstraints::default().obfuscation(ObfuscationMode::Quic)
            )));
        }

        #[test]
        fn singlehop_lwo() {
            insta::assert_yaml_snapshot!(snapshot(Predicate::Singlehop(
                EntryConstraints::default()
                    .obfuscation(ObfuscationMode::Lwo(LwoSettings::default()))
            )));
        }

        #[test]
        fn singlehop_shadowsocks_any_port() {
            insta::assert_yaml_snapshot!(snapshot(Predicate::Singlehop(
                EntryConstraints::default()
                    .obfuscation(ObfuscationMode::Shadowsocks(ShadowsocksSettings::default()))
            )));
        }

        // --- Autohop ---
        // Autohop: DAITA relay is accepted directly; non-DAITA relays are only accepted if an
        // alternate DAITA entry can be found globally.

        #[test]
        fn autohop_default() {
            insta::assert_yaml_snapshot!(snapshot(Predicate::Autohop(EntryConstraints::default())));
        }

        #[test]
        fn autohop_daita() {
            insta::assert_yaml_snapshot!(snapshot(Predicate::Autohop(
                EntryConstraints::default().daita(true)
            )));
        }

        // --- Entry (multihop) ---
        // Shows which relays are eligible as the *entry* hop.

        #[test]
        fn entry_default() {
            insta::assert_yaml_snapshot!(snapshot(
                Predicate::Entry(MultihopConstraints::default())
            ));
        }

        #[test]
        fn entry_daita_entry() {
            insta::assert_yaml_snapshot!(snapshot(Predicate::Entry(
                MultihopConstraints::default().entry(EntryConstraints::default().daita(true))
            )));
        }

        #[test]
        fn entry_mullvad_owned_entry() {
            insta::assert_yaml_snapshot!(snapshot(Predicate::Entry(
                MultihopConstraints::default()
                    .entry(EntryConstraints::default().ownership(Ownership::MullvadOwned))
            )));
        }

        // --- Exit (multihop) ---
        // Shows which relays are eligible as the *exit* hop.

        #[test]
        fn exit_default() {
            insta::assert_yaml_snapshot!(snapshot(Predicate::Exit(MultihopConstraints::default())));
        }

        #[test]
        fn exit_mullvad_owned_exit() {
            insta::assert_yaml_snapshot!(snapshot(Predicate::Exit(
                MultihopConstraints::default()
                    .exit(ExitConstraints::default().ownership(Ownership::MullvadOwned))
            )));
        }
    }
}

mod relay_list_builder {
    use std::{fmt::Display, net::Ipv4Addr};

    use mullvad_relay_selector::{Relay, RelaySelector};
    use mullvad_types::{
        location::Location,
        relay_list::{
            BridgeList, EndpointData, RelayList, RelayListCity, RelayListCountry, WireguardRelay,
            WireguardRelayEndpointData,
        },
        settings::Settings,
    };
    use talpid_types::net::wireguard::PublicKey;

    pub struct RelayListBuilder {
        pub inner: RelayList,
    }

    impl From<RelayListBuilder> for RelaySelector {
        fn from(val: RelayListBuilder) -> Self {
            RelaySelector::from_settings(&Settings::default(), val.finish(), BridgeList::default())
        }
    }

    impl RelayListBuilder {
        pub fn new() -> Self {
            let relay_list = RelayList {
                countries: vec![RelayListCountry {
                    name: "test-country".into(),
                    code: "tc".into(),
                    cities: vec![RelayListCity {
                        name: "test-city".into(),
                        code: "tc".into(),
                        latitude: 0.0,
                        longitude: 0.0,
                        relays: vec![],
                    }],
                }],
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
                    udp2tcp_ports: vec![80, 443, 5001],
                    shadowsocks_port_ranges: vec![100..=200, 1000..=2000],
                },
            };
            Self { inner: relay_list }
        }

        /// Add a relay into the previously added location, which defaults to "test-country".
        pub fn add_relay(&mut self, hostname: impl Display) -> &mut WireguardRelay {
            let country = self
                .inner
                .countries
                .last_mut()
                .expect("Some active country");
            let city = country.cities.last_mut().expect("Some active city");
            city.relays.push(WireguardRelay {
                overridden_ipv4: false,
                overridden_ipv6: false,
                include_in_country: true,
                owned: true,
                provider: "TestProvider".into(),
                endpoint_data: WireguardRelayEndpointData::new(
                    PublicKey::from_base64("BLNHNoGO88LjV/wDBa7CUUwUzPq/fO2UwcGLy56hKy4=").unwrap(),
                ),
                inner: Relay {
                    hostname: hostname.to_string(),
                    ipv4_addr_in: Ipv4Addr::new(0, 0, 0, 0),
                    ipv6_addr_in: None,
                    active: true,
                    weight: 1,
                    location: Location {
                        country_code: country.code.clone(),
                        city: city.name.clone(),
                        latitude: city.latitude,
                        longitude: city.longitude,
                        country: country.name.clone(),
                        city_code: city.code.clone(),
                    },
                },
            });
            city.relays.last_mut().unwrap()
        }

        pub fn add_location(&mut self, country: &str, city: &str) {
            let country = match self.inner.countries.iter_mut().find(|c| c.code == country) {
                Some(country) => country,
                None => {
                    let country = RelayListCountry {
                        code: country.to_string(),
                        name: country.to_string(),
                        cities: vec![],
                    };
                    self.inner.countries.push(country);
                    self.inner.countries.last_mut().unwrap()
                }
            };
            let city = RelayListCity {
                code: city.to_string(),
                name: city.to_string(),
                latitude: 0.0,
                longitude: 0.0,
                relays: vec![],
            };
            country.cities.push(city);
        }

        pub fn finish(self) -> RelayList {
            self.inner
        }
    }
}

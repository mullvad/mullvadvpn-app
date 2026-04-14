//! This module contains various helper functions for the relay selector implementation.

use std::{
    net::{IpAddr, SocketAddr},
    ops::{Deref, RangeBounds, RangeInclusive},
};

use mullvad_types::{
    constraints::Constraint,
    endpoint::MullvadEndpoint,
    relay_constraints::{LwoSettings, ShadowsocksSettings, Udp2TcpObfuscationSettings},
    relay_list::{Relay, WireguardRelay},
};
use rand::{
    Rng,
    seq::{IndexedRandom, IteratorRandom},
};
use talpid_types::net::{IpVersion, obfuscation::ObfuscatorConfig};

#[cfg(feature = "staggered-obfuscation")]
use crate::SelectedObfuscator;

/// Port ranges available for WireGuard relays that have extra IPs for Shadowsocks.
/// For relays that have no additional IPs, only ports provided by the relay list are available.
const SHADOWSOCKS_EXTRA_PORT_RANGES: &[RangeInclusive<u16>] = &[1..=u16::MAX];

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Found no valid port matching the selected settings")]
    NoMatchingPort,
    #[error("Found no valid IP protocol matching the selected settings")]
    NoMatchingAddresses,
    #[error("The selected relay does not support the selected obfuscation method")]
    MissingSupport,
}

/// Picks a relay at random from `relays`, but don't pick `exclude`.
pub fn pick_random_relay_excluding<'a, T>(relays: &'a [T], exclude: &'_ T) -> Option<&'a T>
where
    T: Deref<Target = Relay>,
{
    let filtered_relays = relays.iter().filter(|&a| a.deref() != exclude.deref());
    pick_random_relay_weighted(filtered_relays, |relay: &T| relay.weight)
}

/// Picks a relay using [pick_random_relay_weighted], using the `weight` member of each relay
/// as the weight function.
pub fn pick_random_relay<T>(relays: &[T]) -> Option<&T>
where
    T: Deref<Target = Relay>,
{
    pick_random_relay_weighted(relays.iter(), |relay| relay.weight)
}

/// Pick a random relay from the given slice. Will return `None` if the given slice is empty.
/// If all of the relays have a weight of 0, one will be picked at random without bias,
/// otherwise roulette wheel selection will be used to pick only relays with non-zero
/// weights.
pub fn pick_random_relay_weighted<'a, RelayType>(
    mut relays: impl Iterator<Item = &'a RelayType> + Clone,
    weight: impl Fn(&'a RelayType) -> u64,
) -> Option<&'a RelayType> {
    let total_weight: u64 = relays.clone().map(&weight).sum();
    let mut rng = rand::rng();
    if total_weight == 0 {
        relays.choose(&mut rng)
    } else {
        // Assign each relay a subset of the range 0..total_weight with size equal to its weight.
        // Pick a random number in the range 1..=total_weight. This chooses the relay with a
        // non-zero weight.
        //
        //                           rng(1..=total_weight)
        //                           |
        //                           v
        //   ________________________i_______________________________________________
        // 0|_____________|____________________|___________|_____|________|__________| total_weight
        //  ^             ^                    ^                          ^          ^
        //  |             |                    |                          |          |
        //  ------------------------------------                          ------------
        //         |                  |                                         |
        //   weight(relay 0)     weight(relay 1)    ..       ..     ..    weight(relay n)
        let mut i: u64 = rng.random_range(1..=total_weight);
        Some(
            relays
                .find(|relay| {
                    i = i.saturating_sub(weight(relay));
                    i == 0
                })
                .expect("At least one relay must've had a weight above 0"),
        )
    }
}

/// Create a multiplexer obfuscator config
///
/// # Arguments
/// * `udp2tcp_ports` - Available ports for UDP2TCP obfuscation
/// * `shadowsocks_ports` - Available port ranges for Shadowsocks obfuscation
/// * `obfuscator_relay` - The relay that will host the obfuscation services
/// * `endpoint` - Selected endpoint
#[cfg(feature = "staggered-obfuscation")]
pub fn get_multiplexer_obfuscator(
    udp2tcp_ports: &[u16],
    shadowsocks_ports: &[RangeInclusive<u16>],
    obfuscator_relay: WireguardRelay,
    endpoint: &MullvadEndpoint,
) -> Result<SelectedObfuscator, Error> {
    use talpid_types::net::obfuscation::Obfuscators;

    // Direct (no obfuscation) method
    let direct = Some(endpoint.peer.endpoint);

    // Add obfuscation methods
    let mut configs = vec![];

    let udp2tcp = get_udp2tcp_obfuscator(
        &Udp2TcpObfuscationSettings::default(),
        udp2tcp_ports,
        obfuscator_relay.clone(),
        endpoint,
    )?;
    configs.push(udp2tcp.0);

    let shadowsocks = get_shadowsocks_obfuscator(
        &ShadowsocksSettings::default(),
        shadowsocks_ports,
        obfuscator_relay.clone(),
        endpoint,
    )?;
    configs.push(shadowsocks.0);

    let ip_version = match endpoint.peer.endpoint {
        SocketAddr::V4(_) => Constraint::Only(IpVersion::V4),
        SocketAddr::V6(_) => Constraint::Only(IpVersion::V6),
    };
    if let Some(quic) = get_quic_obfuscator(obfuscator_relay.clone(), ip_version) {
        configs.push(quic.0);
    }

    let config =
        Obfuscators::multiplexer(direct, &configs).expect("non-zero number of obfuscators");

    Ok(SelectedObfuscator {
        config,
        relay: obfuscator_relay,
    })
}

pub fn get_udp2tcp_obfuscator(
    obfuscation_settings_constraint: &Udp2TcpObfuscationSettings,
    udp2tcp_ports: &[u16],
    relay: WireguardRelay,
    endpoint: &MullvadEndpoint,
) -> Result<(ObfuscatorConfig, WireguardRelay), Error> {
    let udp2tcp_endpoint_port =
        get_udp2tcp_obfuscator_port(obfuscation_settings_constraint, udp2tcp_ports)?;
    let config = ObfuscatorConfig::Udp2Tcp {
        endpoint: SocketAddr::new(endpoint.peer.endpoint.ip(), udp2tcp_endpoint_port),
    };

    Ok((config, relay))
}

fn get_udp2tcp_obfuscator_port(
    obfuscation_settings: &Udp2TcpObfuscationSettings,
    udp2tcp_ports: &[u16],
) -> Result<u16, Error> {
    let port = if let Constraint::Only(desired_port) = obfuscation_settings.port {
        udp2tcp_ports
            .iter()
            .find(|&candidate| desired_port == *candidate)
            .copied()
    } else {
        // There are no specific obfuscation settings to take into consideration in this case.
        udp2tcp_ports.choose(&mut rand::rng()).copied()
    };
    port.ok_or(Error::NoMatchingPort)
}

pub fn get_shadowsocks_obfuscator(
    settings: &ShadowsocksSettings,
    non_extra_port_ranges: &[RangeInclusive<u16>],
    relay: WireguardRelay,
    endpoint: &MullvadEndpoint,
) -> Result<(ObfuscatorConfig, WireguardRelay), Error> {
    let port = settings.port;
    let extra_addrs = relay.endpoint_data.shadowsocks_extra_in_addrs();

    let endpoint = get_shadowsocks_obfuscator_inner(
        endpoint.peer.endpoint.ip(),
        non_extra_port_ranges,
        extra_addrs.copied(),
        port,
    )?;

    Ok((ObfuscatorConfig::Shadowsocks { endpoint }, relay))
}

pub fn get_quic_obfuscator(
    relay: WireguardRelay,
    ip_version: Constraint<IpVersion>,
) -> Option<(ObfuscatorConfig, WireguardRelay)> {
    let quic = relay.endpoint().quic()?;
    let config = {
        let hostname = quic.hostname().to_string();
        let addrs: Vec<IpAddr> = match ip_version {
            Constraint::Only(IpVersion::V4) => quic.in_ipv4().map(IpAddr::from).collect(),
            Constraint::Only(IpVersion::V6) => quic.in_ipv6().map(IpAddr::from).collect(),
            // Prefer IPv4, but fall back to IPv6 if the relay's QUIC obfuscator
            // only has IPv6 addresses.
            Constraint::Any => {
                let v4: Vec<IpAddr> = quic.in_ipv4().map(IpAddr::from).collect();
                if v4.is_empty() {
                    quic.in_ipv6().map(IpAddr::from).collect()
                } else {
                    v4
                }
            }
        };
        let &in_ip = addrs.iter().choose(&mut rand::rng())?;
        let endpoint = SocketAddr::from((in_ip, quic.port()));
        let auth_token = quic.auth_token().to_string();
        ObfuscatorConfig::Quic {
            hostname,
            endpoint,
            auth_token,
        }
    };

    Some((config, relay))
}

pub fn get_lwo_obfuscator(
    relay: WireguardRelay,
    endpoint: &MullvadEndpoint,
    port_ranges: &[RangeInclusive<u16>],
    settings: LwoSettings,
) -> Result<(ObfuscatorConfig, WireguardRelay), Error> {
    if !relay.endpoint().lwo {
        return Err(Error::MissingSupport);
    }
    let ip = match endpoint.peer.endpoint {
        SocketAddr::V4(_) => IpAddr::V4(relay.ipv4_addr_in),
        SocketAddr::V6(_) => IpAddr::V6(relay.ipv6_addr_in.ok_or(Error::NoMatchingAddresses)?),
    };
    let port = desired_or_random_port_from_range(port_ranges, settings.port)?;
    let endpoint = SocketAddr::new(ip, port);

    let config = ObfuscatorConfig::Lwo { endpoint };

    Ok((config, relay))
}

/// Return an obfuscation config for the wireguard server at `wg_in_addr` or one of `extra_in_addrs`
/// (unless empty). `wg_in_addr_port_ranges` contains all valid ports for `wg_in_addr`, and
/// `SHADOWSOCKS_EXTRA_PORT_RANGES` contains valid ports for `extra_in_addrs`.
fn get_shadowsocks_obfuscator_inner<R: RangeBounds<u16> + Iterator<Item = u16> + Clone>(
    wg_in_addr: IpAddr,
    wg_in_addr_port_ranges: &[R],
    extra_in_addrs: impl IntoIterator<Item = IpAddr>,
    desired_port: Constraint<u16>,
) -> Result<SocketAddr, Error> {
    // Filter out addresses for the wrong address family
    let extra_in_addrs: Vec<_> = extra_in_addrs
        .into_iter()
        .filter(|addr| addr.is_ipv4() == wg_in_addr.is_ipv4())
        .collect();

    let in_ip = extra_in_addrs
        .iter()
        .choose(&mut rand::rng())
        .copied()
        .unwrap_or(wg_in_addr);

    let selected_port = if extra_in_addrs.is_empty() {
        desired_or_random_port_from_range(wg_in_addr_port_ranges, desired_port)
    } else {
        desired_or_random_port_from_range(SHADOWSOCKS_EXTRA_PORT_RANGES, desired_port)
    }?;

    Ok(SocketAddr::from((in_ip, selected_port)))
}

/// Return `desired_port` if it is specified and included in `port_ranges`.
/// If `desired_port` isn't specified, return a random port from the ranges.
/// If `desired_port` is specified but not in range, return an error.
pub fn desired_or_random_port_from_range<R: RangeBounds<u16> + Iterator<Item = u16> + Clone>(
    port_ranges: &[R],
    desired_port: Constraint<u16>,
) -> Result<u16, Error> {
    match desired_port {
        Constraint::Only(port) => port_if_in_range(port_ranges, port),
        Constraint::Any => select_random_port(port_ranges),
    }
}

/// Return `Ok(port)`, if and only if `port` is in `port_ranges`. Otherwise, return an error.
fn port_if_in_range<R: RangeBounds<u16>>(port_ranges: &[R], port: u16) -> Result<u16, Error> {
    port_ranges
        .iter()
        .find_map(|range| {
            if range.contains(&port) {
                Some(port)
            } else {
                None
            }
        })
        .ok_or(Error::NoMatchingPort)
}

/// Select a random port number from a list of provided port ranges.
///
/// # Parameters
/// - `port_ranges`: A slice of port numbers.
///
/// # Returns
/// - On success, a randomly selected port number within the given ranges. Otherwise, an error is
///   returned.
pub fn select_random_port<R: RangeBounds<u16> + Iterator<Item = u16> + Clone>(
    port_ranges: &[R],
) -> Result<u16, Error> {
    port_ranges
        .iter()
        .cloned()
        .flatten()
        .choose(&mut rand::rng())
        .ok_or(Error::NoMatchingPort)
}

#[cfg(test)]
mod tests {
    use super::{
        SHADOWSOCKS_EXTRA_PORT_RANGES, get_shadowsocks_obfuscator_inner, port_if_in_range,
    };
    use mullvad_types::constraints::Constraint;
    use std::{iter, net::IpAddr, ops::RangeInclusive};

    /// Test whether select ports are available when relay has no extra IPs
    #[test]
    fn test_shadowsocks_no_extra_addrs() {
        const PORT_RANGES: &[RangeInclusive<u16>] = &[100..=200, 1000..=2000];
        const WITHIN_RANGE_PORT: u16 = 100;
        const OUT_OF_RANGE_PORT: u16 = 1;
        let wg_in_ip: IpAddr = "1.2.3.4".parse().unwrap();

        let selected_addr =
            get_shadowsocks_obfuscator_inner(wg_in_ip, PORT_RANGES, iter::empty(), Constraint::Any)
                .expect("should find valid port without constraint");

        assert_eq!(selected_addr.ip(), wg_in_ip);
        assert!(
            port_if_in_range(PORT_RANGES, selected_addr.port()).is_ok(),
            "expected port in port range"
        );

        let selected_addr = get_shadowsocks_obfuscator_inner(
            wg_in_ip,
            PORT_RANGES,
            iter::empty(),
            Constraint::Only(WITHIN_RANGE_PORT),
        )
        .expect("should find within-range port");

        assert_eq!(selected_addr.ip(), wg_in_ip);
        assert!(
            port_if_in_range(PORT_RANGES, selected_addr.port()).is_ok(),
            "expected port in port range"
        );

        let selected_addr = get_shadowsocks_obfuscator_inner(
            wg_in_ip,
            PORT_RANGES,
            iter::empty(),
            Constraint::Only(OUT_OF_RANGE_PORT),
        );
        assert!(
            selected_addr.is_err(),
            "expected no relay for port outside range, found {selected_addr:?}"
        );
    }

    /// All ports should be available when relay has extra IPs, and only extra IPs should be used
    #[test]
    fn test_shadowsocks_extra_addrs() {
        const PORT_RANGES: &[RangeInclusive<u16>] = &[100..=200, 1000..=2000];
        const OUT_OF_RANGE_PORT: u16 = 1;
        let wg_in_ip: IpAddr = "1.2.3.4".parse().unwrap();

        let extra_in_addrs: Vec<IpAddr> =
            vec!["1.3.3.7".parse().unwrap(), "192.0.2.123".parse().unwrap()];

        let selected_addr = get_shadowsocks_obfuscator_inner(
            wg_in_ip,
            PORT_RANGES,
            extra_in_addrs.clone(),
            Constraint::Any,
        )
        .expect("should find valid port without constraint");

        assert!(
            extra_in_addrs.contains(&selected_addr.ip()),
            "expected extra IP to be selected"
        );
        assert!(port_if_in_range(SHADOWSOCKS_EXTRA_PORT_RANGES, selected_addr.port(),).is_ok());

        let selected_addr = get_shadowsocks_obfuscator_inner(
            wg_in_ip,
            PORT_RANGES,
            extra_in_addrs.clone(),
            Constraint::Only(OUT_OF_RANGE_PORT),
        )
        .expect("expected selected address to be returned");
        assert!(
            extra_in_addrs.contains(&selected_addr.ip()),
            "expected extra IP to be selected, got {selected_addr:?}"
        );
        assert_eq!(
            selected_addr.port(),
            OUT_OF_RANGE_PORT,
            "expected selected port, got {selected_addr:?}"
        );
    }

    /// Extra addresses that belong to the wrong IP family should be ignored
    #[test]
    fn test_shadowsocks_irrelevant_extra_addrs() {
        const PORT_RANGES: &[RangeInclusive<u16>] = &[100..=200, 1000..=2000];
        const IN_RANGE_PORT: u16 = 100;
        const OUT_OF_RANGE_PORT: u16 = 1;
        let wg_in_ip: IpAddr = "1.2.3.4".parse().unwrap();

        let extra_in_addrs: Vec<IpAddr> = vec!["::2".parse().unwrap()];

        let selected_addr = get_shadowsocks_obfuscator_inner(
            wg_in_ip,
            PORT_RANGES,
            extra_in_addrs.clone(),
            Constraint::Any,
        )
        .expect("should find valid port without constraint");

        assert_eq!(
            selected_addr.ip(),
            wg_in_ip,
            "expected extra IP to be ignored"
        );

        let selected_addr = get_shadowsocks_obfuscator_inner(
            wg_in_ip,
            PORT_RANGES,
            extra_in_addrs.clone(),
            Constraint::Only(OUT_OF_RANGE_PORT),
        );
        assert!(
            selected_addr.is_err(),
            "expected no match for out-of-range port"
        );

        let selected_addr = get_shadowsocks_obfuscator_inner(
            wg_in_ip,
            PORT_RANGES,
            extra_in_addrs,
            Constraint::Only(IN_RANGE_PORT),
        );
        assert!(
            selected_addr.is_ok(),
            "expected match for within-range port"
        );
    }

    mod quic {
        use super::super::get_quic_obfuscator;
        use mullvad_types::{
            constraints::Constraint,
            location::Location,
            relay_list::{Quic, Relay, WireguardRelay, WireguardRelayEndpointData},
        };
        use std::net::IpAddr;
        use talpid_types::net::{IpVersion, obfuscation::ObfuscatorConfig, wireguard::PublicKey};
        fn dummy_relay(quic_addrs: Vec<IpAddr>) -> WireguardRelay {
            let quic = Quic::new(
                quic_addrs.try_into().expect("quic_addrs must be non-empty"),
                "token".to_string(),
                "example.com".to_string(),
            );
            let mut endpoint_data = WireguardRelayEndpointData::new(PublicKey::from([0u8; 32]));
            endpoint_data.quic = Some(quic);
            WireguardRelay::new(
                false,
                false,
                true,
                true,
                "provider".to_string(),
                endpoint_data,
                Relay {
                    hostname: "se-got-wg-003".to_string(),
                    ipv4_addr_in: "1.2.3.4".parse().unwrap(),
                    ipv6_addr_in: None,
                    active: true,
                    weight: 100,
                    location: Location {
                        country: "Sweden".to_string(),
                        country_code: "se".to_string(),
                        city: "Gothenburg".to_string(),
                        city_code: "got".to_string(),
                        latitude: 57.71,
                        longitude: 11.97,
                    },
                },
            )
        }

        /// IPv4-only runtime with relay that has no IPv4 QUIC addresses should return None.
        #[test]
        fn ipv4_only_no_ipv4_quic_addrs() {
            let relay = dummy_relay(vec!["::1".parse().unwrap(), "::2".parse().unwrap()]);
            let result = get_quic_obfuscator(relay, Constraint::Only(IpVersion::V4));
            assert!(
                result.is_none(),
                "expected no QUIC obfuscator for IPv4-only with only IPv6 QUIC addresses"
            );
        }

        /// IPv6 available at runtime, relay has only IPv6 QUIC addresses - should succeed.
        #[test]
        fn ipv6_available_only_ipv6_quic_addrs() {
            let ipv6_addr: IpAddr = "::1".parse().unwrap();
            let relay = dummy_relay(vec![ipv6_addr]);
            let (config, _relay) = get_quic_obfuscator(relay, Constraint::Only(IpVersion::V6))
                .expect("expected QUIC obfuscator with IPv6 address");
            let ObfuscatorConfig::Quic { endpoint, .. } = config else {
                panic!("expected Quic config");
            };
            assert!(endpoint.ip().is_ipv6(), "expected IPv6 endpoint");
        }

        /// When unconstrained (Any), prefers IPv4 if available, even when IPv6 also exists.
        #[test]
        fn any_prefers_ipv4_when_available() {
            let ipv4: IpAddr = "10.0.0.1".parse().unwrap();
            let ipv6: IpAddr = "::1".parse().unwrap();
            let relay = dummy_relay(vec![ipv4, ipv6]);
            let (config, _relay) =
                get_quic_obfuscator(relay, Constraint::Any).expect("expected QUIC obfuscator");
            let ObfuscatorConfig::Quic { endpoint, .. } = config else {
                panic!("expected Quic config");
            };
            assert!(
                endpoint.ip().is_ipv4(),
                "expected IPv4 endpoint when both are available"
            );
        }

        /// When unconstrained (Any) and relay only has IPv6 QUIC addresses, falls back to IPv6.
        #[test]
        fn any_falls_back_to_ipv6() {
            let ipv6: IpAddr = "::1".parse().unwrap();
            let relay = dummy_relay(vec![ipv6]);
            let (config, _relay) = get_quic_obfuscator(relay, Constraint::Any)
                .expect("expected QUIC obfuscator to fall back to IPv6");
            let ObfuscatorConfig::Quic { endpoint, .. } = config else {
                panic!("expected Quic config");
            };
            assert!(endpoint.ip().is_ipv6(), "expected IPv6 fallback endpoint");
        }

        /// When constrained to IPv6, should only return IPv6 addresses.
        #[test]
        fn ipv6_constraint_ignores_ipv4() {
            let ipv4: IpAddr = "10.0.0.1".parse().unwrap();
            let ipv6: IpAddr = "::1".parse().unwrap();
            let relay = dummy_relay(vec![ipv4, ipv6]);
            let (config, _relay) = get_quic_obfuscator(relay, Constraint::Only(IpVersion::V6))
                .expect("expected QUIC obfuscator");
            let ObfuscatorConfig::Quic { endpoint, .. } = config else {
                panic!("expected Quic config");
            };
            assert!(
                endpoint.ip().is_ipv6(),
                "expected only IPv6 when constrained"
            );
        }
    }
}

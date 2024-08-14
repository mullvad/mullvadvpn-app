//! This module contains various helper functions for the relay selector implementation.

use std::{
    net::{IpAddr, SocketAddr},
    ops::{RangeBounds, RangeInclusive},
};

use mullvad_types::{
    constraints::Constraint,
    endpoint::MullvadWireguardEndpoint,
    relay_constraints::{ShadowsocksSettings, Udp2TcpObfuscationSettings},
    relay_list::Relay,
};
use rand::{
    seq::{IteratorRandom, SliceRandom},
    thread_rng, Rng,
};
use talpid_types::net::obfuscation::ObfuscatorConfig;

use crate::SelectedObfuscator;

/// Port ranges available for WireGuard relays that have extra IPs for Shadowsocks.
/// For relays that have no additional IPs, only ports provided by the relay list are available.
const SHADOWSOCKS_EXTRA_PORT_RANGES: &[RangeInclusive<u16>] = &[1..=u16::MAX];

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Found no valid port matching the selected settings")]
    NoMatchingPort,
}

/// Picks a relay using [pick_random_relay_weighted], using the `weight` member of each relay
/// as the weight function.
pub fn pick_random_relay(relays: &[Relay]) -> Option<&Relay> {
    pick_random_relay_weighted(relays, |relay| relay.weight)
}

/// Pick a random relay from the given slice. Will return `None` if the given slice is empty.
/// If all of the relays have a weight of 0, one will be picked at random without bias,
/// otherwise roulette wheel selection will be used to pick only relays with non-zero
/// weights.
pub fn pick_random_relay_weighted<RelayType>(
    relays: &[RelayType],
    weight: impl Fn(&RelayType) -> u64,
) -> Option<&RelayType> {
    let total_weight: u64 = relays.iter().map(&weight).sum();
    let mut rng = thread_rng();
    if total_weight == 0 {
        relays.choose(&mut rng)
    } else {
        // Assign each relay a subset of the range 0..total_weight with size equal to its weight.
        // Pick a random number in the range 1..=total_weight. This choses the relay with a
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
        let mut i: u64 = rng.gen_range(1..=total_weight);
        Some(
            relays
                .iter()
                .find(|relay| {
                    i = i.saturating_sub(weight(relay));
                    i == 0
                })
                .expect("At least one relay must've had a weight above 0"),
        )
    }
}

pub fn get_udp2tcp_obfuscator(
    obfuscation_settings_constraint: &Udp2TcpObfuscationSettings,
    udp2tcp_ports: &[u16],
    relay: Relay,
    endpoint: &MullvadWireguardEndpoint,
) -> Result<SelectedObfuscator, Error> {
    let udp2tcp_endpoint_port =
        get_udp2tcp_obfuscator_port(obfuscation_settings_constraint, udp2tcp_ports)?;
    let config = ObfuscatorConfig::Udp2Tcp {
        endpoint: SocketAddr::new(endpoint.peer.endpoint.ip(), udp2tcp_endpoint_port),
    };

    Ok(SelectedObfuscator { config, relay })
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
        udp2tcp_ports.choose(&mut thread_rng()).copied()
    };
    port.ok_or(Error::NoMatchingPort)
}

pub fn get_shadowsocks_obfuscator(
    settings: &ShadowsocksSettings,
    non_extra_port_ranges: &[RangeInclusive<u16>],
    relay: Relay,
    endpoint: &MullvadWireguardEndpoint,
) -> Result<SelectedObfuscator, Error> {
    let port = settings.port;
    let extra_addrs = match &relay.endpoint_data {
        mullvad_types::relay_list::RelayEndpointData::Wireguard(wg) => {
            &wg.shadowsocks_extra_addr_in
        }
        _ => panic!("expected wireguard relay"),
    };

    let endpoint = get_shadowsocks_obfuscator_inner(
        endpoint.peer.endpoint.ip(),
        non_extra_port_ranges,
        extra_addrs,
        port,
    )?;

    Ok(SelectedObfuscator {
        config: ObfuscatorConfig::Shadowsocks { endpoint },
        relay,
    })
}

/// Return an obfuscation config for the wireguard server at `wg_in_addr` or one of `extra_in_addrs`
/// (unless empty). `wg_in_addr_port_ranges` contains all valid ports for `wg_in_addr`, and
/// `SHADOWSOCKS_EXTRA_PORT_RANGES` contains valid ports for `extra_in_addrs`.
fn get_shadowsocks_obfuscator_inner<R: RangeBounds<u16> + Iterator<Item = u16> + Clone>(
    wg_in_addr: IpAddr,
    wg_in_addr_port_ranges: &[R],
    extra_in_addrs: &[IpAddr],
    desired_port: Constraint<u16>,
) -> Result<SocketAddr, Error> {
    // Filter out addresses for the wrong address family
    let extra_in_addrs: Vec<_> = extra_in_addrs
        .iter()
        .filter(|addr| addr.is_ipv4() == wg_in_addr.is_ipv4())
        .copied()
        .collect();

    let in_ip = extra_in_addrs
        .iter()
        .choose(&mut rand::thread_rng())
        .copied()
        .unwrap_or(wg_in_addr);

    let selected_port = if extra_in_addrs.is_empty() {
        desired_port_from_range(wg_in_addr_port_ranges, desired_port)
    } else {
        desired_port_from_range(SHADOWSOCKS_EXTRA_PORT_RANGES, desired_port)
    }?;

    Ok(SocketAddr::from((in_ip, selected_port)))
}

fn desired_port_from_range<R: RangeBounds<u16> + Iterator<Item = u16> + Clone>(
    port_ranges: &[R],
    desired_port: Constraint<u16>,
) -> Result<u16, Error> {
    match desired_port {
        // Selected a specific, in-range port
        Constraint::Only(port) if port_in_range(port, port_ranges) => Ok(port),
        // Selected a specific, out-of-range port
        Constraint::Only(_port) => Err(Error::NoMatchingPort),
        // Selected no specific port
        Constraint::Any => select_random_port(port_ranges),
    }
}

/// Selects a random port number from a list of provided port ranges.
///
/// This function iterates over a list of port ranges, each represented as a tuple (u16, u16)
/// where the first element is the start of the range and the second is the end (inclusive),
/// and selects a random port from the set of all ranges.
///
/// # Parameters
/// - `port`: Constraint to apply to the port selection
/// - `port_ranges`: A slice of tuples, each representing a range of valid port numbers.
///
/// # Returns
/// - A randomly selected port number within the given ranges.
/// - An error if `port_ranges` is empty.
pub fn select_random_port<R: RangeBounds<u16> + Iterator<Item = u16> + Clone>(
    port_ranges: &[R],
) -> Result<u16, Error> {
    port_ranges
        .iter()
        .cloned()
        .flatten()
        .choose(&mut rand::thread_rng())
        .ok_or(Error::NoMatchingPort)
}

pub fn port_in_range<R: RangeBounds<u16>>(port: u16, port_ranges: &[R]) -> bool {
    port_ranges.iter().any(|range| range.contains(&port))
}

#[cfg(test)]
mod tests {
    use super::{get_shadowsocks_obfuscator_inner, port_in_range, SHADOWSOCKS_EXTRA_PORT_RANGES};
    use mullvad_types::constraints::Constraint;
    use std::{net::IpAddr, ops::RangeInclusive};

    /// Test whether select ports are available when relay has no extra IPs
    #[test]
    fn test_shadowsocks_no_extra_addrs() {
        const PORT_RANGES: &[RangeInclusive<u16>] = &[100..=200, 1000..=2000];
        const WITHIN_RANGE_PORT: u16 = 100;
        const OUT_OF_RANGE_PORT: u16 = 1;
        let wg_in_ip: IpAddr = "1.2.3.4".parse().unwrap();

        let selected_addr =
            get_shadowsocks_obfuscator_inner(wg_in_ip, PORT_RANGES, &[], Constraint::Any)
                .expect("should find valid port without constraint");

        assert_eq!(selected_addr.ip(), wg_in_ip);
        assert!(
            port_in_range(selected_addr.port(), PORT_RANGES),
            "expected port in port range"
        );

        let selected_addr = get_shadowsocks_obfuscator_inner(
            wg_in_ip,
            PORT_RANGES,
            &[],
            Constraint::Only(WITHIN_RANGE_PORT),
        )
        .expect("should find within-range port");

        assert_eq!(selected_addr.ip(), wg_in_ip);
        assert!(
            port_in_range(selected_addr.port(), PORT_RANGES),
            "expected port in port range"
        );

        let selected_addr = get_shadowsocks_obfuscator_inner(
            wg_in_ip,
            PORT_RANGES,
            &[],
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

        let extra_in_addrs: &[IpAddr] =
            &["1.3.3.7".parse().unwrap(), "192.0.2.123".parse().unwrap()];

        let selected_addr = get_shadowsocks_obfuscator_inner(
            wg_in_ip,
            PORT_RANGES,
            extra_in_addrs,
            Constraint::Any,
        )
        .expect("should find valid port without constraint");

        assert!(
            extra_in_addrs.contains(&selected_addr.ip()),
            "expected extra IP to be selected"
        );
        assert!(port_in_range(
            selected_addr.port(),
            SHADOWSOCKS_EXTRA_PORT_RANGES
        ));

        let selected_addr = get_shadowsocks_obfuscator_inner(
            wg_in_ip,
            PORT_RANGES,
            extra_in_addrs,
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

        let extra_in_addrs: &[IpAddr] = &["::2".parse().unwrap()];

        let selected_addr = get_shadowsocks_obfuscator_inner(
            wg_in_ip,
            PORT_RANGES,
            extra_in_addrs,
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
            extra_in_addrs,
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
}

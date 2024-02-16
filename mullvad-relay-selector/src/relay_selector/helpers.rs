//! This module contains various helper functions for the relay selector implementation.

use std::net::SocketAddr;

use mullvad_types::{
    constraints::Constraint,
    endpoint::MullvadWireguardEndpoint,
    relay_constraints::Udp2TcpObfuscationSettings,
    relay_list::{BridgeEndpointData, Relay, RelayEndpointData},
};
use rand::{
    seq::{IteratorRandom, SliceRandom},
    thread_rng, Rng,
};
use talpid_types::net::{obfuscation::ObfuscatorConfig, proxy::CustomProxy};

use crate::SelectedObfuscator;

/// Pick a random element out of `from`, excluding the element `exclude` from the selection.
pub fn random<'a, A: PartialEq>(
    from: impl IntoIterator<Item = &'a A>,
    exclude: &A,
) -> Option<&'a A> {
    from.into_iter()
        .filter(|&a| a != exclude)
        .choose(&mut thread_rng())
}

/// Picks a relay using [Self::pick_random_relay_fn], using the `weight` member of each relay
/// as the weight function.
pub fn pick_random_relay(relays: &[Relay]) -> Option<&Relay> {
    pick_random_relay_fn(relays, |relay| relay.weight)
}

/// Pick a random relay from the given slice. Will return `None` if the given slice is empty.
/// If all of the relays have a weight of 0, one will be picked at random without bias,
/// otherwise roulette wheel selection will be used to pick only relays with non-zero
/// weights.
pub fn pick_random_relay_fn<RelayType>(
    relays: &[RelayType],
    weight_fn: impl Fn(&RelayType) -> u64,
) -> Option<&RelayType> {
    let total_weight: u64 = relays.iter().map(&weight_fn).sum();
    let mut rng = thread_rng();
    if total_weight == 0 {
        relays.choose(&mut rng)
    } else {
        // Pick a random number in the range 1..=total_weight. This choses the relay with a
        // non-zero weight.
        let mut i: u64 = rng.gen_range(1..=total_weight);
        Some(
            relays
                .iter()
                .find(|relay| {
                    i = i.saturating_sub(weight_fn(relay));
                    i == 0
                })
                .expect("At least one relay must've had a weight above 0"),
        )
    }
}

/// Picks a random bridge from a relay.
pub fn pick_random_bridge(data: &BridgeEndpointData, relay: &Relay) -> Option<CustomProxy> {
    if relay.endpoint_data != RelayEndpointData::Bridge {
        return None;
    }
    let shadowsocks_endpoint = data.shadowsocks.choose(&mut rand::thread_rng());
    if let Some(shadowsocks_endpoint) = shadowsocks_endpoint {
        log::info!(
            "Selected Shadowsocks bridge {} at {}:{}/{}",
            relay.hostname,
            relay.ipv4_addr_in,
            shadowsocks_endpoint.port,
            shadowsocks_endpoint.protocol
        );
    }
    shadowsocks_endpoint
        .map(|endpoint_data| endpoint_data.to_proxy_settings(relay.ipv4_addr_in.into()))
}

pub fn get_udp2tcp_obfuscator(
    obfuscation_settings_constraint: &Constraint<Udp2TcpObfuscationSettings>,
    udp2tcp_ports: &[u16],
    relay: Relay,
    endpoint: &MullvadWireguardEndpoint,
) -> Option<SelectedObfuscator> {
    let udp2tcp_endpoint_port =
        get_udp2tcp_obfuscator_port(obfuscation_settings_constraint, udp2tcp_ports)?;
    let config = ObfuscatorConfig::Udp2Tcp {
        endpoint: SocketAddr::new(endpoint.peer.endpoint.ip(), udp2tcp_endpoint_port),
    };

    Some(SelectedObfuscator { config, relay })
}

pub fn get_udp2tcp_obfuscator_port(
    obfuscation_settings_constraint: &Constraint<Udp2TcpObfuscationSettings>,
    udp2tcp_ports: &[u16],
) -> Option<u16> {
    match obfuscation_settings_constraint {
        Constraint::Only(obfuscation_settings) if obfuscation_settings.port.is_only() => {
            udp2tcp_ports
                .iter()
                .find(|&candidate| obfuscation_settings.port == Constraint::Only(*candidate))
                .copied()
        }
        // There are no specific obfuscation settings to take into consideration in this case.
        Constraint::Any | Constraint::Only(_) => udp2tcp_ports.choose(&mut thread_rng()).copied(),
    }
}

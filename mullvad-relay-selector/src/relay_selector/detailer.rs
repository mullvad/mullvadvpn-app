//! This module implement functions for producing a [`MullvadEndpoint`] given a Wireguard or
//! OpenVPN relay chosen by the relay selector.
//!
//! [`MullvadEndpoint`] contains all the necessary information for establishing a connection
//! between the client and Mullvad VPN. It is the daemon's responsibillity of actually establishing
//! this connection.

use std::net::{IpAddr, SocketAddr, SocketAddrV4};

use ipnetwork::IpNetwork;
use mullvad_types::{
    constraints::Constraint,
    endpoint::MullvadWireguardEndpoint,
    relay_constraints::TransportPort,
    relay_list::{OpenVpnEndpoint, OpenVpnEndpointData, Relay, WireguardEndpointData},
};
use talpid_types::net::{
    all_of_the_internet, wireguard::PeerConfig, Endpoint, IpVersion, TransportProtocol,
};

use crate::constants::WIREGUARD_EXIT_PORT;

use super::{
    query::{BridgeQuery, OpenVpnRelayQuery, WireguardRelayQuery},
    WireguardConfig,
};

/// Given an OpenVPN relay and the original query the relay selector used,
/// fill in all connection details to produce a valid [`Endpoint`] by
/// calling [`to_endpoint`].
///
/// [`to_endpoint`]: OpenVpnDetailer::to_endpoint
pub struct OpenVpnDetailer<'a> {
    openvpn_constraints: &'a OpenVpnRelayQuery,
    exit: &'a Relay,
    data: &'a OpenVpnEndpointData,
}

impl<'a> OpenVpnDetailer<'a> {
    /// Create a new [`OpenVpnDetailer`].
    pub const fn new(
        query: &'a OpenVpnRelayQuery,
        relay: &'a Relay,
        data: &'a OpenVpnEndpointData,
    ) -> OpenVpnDetailer<'a> {
        Self {
            openvpn_constraints: query,
            exit: relay,
            data,
        }
    }

    /// Map `self` to a [`Endpoint`].
    ///
    /// If this endpoint is to be used in conjunction with a bridge, the resulting endpoint is
    /// guaranteed to use transport protocol `TCP`.
    ///
    /// This function can fail if no valid port + transport protocol combination is found.
    /// See [`OpenVpnEndpointData`] for more details.
    pub fn to_endpoint(&self) -> Option<Endpoint> {
        // If `bridge_mode` is true, this function may only return endpoints which use TCP, not UDP.
        if BridgeQuery::should_use_bridge(&self.openvpn_constraints.bridge_settings) {
            self.to_bridged_endpoint()
        } else {
            self.to_singlehop_endpoint()
        }
    }

    /// Configure a single-hop connection using the exit relay data.
    fn to_singlehop_endpoint(&self) -> Option<Endpoint> {
        use rand::seq::IteratorRandom;
        let constraints_port = self.openvpn_constraints.port;
        self.data
            .ports
            .iter()
            .filter(|&endpoint| Self::compatible_port_combo(&constraints_port, endpoint))
            .choose(&mut rand::thread_rng())
            .map(|endpoint| Endpoint::new(self.exit.ipv4_addr_in, endpoint.port, endpoint.protocol))
    }

    /// Configure an endpoint that will be used together with a bridge.
    ///
    /// # Note
    /// In bridge mode, the only viable transport protocol is TCP. Otherwise, this function is
    /// identical to [`Self::to_singlehop_endpoint`].
    fn to_bridged_endpoint(&self) -> Option<Endpoint> {
        use rand::seq::IteratorRandom;
        let constraints_port = self.openvpn_constraints.port;
        self.data
            .ports
            .iter()
            .filter(|endpoint| matches!(endpoint.protocol, TransportProtocol::Tcp))
            .filter(|endpoint| Self::compatible_port_combo(&constraints_port, endpoint))
            .choose(&mut rand::thread_rng())
            .map(|endpoint| Endpoint::new(self.exit.ipv4_addr_in, endpoint.port, endpoint.protocol))
    }

    /// Returns true if `port_constraint` can be used to connect to `endpoint`.
    /// Otherwise, false is returned.
    fn compatible_port_combo(
        port_constraint: &Constraint<TransportPort>,
        endpoint: &OpenVpnEndpoint,
    ) -> bool {
        match port_constraint {
            Constraint::Any => true,
            Constraint::Only(transport_port) => match transport_port.port {
                Constraint::Any => transport_port.protocol == endpoint.protocol,
                Constraint::Only(port) => {
                    port == endpoint.port && transport_port.protocol == endpoint.protocol
                }
            },
        }
    }
}

// -- These shall overthrow the tyranny of the structs --

/// Constructs a [`MullvadWireguardEndpoint`] with details for a Wireguard circuit.
///
/// If entry is `None`, `to_endpoint` configures a single-hop connection using the exit relay data.
/// Otherwise, it constructs a multihop setup using both entry and exit to set up appropriate peer
/// configurations.
///
/// # Returns
/// - A configured Mullvad endpoint for Wireguard, encapsulating either a single-hop or multi-hop connection setup.
/// - Returns `None` if the desired port is not in a valid port range (see
/// [`WireguradRelayQuery::port`]) or relay addresses cannot be resolved.
pub fn wireguard_endpoint(
    query: &WireguardRelayQuery,
    data: &WireguardEndpointData,
    relay: &WireguardConfig,
) -> Option<MullvadWireguardEndpoint> {
    match relay {
        WireguardConfig::Singlehop { exit } => to_singlehop_endpoint(query, data, exit),
        WireguardConfig::Multihop { exit, entry } => to_multihop_endpoint(query, data, exit, entry),
    }
}

/// Configure a single-hop connection using the exit relay data.
fn to_singlehop_endpoint(
    query: &WireguardRelayQuery,
    data: &WireguardEndpointData,
    exit: &Relay,
) -> Option<MullvadWireguardEndpoint> {
    let endpoint = {
        let host = get_address_for_wireguard_relay(query, exit)?;
        let port = get_port_for_wireguard_relay(query, data)?;
        SocketAddr::new(host, port)
    };
    let peer_config = PeerConfig {
        public_key: exit.endpoint_data.unwrap_wireguard_ref().public_key.clone(),
        endpoint,
        allowed_ips: all_of_the_internet(),
        // This will be filled in later, not the relay selector's problem
        psk: None,
    };
    Some(MullvadWireguardEndpoint {
        peer: peer_config,
        exit_peer: None,
        ipv4_gateway: data.ipv4_gateway,
        ipv6_gateway: data.ipv6_gateway,
    })
}

/// Configure a multihop connection using the entry & exit relay data.
///
/// # Note
/// In a multihop circuit, we need to provide an exit peer configuration in addition to the
/// peer configuration.
fn to_multihop_endpoint(
    query: &WireguardRelayQuery,
    data: &WireguardEndpointData,
    exit: &Relay,
    entry: &Relay,
) -> Option<MullvadWireguardEndpoint> {
    let exit_endpoint = {
        let ip = exit.ipv4_addr_in;
        // The port that the exit relay listens for incoming connections from entry
        // relays is *not* derived from the original query / user settings.
        let port = WIREGUARD_EXIT_PORT;
        SocketAddrV4::new(ip, port).into()
    };
    let exit = PeerConfig {
        public_key: exit.endpoint_data.unwrap_wireguard_ref().public_key.clone(),
        endpoint: exit_endpoint,
        // The exit peer should be able to route incomming VPN traffic to the rest of
        // the internet.
        allowed_ips: all_of_the_internet(),
        // This will be filled in later, not the relay selector's problem
        psk: None,
    };

    let entry_endpoint = {
        let host = get_address_for_wireguard_relay(query, entry)?;
        let port = get_port_for_wireguard_relay(query, data)?;
        SocketAddr::new(host, port)
    };
    let entry = PeerConfig {
        public_key: entry
            .endpoint_data
            .unwrap_wireguard_ref()
            .public_key
            .clone(),
        endpoint: entry_endpoint,
        // The entry peer should only be able to route incomming VPN traffic to the
        // exit peer.
        allowed_ips: vec![IpNetwork::from(exit.endpoint.ip())],
        // This will be filled in later
        psk: None,
    };

    Some(MullvadWireguardEndpoint {
        peer: entry,
        exit_peer: Some(exit),
        ipv4_gateway: data.ipv4_gateway,
        ipv6_gateway: data.ipv6_gateway,
    })
}

/// Get the correct IP address for the given relay.
fn get_address_for_wireguard_relay(query: &WireguardRelayQuery, relay: &Relay) -> Option<IpAddr> {
    match query.ip_version {
        Constraint::Any | Constraint::Only(IpVersion::V4) => Some(relay.ipv4_addr_in.into()),
        Constraint::Only(IpVersion::V6) => relay.ipv6_addr_in.map(|addr| addr.into()),
    }
}

// Try to pick a valid Wireguard port.
fn get_port_for_wireguard_relay(
    query: &WireguardRelayQuery,
    data: &WireguardEndpointData,
) -> Option<u16> {
    match query.port {
        Constraint::Any => {
            let random_port = select_random_port(&data.port_ranges);
            if random_port.is_none() {
                log::error!("Port selection algorithm is broken!");
            }
            random_port
        }
        Constraint::Only(port) => {
            if data
                .port_ranges
                .iter()
                .any(|range| (range.0 <= port && port <= range.1))
            {
                Some(port)
            } else {
                None
            }
        }
    }
}

/// Selects a random port number from a list of provided port ranges.
///
/// This function iterates over a list of port ranges, each represented as a tuple (u16, u16)
/// where the first element is the start of the range and the second is the end (inclusive),
/// and selects a random port from the set of all ranges.
///
/// # Parameters
/// - `port_ranges`: A slice of tuples, each representing a range of valid port numbers.
///
/// # Returns
/// - `Option<u16>`: A randomly selected port number within the given ranges, or `None` if
///   the input is empty or the total number of available ports is zero.
fn select_random_port(port_ranges: &[(u16, u16)]) -> Option<u16> {
    use rand::Rng;
    let get_port_amount = |range: &(u16, u16)| -> u64 { (1 + range.1 - range.0) as u64 };
    let port_amount: u64 = port_ranges.iter().map(get_port_amount).sum();

    if port_amount < 1 {
        return None;
    }

    let mut port_index = rand::thread_rng().gen_range(0..port_amount);

    for range in port_ranges.iter() {
        let ports_in_range = get_port_amount(range);
        if port_index < ports_in_range {
            return Some(port_index as u16 + range.0);
        }
        port_index -= ports_in_range;
    }
    None
}

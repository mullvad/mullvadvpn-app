//! This module implement functions for producing a [`MullvadEndpoint`] given a Wireguard or
//! OpenVPN relay chosen by the relay selector.
//!
//! [`MullvadEndpoint`] contains all the necessary information for establishing a connection
//! between the client and Mullvad VPN. It is the daemon's responsibility to establish this
//! connection.
//!
//! [`MullvadEndpoint`]: mullvad_types::endpoint::MullvadEndpoint

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

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("No OpenVPN endpoint could be derived")]
    NoOpenVPNEndpoint,
    #[error("No bridge endpoint could be derived")]
    NoBridgeEndpoint,
    #[error("The selected relay does not support IPv6")]
    NoIPv6(Box<Relay>),
    #[error("Invalid port argument: port {0} is not in any valid Wireguard port range")]
    PortNotInRange(u16),
    #[error("Port selection algorithm is broken")]
    PortSelectionAlgorithm,
}

/// Constructs a [`MullvadWireguardEndpoint`] with details for how to connect to a Wireguard relay.
///
/// # Returns
/// - A configured endpoint for Wireguard relay, encapsulating either a single-hop or multi-hop connection.
/// - Returns [`Option::None`] if the desired port is not in a valid port range (see
/// [`WireguardRelayQuery::port`]) or relay addresses cannot be resolved.
pub fn wireguard_endpoint(
    query: &WireguardRelayQuery,
    data: &WireguardEndpointData,
    relay: &WireguardConfig,
) -> Result<MullvadWireguardEndpoint, Error> {
    match relay {
        WireguardConfig::Singlehop { exit } => wireguard_singlehop_endpoint(query, data, exit),
        WireguardConfig::Multihop { exit, entry } => {
            wireguard_multihop_endpoint(query, data, exit, entry)
        }
    }
}

/// Configure a single-hop connection using the exit relay data.
fn wireguard_singlehop_endpoint(
    query: &WireguardRelayQuery,
    data: &WireguardEndpointData,
    exit: &Relay,
) -> Result<MullvadWireguardEndpoint, Error> {
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
    Ok(MullvadWireguardEndpoint {
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
fn wireguard_multihop_endpoint(
    query: &WireguardRelayQuery,
    data: &WireguardEndpointData,
    exit: &Relay,
    entry: &Relay,
) -> Result<MullvadWireguardEndpoint, Error> {
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
        // The exit peer should be able to route incoming VPN traffic to the rest of
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

    Ok(MullvadWireguardEndpoint {
        peer: entry,
        exit_peer: Some(exit),
        ipv4_gateway: data.ipv4_gateway,
        ipv6_gateway: data.ipv6_gateway,
    })
}

/// Get the correct IP address for the given relay.
fn get_address_for_wireguard_relay(
    query: &WireguardRelayQuery,
    relay: &Relay,
) -> Result<IpAddr, Error> {
    match query.ip_version {
        Constraint::Any | Constraint::Only(IpVersion::V4) => Ok(relay.ipv4_addr_in.into()),
        Constraint::Only(IpVersion::V6) => relay
            .ipv6_addr_in
            .map(|addr| addr.into())
            .ok_or(Error::NoIPv6(Box::new(relay.clone()))),
    }
}

/// Try to pick a valid Wireguard port.
fn get_port_for_wireguard_relay(
    query: &WireguardRelayQuery,
    data: &WireguardEndpointData,
) -> Result<u16, Error> {
    match query.port {
        Constraint::Any => select_random_port(&data.port_ranges),
        Constraint::Only(port) => {
            if data
                .port_ranges
                .iter()
                .any(|range| (range.0 <= port && port <= range.1))
            {
                Ok(port)
            } else {
                Err(Error::PortNotInRange(port))
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
fn select_random_port(port_ranges: &[(u16, u16)]) -> Result<u16, Error> {
    use rand::Rng;
    let get_port_amount = |range: &(u16, u16)| -> u64 { (1 + range.1 - range.0) as u64 };
    let port_amount: u64 = port_ranges.iter().map(get_port_amount).sum();

    if port_amount < 1 {
        return Err(Error::PortSelectionAlgorithm);
    }

    let mut port_index = rand::thread_rng().gen_range(0..port_amount);

    for range in port_ranges.iter() {
        let ports_in_range = get_port_amount(range);
        if port_index < ports_in_range {
            return Ok(port_index as u16 + range.0);
        }
        port_index -= ports_in_range;
    }
    Err(Error::PortSelectionAlgorithm)
}

/// Constructs an [`Endpoint`] with details for how to connect to an OpenVPN relay.
///
/// If this endpoint is to be used in conjunction with a bridge, the resulting endpoint is
/// guaranteed to use transport protocol `TCP`.
///
/// This function can fail if no valid port + transport protocol combination is found.
/// See [`OpenVpnEndpointData`] for more details.
pub fn openvpn_endpoint(
    query: &OpenVpnRelayQuery,
    data: &OpenVpnEndpointData,
    relay: &Relay,
) -> Result<Endpoint, Error> {
    // If `bridge_mode` is true, this function may only return endpoints which use TCP, not UDP.
    if BridgeQuery::should_use_bridge(&query.bridge_settings) {
        openvpn_bridge_endpoint(&query.port, data, relay)
    } else {
        openvpn_singlehop_endpoint(&query.port, data, relay)
    }
}

/// Configure a single-hop connection using the exit relay data.
fn openvpn_singlehop_endpoint(
    port_constraint: &Constraint<TransportPort>,
    data: &OpenVpnEndpointData,
    exit: &Relay,
) -> Result<Endpoint, Error> {
    use rand::seq::IteratorRandom;
    data.ports
        .iter()
        .filter(|&endpoint| compatible_openvpn_port_combo(port_constraint, endpoint))
        .choose(&mut rand::thread_rng())
        .map(|endpoint| Endpoint::new(exit.ipv4_addr_in, endpoint.port, endpoint.protocol))
        .ok_or(Error::NoOpenVPNEndpoint)
}

/// Configure an endpoint that will be used together with a bridge.
///
/// # Note
/// In bridge mode, the only viable transport protocol is TCP. Otherwise, this function is
/// identical to [`Self::to_singlehop_endpoint`].
fn openvpn_bridge_endpoint(
    port_constraint: &Constraint<TransportPort>,
    data: &OpenVpnEndpointData,
    exit: &Relay,
) -> Result<Endpoint, Error> {
    use rand::seq::IteratorRandom;
    data.ports
        .iter()
        .filter(|endpoint| matches!(endpoint.protocol, TransportProtocol::Tcp))
        .filter(|endpoint| compatible_openvpn_port_combo(port_constraint, endpoint))
        .choose(&mut rand::thread_rng())
        .map(|endpoint| Endpoint::new(exit.ipv4_addr_in, endpoint.port, endpoint.protocol))
        .ok_or(Error::NoBridgeEndpoint)
}

/// Returns true if `port_constraint` can be used to connect to `endpoint`.
/// Otherwise, false is returned.
fn compatible_openvpn_port_combo(
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

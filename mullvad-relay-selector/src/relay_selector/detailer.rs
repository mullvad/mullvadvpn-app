//! This module implements functions for producing a [`MullvadEndpoint`] given a Wireguard or
//! OpenVPN relay chosen by the relay selector.
//!
//! [`MullvadEndpoint`] contains all the necessary information for establishing a connection
//! between the client and Mullvad VPN. It is the daemon's responsibility to establish this
//! connection.
//!
//! [`MullvadEndpoint`]: mullvad_types::endpoint::MullvadEndpoint

use std::net::{IpAddr, SocketAddr};

use super::{
    query::{BridgeQuery, OpenVpnRelayQuery, WireguardRelayQuery},
    WireguardConfig,
};
use crate::RuntimeParameters;
use ipnetwork::IpNetwork;
use mullvad_types::{
    constraints::Constraint,
    endpoint::MullvadWireguardEndpoint,
    relay_constraints::TransportPort,
    relay_list::{
        BridgeEndpointData, OpenVpnEndpoint, OpenVpnEndpointData, Relay, RelayEndpointData,
        WireguardEndpointData,
    },
};
use talpid_types::net::{
    all_of_the_internet,
    proxy::Shadowsocks,
    wireguard::{PeerConfig, PublicKey},
    Endpoint, IpVersion, TransportProtocol,
};

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("No OpenVPN endpoint could be derived")]
    NoOpenVpnEndpoint,
    #[error("No bridge endpoint could be derived")]
    NoBridgeEndpoint,
    #[error("OpenVPN relays and bridges does not have a public key. Expected a Wireguard relay")]
    MissingPublicKey,
    #[error("The selected relay does not support IPv6")]
    NoIPv6(Box<Relay>),
    #[error("Failed to select port")]
    PortSelectionError,
}

/// Constructs a [`MullvadWireguardEndpoint`] with details for how to connect to a Wireguard relay.
///
/// # Returns
/// - A configured endpoint for Wireguard relay, encapsulating either a single-hop or multi-hop
///   connection.
/// - Returns [`Option::None`] if the desired port is not in a valid port range (see
///   [`WireguardRelayQuery::port`]) or relay addresses cannot be resolved.
pub fn wireguard_endpoint(
    query: &WireguardRelayQuery,
    data: &WireguardEndpointData,
    relay: &WireguardConfig,
    runtime_parameters: RuntimeParameters,
) -> Result<MullvadWireguardEndpoint, Error> {
    match relay {
        WireguardConfig::Singlehop { exit } => {
            wireguard_singlehop_endpoint(query, data, exit, runtime_parameters)
        }
        WireguardConfig::Multihop { exit, entry } => {
            wireguard_multihop_endpoint(query, data, exit, entry, runtime_parameters)
        }
    }
}

/// Configure a single-hop connection using the exit relay data.
fn wireguard_singlehop_endpoint(
    query: &WireguardRelayQuery,
    data: &WireguardEndpointData,
    exit: &Relay,
    runtime_parameters: RuntimeParameters,
) -> Result<MullvadWireguardEndpoint, Error> {
    let endpoint = {
        let host = get_address_for_wireguard_relay(query, exit, runtime_parameters)?;
        let port = get_port_for_wireguard_relay(query, data)?;
        SocketAddr::new(host, port)
    };
    let peer_config = PeerConfig {
        public_key: get_public_key(exit)?.clone(),
        endpoint,
        allowed_ips: all_of_the_internet(),
        // This will be filled in later, not the relay selector's problem
        psk: None,
        // This will be filled in later
        #[cfg(daita)]
        constant_packet_size: false,
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
    runtime_parameters: RuntimeParameters,
) -> Result<MullvadWireguardEndpoint, Error> {
    /// The standard port on which an exit relay accepts connections from an entry relay in a
    /// multihop circuit.
    const WIREGUARD_EXIT_PORT: u16 = 51820;
    let exit_endpoint = {
        let ip = exit.ipv4_addr_in;
        // The port that the exit relay listens for incoming connections from entry
        // relays is *not* derived from the original query / user settings.
        let port = WIREGUARD_EXIT_PORT;
        SocketAddr::from((ip, port))
    };
    let exit = PeerConfig {
        public_key: get_public_key(exit)?.clone(),
        endpoint: exit_endpoint,
        // The exit peer should be able to route incoming VPN traffic to the rest of
        // the internet.
        allowed_ips: all_of_the_internet(),
        // This will be filled in later, not the relay selector's problem
        psk: None,
        // This will be filled in later
        #[cfg(daita)]
        constant_packet_size: false,
    };

    let entry_endpoint = {
        let host = get_address_for_wireguard_relay(query, entry, runtime_parameters)?;
        let port = get_port_for_wireguard_relay(query, data)?;
        SocketAddr::from((host, port))
    };
    let entry = PeerConfig {
        public_key: get_public_key(entry)?.clone(),
        endpoint: entry_endpoint,
        // The entry peer should only be able to route incoming VPN traffic to the
        // exit peer.
        allowed_ips: vec![IpNetwork::from(exit.endpoint.ip())],
        // This will be filled in later
        psk: None,
        // This will be filled in later
        #[cfg(daita)]
        constant_packet_size: false,
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
    runtime_parameters: RuntimeParameters,
) -> Result<IpAddr, Error> {
    match resolve_ip_version(query.ip_version, runtime_parameters.ipv4) {
        IpVersion::V4 => Ok(relay.ipv4_addr_in.into()),
        IpVersion::V6 => relay
            .ipv6_addr_in
            .map(|addr| addr.into())
            .ok_or(Error::NoIPv6(Box::new(relay.clone()))),
    }
}

pub fn resolve_ip_version(ip_version: Constraint<IpVersion>, ip_v4_available: bool) -> IpVersion {
    match ip_version {
        Constraint::Any => {
            if ip_v4_available {
                IpVersion::V4
            } else {
                IpVersion::V6
            }
        }
        Constraint::Only(IpVersion::V4) => IpVersion::V4,
        Constraint::Only(IpVersion::V6) => IpVersion::V6,
    }
}

/// Try to pick a valid Wireguard port.
fn get_port_for_wireguard_relay(
    query: &WireguardRelayQuery,
    data: &WireguardEndpointData,
) -> Result<u16, Error> {
    super::helpers::desired_or_random_port_from_range(&data.port_ranges, query.port)
        .map_err(|_err| Error::PortSelectionError)
}

/// Read the [`PublicKey`] of a relay. This will only succeed if [relay][`Relay`] is a
/// [Wireguard][`RelayEndpointData::Wireguard`] relay.
const fn get_public_key(relay: &Relay) -> Result<&PublicKey, Error> {
    match &relay.endpoint_data {
        RelayEndpointData::Wireguard(endpoint) => Ok(&endpoint.public_key),
        RelayEndpointData::Openvpn | RelayEndpointData::Bridge => Err(Error::MissingPublicKey),
    }
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
        .ok_or(Error::NoOpenVpnEndpoint)
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

/// Picks a random bridge from a relay.
pub fn bridge_endpoint(data: &BridgeEndpointData, relay: &Relay) -> Option<Shadowsocks> {
    use rand::seq::SliceRandom;
    if relay.endpoint_data != RelayEndpointData::Bridge {
        return None;
    }
    data.shadowsocks
        .choose(&mut rand::thread_rng())
        .inspect(|shadowsocks_endpoint| {
            log::info!(
                "Selected Shadowsocks bridge {} at {}:{}/{}",
                relay.hostname,
                relay.ipv4_addr_in,
                shadowsocks_endpoint.port,
                shadowsocks_endpoint.protocol
            );
        })
        .map(|shadowsocks_endpoint| {
            shadowsocks_endpoint.to_proxy_settings(relay.ipv4_addr_in.into())
        })
}

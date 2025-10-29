//! This module implements functions for producing a [`MullvadEndpoint`] given a Wireguard
//! relay chosen by the relay selector.
//!
//! [`MullvadEndpoint`] contains all the necessary information for establishing a connection
//! between the client and Mullvad VPN. It is the daemon's responsibility to establish this
//! connection.
//!
//! [`MullvadEndpoint`]: mullvad_types::endpoint::MullvadEndpoint

use std::net::{IpAddr, SocketAddr};

use ipnetwork::IpNetwork;
use mullvad_types::{
    constraints::Constraint,
    endpoint::MullvadEndpoint,
    relay_constraints::allowed_ip::resolve_from_constraint,
    relay_list::{BridgeEndpointData, EndpointData, Relay, RelayEndpointData},
};
use rand::seq::IndexedRandom;
use talpid_types::net::{
    IpVersion,
    proxy::Shadowsocks,
    wireguard::{PeerConfig, PublicKey},
};

use crate::query::ObfuscationQuery;

use super::{WireguardConfig, query::WireguardRelayQuery};

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("No bridge endpoint could be derived")]
    NoBridgeEndpoint,
    #[error("Bridges do not have a public key. Expected a Wireguard relay")]
    MissingPublicKey,
    #[error("The selected relay does not support IPv6")]
    NoIPv6(Box<Relay>),
    #[error("Failed to select port ({port})")]
    PortSelectionError { port: Constraint<u16> },
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
    data: &EndpointData,
    relay: &WireguardConfig,
) -> Result<MullvadEndpoint, Error> {
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
    data: &EndpointData,
    exit: &Relay,
) -> Result<MullvadEndpoint, Error> {
    let endpoint = {
        let host = get_address_for_wireguard_relay(query, exit)?;
        let port = get_port_for_wireguard_relay(query, data)?;
        SocketAddr::new(host, port)
    };
    let peer_config = PeerConfig {
        public_key: get_public_key(exit)?.clone(),
        endpoint,
        // The peer should be able to route incoming VPN traffic to the given user given IP
        // ranges, if any, else the rest of the internet.
        allowed_ips: resolve_from_constraint(
            &query.allowed_ips,
            Some(data.ipv4_gateway),
            Some(data.ipv6_gateway),
        ),
        // This will be filled in later, not the relay selector's problem
        psk: None,
        // This will be filled in later
        #[cfg(daita)]
        constant_packet_size: false,
    };
    Ok(MullvadEndpoint {
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
    data: &EndpointData,
    exit: &Relay,
    entry: &Relay,
) -> Result<MullvadEndpoint, Error> {
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
        // The exit peer should be able to route incoming VPN traffic to the given user given IP
        // ranges, if any, else the rest of the internet.
        allowed_ips: resolve_from_constraint(
            &query.allowed_ips,
            Some(data.ipv4_gateway),
            Some(data.ipv6_gateway),
        ),
        // This will be filled in later, not the relay selector's problem
        psk: None,
        // This will be filled in later
        #[cfg(daita)]
        constant_packet_size: false,
    };

    let entry_endpoint = {
        let host = get_address_for_wireguard_relay(query, entry)?;
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

    Ok(MullvadEndpoint {
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
    match resolve_ip_version(query.ip_version) {
        IpVersion::V4 => Ok(relay.ipv4_addr_in.into()),
        IpVersion::V6 => relay
            .ipv6_addr_in
            .map(|addr| addr.into())
            .ok_or(Error::NoIPv6(Box::new(relay.clone()))),
    }
}

pub fn resolve_ip_version(ip_version: Constraint<IpVersion>) -> IpVersion {
    match ip_version {
        Constraint::Any | Constraint::Only(IpVersion::V4) => IpVersion::V4,
        Constraint::Only(IpVersion::V6) => IpVersion::V6,
    }
}

/// Try to pick a valid Wireguard port.
fn get_port_for_wireguard_relay(
    query: &WireguardRelayQuery,
    data: &EndpointData,
) -> Result<u16, Error> {
    let port = if let ObfuscationQuery::Port(port) = query.obfuscation {
        Constraint::Only(port)
    } else {
        Constraint::Any
    };

    super::helpers::desired_or_random_port_from_range(&data.port_ranges, port)
        .map_err(|_err| Error::PortSelectionError { port })
}

/// Read the [`PublicKey`] of a relay. This will only succeed if [relay][`Relay`] is a
/// [Wireguard][`RelayEndpointData::Wireguard`] relay.
const fn get_public_key(relay: &Relay) -> Result<&PublicKey, Error> {
    match &relay.endpoint_data {
        RelayEndpointData::Wireguard(endpoint) => Ok(&endpoint.public_key),
        RelayEndpointData::Bridge => Err(Error::MissingPublicKey),
    }
}

/// Picks a random bridge from a relay.
pub fn bridge_endpoint(data: &BridgeEndpointData, relay: &Relay) -> Option<Shadowsocks> {
    if relay.endpoint_data != RelayEndpointData::Bridge {
        return None;
    }
    data.shadowsocks
        .choose(&mut rand::rng())
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

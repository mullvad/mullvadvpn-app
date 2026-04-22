//! This module implements functions for producing a [`MullvadEndpoint`] given a Wireguard
//! relay chosen by the relay selector.
//!
//! [`MullvadEndpoint`] contains all the necessary information for establishing a connection
//! between the client and Mullvad VPN. It is the daemon's responsibility to establish this
//! connection.
//!
//! [`MullvadEndpoint`]: mullvad_types::endpoint::MullvadEndpoint

use std::net::SocketAddr;

use ipnetwork::IpNetwork;
use mullvad_types::{
    constraints::Constraint,
    endpoint::MullvadEndpoint,
    relay_constraints::{AllowedIps, allowed_ip::resolve_from_constraint},
    relay_list::{Bridge, BridgeEndpointData, EndpointData, WireguardRelay},
};
use rand::seq::IndexedRandom;
use talpid_types::net::{IpVersion, proxy::Shadowsocks, wireguard::PeerConfig};

use super::WireguardConfig;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("No bridge endpoint could be derived")]
    NoBridgeEndpoint,
    #[error("Bridges do not have a public key. Expected a Wireguard relay")]
    MissingPublicKey,
    #[error("The selected relay does not support IPv6")]
    NoIPv6(Box<WireguardRelay>),
    #[error("Failed to select port ({port})")]
    PortSelectionError { port: Constraint<u16> },
}

/// Constructs a [`MullvadEndpoint`] with details for how to connect to a Wireguard relay.
///
/// # Returns
/// - A configured endpoint for Wireguard relay, encapsulating either a single-hop or multi-hop
///   connection.
pub fn wireguard_endpoint(
    allowed_ips: &Constraint<AllowedIps>,
    data: &EndpointData,
    relay: &WireguardConfig,
    entry_endpoint: SocketAddr,
) -> MullvadEndpoint {
    match relay {
        WireguardConfig::Singlehop { exit } => {
            wireguard_singlehop_endpoint(allowed_ips, data, exit, entry_endpoint)
        }
        WireguardConfig::Multihop { exit, entry } => {
            wireguard_multihop_endpoint(allowed_ips, data, exit, entry, entry_endpoint)
        }
    }
}

/// Configure a single-hop connection using the exit relay data.
fn wireguard_singlehop_endpoint(
    allowed_ips: &Constraint<AllowedIps>,
    data: &EndpointData,
    exit: &WireguardRelay,
    endpoint: SocketAddr,
) -> MullvadEndpoint {
    let peer_config = PeerConfig {
        public_key: exit.get_public_key().clone(),
        endpoint,
        // The peer should be able to route incoming VPN traffic to the given user given IP
        // ranges, if any, else the rest of the internet.
        allowed_ips: resolve_from_constraint(
            allowed_ips,
            Some(data.ipv4_gateway),
            Some(data.ipv6_gateway),
        ),
        // This will be filled in later, not the relay selector's problem
        psk: None,
        // This will be filled in later
        #[cfg(daita)]
        constant_packet_size: false,
    };
    MullvadEndpoint {
        peer: peer_config,
        exit_peer: None,
        ipv4_gateway: data.ipv4_gateway,
        ipv6_gateway: data.ipv6_gateway,
    }
}

/// Configure a multihop connection using the entry & exit relay data.
///
/// # Note
/// In a multihop circuit, we need to provide an exit peer configuration in addition to the
/// peer configuration.
fn wireguard_multihop_endpoint(
    allowed_ips: &Constraint<AllowedIps>,
    data: &EndpointData,
    exit: &WireguardRelay,
    entry: &WireguardRelay,
    entry_endpoint: SocketAddr,
) -> MullvadEndpoint {
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
        public_key: exit.get_public_key().clone(),
        endpoint: exit_endpoint,
        // The exit peer should be able to route incoming VPN traffic to the given user given IP
        // ranges, if any, else the rest of the internet.
        allowed_ips: resolve_from_constraint(
            allowed_ips,
            Some(data.ipv4_gateway),
            Some(data.ipv6_gateway),
        ),
        // This will be filled in later, not the relay selector's problem
        psk: None,
        // This will be filled in later
        #[cfg(daita)]
        constant_packet_size: false,
    };

    let entry = PeerConfig {
        public_key: entry.get_public_key().clone(),
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

    MullvadEndpoint {
        peer: entry,
        exit_peer: Some(exit),
        ipv4_gateway: data.ipv4_gateway,
        ipv6_gateway: data.ipv6_gateway,
    }
}

pub fn resolve_ip_version(ip_version: Constraint<&IpVersion>) -> IpVersion {
    match ip_version {
        Constraint::Any | Constraint::Only(IpVersion::V4) => IpVersion::V4,
        Constraint::Only(IpVersion::V6) => IpVersion::V6,
    }
}

/// Picks a random bridge from a relay.
pub fn bridge_endpoint(data: &BridgeEndpointData, relay: &Bridge) -> Option<Shadowsocks> {
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

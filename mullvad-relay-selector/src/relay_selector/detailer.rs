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
    endpoint::{MullvadEndpoint, MullvadWireguardEndpoint},
    relay_constraints::TransportPort,
    relay_list::{OpenVpnEndpoint, OpenVpnEndpointData, Relay, WireguardEndpointData},
};
use talpid_types::net::{all_of_the_internet, wireguard::PeerConfig, Endpoint, IpVersion};

use super::{
    query::{OpenVpnRelayQuery, WireguardRelayQuery},
    WireguardConfig,
};

/// Given a Wireguad relay (and optionally an entry relay if multihop is used) and the original
/// query the relay selector used, fill in all connection details to produce a valid
/// [`MullvadEndpoint`] by calling [`to_endpoint`].
///
/// [`to_endpoint`]: WireguardDetailer::to_endpoint
pub struct WireguardDetailer {
    wireguard_constraints: WireguardRelayQuery,
    config: WireguardConfig,
    data: WireguardEndpointData,
}

impl WireguardDetailer {
    /// The standard port on which an exit relay accepts connections from an entry relay in a
    /// multihop circuit.
    pub const WIREGUARD_EXIT_PORT: u16 = 51820;

    /// Create a new [`WireguardDetailer`].
    pub const fn new(
        query: WireguardRelayQuery,
        config: WireguardConfig,
        data: WireguardEndpointData,
    ) -> Self {
        Self {
            wireguard_constraints: query,
            config,
            data,
        }
    }

    /// Constructs a `MullvadEndpoint` with details for a Wireguard circuit.
    ///
    /// If entry is `None`, `to_endpoint` configures a single-hop connection using the exit relay data.
    /// Otherwise, it constructs a multihop setup using both entry and exit to set up appropriate peer
    /// configurations.
    ///
    /// # Returns
    /// - A configured Mullvad endpoint for Wireguard, encapsulating either a single-hop or multi-hop connection setup.
    /// - Returns `None` if the desired port is not in a valid port range (see
    /// [`WireguradRelayQuery::port`]) or relay addresses cannot be resolved.
    pub fn to_endpoint(&self) -> Option<MullvadEndpoint> {
        match &self.config {
            WireguardConfig::Singlehop { exit } => self.tmp_singlehop(exit),
            WireguardConfig::Multihop { exit, entry } => self.tmp_multihop(exit, entry),
        }
    }

    /// Configure a single-hop connection using the exit relay data.
    /// TODO(markus): Rename
    fn tmp_singlehop(&self, exit: &Relay) -> Option<MullvadEndpoint> {
        let endpoint = {
            let host = self.get_address_for_wireguard_relay(exit)?;
            let port = self.get_port_for_wireguard_relay(&self.data)?;
            SocketAddr::new(host, port)
        };
        let peer_config = PeerConfig {
            public_key: exit.endpoint_data.unwrap_wireguard_ref().public_key.clone(),
            endpoint,
            allowed_ips: all_of_the_internet(),
            // This will be filled in later, not the relay selector's problem
            psk: None,
        };
        Some(MullvadEndpoint::Wireguard(MullvadWireguardEndpoint {
            peer: peer_config,
            exit_peer: None,
            ipv4_gateway: self.data.ipv4_gateway,
            ipv6_gateway: self.data.ipv6_gateway,
        }))
    }

    /// Configure a multihop connection using the entry & exit relay data.
    ///
    /// # Note
    /// In a multihop circuit, we need to provide an exit peer configuration in addition to the
    /// peer configuration.
    /// TODO(markus): Rename
    fn tmp_multihop(&self, exit: &Relay, entry: &Relay) -> Option<MullvadEndpoint> {
        let exit_endpoint = {
            let ip = exit.ipv4_addr_in;
            // The port that the exit relay listens for incoming connections from entry
            // relays is *not* derived from the original query / user settings.
            let port = Self::WIREGUARD_EXIT_PORT;
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
            let host = self.get_address_for_wireguard_relay(entry)?;
            let port = self.get_port_for_wireguard_relay(&self.data)?;
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

        Some(MullvadEndpoint::Wireguard(MullvadWireguardEndpoint {
            peer: entry,
            exit_peer: Some(exit),
            ipv4_gateway: self.data.ipv4_gateway,
            ipv6_gateway: self.data.ipv6_gateway,
        }))
    }

    /// Get the correct IP address for the given relay.
    fn get_address_for_wireguard_relay(&self, relay: &Relay) -> Option<IpAddr> {
        match self.wireguard_constraints.ip_version {
            Constraint::Any | Constraint::Only(IpVersion::V4) => Some(relay.ipv4_addr_in.into()),
            Constraint::Only(IpVersion::V6) => relay.ipv6_addr_in.map(|addr| addr.into()),
        }
    }

    // Try to pick a valid Wireguard port.
    fn get_port_for_wireguard_relay(&self, data: &WireguardEndpointData) -> Option<u16> {
        match self.wireguard_constraints.port {
            Constraint::Any => {
                let random_port = Self::select_random_port(&data.port_ranges);
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
}

/// Given an OpenVPN relay and the original query the relay selector used,
/// fill in all connection details to produce a valid [`MullvadEndpoint`]
/// by calling [`to_endpoint`].
///
/// [`to_endpoint`]: OpenVpnDetailer::to_endpoint
pub struct OpenVpnDetailer {
    openvpn_constraints: OpenVpnRelayQuery,
    exit: Relay,
    data: OpenVpnEndpointData,
}

impl OpenVpnDetailer {
    /// Create a new [`OpenVpnDetailer`].
    pub const fn new(query: OpenVpnRelayQuery, relay: Relay, data: OpenVpnEndpointData) -> Self {
        Self {
            openvpn_constraints: query,
            exit: relay,
            data,
        }
    }

    /// Map `self` to a [`MullvadEndpoint`].
    ///
    /// This function can fail if no valid port + transport protocol combination is found.
    /// See [`OpenVpnEndpointData`] for more details.
    pub fn to_endpoint(&self) -> Option<MullvadEndpoint> {
        self.get_random_transport_port().map(|endpoint| {
            MullvadEndpoint::OpenVpn(Endpoint::new(
                self.exit.ipv4_addr_in,
                endpoint.port,
                endpoint.protocol,
            ))
        })
    }

    /// Try to pick a valid OpenVPN port.
    fn get_random_transport_port(&self) -> Option<&OpenVpnEndpoint> {
        use rand::seq::IteratorRandom;
        let constraints_port = self.openvpn_constraints.port;
        self.data
            .ports
            .iter()
            .filter(|endpoint| Self::compatible_port_combo(constraints_port, endpoint))
            .choose(&mut rand::thread_rng())
    }

    /// Returns true if `port_constraint` can be used to connect to `endpoint`.
    /// Otherwise, false is returned.
    fn compatible_port_combo(
        port_constraint: Constraint<TransportPort>,
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

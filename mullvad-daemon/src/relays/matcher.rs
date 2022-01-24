use mullvad_types::{
    endpoint::{MullvadEndpoint, MullvadWireguardEndpoint},
    relay_constraints::{
        Constraint, LocationConstraint, Match, OpenVpnConstraints, Providers, RelayConstraints,
        TransportPort, WireguardConstraints,
    },
    relay_list::{Relay, RelayTunnels, WireguardEndpointData},
};
use rand::{seq::SliceRandom, Rng};
use std::net::{IpAddr, SocketAddr};
use talpid_types::net::{all_of_the_internet, wireguard, IpVersion, TransportProtocol, TunnelType};

#[derive(Clone)]
pub struct RelayMatcher<T: TunnelMatcher> {
    pub location: Constraint<LocationConstraint>,
    pub providers: Constraint<Providers>,
    pub tunnel: T,
}

impl From<RelayConstraints> for RelayMatcher<AnyTunnelMatcher> {
    fn from(constraints: RelayConstraints) -> Self {
        Self {
            location: constraints.location,
            providers: constraints.providers,
            tunnel: AnyTunnelMatcher {
                wireguard: constraints.wireguard_constraints.into(),
                openvpn: constraints.openvpn_constraints,
                tunnel_type: constraints.tunnel_protocol,
            },
        }
    }
}

impl RelayMatcher<AnyTunnelMatcher> {
    pub fn to_wireguard_matcher(self) -> RelayMatcher<WireguardMatcher> {
        RelayMatcher {
            tunnel: self.tunnel.wireguard,
            location: self.location,
            providers: self.providers,
        }
    }
}

impl RelayMatcher<WireguardMatcher> {
    pub fn set_peer(&mut self, peer: Relay) {
        self.tunnel.peer = Some(peer);
    }
}

impl<T: TunnelMatcher> RelayMatcher<T> {
    /// Filter a relay and its endpoints based on constraints.
    /// Only matching endpoints are included in the returned Relay.
    pub fn filter_matching_relay(&self, relay: &Relay) -> Option<Relay> {
        if !self.location.matches(relay) || !self.providers.matches(relay) {
            return None;
        }

        self.tunnel.filter_matching_endpoints(relay)
    }

    pub fn mullvad_endpoint(&self, relay: &Relay) -> Option<MullvadEndpoint> {
        self.tunnel.mullvad_endpoint(relay)
    }
}

/// TunnelMatcher allows to abstract over different tunnel-specific constraints,
/// as to not have false dependencies on OpenVpn specific constraints when
/// selecting only WireGuard tunnels.
pub trait TunnelMatcher: Clone {
    /// Filter a relay and its endpoints based on constraints.
    /// Only matching endpoints are included in the returned Relay.
    fn filter_matching_endpoints(&self, relay: &Relay) -> Option<Relay>;
    /// Constructs a MullvadEndpoint for a given Relay using extra data from the relay matcher
    /// itself.
    fn mullvad_endpoint(&self, relay: &Relay) -> Option<MullvadEndpoint>;
}

impl TunnelMatcher for OpenVpnMatcher {
    fn filter_matching_endpoints(&self, relay: &Relay) -> Option<Relay> {
        let tunnels = relay
            .tunnels
            .openvpn
            .iter()
            .filter(|endpoint| self.matches(endpoint))
            .cloned()
            .collect::<Vec<_>>();
        if tunnels.is_empty() {
            return None;
        }
        let mut relay = relay.clone();
        relay.tunnels = RelayTunnels {
            openvpn: tunnels,
            wireguard: vec![],
        };
        Some(relay)
    }

    fn mullvad_endpoint(&self, relay: &Relay) -> Option<MullvadEndpoint> {
        relay
            .tunnels
            .openvpn
            .choose(&mut rand::thread_rng())
            .cloned()
            .map(|endpoint| endpoint.into_mullvad_endpoint(relay.ipv4_addr_in.into()))
    }
}

pub type OpenVpnMatcher = OpenVpnConstraints;

#[derive(Clone)]
pub struct AnyTunnelMatcher {
    wireguard: WireguardMatcher,
    openvpn: OpenVpnMatcher,
    /// in the case that a user hasn't specified a tunnel protocol, the relay
    /// selector might still construct preferred constraints that do select a
    /// specific tunnel protocol, which is why the tunnel type may be specified
    /// in the `AnyTunnelMatcher`.
    tunnel_type: Constraint<TunnelType>,
}

impl TunnelMatcher for AnyTunnelMatcher {
    fn filter_matching_endpoints(&self, relay: &Relay) -> Option<Relay> {
        match self.tunnel_type {
            Constraint::Any => {
                let wireguard_relay = self.wireguard.filter_matching_endpoints(relay);
                let openvpn_relay = self.openvpn.filter_matching_endpoints(relay);

                match (wireguard_relay, openvpn_relay) {
                    (Some(mut matched_relay), Some(openvpn_relay)) => {
                        matched_relay.tunnels.openvpn = openvpn_relay.tunnels.openvpn;
                        Some(matched_relay)
                    }
                    (Some(relay), None) | (None, Some(relay)) => Some(relay),
                    _ => None,
                }
            }
            Constraint::Only(TunnelType::OpenVpn) => self.openvpn.filter_matching_endpoints(relay),
            Constraint::Only(TunnelType::Wireguard) => {
                self.wireguard.filter_matching_endpoints(relay)
            }
        }
    }

    fn mullvad_endpoint(&self, relay: &Relay) -> Option<MullvadEndpoint> {
        #[cfg(not(target_os = "android"))]
        match self.tunnel_type {
            Constraint::Any => vec![
                self.openvpn.mullvad_endpoint(relay),
                self.wireguard.mullvad_endpoint(relay),
            ]
            .into_iter()
            .filter_map(|relay| relay)
            .collect::<Vec<_>>()
            .choose(&mut rand::thread_rng())
            .cloned(),
            Constraint::Only(TunnelType::OpenVpn) => self.openvpn.mullvad_endpoint(relay),
            Constraint::Only(TunnelType::Wireguard) => self.wireguard.mullvad_endpoint(relay),
        }

        #[cfg(target_os = "android")]
        self.wireguard.mullvad_endpoint(relay)
    }
}

#[derive(Clone)]
pub struct WireguardMatcher {
    /// The peer is an already selected peer relay to be used with multihop.
    /// It's stored here so we can exclude it from further selections being made.
    pub peer: Option<Relay>,
    pub port: Constraint<TransportPort>,
    pub ip_version: Constraint<IpVersion>,
}

impl WireguardMatcher {
    fn wg_data_to_endpoint(
        &self,
        relay: &Relay,
        data: WireguardEndpointData,
    ) -> Option<MullvadEndpoint> {
        let host = self.get_address_for_wireguard_relay(relay)?;
        let port = self.get_port_for_wireguard_relay(&data)?;
        let peer_config = wireguard::PeerConfig {
            public_key: data.public_key,
            endpoint: SocketAddr::new(host, port),
            allowed_ips: all_of_the_internet(),
            protocol: self
                .port
                .map(|port| port.protocol)
                .unwrap_or(TransportProtocol::Udp),
        };
        Some(MullvadEndpoint::Wireguard(MullvadWireguardEndpoint {
            peer: peer_config,
            exit_peer: None,
            ipv4_gateway: data.ipv4_gateway,
            ipv6_gateway: data.ipv6_gateway,
        }))
    }

    fn get_address_for_wireguard_relay(&self, relay: &Relay) -> Option<IpAddr> {
        match self.ip_version {
            Constraint::Any | Constraint::Only(IpVersion::V4) => Some(relay.ipv4_addr_in.into()),
            Constraint::Only(IpVersion::V6) => relay.ipv6_addr_in.map(|addr| addr.into()),
        }
    }

    fn get_port_for_wireguard_relay(&self, data: &WireguardEndpointData) -> Option<u16> {
        match self
            .port
            .as_ref()
            .map(|port| port.port)
            .unwrap_or(Constraint::Any)
        {
            Constraint::Any => {
                let get_port_amount =
                    |range: &(u16, u16)| -> u64 { (1 + range.1 - range.0) as u64 };
                let port_amount: u64 = data.port_ranges.iter().map(get_port_amount).sum();

                if port_amount < 1 {
                    return None;
                }

                let mut port_index = rand::thread_rng().gen_range(0, port_amount);

                for range in data.port_ranges.iter() {
                    let ports_in_range = get_port_amount(range);
                    if port_index < ports_in_range {
                        return Some(port_index as u16 + range.0);
                    }
                    port_index -= ports_in_range;
                }
                log::error!("Port selection algorithm is broken!");
                None
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
}

impl From<WireguardConstraints> for WireguardMatcher {
    fn from(constraints: WireguardConstraints) -> Self {
        Self {
            peer: None,
            port: constraints.port,
            ip_version: constraints.ip_version,
        }
    }
}

impl Match<WireguardEndpointData> for WireguardMatcher {
    fn matches(&self, endpoint: &WireguardEndpointData) -> bool {
        match self
            .port
            .as_ref()
            .map(|port| port.port)
            .unwrap_or(Constraint::Any)
        {
            Constraint::Any => true,
            Constraint::Only(port) => endpoint
                .port_ranges
                .iter()
                .any(|range| (port >= range.0 && port <= range.1)),
        }
    }
}

impl TunnelMatcher for WireguardMatcher {
    fn filter_matching_endpoints(&self, relay: &Relay) -> Option<Relay> {
        if self
            .peer
            .as_ref()
            .map(|peer_relay| peer_relay.hostname == relay.hostname)
            .unwrap_or(false)
        {
            return None;
        }

        let tunnels = relay
            .tunnels
            .wireguard
            .iter()
            .filter(|endpoint| self.matches(*endpoint))
            .cloned()
            .collect::<Vec<_>>();
        if tunnels.is_empty() {
            return None;
        }
        let mut relay = relay.clone();
        relay.tunnels = RelayTunnels {
            wireguard: tunnels,
            openvpn: vec![],
        };
        Some(relay)
    }

    fn mullvad_endpoint(&self, relay: &Relay) -> Option<MullvadEndpoint> {
        relay
            .tunnels
            .wireguard
            .choose(&mut rand::thread_rng())
            .cloned()
            .and_then(|wg_tunnel| self.wg_data_to_endpoint(relay, wg_tunnel))
    }
}

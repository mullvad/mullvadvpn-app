use mullvad_types::{
    endpoint::{MullvadEndpoint, MullvadWireguardEndpoint},
    relay_constraints::{
        Constraint, LocationConstraint, Match, OpenVpnConstraints, Ownership, Providers,
        RelayConstraints, WireguardConstraints,
    },
    relay_list::{
        OpenVpnEndpoint, OpenVpnEndpointData, Relay, RelayEndpointData, WireguardEndpointData,
    },
};
use rand::{
    seq::{IteratorRandom, SliceRandom},
    Rng,
};
use std::net::{IpAddr, SocketAddr};
use talpid_types::net::{all_of_the_internet, wireguard, Endpoint, IpVersion, TunnelType};

#[derive(Clone)]
pub struct RelayMatcher<T: EndpointMatcher> {
    pub location: Constraint<LocationConstraint>,
    pub providers: Constraint<Providers>,
    pub ownership: Constraint<Ownership>,
    pub endpoint_matcher: T,
}

impl RelayMatcher<AnyTunnelMatcher> {
    pub fn new(
        constraints: RelayConstraints,
        openvpn_data: OpenVpnEndpointData,
        wireguard_data: WireguardEndpointData,
    ) -> Self {
        Self {
            location: constraints.location,
            providers: constraints.providers,
            ownership: constraints.ownership,
            endpoint_matcher: AnyTunnelMatcher {
                wireguard: WireguardMatcher::new(constraints.wireguard_constraints, wireguard_data),
                openvpn: OpenVpnMatcher::new(constraints.openvpn_constraints, openvpn_data),
                tunnel_type: constraints.tunnel_protocol,
            },
        }
    }

    pub fn into_wireguard_matcher(self) -> RelayMatcher<WireguardMatcher> {
        RelayMatcher {
            endpoint_matcher: self.endpoint_matcher.wireguard,
            location: self.location,
            providers: self.providers,
            ownership: self.ownership,
        }
    }
}

impl RelayMatcher<WireguardMatcher> {
    pub fn set_peer(&mut self, peer: Relay) {
        self.endpoint_matcher.peer = Some(peer);
    }
}

impl<T: EndpointMatcher> RelayMatcher<T> {
    /// Filter a list of relays and their endpoints based on constraints.
    /// Only relays with (and including) matching endpoints are returned.
    pub fn filter_matching_relay_list(&self, relays: &[Relay]) -> Vec<Relay> {
        let matches = relays
            .iter()
            .filter(|relay| self.pre_filter_matching_relay(relay));

        let ignore_include_in_country = !matches.clone().any(|relay| relay.include_in_country);

        matches
            .filter(|relay| self.post_filter_matching_relay(relay, ignore_include_in_country))
            .cloned()
            .collect()
    }

    /// Filter a relay based on constraints and endpoint type, 1st pass.
    fn pre_filter_matching_relay(&self, relay: &Relay) -> bool {
        relay.active
            && self.providers.matches(relay)
            && self.ownership.matches(relay)
            && self.location.matches_with_opts(relay, true)
            && self.endpoint_matcher.is_matching_relay(relay)
    }

    /// Filter a relay based on constraints and endpoint type, 2nd pass.
    fn post_filter_matching_relay(&self, relay: &Relay, ignore_include_in_country: bool) -> bool {
        self.location
            .matches_with_opts(relay, ignore_include_in_country)
    }

    pub fn mullvad_endpoint(&self, relay: &Relay) -> Option<MullvadEndpoint> {
        self.endpoint_matcher.mullvad_endpoint(relay)
    }
}

/// EndpointMatcher allows to abstract over different tunnel-specific or bridge constraints.
/// This enables one to not have false dependencies on OpenVpn specific constraints when
/// selecting only WireGuard tunnels.
pub trait EndpointMatcher: Clone {
    /// Returns whether the relay has matching endpoints.
    fn is_matching_relay(&self, relay: &Relay) -> bool;
    /// Constructs a MullvadEndpoint for a given Relay using extra data from the relay matcher
    /// itself.
    fn mullvad_endpoint(&self, relay: &Relay) -> Option<MullvadEndpoint>;
}

impl EndpointMatcher for OpenVpnMatcher {
    fn is_matching_relay(&self, relay: &Relay) -> bool {
        self.matches(&self.data) && matches!(relay.endpoint_data, RelayEndpointData::Openvpn)
    }

    fn mullvad_endpoint(&self, relay: &Relay) -> Option<MullvadEndpoint> {
        if !self.is_matching_relay(relay) {
            return None;
        }

        self.get_transport_port().map(|endpoint| {
            MullvadEndpoint::OpenVpn(Endpoint::new(
                relay.ipv4_addr_in,
                endpoint.port,
                endpoint.protocol,
            ))
        })
    }
}

#[derive(Debug, Clone)]
pub struct OpenVpnMatcher {
    pub constraints: OpenVpnConstraints,
    pub data: OpenVpnEndpointData,
}

impl OpenVpnMatcher {
    pub fn new(constraints: OpenVpnConstraints, data: OpenVpnEndpointData) -> Self {
        Self { constraints, data }
    }

    fn get_transport_port(&self) -> Option<&OpenVpnEndpoint> {
        match self.constraints.port {
            Constraint::Any => self.data.ports.choose(&mut rand::thread_rng()),
            Constraint::Only(transport_port) => self
                .data
                .ports
                .iter()
                .filter(|endpoint| {
                    transport_port
                        .port
                        .map(|port| port == endpoint.port)
                        .unwrap_or(true)
                        && transport_port.protocol == endpoint.protocol
                })
                .choose(&mut rand::thread_rng()),
        }
    }
}

impl Match<OpenVpnEndpointData> for OpenVpnMatcher {
    fn matches(&self, endpoint: &OpenVpnEndpointData) -> bool {
        match self.constraints.port {
            Constraint::Any => true,
            Constraint::Only(transport_port) => endpoint.ports.iter().any(|endpoint| {
                transport_port.protocol == endpoint.protocol
                    && (transport_port.port.is_any()
                        || transport_port.port == Constraint::Only(endpoint.port))
            }),
        }
    }
}

#[derive(Clone)]
pub struct AnyTunnelMatcher {
    pub wireguard: WireguardMatcher,
    pub openvpn: OpenVpnMatcher,
    /// in the case that a user hasn't specified a tunnel protocol, the relay
    /// selector might still construct preferred constraints that do select a
    /// specific tunnel protocol, which is why the tunnel type may be specified
    /// in the `AnyTunnelMatcher`.
    pub tunnel_type: Constraint<TunnelType>,
}

impl EndpointMatcher for AnyTunnelMatcher {
    fn is_matching_relay(&self, relay: &Relay) -> bool {
        match self.tunnel_type {
            Constraint::Any => {
                self.wireguard.is_matching_relay(relay) || self.openvpn.is_matching_relay(relay)
            }
            Constraint::Only(TunnelType::OpenVpn) => self.openvpn.is_matching_relay(relay),
            Constraint::Only(TunnelType::Wireguard) => self.wireguard.is_matching_relay(relay),
        }
    }

    fn mullvad_endpoint(&self, relay: &Relay) -> Option<MullvadEndpoint> {
        #[cfg(not(target_os = "android"))]
        match self.tunnel_type {
            Constraint::Any => self
                .openvpn
                .mullvad_endpoint(relay)
                .or_else(|| self.wireguard.mullvad_endpoint(relay)),
            Constraint::Only(TunnelType::OpenVpn) => self.openvpn.mullvad_endpoint(relay),
            Constraint::Only(TunnelType::Wireguard) => self.wireguard.mullvad_endpoint(relay),
        }

        #[cfg(target_os = "android")]
        self.wireguard.mullvad_endpoint(relay)
    }
}

#[derive(Default, Clone)]
pub struct WireguardMatcher {
    /// The peer is an already selected peer relay to be used with multihop.
    /// It's stored here so we can exclude it from further selections being made.
    pub peer: Option<Relay>,
    pub port: Constraint<u16>,
    pub ip_version: Constraint<IpVersion>,

    pub data: WireguardEndpointData,
}

impl WireguardMatcher {
    pub fn new(constraints: WireguardConstraints, data: WireguardEndpointData) -> Self {
        Self {
            peer: None,
            port: constraints.port,
            ip_version: constraints.ip_version,
            data,
        }
    }

    pub fn from_endpoint(data: WireguardEndpointData) -> Self {
        Self {
            data,
            ..Default::default()
        }
    }

    fn wg_data_to_endpoint(
        &self,
        relay: &Relay,
        data: &WireguardEndpointData,
    ) -> Option<MullvadEndpoint> {
        let host = self.get_address_for_wireguard_relay(relay)?;
        let port = self.get_port_for_wireguard_relay(data)?;
        let peer_config = wireguard::PeerConfig {
            public_key: relay
                .endpoint_data
                .unwrap_wireguard_ref()
                .public_key
                .clone(),
            endpoint: SocketAddr::new(host, port),
            allowed_ips: all_of_the_internet(),
            psk: None,
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
        match self.port {
            Constraint::Any => {
                let get_port_amount =
                    |range: &(u16, u16)| -> u64 { (1 + range.1 - range.0) as u64 };
                let port_amount: u64 = data.port_ranges.iter().map(get_port_amount).sum();

                if port_amount < 1 {
                    return None;
                }

                let mut port_index = rand::thread_rng().gen_range(0..port_amount);

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

impl EndpointMatcher for WireguardMatcher {
    fn is_matching_relay(&self, relay: &Relay) -> bool {
        !self
            .peer
            .as_ref()
            .map(|peer_relay| peer_relay.hostname == relay.hostname)
            .unwrap_or(false)
            && matches!(relay.endpoint_data, RelayEndpointData::Wireguard(..))
    }

    fn mullvad_endpoint(&self, relay: &Relay) -> Option<MullvadEndpoint> {
        if !self.is_matching_relay(relay) {
            return None;
        }
        self.wg_data_to_endpoint(relay, &self.data)
    }
}

#[derive(Clone)]
pub struct BridgeMatcher(pub ());

impl EndpointMatcher for BridgeMatcher {
    fn is_matching_relay(&self, relay: &Relay) -> bool {
        matches!(relay.endpoint_data, RelayEndpointData::Bridge)
    }

    fn mullvad_endpoint(&self, _relay: &Relay) -> Option<MullvadEndpoint> {
        None
    }
}

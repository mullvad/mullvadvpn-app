//! This module is responsible for filtering the whole relay list based on queries.
use mullvad_types::{
    constraints::{Constraint, Match},
    custom_list::CustomListsSettings,
    relay_constraints::{
        BridgeState, InternalBridgeConstraints, Ownership, Providers, ResolvedLocationConstraint,
        TransportPort,
    },
    relay_list::{
        OpenVpnEndpoint, OpenVpnEndpointData, Relay, RelayEndpointData, WireguardEndpointData,
    },
};
use talpid_types::net::{IpVersion, TransportProtocol, TunnelType};

use super::query::{OpenVpnRelayQuery, RelayQuery, WireguardRelayQuery};

#[derive(Clone)]
pub struct RelayMatcher<T: EndpointMatcher> {
    /// Locations allowed to be picked from. In the case of custom lists this may be multiple
    /// locations. In normal circumstances this contains only 1 location.
    pub locations: Constraint<ResolvedLocationConstraint>,
    /// Relay providers allowed to be picked from.
    pub providers: Constraint<Providers>,
    /// Relay ownership allowed to be picked from.
    pub ownership: Constraint<Ownership>,
    /// Concrete representation of [`RelayConstraints`] or [`BridgeConstraints`].
    pub endpoint_matcher: T,
}

impl RelayMatcher<AnyTunnelMatcher> {
    pub fn new(
        constraints: RelayQuery,
        openvpn_data: OpenVpnEndpointData,
        brige_state: BridgeState,
        wireguard_data: WireguardEndpointData,
        custom_lists: &CustomListsSettings,
    ) -> Self {
        Self {
            locations: ResolvedLocationConstraint::from_constraint(
                constraints.location,
                custom_lists,
            ),
            providers: constraints.providers,
            ownership: constraints.ownership,
            endpoint_matcher: AnyTunnelMatcher {
                wireguard: WireguardMatcher::new(constraints.wireguard_constraints, wireguard_data),
                openvpn: OpenVpnMatcher::new(
                    constraints.openvpn_constraints,
                    openvpn_data,
                    brige_state,
                ),
                tunnel_type: constraints.tunnel_protocol,
            },
        }
    }
}

impl<T: EndpointMatcher> RelayMatcher<T> {
    /// Filter a list of relays and their endpoints based on constraints.
    /// Only relays with (and including) matching endpoints are returned.
    pub fn filter_matching_relay_list<'a, R: Iterator<Item = &'a Relay> + Clone>(
        &self,
        relays: R,
    ) -> Vec<Relay> {
        let shortlist = relays
            // Filter on active relays
            .filter(|relay| filter_on_active(relay))
            // Filter by location
            .filter(|relay| filter_on_location(&self.locations, relay))
            // Filter by ownership
            .filter(|relay| filter_on_ownership(&self.ownership, relay))
            // Filter by providers
            .filter(|relay| filter_on_providers(&self.providers, relay))
            // Filter on relay type & relay specific properties
            .filter(|relay| self.endpoint_matcher.is_matching_relay(relay));

        // The last filtering to be done is on the `include_in_country` attribute found on each
        // relay. A regular, user-facing relay will have `include_in_country` set to true.
        // If a relay has `include_in_country` set to false, they are purposely hidden than
        // other relays. We should only consider those if there are no regular candidates left.
        let ignore_include_in_country = !shortlist.clone().any(|relay| relay.include_in_country);
        shortlist
            .filter(|relay| {
                self.locations
                    .matches_with_opts(relay, ignore_include_in_country)
            })
            .cloned()
            .collect()
    }
}

/// EndpointMatcher allows to abstract over different tunnel-specific or bridge constraints.
/// This enables one to not have false dependencies on OpenVpn specific constraints when
/// selecting only WireGuard tunnels.
pub trait EndpointMatcher: Clone {
    /// Returns whether the relay has matching endpoints.
    fn is_matching_relay(&self, relay: &Relay) -> bool;
}

#[derive(Clone)]
pub struct AnyTunnelMatcher {
    /// The [`WireguardMatcher`] to be used in case we should filter Wireguard relays.
    pub wireguard: WireguardMatcher,
    /// The [`OpenVpnMatcher`] to be used in case we should filter OpenVPN relays.
    pub openvpn: OpenVpnMatcher,
    /// If the user hasn't specified a tunnel protocol the relay selector might
    /// still prefer a specific tunnel protocol, which is why the tunnel type
    /// may be specified in the `AnyTunnelMatcher`.
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
}

#[derive(Default, Clone)]
pub struct WireguardMatcher {
    pub port: Constraint<u16>,
    pub ip_version: Constraint<IpVersion>,
    pub data: WireguardEndpointData,
}

/// Filter suitable Wireguard relays from the relay list
impl WireguardMatcher {
    pub fn new(constraints: WireguardRelayQuery, data: WireguardEndpointData) -> Self {
        Self {
            port: constraints.port,
            ip_version: constraints.ip_version,
            data,
        }
    }

    pub fn new_matcher(
        constraints: RelayQuery,
        data: WireguardEndpointData,
        custom_lists: &CustomListsSettings,
    ) -> RelayMatcher<Self> {
        RelayMatcher {
            locations: ResolvedLocationConstraint::from_constraint(
                constraints.location,
                custom_lists,
            ),
            providers: constraints.providers,
            ownership: constraints.ownership,
            endpoint_matcher: WireguardMatcher::new(constraints.wireguard_constraints, data),
        }
    }
}

impl EndpointMatcher for WireguardMatcher {
    fn is_matching_relay(&self, relay: &Relay) -> bool {
        filter_wireguard(relay)
    }
}

/// Filter suitable OpenVPN relays from the relay list
#[derive(Debug, Clone)]
pub struct OpenVpnMatcher {
    pub constraints: OpenVpnRelayQuery,
    pub data: OpenVpnEndpointData,
}

impl OpenVpnMatcher {
    pub fn new(
        mut constraints: OpenVpnRelayQuery,
        data: OpenVpnEndpointData,
        bridge_state: BridgeState,
    ) -> Self {
        // TODO(markus): Seems like a hack
        if constraints.port.is_any() && bridge_state == BridgeState::On {
            constraints.port = Constraint::Only(TransportPort {
                protocol: TransportProtocol::Tcp,
                port: Constraint::Any,
            });
        }
        Self { constraints, data }
    }
}

impl EndpointMatcher for OpenVpnMatcher {
    fn is_matching_relay(&self, relay: &Relay) -> bool {
        filter_openvpn(relay) && openvpn_filter_on_port(self.constraints.port, &self.data)
    }
}

#[derive(Clone)]
pub struct BridgeMatcher;

impl BridgeMatcher {
    pub fn new_matcher(
        relay_constraints: InternalBridgeConstraints,
        custom_lists: &CustomListsSettings,
    ) -> RelayMatcher<Self> {
        RelayMatcher {
            locations: ResolvedLocationConstraint::from_constraint(
                relay_constraints.location,
                custom_lists,
            ),
            providers: relay_constraints.providers,
            ownership: relay_constraints.ownership,
            endpoint_matcher: BridgeMatcher,
        }
    }
}

impl EndpointMatcher for BridgeMatcher {
    fn is_matching_relay(&self, relay: &Relay) -> bool {
        filter_bridge(relay)
    }
}

// --- Define relay filters as simple functions / predicates ---
// The intent is to make it easier to re-use in iterator chains.

/// Returns whether `relay` is active.
pub const fn filter_on_active(relay: &Relay) -> bool {
    relay.active
}

/// Returns whether `relay` satisfy the location constraint posed by `filter`.
pub fn filter_on_location(filter: &Constraint<ResolvedLocationConstraint>, relay: &Relay) -> bool {
    let ignore_include_in_countries = true;
    filter.matches_with_opts(relay, ignore_include_in_countries)
}

/// Returns whether `relay` satisfy the ownership constraint posed by `filter`.
pub fn filter_on_ownership(filter: &Constraint<Ownership>, relay: &Relay) -> bool {
    filter.matches(relay)
}

/// Returns whether `relay` satisfy the providers constraint posed by `filter`.
pub fn filter_on_providers(filter: &Constraint<Providers>, relay: &Relay) -> bool {
    filter.matches(relay)
}

/// Returns whether the relay is an OpenVPN relay.
pub const fn filter_openvpn(relay: &Relay) -> bool {
    matches!(relay.endpoint_data, RelayEndpointData::Openvpn)
}

/// Returns whether the relay is a Wireguard relay.
pub const fn filter_wireguard(relay: &Relay) -> bool {
    matches!(relay.endpoint_data, RelayEndpointData::Wireguard(_))
}

/// Returns whether the relay is a bridge.
pub const fn filter_bridge(relay: &Relay) -> bool {
    matches!(relay.endpoint_data, RelayEndpointData::Bridge)
}

// --- OpenVPN specific filter ---

/// Returns wheter a relay (endpoint) satisfy the port constraints (transport protocol + port
/// number) posed by `filter`.
fn openvpn_filter_on_port(port: Constraint<TransportPort>, endpoint: &OpenVpnEndpointData) -> bool {
    let compatible_port =
        |transport_port: TransportPort, endpoint: &OpenVpnEndpoint| match transport_port.port {
            Constraint::Any => true,
            Constraint::Only(port) => port == endpoint.port,
        };

    match port {
        Constraint::Any => true,
        Constraint::Only(transport_port) => endpoint
            .ports
            .iter()
            .filter(|endpoint| endpoint.protocol == transport_port.protocol)
            .any(|port| compatible_port(transport_port, port)),
    }
}

//! This module is responsible for filtering the whole relay list based on queries.
use mullvad_types::{
    constraints::{Constraint, Match},
    custom_list::CustomListsSettings,
    relay_constraints::{
        BridgeState, GeographicLocationConstraint, InternalBridgeConstraints, LocationConstraint,
        Ownership, Providers, TransportPort,
    },
    relay_list::{OpenVpnEndpointData, Relay, RelayEndpointData, WireguardEndpointData},
};
use talpid_types::net::{IpVersion, TransportProtocol, TunnelType};

use super::query::{OpenVpnRelayQuery, RelayQuery, WireguardRelayQuery};

/// The relay matcher decomposes a [`RelayQuery`] and exposes functions for filtering a set of relays
/// to a subset which matches the original query.
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

impl<'a> RelayMatcher<AnyTunnelMatcher<'a>> {
    pub fn new(
        query: RelayQuery,
        openvpn_data: &'a OpenVpnEndpointData,
        bridge_state: BridgeState,
        wireguard_data: &'a WireguardEndpointData,
        custom_lists: &CustomListsSettings,
    ) -> RelayMatcher<AnyTunnelMatcher<'a>> {
        let endpoint_matcher = AnyTunnelMatcher {
            wireguard: WireguardMatcher::new(query.wireguard_constraints.clone(), wireguard_data),
            openvpn: OpenVpnMatcher::new(
                query.openvpn_constraints.clone(),
                openvpn_data,
                bridge_state,
            ),
            tunnel_type: query.tunnel_protocol,
        };
        Self::using(query, custom_lists, endpoint_matcher)
    }
}

impl<T: EndpointMatcher> RelayMatcher<T> {
    pub fn using(
        query: RelayQuery,
        custom_lists: &CustomListsSettings,
        endpoint_matcher: T,
    ) -> RelayMatcher<T> {
        RelayMatcher {
            locations: ResolvedLocationConstraint::from_constraint(query.location, custom_lists),
            providers: query.providers,
            ownership: query.ownership,
            endpoint_matcher,
        }
    }

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
        // relay. When the location constraint is based on country, a relay which has `include_in_country`
        // set to true should always be prioritized over relays which has this flag set to false.
        // We should only consider relays with `include_in_country` set to false if there are no
        // other candidates left.
        match &self.locations {
            Constraint::Any => shortlist.cloned().collect(),
            Constraint::Only(locations) => {
                let mut included = std::collections::HashSet::new();
                let mut excluded = std::collections::HashSet::new();
                for relay in shortlist {
                    for location in locations {
                        if location.is_country() && relay.include_in_country {
                            included.insert(relay.clone());
                        } else {
                            excluded.insert(relay.clone());
                        };
                    }
                }
                if included.is_empty() {
                    excluded.into_iter().collect()
                } else {
                    included.into_iter().collect()
                }
            }
        }
    }
}

/// EndpointMatcher allows to abstract over different tunnel-specific or bridge constraints.
/// This enables one to not have false dependencies on OpenVpn specific constraints when
/// selecting only WireGuard tunnels.
pub trait EndpointMatcher {
    /// Returns whether the relay has matching endpoints.
    fn is_matching_relay(&self, relay: &Relay) -> bool;
}

pub struct AnyTunnelMatcher<'a> {
    /// The [`WireguardMatcher`] to be used in case we should filter Wireguard relays.
    pub wireguard: WireguardMatcher<'a>,
    /// The [`OpenVpnMatcher`] to be used in case we should filter OpenVPN relays.
    pub openvpn: OpenVpnMatcher<'a>,
    /// If the user hasn't specified a tunnel protocol the relay selector might
    /// still prefer a specific tunnel protocol, which is why the tunnel type
    /// may be specified in the `AnyTunnelMatcher`.
    pub tunnel_type: Constraint<TunnelType>,
}

impl EndpointMatcher for AnyTunnelMatcher<'_> {
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

#[derive(Debug)]
pub struct WireguardMatcher<'a> {
    pub port: Constraint<u16>,
    pub ip_version: Constraint<IpVersion>,
    pub data: &'a WireguardEndpointData,
}

/// Filter suitable Wireguard relays from the relay list
impl<'a> WireguardMatcher<'a> {
    pub fn new(
        constraints: WireguardRelayQuery,
        data: &'a WireguardEndpointData,
    ) -> WireguardMatcher<'a> {
        Self {
            port: constraints.port,
            ip_version: constraints.ip_version,
            data,
        }
    }
}

impl<'a> EndpointMatcher for WireguardMatcher<'a> {
    fn is_matching_relay(&self, relay: &Relay) -> bool {
        filter_wireguard(relay)
    }
}

/// Filter suitable OpenVPN relays from the relay list
#[derive(Debug)]
pub struct OpenVpnMatcher<'a> {
    pub constraints: OpenVpnRelayQuery,
    pub data: &'a OpenVpnEndpointData,
}

impl<'a> OpenVpnMatcher<'a> {
    pub fn new(
        mut constraints: OpenVpnRelayQuery,
        data: &'a OpenVpnEndpointData,
        bridge_state: BridgeState,
    ) -> Self {
        // Using bridges demands the selected endpoint to use TCP.
        //
        // If the user has not set any specific port constraint, and bridge mode is explicitly
        // turned on, sneakily set the transport protocol of the resulting matcher to TCP.
        // This will correctly filter out matching relays later.
        if constraints.port.is_any() && bridge_state == BridgeState::On {
            constraints.port = Constraint::Only(TransportPort {
                protocol: TransportProtocol::Tcp,
                port: Constraint::Any,
            });
        }
        Self { constraints, data }
    }
}

impl EndpointMatcher for OpenVpnMatcher<'_> {
    fn is_matching_relay(&self, relay: &Relay) -> bool {
        filter_openvpn(relay) && openvpn_filter_on_port(self.constraints.port, self.data)
    }
}

#[derive(Debug)]
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
    filter.matches(relay)
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

/// Returns whether a relay (endpoint) satisfy the port constraints (transport protocol + port
/// number) posed by `filter`.
fn openvpn_filter_on_port(port: Constraint<TransportPort>, endpoint: &OpenVpnEndpointData) -> bool {
    match port {
        Constraint::Any => true,
        Constraint::Only(transport_port) => endpoint
            .ports
            .iter()
            .filter(|endpoint| endpoint.protocol == transport_port.protocol)
            .any(|endpoint| match transport_port.port {
                Constraint::Any => true,
                Constraint::Only(port) => port == endpoint.port,
            }),
    }
}

// -- Wrapper around LocationConstraint --

#[derive(Debug, Clone)]
pub struct ResolvedLocationConstraint(Vec<GeographicLocationConstraint>);

impl<'a> IntoIterator for &'a ResolvedLocationConstraint {
    type Item = &'a GeographicLocationConstraint;

    type IntoIter = core::slice::Iter<'a, GeographicLocationConstraint>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}

impl IntoIterator for ResolvedLocationConstraint {
    type Item = GeographicLocationConstraint;

    type IntoIter = std::vec::IntoIter<GeographicLocationConstraint>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl FromIterator<GeographicLocationConstraint> for ResolvedLocationConstraint {
    fn from_iter<T: IntoIterator<Item = GeographicLocationConstraint>>(iter: T) -> Self {
        Self(Vec::from_iter(iter))
    }
}

impl ResolvedLocationConstraint {
    pub fn from_constraint(
        location_constraint: Constraint<LocationConstraint>,
        custom_lists: &CustomListsSettings,
    ) -> Constraint<ResolvedLocationConstraint> {
        location_constraint.map(|location| Self::from_location_constraint(location, custom_lists))
    }

    fn from_location_constraint(
        location: LocationConstraint,
        custom_lists: &CustomListsSettings,
    ) -> ResolvedLocationConstraint {
        match location {
            LocationConstraint::Location(location) => Self::from_iter(std::iter::once(location)),
            LocationConstraint::CustomList { list_id } => custom_lists
                .iter()
                .find(|list| list.id == list_id)
                .map(|custom_list| Self::from_iter(custom_list.locations.clone()))
                .unwrap_or_else(|| {
                    log::warn!("Resolved non-existent custom list");
                    Self::from_iter(std::iter::empty())
                }),
        }
    }
}

impl Match<Relay> for ResolvedLocationConstraint {
    fn matches(&self, relay: &Relay) -> bool {
        self.into_iter().any(|location| location.matches(relay))
    }
}

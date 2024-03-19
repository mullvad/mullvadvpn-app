//! This module is responsible for filtering the whole relay list based on queries.
use std::collections::HashSet;

use mullvad_types::{
    constraints::{Constraint, Match},
    custom_list::CustomListsSettings,
    relay_constraints::{
        GeographicLocationConstraint, InternalBridgeConstraints, LocationConstraint, Ownership,
        Providers,
    },
    relay_list::{Relay, RelayEndpointData},
};
use talpid_types::net::TunnelType;

use super::query::RelayQuery;

/// Filter a list of relays and their endpoints based on constraints.
/// Only relays with (and including) matching endpoints are returned.
pub fn new_filter_matching_relay_list<'a, R: Iterator<Item = &'a Relay> + Clone>(
    query: &RelayQuery,
    relays: R,
    custom_lists: &CustomListsSettings,
) -> Vec<Relay> {
    // TODO: `ResolvedLocationConstraint` does not need to take any ownership of anything (?)
    let locations =
        ResolvedLocationConstraint::from_constraint(query.location.clone(), custom_lists);
    let shortlist = relays
            // Filter on tunnel type
            .filter(|relay| filter_tunnel_type(&query.tunnel_protocol, relay))
            // Filter on active relays
            .filter(|relay| filter_on_active(relay))
            // Filter by location
            .filter(|relay| filter_on_location(&locations, relay))
            // Filter by ownership
            .filter(|relay| filter_on_ownership(&query.ownership, relay))
            // Filter by providers
            .filter(|relay| filter_on_providers(&query.providers, relay));

    // The last filtering to be done is on the `include_in_country` attribute found on each
    // relay. When the location constraint is based on country, a relay which has `include_in_country`
    // set to true should always be prioritized over relays which has this flag set to false.
    // We should only consider relays with `include_in_country` set to false if there are no
    // other candidates left.
    match &locations {
        Constraint::Any => shortlist.cloned().collect(),
        Constraint::Only(locations) => {
            let mut included = HashSet::new();
            let mut excluded = HashSet::new();
            for location in locations {
                let (included_in_country, not_included_in_country): (Vec<_>, Vec<_>) = shortlist
                    .clone()
                    .partition(|relay| location.is_country() && relay.include_in_country);
                included.extend(included_in_country);
                excluded.extend(not_included_in_country);
            }
            if included.is_empty() {
                excluded.into_iter().cloned().collect()
            } else {
                included.into_iter().cloned().collect()
            }
        }
    }
}

pub fn filter_matching_bridges<'a, R: Iterator<Item = &'a Relay> + Clone>(
    constraints: &InternalBridgeConstraints,
    relays: R,
    custom_lists: &CustomListsSettings,
) -> Vec<Relay> {
    // TODO: Remove clone
    let locations =
        ResolvedLocationConstraint::from_constraint(constraints.location.clone(), custom_lists);
    relays
            // Filter on active relays
            .filter(|relay| filter_on_active(relay))
            // Filter on bridge type
            .filter(|relay| filter_bridge(relay))
            // Filter by location
            .filter(|relay| filter_on_location(&locations, relay))
            // Filter by ownership
            .filter(|relay| filter_on_ownership(&constraints.ownership, relay))
            // Filter by constraints
            .filter(|relay| filter_on_providers(&constraints.providers, relay))
            .cloned()
            .collect()
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
pub const fn filter_tunnel_type(filter: &Constraint<TunnelType>, relay: &Relay) -> bool {
    match filter {
        Constraint::Any => true,
        Constraint::Only(typ) => match typ {
            TunnelType::OpenVpn => filter_openvpn(relay),
            TunnelType::Wireguard => filter_wireguard(relay),
        },
    }
}

/// Returns whether the relay is a Wireguard relay.
pub const fn filter_wireguard(relay: &Relay) -> bool {
    matches!(relay.endpoint_data, RelayEndpointData::Wireguard(_))
}

/// Returns whether the relay is a bridge.
pub const fn filter_bridge(relay: &Relay) -> bool {
    matches!(relay.endpoint_data, RelayEndpointData::Bridge)
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

//! Relay filter predicates used by the verdict-based partition functions.

use mullvad_types::{
    constraints::{Constraint, Match},
    custom_list::CustomListsSettings,
    relay_constraints::{GeographicLocationConstraint, LocationConstraint, Ownership, Providers},
    relay_list::{WireguardRelay, WireguardRelayEndpointData},
};

/// Returns whether `relay` satisfy the location constraint posed by `filter`.
pub fn filter_on_location(
    filter: Constraint<&ResolvedLocationConstraint<'_>>,
    relay: &WireguardRelay,
) -> bool {
    filter.matches(relay)
}

/// Returns whether `relay` satisfy the ownership constraint posed by `filter`.
pub fn filter_on_ownership(filter: Constraint<&Ownership>, relay: &WireguardRelay) -> bool {
    filter.matches(relay)
}

/// Returns whether `relay` satisfy the providers constraint posed by `filter`.
pub fn filter_on_providers(filter: Constraint<&Providers>, relay: &WireguardRelay) -> bool {
    filter.matches(relay)
}

/// Returns whether `relay` satisfy the daita constraint posed by `filter`.
pub fn filter_on_daita(filter: Constraint<bool>, relay: &WireguardRelay) -> bool {
    match (filter, &relay.endpoint_data) {
        // Only a subset of relays support DAITA, so filter out ones that don't.
        (Constraint::Only(true), WireguardRelayEndpointData { daita, .. }) => *daita,
        // If we don't require DAITA, any relay works.
        _ => true,
    }
}

/// Returns `true` if no city- or hostname-level [`GeographicLocationConstraint`] in `location`
/// matches `relay`. This covers both `Constraint::Any` (no constraint at all, meaning the relay
/// is not specifically targeted) and constraints that only mention the relay's country.
///
/// Used to determine whether a relay with `include_in_country = false` should be treated as a
/// fallback: if the user has pinpointed a specific city or hostname that contains this relay,
/// we honour that explicit choice and promote it to a primary match.
pub fn is_country_only_match(
    location: Constraint<&ResolvedLocationConstraint<'_>>,
    relay: &WireguardRelay,
) -> bool {
    match location {
        // No location constraint — relay is not specifically targeted.
        Constraint::Any => true,
        Constraint::Only(resolved) => {
            // It is a country-only match as long as none of the matching constraints
            // is more specific than a country (i.e. city or hostname).
            !resolved
                .into_iter()
                .filter(|loc| loc.matches(relay))
                .any(|loc| !loc.is_country())
        }
    }
}

/// Wrapper around [`GeographicLocationConstraint`].
/// Useful for iterating over a set of [`GeographicLocationConstraint`] where custom lists
/// are considered.
#[derive(Debug, Clone)]
pub struct ResolvedLocationConstraint<'a>(Vec<&'a GeographicLocationConstraint>);

impl<'a> ResolvedLocationConstraint<'a> {
    /// Define the mapping from a [location][`LocationConstraint`] and a set of
    /// [custom lists][`CustomListsSettings`] to [`ResolvedLocationConstraint`].
    pub fn from_constraint(
        location_constraint: Constraint<&'a LocationConstraint>,
        custom_lists: &'a CustomListsSettings,
    ) -> Constraint<ResolvedLocationConstraint<'a>> {
        match location_constraint {
            Constraint::Any => Constraint::Any,
            Constraint::Only(location) => Constraint::Only(match location {
                LocationConstraint::Location(location) => {
                    ResolvedLocationConstraint(vec![location])
                }
                LocationConstraint::CustomList { list_id } => custom_lists
                    .iter()
                    .find(|list| list.id() == *list_id)
                    .map(|custom_list| {
                        ResolvedLocationConstraint(custom_list.locations.iter().collect())
                    })
                    .unwrap_or_else(|| {
                        log::warn!("Resolved non-existent custom list with id {list_id:?}");
                        ResolvedLocationConstraint(vec![])
                    }),
            }),
        }
    }
}

impl<'a> IntoIterator for &'a ResolvedLocationConstraint<'a> {
    type Item = &'a GeographicLocationConstraint;
    type IntoIter = std::iter::Copied<std::slice::Iter<'a, &'a GeographicLocationConstraint>>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter().copied()
    }
}

impl Match<WireguardRelay> for &ResolvedLocationConstraint<'_> {
    fn matches(&self, relay: &WireguardRelay) -> bool {
        self.into_iter().any(|location| location.matches(relay))
    }
}

//! This module is responsible for filtering the whole relay list based on queries.
use std::{collections::HashSet, ops::RangeInclusive};

use super::query::{ObfuscationQuery, RelayQuery, WireguardRelayQuery};
use crate::RuntimeParameters;
use mullvad_types::{
    constraints::{Constraint, Match},
    custom_list::CustomListsSettings,
    relay_constraints::{
        GeographicLocationConstraint, InternalBridgeConstraints, LocationConstraint, Ownership,
        Providers, ShadowsocksSettings,
    },
    relay_list::{Relay, RelayEndpointData, RelayList, WireguardRelayEndpointData},
};
use talpid_types::net::{IpVersion, TunnelType};

/// Filter a list of relays and their endpoints based on constraints.
/// Only relays with (and including) matching endpoints are returned.
pub fn filter_matching_relay_list(
    query: &RelayQuery,
    relay_list: &RelayList,
    custom_lists: &CustomListsSettings,
    runtime_parameters: RuntimeParameters,
) -> Vec<Relay> {
    let relays = relay_list.relays();

    let locations = ResolvedLocationConstraint::from_constraint(query.location(), custom_lists);
    let shortlist = relays
            // Filter on tunnel type
            .filter(|relay| filter_tunnel_type(&query.tunnel_protocol(), relay))
            // Filter on active relays
            .filter(|relay| filter_on_active(relay))
            // Filter by location
            .filter(|relay| filter_on_location(&locations, relay))
            // Filter by ownership
            .filter(|relay| filter_on_ownership(&query.ownership(), relay))
            // Filter by providers
            .filter(|relay| filter_on_providers(query.providers(), relay))
            // Filter by DAITA support
            .filter(|relay| filter_on_daita(&query.wireguard_constraints().daita, relay))
            // Filter by obfuscation support
            .filter(|relay| filter_on_obfuscation(query.wireguard_constraints(), relay_list, relay, runtime_parameters.clone()));

    // The last filtering to be done is on the `include_in_country` attribute found on each
    // relay. When the location constraint is based on country, a relay which has
    // `include_in_country` set to true should always be prioritized over relays which has this
    // flag set to false. We should only consider relays with `include_in_country` set to false
    // if there are no other candidates left.
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
    let locations =
        ResolvedLocationConstraint::from_constraint(&constraints.location, custom_lists);
    relays
            // Filter on active relays
            .filter(|relay| filter_on_active(relay))
            // Filter on bridge type
            .filter(|relay| filter_bridge(relay))
            // Filter by location
            .filter(|relay| filter_on_location(&locations, relay))
            // Filter by ownership
            .filter(|relay| filter_on_ownership(&constraints.ownership, relay))
            // Filter by providers
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
pub fn filter_on_location(
    filter: &Constraint<ResolvedLocationConstraint<'_>>,
    relay: &Relay,
) -> bool {
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

/// Returns whether `relay` satisfy the daita constraint posed by `filter`.
pub fn filter_on_daita(filter: &Constraint<bool>, relay: &Relay) -> bool {
    match (filter, &relay.endpoint_data) {
        // Only a subset of relays support DAITA, so filter out ones that don't.
        (
            Constraint::Only(true),
            RelayEndpointData::Wireguard(WireguardRelayEndpointData { daita, .. }),
        ) => *daita,
        // If we don't require DAITA, any relay works.
        _ => true,
    }
}

/// Returns whether `relay` satisfies the obfuscation settings.
fn filter_on_obfuscation(
    query: &WireguardRelayQuery,
    relay_list: &RelayList,
    relay: &Relay,
    runtime_parameters: RuntimeParameters,
) -> bool {
    match &query.obfuscation {
        // Shadowsocks has relay-specific constraints
        ObfuscationQuery::Shadowsocks(settings) => {
            let wg_data = &relay_list.wireguard;
            filter_on_shadowsocks(
                &wg_data.shadowsocks_port_ranges,
                &query.ip_version,
                settings,
                relay,
                runtime_parameters,
            )
        }

        // If Shadowsocks is not a requirement, then there are no relay-specific constraints
        _ => true,
    }
}

/// Returns whether `relay` satisfies the Shadowsocks filter posed by `port`.
fn filter_on_shadowsocks(
    port_ranges: &[RangeInclusive<u16>],
    ip_version: &Constraint<IpVersion>,
    settings: &ShadowsocksSettings,
    relay: &Relay,
    runtime_parameters: RuntimeParameters,
) -> bool {
    let ip_version = super::detailer::resolve_ip_version(*ip_version, runtime_parameters.ipv4);

    match (settings, &relay.endpoint_data) {
        // If Shadowsocks is specifically asked for, we must check if the specific relay supports
        // our port. If there are extra addresses, then all ports are available, so we do
        // not need to do this.
        (
            ShadowsocksSettings {
                port: Constraint::Only(desired_port),
            },
            RelayEndpointData::Wireguard(wg_data),
        ) => {
            let filtered_extra_addrs = wg_data
                .shadowsocks_extra_addr_in
                .iter()
                .find(|&&addr| IpVersion::from(addr) == ip_version);

            filtered_extra_addrs.is_some()
                || port_ranges.iter().any(|range| range.contains(desired_port))
        }

        // Otherwise, any relay works.
        _ => true,
    }
}

/// Returns whether the relay is an OpenVPN relay.
pub const fn filter_openvpn(relay: &Relay) -> bool {
    matches!(relay.endpoint_data, RelayEndpointData::Openvpn)
}

/// Returns whether the relay matches the tunnel constraint `filter`
#[cfg(not(target_os = "android"))]
pub const fn filter_tunnel_type(filter: &TunnelType, relay: &Relay) -> bool {
    match filter {
        TunnelType::OpenVpn => filter_openvpn(relay),
        TunnelType::Wireguard => filter_wireguard(relay),
    }
}

/// Returns whether the relay matches the tunnel constraint `filter`
#[cfg(target_os = "android")]
pub const fn filter_tunnel_type(_: &TunnelType, relay: &Relay) -> bool {
    // Only keep Wireguard relays on Android (i.e. filter out OpenVPN relays)
    filter_wireguard(relay)
}

/// Returns whether the relay is a Wireguard relay.
pub const fn filter_wireguard(relay: &Relay) -> bool {
    matches!(relay.endpoint_data, RelayEndpointData::Wireguard(_))
}

/// Returns whether the relay is a bridge.
pub const fn filter_bridge(relay: &Relay) -> bool {
    matches!(relay.endpoint_data, RelayEndpointData::Bridge)
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
        location_constraint: &'a Constraint<LocationConstraint>,
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
                    .find(|list| list.id == *list_id)
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

impl Match<Relay> for ResolvedLocationConstraint<'_> {
    fn matches(&self, relay: &Relay) -> bool {
        self.into_iter().any(|location| location.matches(relay))
    }
}

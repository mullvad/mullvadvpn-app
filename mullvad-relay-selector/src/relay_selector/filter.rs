use super::{
    AnnotatedRelayList, RelaySelector,
    endpoint_set::{RelayEndpointSet, Verdict, VerdictExt},
};
use mullvad_types::{
    constraints::{Constraint, Match},
    custom_list::CustomListsSettings,
    relay_constraints::{GeographicLocationConstraint, LocationConstraint},
    relay_list::{WireguardRelay, WireguardRelayEndpointData},
    relay_selector::{
        EntryConstraints, EntrySpecificConstraints, ExitConstraints, MultihopConstraints,
        Predicate, Reason, RelayPartitions,
    },
};

use either::Either;
use itertools::Itertools;

pub(crate) struct MultiHopPartitions {
    pub(crate) entries: RelayPartitions,
    pub(crate) exits: RelayPartitions,
}

/// The combined result of partitioning relays for autohop.
///
/// Contains both the singlehop and multihop partitions so that the caller can
/// decide which configuration to use, or collapse the two with
/// [`AutohopPartition::into_relay_partitions`].
pub(super) struct AutohopPartition {
    pub(super) singlehop: RelayPartitions,
    pub(super) multihop: MultiHopPartitions,
}

impl AutohopPartition {
    /// Collapse to a [`RelayPartitions`] where:
    /// - `matches` = exits valid for singlehop **or** for multihop (requires valid entry)
    /// - `discards` = exits valid for neither
    pub(super) fn into_relay_partitions(self) -> RelayPartitions {
        let AutohopPartition {
            singlehop,
            multihop,
        } = self;

        if multihop.entries.matches.is_empty() {
            // No valid entry → multihop unavailable. Autohop reduces to singlehop, so
            // the singlehop partition (matches and discard reasons) is the right answer.
            return singlehop;
        }

        // Multihop is available. A relay matches autohop if it works as singlehop OR as
        // a multihop exit. For discards, prefer multihop.exits reasons over singlehop's:
        // multihop is the more permissive path here, so its reasons describe the minimum
        // fix to unblock the relay in autohop. Singlehop adds entry-specific reasons
        // (DAITA / obfuscation / ip_version) that don't apply to a relay used as a
        // multihop exit and would over-report what the user needs to change.
        let mut matches = singlehop.matches;
        for relay in multihop.exits.matches {
            if !matches.contains(&relay) {
                matches.push(relay);
            }
        }

        let discards = multihop
            .exits
            .discards
            .into_iter()
            .filter(|(r, _)| !matches.contains(r))
            .collect();

        RelayPartitions { matches, discards }
    }
}

impl RelaySelector {
    /// As opposed to the prior [`Self::get_relay_by_query`], this function is stateless with
    /// regards to any particular config / settings, but is stateful in the sense that it works with
    /// the [`RelaySelector`]s current relay list. [`RelaySelector::partition_relays`] is idempotent
    /// if the relay list is pinned.
    pub fn partition_relays(&self, predicate: Predicate) -> RelayPartitions {
        let relays = self.relays.read().unwrap();
        let custom_lists = self.custom_lists();
        match predicate {
            Predicate::Singlehop(constraints) => {
                partition_entry(&relays, &constraints, &custom_lists)
            }
            Predicate::Autohop(constraints) => {
                partition_autohop(&relays, constraints, &custom_lists).into_relay_partitions()
            }
            Predicate::Entry(multihop_constraints) => {
                partition_multihop(&relays, multihop_constraints, &custom_lists).entries
            }
            Predicate::Exit(multihop_constraints) => {
                partition_multihop(&relays, multihop_constraints, &custom_lists).exits
            }
        }
    }
}

// Evaluate a verdict function over every relay in the current relay list and partition the
// results into matches and discards.
fn partition_by_verdict(
    relays: &AnnotatedRelayList,
    f: impl Fn(&WireguardRelay, &RelayEndpointSet) -> Verdict,
) -> RelayPartitions {
    let (matches, discards) = relays
        .inner
        .relays()
        .filter_map(|relay| {
            let set = relays.endpoint_set_for(relay).or_else(|| {
                log::warn!(
                    "Relay {} has no valid WireGuard port ranges; skipping",
                    relay.hostname
                );
                None
            })?;
            Some((relay, set))
        })
        .partition_map(|(relay, set)| match f(relay, set) {
            Verdict::Accept => Either::Left(relay.clone()),
            Verdict::Reject(reasons) => Either::Right((relay.clone(), reasons)),
        });
    RelayPartitions { matches, discards }
}

pub(super) fn partition_entry(
    relays: &AnnotatedRelayList,
    constraints: &EntryConstraints,
    custom_lists: &CustomListsSettings,
) -> RelayPartitions {
    partition_by_verdict(relays, |relay, endpoint_set| {
        usable_as_entry(relay, endpoint_set, &constraints.entry_specific).and(usable_as_exit(
            relay,
            &constraints.general,
            custom_lists,
        ))
    })
}

pub(super) fn partition_autohop(
    relays: &AnnotatedRelayList,
    constraints: EntryConstraints,
    custom_lists: &CustomListsSettings,
) -> AutohopPartition {
    AutohopPartition {
        singlehop: partition_entry(relays, &constraints, custom_lists),
        multihop: partition_multihop(relays, constraints.into_autohop(), custom_lists),
    }
}

pub(super) fn partition_multihop(
    relays: &AnnotatedRelayList,
    MultihopConstraints { entry, exit }: MultihopConstraints,
    custom_lists: &CustomListsSettings,
) -> MultiHopPartitions {
    let mut entries = partition_entry(relays, &entry, custom_lists);
    let mut exits = partition_exit(relays, &exit, custom_lists);

    remove_conflicting_relay(&mut entries, &mut exits);

    MultiHopPartitions { entries, exits }
}

pub(super) fn partition_exit(
    relays: &AnnotatedRelayList,
    constraints: &ExitConstraints,
    custom_lists: &CustomListsSettings,
) -> RelayPartitions {
    partition_by_verdict(relays, |relay, _endpoint_set| {
        usable_as_exit(relay, constraints, custom_lists)
    })
}

/// Check that the relay satisfies the entry specific criteria. Note that this does not check exit constraints.
///
/// Here we consider only entry specific constraints, i.e. DAITA, obfuscation and IP version.
fn usable_as_entry(
    relay: &WireguardRelay,
    endpoint_set: &RelayEndpointSet,
    constraints: &EntrySpecificConstraints,
) -> Verdict {
    let daita = filter_on_daita(constraints.daita, relay).if_false(Reason::Daita);

    let obfuscation_verdict = endpoint_set.obfuscation_verdict(constraints);
    daita.and(obfuscation_verdict)
}

/// Check that the relay satisfies the exit criteria.
fn usable_as_exit(
    relay: &WireguardRelay,
    ExitConstraints {
        location,
        providers,
        ownership,
    }: &ExitConstraints,
    custom_lists: &CustomListsSettings,
) -> Verdict {
    let ownership = ownership.matches(relay).if_false(Reason::Ownership);
    let providers = providers.matches(relay).if_false(Reason::Providers);
    let location = location_criteria(relay, location, custom_lists);
    let active = relay.active.if_false(Reason::Inactive);

    ownership.and(providers).and(location).and(active)
}

fn location_criteria(
    relay: &WireguardRelay,
    location: &Constraint<LocationConstraint>,
    custom_lists: &CustomListsSettings,
) -> Verdict {
    let location_constraint =
        ResolvedLocationConstraint::from_constraint(location.as_ref(), custom_lists);

    if !location_constraint.matches(relay) {
        return Verdict::reject(Reason::Location);
    }

    filter_include_in_country(relay, location_constraint)
}

/// Ensure the same relay cannot be chosen as both entry and exit.
///
/// If either side's `matches` contains a single relay that also appears in the other
/// side's `matches`, that relay is moved to the other side's `discards` with
/// [`Reason::Conflict`]. The two directions are evaluated sequentially, so when a relay
/// is uniquely the match on both sides it is labeled `Conflict` on only one side and
/// remains in the other side's `matches`.
fn remove_conflicting_relay(entries: &mut RelayPartitions, exits: &mut RelayPartitions) {
    move_unique_conflict(entries, exits);
    move_unique_conflict(exits, entries);
}

/// If `from.matches` is a singleton that also appears in `into.matches`, move that relay
/// from `into.matches` into `into.discards` with [`Reason::Conflict`].
fn move_unique_conflict(from: &RelayPartitions, into: &mut RelayPartitions) {
    let [unique] = from.matches.as_slice() else {
        return;
    };
    let Some(pos) = into.matches.iter().position(|r| r == unique) else {
        return;
    };
    let relay = into.matches.remove(pos);
    into.discards.push((relay, vec![Reason::Conflict]));
}

/// Returns whether `relay` satisfy the daita constraint posed by `filter`.
fn filter_on_daita(filter: Constraint<bool>, relay: &WireguardRelay) -> bool {
    match (filter, &relay.endpoint_data) {
        // Only a subset of relays support DAITA, so filter out ones that don't.
        (Constraint::Only(true), WireguardRelayEndpointData { daita, .. }) => *daita,
        // If we don't require DAITA, any relay works.
        _ => true,
    }
}

/// Relays with `include_in_country = false` are only selectable when the location
/// constraint targets them at city or hostname level.
fn filter_include_in_country(
    relay: &WireguardRelay,
    location_constraint: Constraint<ResolvedLocationConstraint<'_>>,
) -> Verdict {
    if !relay.include_in_country && {
        match location_constraint {
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
    } {
        Verdict::reject(Reason::IncludeInCountry)
    } else {
        Verdict::Accept
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

impl Match<WireguardRelay> for ResolvedLocationConstraint<'_> {
    fn matches(&self, relay: &WireguardRelay) -> bool {
        self.into_iter().any(|location| location.matches(relay))
    }
}

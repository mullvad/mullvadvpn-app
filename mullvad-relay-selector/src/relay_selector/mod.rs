//! The implementation of the relay selector.

pub mod detailer;
pub mod endpoint_set;
mod helpers;
pub mod matcher;
pub mod query;
pub mod relays;

use relays::{Multihop, Singlehop, WireguardConfig};

use crate::{
    detailer::wireguard_endpoint,
    error::Error,
    query::{Constraints, RelayQuery, WireguardRelayQuery, obfuscation_constraint_from_settings},
};

pub use mullvad_types::relay_list::Relay;
use mullvad_types::relay_selector::MultihopConstraints;
use mullvad_types::{
    Intersection,
    constraints::Constraint,
    custom_list::CustomListsSettings,
    endpoint::MullvadEndpoint,
    location::Coordinates,
    relay_constraints::{RelaySettings, WireguardConstraints},
    relay_list::{Bridge, BridgeList, RelayList, WireguardRelay},
    settings::Settings,
};
use std::ops::Deref;
use std::{
    collections::HashMap,
    sync::{Arc, LazyLock, Mutex, RwLock},
};
use talpid_types::net::{IpAvailability, IpVersion, obfuscation::Obfuscators, proxy::Shadowsocks};

/// [`RETRY_ORDER`] defines an ordered set of relay parameters which the relay selector
/// should prioritize on successive connection attempts. Note that these will *never* override user
/// preferences. See [the documentation on `RelayQuery`][RelayQuery] for further details.
///
/// Each entry is a [`RelayQuery`] that specifies only the axes that vary between
/// retry attempts (`ip_version` and `obfuscation`). All other fields are left as
/// `Constraint::Any` so that intersecting with user preferences preserves them.
///
/// This list should be kept in sync with the expected behavior defined in `docs/relay-selector.md`
pub static RETRY_ORDER: LazyLock<Vec<RelayQuery>> = LazyLock::new(|| {
    use query::builder::RelayQueryBuilder;
    vec![
        // 1 This works with any wireguard relay
        RelayQueryBuilder::new().build(),
        // 2
        RelayQueryBuilder::new().ip_version(IpVersion::V6).build(),
        // 3
        RelayQueryBuilder::new().shadowsocks().build(),
        // 4
        RelayQueryBuilder::new().quic().build(),
        // 5
        RelayQueryBuilder::new().udp2tcp().build(),
        // 6
        RelayQueryBuilder::new()
            .udp2tcp()
            .ip_version(IpVersion::V6)
            .build(),
        // 7
        RelayQueryBuilder::new().lwo().build(),
    ]
});

/// A [`RelayList`] together with pre-computed [`endpoint_set::RelayEndpointSet`]s for every
/// relay. Both are stored under the same [`RwLock`] in [`RelaySelector`] so that the cache
/// is always consistent with the list.
struct AnnotatedRelayList {
    inner: RelayList,
    /// Maps relay hostname → pre-computed endpoint set.
    /// Relays whose WireGuard port ranges are empty are absent from this map.
    endpoint_sets: HashMap<String, endpoint_set::RelayEndpointSet>,
}

impl AnnotatedRelayList {
    fn new(list: RelayList) -> Self {
        let endpoint_sets = list
            .relays()
            .filter_map(|relay| {
                let set = endpoint_set::RelayEndpointSet::new(relay, &list.wireguard)?;
                Some((relay.hostname.clone(), set))
            })
            .collect();
        Self {
            inner: list,
            endpoint_sets,
        }
    }

    fn endpoint_set_for(&self, relay: &WireguardRelay) -> Option<&endpoint_set::RelayEndpointSet> {
        self.endpoint_sets.get(&relay.hostname)
    }
}

#[derive(Clone)]
pub struct RelaySelector {
    config: Arc<Mutex<Config>>,
    // Relays are updated very infrequently, but might conceivably be accessed by multiple readers at
    // the same time.
    relays: Arc<RwLock<AnnotatedRelayList>>,
    bridges: Arc<RwLock<BridgeList>>,
}

/// Relay selector configuration. This datastructure keeps the relay selector in sync with
/// mullvad-daemon.
///
/// Carries the pre-computed [`RelayQuery`] derived from the user's settings together with the
/// custom lists needed for location filtering. When the user has configured a custom tunnel
/// endpoint the relay selector is never queried, so a dormant default config is used.
#[derive(Debug, Clone, Default)]
struct Config {
    query: RelayQuery,
    custom_lists: CustomListsSettings,
}

impl From<&Settings> for Config {
    fn from(settings: &Settings) -> Self {
        let RelaySettings::Normal(user_preferences) = &settings.relay_settings else {
            // Custom tunnel endpoints bypass the relay selector entirely.
            return Config::default();
        };

        let WireguardConstraints {
            ip_version,
            allowed_ips,
            use_multihop,
            entry_location,
            entry_providers,
            entry_ownership,
        } = user_preferences.wireguard_constraints.clone();

        #[cfg(daita)]
        let daita = settings.tunnel_options.wireguard.daita.enabled;
        #[cfg(not(daita))]
        let daita = false;

        #[cfg(daita)]
        let daita_use_multihop_if_necessary = settings
            .tunnel_options
            .wireguard
            .daita
            .use_multihop_if_necessary;
        #[cfg(not(daita))]
        let daita_use_multihop_if_necessary = false;

        let quantum_resistant = settings.tunnel_options.wireguard.quantum_resistant;

        let wireguard_constraints = WireguardRelayQuery {
            ip_version,
            allowed_ips,
            use_multihop: Constraint::Only(use_multihop),
            entry_location,
            entry_providers,
            entry_ownership,
            obfuscation: obfuscation_constraint_from_settings(
                settings.obfuscation_settings.clone(),
            ),
            daita: Constraint::Only(daita),
            daita_use_multihop_if_necessary: Constraint::Only(daita_use_multihop_if_necessary),
            quantum_resistant: Constraint::Only(quantum_resistant),
        };

        Config {
            query: RelayQuery::new(
                user_preferences.location.clone(),
                user_preferences.providers.clone(),
                user_preferences.ownership,
                wireguard_constraints,
            ),
            custom_lists: settings.custom_lists.clone(),
        }
    }
}

impl From<RelayQuery> for Config {
    fn from(query: RelayQuery) -> Self {
        Config {
            query,
            custom_lists: CustomListsSettings::default(),
        }
    }
}

/// The return type of [`RelaySelector::get_relay`].
#[derive(Clone, Debug)]
pub struct GetRelay {
    pub endpoint: MullvadEndpoint,
    pub obfuscator: Option<Obfuscators>,
    pub inner: WireguardConfig,
}

impl TryFrom<Settings> for RelayQuery {
    type Error = crate::Error;

    fn try_from(value: Settings) -> Result<Self, Self::Error> {
        match &value.relay_settings {
            RelaySettings::Normal(_) => Ok(Config::from(&value).query),
            RelaySettings::CustomTunnelEndpoint(_) => Err(Error::InvalidConstraints),
        }
    }
}

impl RelaySelector {
    /// Create a new `RelaySelector` from a set of relays and bridges.
    pub fn from_query(query: RelayQuery, relays: RelayList, bridges: BridgeList) -> Self {
        RelaySelector {
            config: Arc::new(Mutex::new(Config::from(query))),
            relays: Arc::new(RwLock::new(AnnotatedRelayList::new(relays))),
            bridges: Arc::new(RwLock::new(bridges)),
        }
    }

    pub fn from_settings(config: &Settings, relays: RelayList, bridges: BridgeList) -> Self {
        RelaySelector {
            config: Arc::new(Mutex::new(Config::from(config))),
            relays: Arc::new(RwLock::new(AnnotatedRelayList::new(relays))),
            bridges: Arc::new(RwLock::new(bridges)),
        }
    }

    /// Update the relay selector config.
    pub fn set_config(&self, settings: &Settings) {
        *self.config.lock().unwrap() = Config::from(settings);
    }

    /// Update only the custom list settings used for location filtering.
    pub fn set_custom_lists(&self, custom_lists: CustomListsSettings) {
        self.config.lock().unwrap().custom_lists = custom_lists;
    }

    /// Peek the relay list.
    pub fn relay_list<T>(&self, f: impl Fn(&RelayList) -> T) -> T {
        let relays = self.relays.read().unwrap();
        f(&relays.inner)
    }

    pub fn bridge_list<T>(&self, f: impl Fn(&BridgeList) -> T) -> T {
        let bridges = &self.bridges.read().unwrap();
        f(bridges)
    }

    fn custom_lists(&self) -> CustomListsSettings {
        self.config.lock().unwrap().custom_lists.clone()
    }

    /// Update the list of relays
    pub fn set_relays(&self, relays: RelayList) {
        log::trace!("Updating relay list");
        *self.relays.write().unwrap() = AnnotatedRelayList::new(relays);
    }

    /// Update the list of bridges
    pub fn set_bridges(&self, bridges: BridgeList) {
        log::trace!("Updating bridge list");
        let mut key = self.bridges.write().unwrap();
        *key = bridges;
    }

    /// Returns all countries and cities. The cities in the object returned does not have any
    /// relays in them.
    pub fn get_relays(&self) -> RelayList {
        self.relay_list(RelayList::clone)
    }

    /// Returns all bridges.
    pub fn get_bridges(&self) -> BridgeList {
        self.bridge_list(BridgeList::clone)
    }

    /// Returns a shadowsocks endpoint for any [`Bridge`] in [`BridgeList`].
    pub fn get_bridge_forced(&self) -> Option<Shadowsocks> {
        self.bridge_list(Self::get_proxy_settings)
            .map(|(endpoint, _bridge)| endpoint)
            .inspect_err(|error| log::error!("Failed to get bridge: {error}"))
            .ok()
    }
    /// Returns a random relay and relay endpoint matching the current constraints corresponding to
    /// `retry_attempt` in one of the retry orders while considering the [`Config`].
    pub fn get_relay(
        &self,
        retry_attempt: usize,
        runtime_ip_availability: IpAvailability,
    ) -> Result<GetRelay, Error> {
        self.get_relay_with_custom_params(retry_attempt, &RETRY_ORDER, runtime_ip_availability)
    }

    /// Returns a random relay and relay endpoint matching the current constraints defined by
    /// `retry_order` corresponding to `retry_attempt`.
    pub fn get_relay_with_custom_params(
        &self,
        retry_attempt: usize,
        retry_order: &[RelayQuery],
        runtime_ip_availability: IpAvailability,
    ) -> Result<GetRelay, Error> {
        // Extract data from config and release the lock immediately.
        // The lock must not be held when calling get_wireguard_relay_inner,
        // because usable_as_exit → location_criteria → custom_lists() re-acquires it.
        let mut user_query = self.config.lock().unwrap().query.clone();

        // Runtime parameters may affect which of the default queries that are considered.
        // For example, queries which rely on IPv6 will not be considered if
        // working IPv6 is not available at runtime.
        apply_ip_availability(runtime_ip_availability, &mut user_query)?;
        log::trace!("Merging user preferences {user_query:?} with default retry strategy");

        // Select a relay using the user's preferences merged with the nth compatible query
        // in `retry_order`, looping back to the start of `retry_order` if necessary.
        let maybe_relay = retry_order
            .iter()
            .filter_map(|query| query.clone().intersection(user_query.clone()))
            .filter_map(|query| self.get_relay_by_query(query).ok())
            .cycle()
            .nth(retry_attempt);

        match maybe_relay {
            Some(v) => Ok(v),
            // If none of the queries in `retry_order` merged with `user_query` yield any relays,
            // attempt to only consider the user's preferences.
            None => self.get_relay_by_query(user_query),
        }
    }

    /// Returns random relay and relay endpoint matching `query`.
    /// Note that this does not take custom config into consideration.
    pub fn get_relay_by_query(&self, query: RelayQuery) -> Result<GetRelay, Error> {
        let inner = self.select_wireguard_relay(query.resolve(), &query)?;

        // Build endpoint and obfuscator using pre-computed endpoint sets.
        let entry = match &inner {
            WireguardConfig::Singlehop { exit } => exit,
            WireguardConfig::Multihop { entry, .. } => entry,
        };

        let annotated = self.relays.read().unwrap();
        let endpoint_set = annotated
            .endpoint_set_for(entry)
            .ok_or_else(|| Error::NoRelay(Box::new(query.clone())))?;

        let WireguardRelayQuery {
            ip_version,
            allowed_ips,
            obfuscation,
            ..
        } = query.wireguard_constraints();

        let (wg_addr, obfuscator) =
            endpoint_set.get_wireguard_obfuscator(obfuscation, *ip_version)?;

        let endpoint = wireguard_endpoint(allowed_ips, &annotated.inner.wireguard, &inner, wg_addr);

        Ok(GetRelay {
            endpoint,
            obfuscator,
            inner,
        })
    }

    /// Select relay(s) matching the constraints, handling singlehop, autohop, and multihop routing.
    fn select_wireguard_relay(
        &self,
        constraints: Constraints,
        original_query: &RelayQuery,
    ) -> Result<WireguardConfig, Error> {
        let relays = self.relays.read().unwrap();
        match constraints {
            Constraints::Singlehop(constraints) => {
                let partitions = self.partition_entry(&relays, &constraints);
                match helpers::pick_random_relay(&partitions.matches) {
                    Some(exit) => Ok(WireguardConfig::from(Singlehop::new(exit.clone()))),
                    None => Err(Error::NoRelay(Box::new(original_query.clone()))),
                }
            }
            Constraints::Autohop(constraints) => {
                let autohop = self.partition_autohop(&relays, constraints.clone());
                // Attempt to pick a single relay that matches all constraints
                if let Some(exit) = helpers::pick_random_relay(&autohop.singlehop.matches) {
                    return Ok(WireguardConfig::from(Singlehop::new(exit.clone())));
                }
                // Otherwise fall through to multihop using the pre-computed partition.
                let multihop_constraints = constraints.clone().into_autohop();
                self.select_from_multihop_partitions(autohop.multihop, multihop_constraints)
            }
            Constraints::Multihop(constraints) => {
                let partitions = self.partition_multihop(&relays, constraints.clone());
                self.select_from_multihop_partitions(partitions, constraints)
            }
        }
    }

    /// Select separate entry and exit relays for a multihop configuration.
    ///
    /// If the entry location constraint is [`Constraint::Any`] (autohop), the entry relay
    /// is chosen globally and biased towards the geographically closest relay to the exit.
    /// Otherwise, entry and exit are picked randomly within their respective constraints.
    fn select_from_multihop_partitions(
        &self,
        partitions: partition_relays::MultiHopPartitions,
        multihop_constraints: MultihopConstraints,
    ) -> Result<WireguardConfig, Error> {
        let MultihopConstraints {
            entry: entry_constraints,
            exit: exit_constraints,
        } = multihop_constraints;

        let exit = helpers::pick_random_relay(&partitions.exits.matches)
            .ok_or_else(|| Error::NoRelayExit(Box::new(exit_constraints)))?;

        let entry = if matches!(entry_constraints.general.location, Constraint::Any) {
            // `Constraint::Any` implies an automatic entry selection with no geographical constraints.
            // Bias this selection towards the closest relay to the exit.
            let mut candidates: Vec<_> = partitions
                .entries
                .matches
                .iter()
                .map(|e| RelayWithDistance::new_with_distance_from(e.clone(), &exit.location))
                .collect();
            candidates.sort_unstable_by(|a, b| a.distance.total_cmp(&b.distance));
            let min_distance = candidates.first().map(|r| r.distance).unwrap_or_default();
            let closest: Vec<_> = candidates
                .into_iter()
                .take_while(|r| r.distance <= min_distance)
                .map(|r| r.relay)
                .collect();
            helpers::pick_random_relay_excluding(&closest, exit)
                .ok_or_else(|| Error::NoRelayEntry(Box::new(entry_constraints)))?
                .clone()
        } else {
            helpers::pick_random_relay_excluding(&partitions.entries.matches, exit)
                .ok_or_else(|| Error::NoRelayEntry(Box::new(entry_constraints)))?
                .clone()
        };

        Ok(WireguardConfig::from(Multihop::new(entry, exit.clone())))
    }

    /// Try to get a bridge that matches the given `constraints`.
    ///
    /// The connection details are returned alongside the relay hosting the bridge.
    fn get_proxy_settings(bridge_list: &BridgeList) -> Result<(Shadowsocks, Bridge), Error> {
        // Filter on active relays
        let bridges: Vec<Bridge> = bridge_list
            .bridges()
            .iter()
            .filter(|bridge| bridge.active)
            .cloned()
            .collect();

        let bridge = helpers::pick_random_relay(&bridges)
            .cloned()
            .ok_or(Error::NoBridge)?;
        let endpoint = detailer::bridge_endpoint(&bridge_list.bridge_endpoint, &bridge)
            .ok_or(Error::NoBridge)?;
        Ok((endpoint, bridge))
    }
}

fn apply_ip_availability(
    runtime_ip_availability: IpAvailability,
    user_query: &mut RelayQuery,
) -> Result<(), Error> {
    let ip_version = match runtime_ip_availability {
        IpAvailability::Ipv4 => Constraint::Only(IpVersion::V4),
        IpAvailability::Ipv6 => Constraint::Only(IpVersion::V6),
        IpAvailability::Ipv4AndIpv6 => Constraint::Any,
    };
    let merged = user_query
        .wireguard_constraints()
        .ip_version
        .intersection(ip_version)
        .ok_or_else(|| {
            // It is safe to call `unwrap` on `wireguard_constraints().ip_version` here
            // because this will only be called if intersection returns None
            // and the only way None can be returned is if both
            // ip_version and wireguard_constraints.ip_version are Constraint::Only and thus
            // guarantees that wireguard_constraints.ip_version is Constraint::Only
            let family = user_query.wireguard_constraints().ip_version.unwrap();
            Error::IpVersionUnavailable { family }
        })?;
    user_query.wireguard_constraints_mut().ip_version = merged;
    Ok(())
}

#[derive(Clone)]
struct RelayWithDistance<T> {
    distance: f64,
    relay: T,
}

impl<T> RelayWithDistance<T> {
    fn new_with_distance_from(relay: T, from: impl Into<Coordinates>) -> Self
    where
        T: Deref<Target = Relay> + Clone,
    {
        let distance = relay.location.distance_from(from);
        RelayWithDistance {
            relay: relay.clone(),
            distance,
        }
    }
}

mod partition_relays {
    use super::{
        AnnotatedRelayList, RelaySelector,
        endpoint_set::{RelayEndpointSet, Verdict, VerdictExt},
        matcher,
    };
    use mullvad_types::{
        constraints::Constraint,
        custom_list::CustomListsSettings,
        relay_constraints::LocationConstraint,
        relay_list::WireguardRelay,
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

            // Start with singlehop matches (exits that are also their own valid entry).
            let mut matches = singlehop.matches;

            // Add multihop exits only when at least one valid entry actually exists.
            // Without any valid entry, no autohop configuration can be established.
            let (multihop_exit_matches_added, multihop_exit_matches_stranded) =
                if !multihop.entries.matches.is_empty() {
                    (multihop.exits.matches, vec![])
                } else {
                    (vec![], multihop.exits.matches)
                };

            // Also covers the conflict edge-case: a relay valid as singlehop even
            // though it was moved to multihop.exits.discards with Reason::Conflict.
            for relay in multihop_exit_matches_added {
                if !matches.contains(&relay) {
                    matches.push(relay);
                }
            }

            // Discards = exits that appear in neither matches list.
            // Includes exits stranded by empty entries (added with no reasons).
            let mut discards: Vec<(WireguardRelay, Vec<Reason>)> = multihop
                .exits
                .discards
                .into_iter()
                .filter(|(r, _)| !matches.contains(r))
                .collect();
            discards.extend(
                multihop_exit_matches_stranded
                    .into_iter()
                    .filter(|r| !matches.contains(r))
                    .map(|r| (r, vec![])),
            );

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
            match predicate {
                Predicate::Singlehop(constraints) => self.partition_entry(&relays, &constraints),
                Predicate::Autohop(constraints) => self
                    .partition_autohop(&relays, constraints)
                    .into_relay_partitions(),
                Predicate::Entry(multihop_constraints) => {
                    self.partition_multihop(&relays, multihop_constraints)
                        .entries
                }
                Predicate::Exit(multihop_constraints) => {
                    self.partition_multihop(&relays, multihop_constraints).exits
                }
            }
        }

        // Evaluate a verdict function over every relay in the current relay list and partition the
        // results into matches and discards.
        pub(super) fn partition_by_verdict(
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
            let mut partitions = RelayPartitions { matches, discards };
            rescue_fallbacks(&mut partitions);
            partitions
        }

        pub(super) fn partition_entry(
            &self,
            relays: &AnnotatedRelayList,
            constraints: &EntryConstraints,
        ) -> RelayPartitions {
            Self::partition_by_verdict(relays, |relay, endpoint_set| {
                self.usable_as_entry(relay, endpoint_set, &constraints.entry_specific)
                    .and(self.usable_as_exit(relay, &constraints.general))
            })
        }

        pub(super) fn partition_autohop(
            &self,
            relays: &AnnotatedRelayList,
            constraints: EntryConstraints,
        ) -> AutohopPartition {
            AutohopPartition {
                singlehop: self.partition_entry(relays, &constraints),
                multihop: self.partition_multihop(relays, constraints.into_autohop()),
            }
        }

        pub(super) fn partition_multihop(
            &self,
            relays: &AnnotatedRelayList,
            MultihopConstraints { entry, exit }: MultihopConstraints,
        ) -> MultiHopPartitions {
            let mut entries = self.partition_entry(relays, &entry);
            let mut exits = self.partition_exit(relays, &exit);

            remove_conflicting_relay(&mut entries, &mut exits);

            // Conflict removal may have emptied a side. Rescue IncludeInCountry fallbacks so
            // that a pool where every relay has include_in_country=false can still form a
            // valid pair.
            rescue_fallbacks(&mut entries);
            rescue_fallbacks(&mut exits);

            MultiHopPartitions { entries, exits }
        }

        pub(super) fn partition_exit(
            &self,
            relays: &AnnotatedRelayList,
            constraints: &ExitConstraints,
        ) -> RelayPartitions {
            Self::partition_by_verdict(relays, |relay, _endpoint_set| {
                self.usable_as_exit(relay, constraints)
            })
        }

        /// Check that the relay satisfies the entry specific criteria. Note that this does not check exit constraints.
        ///
        /// Here we consider only entry specific constraints, i.e. DAITA, obfuscation and IP version.
        pub(crate) fn usable_as_entry(
            &self,
            relay: &WireguardRelay,
            endpoint_set: &RelayEndpointSet,
            constraints: &EntrySpecificConstraints,
        ) -> Verdict {
            let daita_on = constraints.daita;
            let daita = matcher::filter_on_daita(daita_on, relay).if_false(Reason::Daita);

            let obfuscation_verdict = endpoint_set.obfuscation_verdict(constraints);
            daita.and(obfuscation_verdict)
        }

        /// Check that the relay satisfies the exit criteria.
        pub(crate) fn usable_as_exit(
            &self,
            relay: &WireguardRelay,
            ExitConstraints {
                location,
                providers,
                ownership,
            }: &ExitConstraints,
        ) -> Verdict {
            let ownership =
                matcher::filter_on_ownership(ownership.as_ref(), relay).if_false(Reason::Ownership);
            let providers =
                matcher::filter_on_providers(providers.as_ref(), relay).if_false(Reason::Providers);
            let location = self.location_criteria(relay, location);
            let active = relay.active.if_false(Reason::Inactive);

            ownership.and(providers).and(location).and(active)
        }

        pub(crate) fn location_criteria(
            &self,
            relay: &WireguardRelay,
            location: &Constraint<LocationConstraint>,
        ) -> Verdict {
            let custom_lists: CustomListsSettings = self.custom_lists();

            let resolved = matcher::ResolvedLocationConstraint::from_constraint(
                location.as_ref(),
                &custom_lists,
            );

            if !matcher::filter_on_location(resolved.as_ref(), relay) {
                return Verdict::reject(Reason::Location);
            }

            // Relays with `include_in_country = false` are deprioritized when the location
            // constraint only targets the country (or is unconstrained). A city- or
            // hostname-level constraint that matches the relay overrides this — the user
            // has made an explicit, specific choice.
            if !relay.include_in_country && matcher::is_country_only_match(resolved.as_ref(), relay)
            {
                Verdict::reject(Reason::IncludeInCountry)
            } else {
                Verdict::Accept
            }
        }
    }

    /// Promote relays whose sole discard reason is [`Reason::IncludeInCountry`] into `matches`
    /// when no primary relay is available. This implements the "use only when necessary"
    /// semantics of `include_in_country = false`.
    pub(super) fn rescue_fallbacks(partitions: &mut RelayPartitions) {
        if !partitions.matches.is_empty() {
            return;
        }
        let mut rescued = vec![];
        partitions.discards.retain(|(relay, reasons)| {
            if reasons.as_slice() == [Reason::IncludeInCountry] {
                rescued.push(relay.clone());
                false
            } else {
                true
            }
        });
        partitions.matches.extend(rescued);
    }

    /// Ensure the same relay cannot be chosen as both entry and exit.
    ///
    /// If either side's `matches` contains a single relay that also appears in the other
    /// side's `matches`, that relay is moved to the other side's `discards` with
    /// [`Reason::Conflict`]. The two directions are evaluated sequentially, so when a relay
    /// is uniquely the match on both sides it is labeled `Conflict` on only one side and
    /// remains in the other side's `matches` — which keeps the multihop pair formable once
    /// the other side falls back to an [`Reason::IncludeInCountry`] relay via
    /// [`rescue_fallbacks`].
    pub(crate) fn remove_conflicting_relay(
        entries: &mut RelayPartitions,
        exits: &mut RelayPartitions,
    ) {
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
}

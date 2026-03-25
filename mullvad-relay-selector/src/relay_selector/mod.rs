//! The implementation of the relay selector.

pub mod detailer;
mod helpers;
pub mod matcher;
pub mod query;
pub mod relays;

use detailer::resolve_ip_version;
use matcher::{filter_matching_relay_list, filter_matching_relay_list_include_all};
use relays::{Multihop, Singlehop, WireguardConfig};

use crate::{
    detailer::wireguard_endpoint,
    error::{EndpointErrorDetails, Error},
    query::{ObfuscationQuery, RelayQuery, RelayQueryExt, WireguardRelayQuery},
};

use either::Either;
use itertools::Itertools;
pub use mullvad_types::relay_list::Relay;
use mullvad_types::{
    CustomTunnelEndpoint, Intersection,
    constraints::Constraint,
    custom_list::CustomListsSettings,
    endpoint::MullvadEndpoint,
    location::Coordinates,
    relay_constraints::{
        LocationConstraint, ObfuscationSettings, RelayConstraints, RelaySettings,
        WireguardConstraints,
    },
    relay_list::{Bridge, BridgeList, RelayList, WireguardRelay},
    relay_selector::{
        EntryConstraints, ExitConstraints, MultihopConstraints, Predicate, Reason, RelayPartitions,
    },
    settings::Settings,
    wireguard::QuantumResistantState,
};
use std::{
    borrow::Borrow,
    ops::RangeInclusive,
    sync::{Arc, LazyLock, Mutex, RwLock},
};
use std::{net::IpAddr, ops::Deref};
use talpid_types::net::{
    IpAvailability, IpVersion,
    obfuscation::{ObfuscatorConfig, Obfuscators},
    proxy::Shadowsocks,
};

/// [`RETRY_ORDER`] defines an ordered set of relay parameters which the relay selector
/// should prioritize on successive connection attempts. Note that these will *never* override user
/// preferences. See [the documentation on `RelayQuery`][RelayQuery] for further details.
///
/// This list should be kept in sync with the expected behavior defined in `docs/relay-selector.md`
pub static RETRY_ORDER: LazyLock<Vec<RelayQuery>> = LazyLock::new(|| {
    use query::builder::{IpVersion, RelayQueryBuilder};
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

#[derive(Clone)]
pub struct RelaySelector {
    config: Arc<Mutex<SelectorConfig>>,
    // Relays are updated very infrequently, but might conceivably be accessed by multiple readers at
    // the same time.
    relays: Arc<RwLock<RelayList>>,
    bridges: Arc<RwLock<BridgeList>>,
}

// TODO: Rename to simply `Config`
#[derive(Clone)]
pub struct SelectorConfig {
    // Normal relay settings
    pub relay_settings: RelaySettings,
    pub additional_constraints: AdditionalWireguardConstraints,
    pub custom_lists: CustomListsSettings,
    // Wireguard specific data
    pub obfuscation_settings: ObfuscationSettings,
}

impl SelectorConfig {
    pub fn from_settings(settings: &Settings) -> Self {
        let additional_constraints = AdditionalWireguardConstraints {
            #[cfg(daita)]
            daita: settings.tunnel_options.wireguard.daita.enabled,
            #[cfg(daita)]
            daita_use_multihop_if_necessary: settings
                .tunnel_options
                .wireguard
                .daita
                .use_multihop_if_necessary,

            #[cfg(not(daita))]
            daita: false,
            #[cfg(not(daita))]
            daita_use_multihop_if_necessary: false,

            quantum_resistant: settings.tunnel_options.wireguard.quantum_resistant,
        };

        Self {
            relay_settings: settings.relay_settings.clone(),
            additional_constraints,
            obfuscation_settings: settings.obfuscation_settings.clone(),
            custom_lists: settings.custom_lists.clone(),
        }
    }
}

/// Extra relay constraints not specified in `relay_settings`.
#[derive(Default, Debug, Clone, Eq, PartialEq)]
pub struct AdditionalWireguardConstraints {
    /// If true, select WireGuard relays that support DAITA. If false, select any
    /// server.
    pub daita: bool,

    /// If true and multihop is disabled, will set up multihop with an automatic entry relay if
    /// DAITA is enabled.
    pub daita_use_multihop_if_necessary: bool,

    /// If enabled, select relays that support PQ.
    pub quantum_resistant: QuantumResistantState,
}

/// This enum exists to separate the two types of [`SelectorConfig`] that exists.
///
/// The first one is a "regular" config, where [`SelectorConfig::relay_settings`] is
/// [`RelaySettings::Normal`]. This is the most common variant, and there exists a
/// mapping from this variant to [`RelayQueryBuilder`]. Being able to implement
/// `From<NormalSelectorConfig> for RelayQueryBuilder` was the main motivator for introducing these
/// seemingly useless derivatives of [`SelectorConfig`].
///
/// The second one is a custom config, where [`SelectorConfig::relay_settings`] is
/// [`RelaySettings::CustomTunnelEndpoint`]. For this variant, the endpoint where the client should
/// connect to is already specified inside of the variant, so in practice the relay selector becomes
/// superfluous. Also, there exists no mapping to [`RelayQueryBuilder`].
///
/// [`RelayQueryBuilder`]: query::builder::RelayQueryBuilder
#[derive(Debug, Clone)]
enum SpecializedSelectorConfig<'a> {
    // This variant implements `From<NormalSelectorConfig> for RelayQuery`
    Normal(NormalSelectorConfig<'a>),
    // This variant does not
    Custom(&'a CustomTunnelEndpoint),
}

/// A special-cased variant of [`SelectorConfig`].
///
/// For context, see [`SpecializedSelectorConfig`].
#[derive(Debug, Clone)]
struct NormalSelectorConfig<'a> {
    user_preferences: &'a RelayConstraints,
    additional_preferences: &'a AdditionalWireguardConstraints,
    custom_lists: &'a CustomListsSettings,
    // Wireguard specific data
    obfuscation_settings: &'a ObfuscationSettings,
}

/// The return type of [`RelaySelector::get_relay`].
// There won't ever be many instances of GetRelay floating around, so the 'large' difference in
// size between its variants is negligible.
#[expect(clippy::large_enum_variant)]
#[derive(Clone, Debug)]
pub enum GetRelay {
    Mullvad {
        endpoint: MullvadEndpoint,
        obfuscator: Option<SelectedObfuscator>,
        // This field seems really weird to me, can we get rid of
        inner: WireguardConfig,
    },
    Custom(CustomTunnelEndpoint),
}

#[derive(Clone, Debug)]
pub struct SelectedObfuscator {
    pub config: Obfuscators,
    pub relay: WireguardRelay,
}

impl From<(ObfuscatorConfig, WireguardRelay)> for SelectedObfuscator {
    fn from((config, relay): (ObfuscatorConfig, WireguardRelay)) -> Self {
        SelectedObfuscator {
            config: Obfuscators::Single(config),
            relay,
        }
    }
}

impl Default for SelectorConfig {
    fn default() -> Self {
        let default_settings = Settings::default();
        SelectorConfig {
            relay_settings: default_settings.relay_settings,
            additional_constraints: AdditionalWireguardConstraints::default(),
            obfuscation_settings: default_settings.obfuscation_settings,
            custom_lists: default_settings.custom_lists,
        }
    }
}

impl TryFrom<Settings> for RelayQuery {
    type Error = crate::Error;

    fn try_from(value: Settings) -> Result<Self, Self::Error> {
        let selector_config = SelectorConfig::from_settings(&value);
        let specialized_selector_config = SpecializedSelectorConfig::from(&selector_config);
        let SpecializedSelectorConfig::Normal(normal_selector_config) = specialized_selector_config
        else {
            return Err(Error::InvalidConstraints);
        };

        RelayQuery::try_from(normal_selector_config)
    }
}

impl<'a> From<&'a SelectorConfig> for SpecializedSelectorConfig<'a> {
    fn from(value: &'a SelectorConfig) -> SpecializedSelectorConfig<'a> {
        match &value.relay_settings {
            RelaySettings::CustomTunnelEndpoint(custom_tunnel_endpoint) => {
                SpecializedSelectorConfig::Custom(custom_tunnel_endpoint)
            }
            RelaySettings::Normal(user_preferences) => {
                SpecializedSelectorConfig::Normal(NormalSelectorConfig {
                    user_preferences,
                    additional_preferences: &value.additional_constraints,
                    obfuscation_settings: &value.obfuscation_settings,
                    custom_lists: &value.custom_lists,
                })
            }
        }
    }
}

// TODO: Implement From instead
impl<'a> TryFrom<NormalSelectorConfig<'a>> for RelayQuery {
    type Error = crate::Error;

    /// Map user settings to [`RelayQuery`].
    fn try_from(value: NormalSelectorConfig<'a>) -> Result<Self, Self::Error> {
        /// Map the Wireguard-specific bits of `value` to [`WireguardRelayQuery`]
        fn wireguard_constraints(
            wireguard_constraints: WireguardConstraints,
            additional_constraints: AdditionalWireguardConstraints,
            obfuscation_settings: ObfuscationSettings,
        ) -> WireguardRelayQuery {
            let WireguardConstraints {
                ip_version,
                allowed_ips,
                use_multihop,
                entry_location,
                entry_providers,
                entry_ownership,
            } = wireguard_constraints;
            let AdditionalWireguardConstraints {
                daita,
                daita_use_multihop_if_necessary,
                quantum_resistant,
            } = additional_constraints;
            WireguardRelayQuery {
                ip_version,
                allowed_ips,
                use_multihop: Constraint::Only(use_multihop),
                entry_location,
                entry_providers,
                entry_ownership,
                obfuscation: ObfuscationQuery::from(obfuscation_settings),
                daita: Constraint::Only(daita),
                daita_use_multihop_if_necessary: Constraint::Only(daita_use_multihop_if_necessary),
                quantum_resistant: Constraint::Only(quantum_resistant),
            }
        }

        let wireguard_constraints = wireguard_constraints(
            value.user_preferences.wireguard_constraints.clone(),
            value.additional_preferences.clone(),
            value.obfuscation_settings.clone(),
        );
        Ok(RelayQuery::new(
            value.user_preferences.location.clone(),
            value.user_preferences.providers.clone(),
            value.user_preferences.ownership,
            wireguard_constraints,
        ))
    }
}

impl RelaySelector {
    /// Create a new `RelaySelector` from a set of relays and bridges.
    pub fn new(config: SelectorConfig, relays: RelayList, bridges: BridgeList) -> Self {
        RelaySelector {
            config: Arc::new(Mutex::new(config)),
            relays: Arc::new(RwLock::new(relays)),
            bridges: Arc::new(RwLock::new(bridges)),
        }
    }

    /// Update the relay selector config.
    pub fn set_config(&self, config: SelectorConfig) {
        *self.config.lock().unwrap() = config;
    }

    /// Peek the relay list.
    pub fn relay_list<T>(&self, f: impl Fn(&RelayList) -> T) -> T {
        let relays = &self.relays.read().unwrap();
        f(relays)
    }

    pub fn bridge_list<T>(&self, f: impl Fn(&BridgeList) -> T) -> T {
        let bridges = &self.bridges.read().unwrap();
        f(bridges)
    }

    fn custom_lists(&self) -> CustomListsSettings {
        let config_guard = self.config.lock().unwrap();
        let SpecializedSelectorConfig::Normal(config) =
            SpecializedSelectorConfig::from(&*config_guard)
        else {
            panic!("Custom lists are not supported with custom relays")
        };
        config.custom_lists.clone()
    }

    /// Update the list of relays
    pub fn set_relays(&self, relays: RelayList) {
        log::trace!("Updating relay list");
        let mut key = self.relays.write().unwrap();
        *key = relays;
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

    /// Returns random relay and relay endpoint matching `query`.
    pub fn get_relay_by_query(&self, query: RelayQuery) -> Result<GetRelay, Error> {
        let config_guard = self.config.lock().unwrap();
        let config = SpecializedSelectorConfig::from(&*config_guard);
        match config {
            SpecializedSelectorConfig::Custom(custom_config) => {
                Ok(GetRelay::Custom(custom_config.clone()))
            }
            SpecializedSelectorConfig::Normal(normal_config) => {
                let relay_list = self.get_relays();
                Self::get_wireguard_relay_inner(&query, normal_config.custom_lists, &relay_list)
            }
        }
    }

    /// Returns a random relay and relay endpoint matching the current constraints corresponding to
    /// `retry_attempt` in one of the retry orders while considering the [`SelectorConfig`].
    pub fn get_relay(
        &self,
        retry_attempt: usize,
        runtime_ip_availability: IpAvailability,
    ) -> Result<GetRelay, Error> {
        let config_guard = self.config.lock().unwrap();
        let config = SpecializedSelectorConfig::from(&*config_guard);
        match config {
            SpecializedSelectorConfig::Custom(custom_config) => {
                Ok(GetRelay::Custom(custom_config.clone()))
            }
            SpecializedSelectorConfig::Normal(_normal_config) => {
                drop(config_guard);
                self.get_relay_with_custom_params(
                    retry_attempt,
                    &RETRY_ORDER,
                    runtime_ip_availability,
                )
            }
        }
    }

    /// Returns a random relay and relay endpoint matching the current constraints defined by
    /// `retry_order` corresponding to `retry_attempt`.
    pub fn get_relay_with_custom_params(
        &self,
        retry_attempt: usize,
        retry_order: &[RelayQuery],
        runtime_ip_availability: IpAvailability,
    ) -> Result<GetRelay, Error> {
        let config_guard = self.config.lock().unwrap();
        let config = SpecializedSelectorConfig::from(&*config_guard);

        // Short-circuit if a custom tunnel endpoint is to be used - don't have to involve the
        // relay selector further!
        match config {
            SpecializedSelectorConfig::Custom(custom_config) => {
                Ok(GetRelay::Custom(custom_config.clone()))
            }
            SpecializedSelectorConfig::Normal(normal_config) => {
                let parsed_relays = self.get_relays();
                // Merge user preferences with the relay selector's default preferences.
                let custom_lists = normal_config.custom_lists;
                let mut user_query = RelayQuery::try_from(normal_config)?;
                // Runtime parameters may affect which of the default queries that are considered.
                // For example, queries which rely on IPv6 will not be considered if
                // working IPv6 is not available at runtime.
                apply_ip_availability(runtime_ip_availability, &mut user_query)?;
                log::trace!("Merging user preferences {user_query:?} with default retry strategy");
                // Select a relay using the user's preferences merged with the nth compatible query
                // in `retry_order`, looping back to the start of `retry_order` if
                // necessary.
                let maybe_relay = retry_order
                    .iter()
                    .filter_map(|query| query.clone().intersection(user_query.clone()))
                    .filter_map(|query| Self::get_wireguard_relay_inner(&query, custom_lists, &parsed_relays).ok())
                    .cycle() // If the above filters remove all relays, cycle will also return an empty iterator
                    .nth(retry_attempt);
                match maybe_relay {
                    Some(v) => Ok(v),
                    // If none of the queries in `retry_order` merged with `user_preferences` yield any relays,
                    // attempt to only consider the user's preferences.
                    None => {
                        Self::get_wireguard_relay_inner(&user_query, custom_lists, &parsed_relays)
                    }
                }
            }
        }
    }

    /// Derive a valid relay configuration from `query`.
    ///
    /// # Parameters
    /// - `query`: Constraints that filter the available relays, such as geographic location or
    ///   tunnel protocol.
    /// - `parsed_relays`: The complete set of parsed relays available for selection.
    /// - `custom_lists`
    ///
    /// # Returns
    /// * An `Err` if no exit relay can be chosen
    /// * An `Err` if no entry relay can be chosen (if multihop is enabled on `query`)
    /// * an `Err` if no [`MullvadEndpoint`] can be derived from the selected relay(s).
    /// * `Ok(GetRelay::Wireguard)` otherwise
    ///
    /// [`MullvadEndpoint`]: mullvad_types::endpoint::MullvadEndpoint
    fn get_wireguard_relay_inner(
        query: &RelayQuery,
        custom_lists: &CustomListsSettings,
        parsed_relays: &RelayList,
    ) -> Result<GetRelay, Error> {
        let inner = Self::get_wireguard_relay_config(query, custom_lists, parsed_relays)?;
        let endpoint = Self::get_wireguard_endpoint(query, parsed_relays, &inner)?;
        let obfuscator =
            Self::get_wireguard_obfuscator(query, inner.clone(), &endpoint, parsed_relays)?;

        Ok(GetRelay::Mullvad {
            endpoint,
            obfuscator,
            inner,
        })
    }

    /// Derive a valid Wireguard relay configuration from `query`.
    ///
    /// # Returns
    /// * An `Err` if no exit relay can be chosen
    /// * An `Err` if no entry relay can be chosen (if multihop is enabled on `query`)
    /// * `Ok(WireguardConfig)` otherwise
    fn get_wireguard_relay_config(
        query: &RelayQuery,
        custom_lists: &CustomListsSettings,
        parsed_relays: &RelayList,
    ) -> Result<WireguardConfig, Error> {
        let inner = if query.singlehop() {
            match Self::get_wireguard_singlehop_config(query, custom_lists, parsed_relays) {
                Some(exit) => WireguardConfig::from(exit),
                None => {
                    // TODO: This autohop logic should be replaced

                    // If we found no matching relays because DAITA was enabled, and
                    // `use_multihop_if_necessary` is enabled, try enabling
                    // multihop and connecting using an automatically selected
                    // entry relay.
                    if query.using_daita() && query.use_multihop_if_necessary() {
                        let multihop = Self::get_wireguard_auto_multihop_config(
                            query,
                            custom_lists,
                            parsed_relays,
                        )?;
                        WireguardConfig::from(multihop)
                    } else {
                        return Err(Error::NoRelay(Box::new(query.clone())));
                    }
                }
            }
        } else {
            // A DAITA compatible entry should be used even when the exit is DAITA compatible.
            // This only makes sense in context: The user is no longer able to explicitly choose an
            // entry relay with smarting routing enabled, even if multihop is turned on
            // Also implied: Multihop is enabled.
            let multihop = if query.using_daita() && query.use_multihop_if_necessary() {
                Self::get_wireguard_auto_multihop_config(query, custom_lists, parsed_relays)?
            } else {
                Self::get_wireguard_multihop_config(query, custom_lists, parsed_relays)?
            };
            WireguardConfig::from(multihop)
        };

        Ok(inner)
    }

    /// Select a valid Wireguard exit relay.
    ///
    /// # Returns
    /// * `Ok(Singlehop)` if an exit relay was selected
    /// * `None` otherwise
    fn get_wireguard_singlehop_config(
        query: &RelayQuery,
        custom_lists: &CustomListsSettings,
        parsed_relays: &RelayList,
    ) -> Option<Singlehop> {
        let candidates = filter_matching_relay_list(query, parsed_relays, custom_lists);
        helpers::pick_random_relay(&candidates)
            .cloned()
            .map(Singlehop::new)
    }

    /// Select a valid Wireguard exit relay, together with with an automatically chosen entry relay.
    ///
    /// # Returns
    /// * An `Err` if no entry/exit relay can be chosen
    /// * `Ok(Multihop)` otherwise
    fn get_wireguard_auto_multihop_config(
        query: &RelayQuery,
        custom_lists: &CustomListsSettings,
        parsed_relays: &RelayList,
    ) -> Result<Multihop, Error> {
        let mut exit_relay_query = query.clone();

        // DAITA & obfuscation should only be enabled for the entry relay
        let mut wireguard_constraints = exit_relay_query.wireguard_constraints().clone();
        wireguard_constraints.daita = Constraint::Only(false);
        wireguard_constraints.obfuscation = ObfuscationQuery::Off;
        exit_relay_query.set_wireguard_constraints(wireguard_constraints);

        let exit_candidates =
            filter_matching_relay_list(&exit_relay_query, parsed_relays, custom_lists);
        let exit = helpers::pick_random_relay(&exit_candidates)
            .ok_or_else(|| Error::NoRelayExit(Box::new(exit_relay_query)))?;

        // generate a list of potential entry relays, disregarding any location constraint
        let mut entry_query = query.clone();
        entry_query.set_location(Constraint::Any);
        let mut entry_candidates =
            filter_matching_relay_list(&entry_query, parsed_relays, custom_lists)
                .into_iter()
                .map(|entry| RelayWithDistance::new_with_distance_from(entry, &exit.location))
                .collect_vec();

        // TODO: We need to keep logic for sorting/weighing the selection
        // sort entry relay candidates by distance, and pick one from those that are closest
        entry_candidates.sort_unstable_by(|a, b| a.distance.total_cmp(&b.distance));
        let smallest_distance = entry_candidates.first().map(|relay| relay.distance);
        let smallest_distance = smallest_distance.unwrap_or_default();
        let entry_candidates = entry_candidates
            .into_iter()
            // only consider the relay(s) with the smallest distance. note that the list is sorted.
            // NOTE: we could relax this requirement, but since so few relays support DAITA
            // (and this function is only used for daita) we might end up picking relays that are
            // needlessly far away. Consider making this closure  configurable if needed.
            .take_while(|relay| relay.distance <= smallest_distance)
            .map(|relay_with_distance| relay_with_distance.relay)
            .collect_vec();
        let entry = helpers::pick_random_relay_excluding(&entry_candidates, exit)
            .ok_or_else(|| Error::NoRelayEntry(Box::new(entry_query)))?;

        Ok(Multihop::new(entry.clone(), exit.clone()))
    }

    /// This function selects a valid entry and exit relay to be used in a multihop configuration.
    ///
    /// # Returns
    /// * An `Err` if no exit relay can be chosen
    /// * An `Err` if no entry relay can be chosen
    /// * An `Err` if the chosen entry and exit relays are the same
    /// * `Ok(WireguardConfig::Multihop)` otherwise
    fn get_wireguard_multihop_config(
        query: &RelayQuery,
        custom_lists: &CustomListsSettings,
        parsed_relays: &RelayList,
    ) -> Result<Multihop, Error> {
        // Here, we modify the original query just a bit.
        // The actual query for an entry relay is identical as for an exit relay, with the
        // exception that the location is different and that the entry filters may be different.
        // The location is dictated by the query's multihop constraint.
        let mut entry_relay_query = query.clone();
        entry_relay_query.set_location(query.wireguard_constraints().entry_location.clone());
        entry_relay_query.set_providers(query.wireguard_constraints().entry_providers.clone());
        entry_relay_query.set_ownership(query.wireguard_constraints().entry_ownership);
        // After we have our two queries (one for the exit relay & one for the entry relay),
        // we can query for all exit & entry candidates! All candidates are needed for the next
        // step.
        let mut exit_relay_query = query.clone();

        // DAITA & Obfuscation should only be enabled for the entry relay
        let mut wg_constraints = exit_relay_query.wireguard_constraints().clone();
        wg_constraints.daita = Constraint::Only(false);
        wg_constraints.obfuscation = ObfuscationQuery::Off;
        exit_relay_query.set_wireguard_constraints(wg_constraints);

        // Opportunistically filter on `include_in_country`.
        let exit_candidates =
            filter_matching_relay_list(&exit_relay_query, parsed_relays, custom_lists);
        let entry_candidates =
            filter_matching_relay_list(&entry_relay_query, parsed_relays, custom_lists);

        Self::pick_working_entry_exit_combo(
            query,
            exit_candidates.as_slice(),
            entry_candidates.as_slice(),
        )
        .map(|(exit, entry)| Multihop::new(entry.clone(), exit.clone()))
        .or_else(|_e| {
            // Sometimes, the set of relays is too small to consider the `include_in_country`
            // flag. It might just be that if we disregard the `include_in_country` flag, we
            // manage to find candidate relays. This is rather unlikely, but it might just
            // happen.
            let exit_candidates = filter_matching_relay_list_include_all(
                &exit_relay_query,
                parsed_relays,
                custom_lists,
            );
            let entry_candidates = filter_matching_relay_list_include_all(
                &entry_relay_query,
                parsed_relays,
                custom_lists,
            );
            Self::pick_working_entry_exit_combo(
                query,
                exit_candidates.as_slice(),
                entry_candidates.as_slice(),
            )
            .map(|(exit, entry)| Multihop::new(entry.clone(), exit.clone()))
        })
    }

    /// Avoid picking the same relay for entry and exit by choosing one and excluding it when
    /// choosing the other.
    fn pick_working_entry_exit_combo<'a>(
        query: &RelayQuery,
        exit_candidates: &'a [WireguardRelay],
        entry_candidates: &'a [WireguardRelay],
    ) -> Result<(&'a WireguardRelay, &'a WireguardRelay), Error> {
        match (exit_candidates, entry_candidates) {
            // In the case where there is only one entry to choose from, we have to pick it before
            // the exit
            (exits, [entry]) if exits.contains(entry) => {
                helpers::pick_random_relay_excluding(exits, entry)
                    .map(|exit| (exit, entry))
                    .ok_or_else(|| Error::NoRelayExit(Box::new(query.clone())))
            }
            // Vice versa for the case of only one exit
            ([exit], entries) if entries.contains(exit) => {
                helpers::pick_random_relay_excluding(entries, exit)
                    .map(|entry| (exit, entry))
                    .ok_or_else(|| Error::NoRelayEntry(Box::new(query.clone())))
            }
            (exits, entries) => {
                let exit = helpers::pick_random_relay(exits);
                match exit {
                    None => Err(Error::NoRelayExit(Box::new(query.clone()))),
                    Some(exit) => helpers::pick_random_relay_excluding(entries, exit)
                        .map(|entry| (exit, entry))
                        .ok_or_else(|| Error::NoRelayEntry(Box::new(query.clone()))),
                }
            }
        }
    }

    /// Constructs a [`MullvadEndpoint`] with details for how to connect to `relay`.
    ///
    /// [`MullvadEndpoint`]: mullvad_types::endpoint::MullvadEndpoint
    fn get_wireguard_endpoint(
        query: &RelayQuery,
        parsed_relays: &RelayList,
        relay: &WireguardConfig,
    ) -> Result<MullvadEndpoint, Error> {
        wireguard_endpoint(
            query.wireguard_constraints(),
            &parsed_relays.wireguard,
            relay,
        )
        .map_err(|internal| Error::NoEndpoint {
            internal,
            relay: EndpointErrorDetails::from_wireguard(relay.clone()),
        })
    }

    fn get_wireguard_obfuscator(
        query: &RelayQuery,
        relay: WireguardConfig,
        endpoint: &MullvadEndpoint,
        parsed_relays: &RelayList,
    ) -> Result<Option<SelectedObfuscator>, Error> {
        let obfuscator_relay = match relay {
            WireguardConfig::Singlehop { exit } => exit,
            WireguardConfig::Multihop { entry, .. } => entry,
        };
        let box_obfuscation_error = |error: helpers::Error| Error::NoObfuscator(Box::new(error));

        match &query.wireguard_constraints().obfuscation {
            ObfuscationQuery::Off => Ok(None),
            #[cfg(not(feature = "staggered-obfuscation"))]
            ObfuscationQuery::Auto => Ok(None),
            ObfuscationQuery::Port(_) => Ok(None),
            #[cfg(feature = "staggered-obfuscation")]
            ObfuscationQuery::Auto => {
                let shadowsocks_ports = &parsed_relays.wireguard.shadowsocks_port_ranges;
                let udp2tcp_ports = &parsed_relays.wireguard.udp2tcp_ports;
                helpers::get_multiplexer_obfuscator(
                    udp2tcp_ports,
                    shadowsocks_ports,
                    obfuscator_relay,
                    endpoint,
                )
                .map(Some)
                .map_err(box_obfuscation_error)
            }
            ObfuscationQuery::Udp2tcp(settings) => {
                let udp2tcp_ports = &parsed_relays.wireguard.udp2tcp_ports;

                helpers::get_udp2tcp_obfuscator(settings, udp2tcp_ports, obfuscator_relay, endpoint)
                    .map(|obfs| Some(obfs.into()))
                    .map_err(box_obfuscation_error)
            }
            ObfuscationQuery::Shadowsocks(settings) => {
                let port_ranges = &parsed_relays.wireguard.shadowsocks_port_ranges;
                let obfuscation = helpers::get_shadowsocks_obfuscator(
                    settings,
                    port_ranges,
                    obfuscator_relay,
                    endpoint,
                )
                .map(|obfs| obfs.into())
                .map_err(box_obfuscation_error)?;

                Ok(Some(obfuscation))
            }
            ObfuscationQuery::Quic => {
                let ip_version =
                    resolve_ip_version(query.wireguard_constraints().ip_version.as_ref());
                Ok(helpers::get_quic_obfuscator(obfuscator_relay, ip_version).map(Into::into))
            }
            ObfuscationQuery::Lwo => {
                Ok(helpers::get_lwo_obfuscator(obfuscator_relay, endpoint).map(Into::into))
            }
        }
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

    // == NEW relay selector API. ==
    // Starting afresh, but this should be used in existing functions.

    /// As opposed to the prior [`Self::get_relay_by_query`], this function is stateless with
    /// regards to any particular config / settings, but is stateful in the sense that it works with
    /// the [`RelaySelector`]s current relay list. [`RelaySelector::partition_relays`] is idempotent
    /// if the relay list is pinned.
    //
    // # Algorithm
    // pseudo-code
    //
    // let criterias := [<is relay active?>, <is relay in expected location?>, ..]
    //
    // for each relay in relay list ..
    // let mut reject_reasons := []
    // for each criteria ..
    // if let Reject(reason) = critera.eval(relay) {
    //   reject_reasons.push(reason)
    // }
    // ..
    // if rejections_reasons.empty() {
    //   (relay, Accept),
    // } else {
    //   (relay, Reject(reject_reasons))
    // }
    // ..
    pub fn partition_relays(&self, predicate: Predicate) -> RelayPartitions {
        let criteria = self.criteria(predicate);
        // The relay selection algorithm is embarrassingly parallel: https://en.wikipedia.org/wiki/Embarrassingly_parallel.
        // We may explore the entire search space (`relays` x `criteria`) without any synchronization between different
        // branches if we really wanted to.
        let (matches, discards) = self.get_relays()
            .into_relays()
            .map(|relay| {
                let verdict = criteria.eval(&relay);
                (relay, verdict)
            })
            // After this mapping, a single reduce is performed to partition the relays based on
            // their assigned verdict.
            .partition_map(|(relay, verdict)| match verdict {
                Verdict::Accept => Either::Left(relay),
                Verdict::Reject(rejected) => Either::Right((relay, rejected)),
            });
        RelayPartitions { matches, discards }
    }

    /// Calculate the set of criteria each predicate will render for scrutinizing relays.
    fn criteria(&self, predicate: Predicate) -> Criteria<'_, WireguardRelay> {
        let shadowsocks_port_ranges =
            self.relay_list(|rl| rl.wireguard.shadowsocks_port_ranges.clone());
        let custom_lists: CustomListsSettings = self.custom_lists();

        match predicate {
            Predicate::Singlehop(constraints) => {
                // TODO: get rid of clone?
                let usable_as_exit =
                    Self::usable_as_exit(constraints.general.clone(), custom_lists);
                let usable_as_entry = Self::usable_as_entry(constraints, shadowsocks_port_ranges);

                usable_as_entry.and(usable_as_exit)
            }
            Predicate::Autohop(constraints) => {
                // This case is identical to `singlehop`, except that it does not generally care about obfuscation, DAITA, etc.
                // In those cases, the VPN traffic may be routed through an alternative entry relay.

                // If a specific exit is to be selected, it could occupy the only possible entry relay.
                // We may run `partition_relays` searching for the entry relay. If the result yields one
                // (and only one) specific relay, we know that it must be excluded from the list of
                // exit relays.
                let can_find_autohop_entry = {
                    // Search globally for an alternate entry relay.
                    let global_predicate = {
                        let mut constraints = constraints.clone();
                        // TODO: Clear the entire `general`, i.e. provider and ownership filters too
                        constraints.general.location = Constraint::Any;
                        Predicate::Singlehop(constraints)
                    };
                    // Compare with the equiv predicate for the `Predicate::Exit` case.
                    let RelayPartitions { matches, .. } = self.partition_relays(global_predicate);

                    match matches.into_iter().at_most_one() {
                        Ok(None) => {
                            // Globally, there are no matching relays.
                            // The most sane thing we can do is to convert the original Predicate from
                            // `Autohop` to `Singlehop`, and re-evaluate those criteria for each relay
                            // to retrieve accurate reject reasons.
                            return self.criteria(Predicate::Singlehop(constraints));
                        }
                        Ok(Some(entry_relay)) => Criteria::new(move |relay: &WireguardRelay| {
                            (relay.inner == entry_relay.inner).if_true(Reason::Conflict)
                        }),
                        Err(_) => {
                            // There where more than 1 possible entry relays for the provided entry relay
                            // predicate, any exit relay goes.
                            Criteria::new(|_| Verdict::Accept)
                        }
                    }
                };

                // Check criteria that apply to both exits and entries
                let usable_as_exit =
                    Self::usable_as_exit(constraints.general.clone(), custom_lists);

                // Check criteria that apply specifically to entries
                let usable_as_entry = Self::usable_as_entry(constraints, shadowsocks_port_ranges);

                usable_as_exit.and(
                    // The relay must also be a valid entry.
                    usable_as_entry.or(
                        // Else another entry must be found.
                        can_find_autohop_entry,
                    ),
                )
            }
            Predicate::Entry(MultihopConstraints { entry, exit }) => {
                // If an exit is already selected, it should be rejected as a possible entry relay.
                // To find out if a certain location is already selected as an exit relay, we may
                // run `partition_relays` searching for the exit relay. If the result yields one
                // (and only one) specific relay, we know that it must be excluded from the list of
                // entry relays.
                // NOTE: We don't handle the case where there are zero exit relays here, you must
                // call the function again with `Predicate::Exit` to find that out.
                let doesnt_collide_with_exit = {
                    let exit_relay = self
                            // Compare with the equiv predicate for the `Predicate::Exit` case.
                            .partition_relays(Predicate::Singlehop(EntryConstraints { general: exit, ..Default::default()} ))
                            .matches
                            .into_iter()
                            .exactly_one();
                    match exit_relay {
                        Ok(entry_relay) => Criteria::new(move |relay: &WireguardRelay| {
                            (relay.inner == entry_relay.inner).if_true(Reason::Conflict)
                        }),
                        Err(_) => {
                            // There where more than 1 possible entry relays for the provided entry relay
                            // predicate, any exit relay goes.
                            Criteria::new(|_| Verdict::Accept)
                        }
                    }
                };

                // Except for the `usable_as_exit` condition, the remainder of the work is
                // ~equiv to `Predicate::Singlehop`.
                let usable_as_entry = Self::usable_as_entry(entry.clone(), shadowsocks_port_ranges);
                let usable_as_exit = Self::usable_as_exit(entry.general, custom_lists);
                usable_as_entry
                    .and(usable_as_exit)
                    .and(doesnt_collide_with_exit)
            }
            Predicate::Exit(MultihopConstraints { entry, exit }) => {
                // If an entry is already selected, it should be rejected as a possible exit relay.
                // To find out if a certain location is already selected as an entry relay, we may
                // run `partition_relays` searching for the entry relay. If the result yields one
                // (and only one) specific relay, we know that it must be excluded from the list of
                // exit relays.
                let doesnt_collide_with_entry = {
                    let entry_relay = self
                        .partition_relays(Predicate::Singlehop(entry))
                        .matches
                        .into_iter()
                        .exactly_one();
                    match entry_relay {
                        Ok(entry_relay) => Criteria::new(move |relay: &WireguardRelay| {
                            (relay.inner == entry_relay.inner).if_true(Reason::Conflict)
                        }),
                        Err(_) => {
                            // There where more than 1 possible entry relays for the provided entry relay
                            // predicate, any exit relay goes.
                            Criteria::new(|_| Verdict::Accept)
                        }
                    }
                };

                let usable_as_exit = Self::usable_as_exit(exit, custom_lists);
                usable_as_exit.and(doesnt_collide_with_entry)
            }
        }
    }

    /// Check that the relay satisfies the entry specific criteria. Note that this does not check exit constraints.
    ///
    /// Here we consider only entry specific constraints, i.e. DAITA, obfuscation and IP version.
    fn usable_as_entry(
        constraints: EntryConstraints,
        shadowsocks_port_ranges: Vec<RangeInclusive<u16>>,
    ) -> Criteria<'static, WireguardRelay> {
        let daita_on = constraints.daita.as_ref().map(|settings| settings.enabled);
        let daita = Criteria::new(move |relay| {
            matcher::filter_on_daita(daita_on, relay).if_false(Reason::Daita)
        });

        let obfuscation_ipversion_port = Criteria::new(move |relay: &WireguardRelay| {
            let wg_endpoint_ip_version = match constraints.ip_version {
                Constraint::Any => Verdict::Accept,
                Constraint::Only(IpVersion::V4) => Verdict::Accept,
                Constraint::Only(IpVersion::V6) => {
                    relay.ipv6_addr_in.is_some().if_false(Reason::IpVersion)
                }
            };

            match obfuscation_criteria(&shadowsocks_port_ranges, relay, &constraints) {
                ObfuscationVerdict::AcceptWireguardEndpoint => wg_endpoint_ip_version,
                ObfuscationVerdict::AcceptObfuscationEndpoint => Verdict::Accept,
                ObfuscationVerdict::Reject(reason) => {
                    Verdict::reject(reason).and(wg_endpoint_ip_version)
                }
            }
        });
        daita.and(obfuscation_ipversion_port)
    }

    /// Check that the relay satisfies the exit criteria.
    fn usable_as_exit(
        ExitConstraints {
            location,
            providers,
            ownership,
        }: ExitConstraints,
        custom_lists: CustomListsSettings,
    ) -> Criteria<'static, WireguardRelay> {
        let ownership = Criteria::new(move |relay| {
            matcher::filter_on_ownership(ownership.as_ref(), relay).if_false(Reason::Ownership)
        });
        let providers = Criteria::new(move |relay| {
            matcher::filter_on_providers(providers.as_ref(), relay).if_false(Reason::Providers)
        });
        let location = Self::location_criteria(location, custom_lists);
        let active =
            Criteria::new(|relay: &WireguardRelay| relay.active.if_false(Reason::Inactive));

        ownership.and(providers).and(location).and(active)
    }

    fn location_criteria(
        location: Constraint<LocationConstraint>,
        custom_lists: CustomListsSettings,
    ) -> Criteria<'static, WireguardRelay> {
        Criteria::new(move |relay| {
            let location = matcher::ResolvedLocationConstraint::from_constraint(
                location.as_ref(),
                &custom_lists,
            );
            matcher::filter_on_location(location.as_ref(), relay).if_false(Reason::Location)
        })
    }
}

/// Verdict for connecting using an obfuscation method.
enum ObfuscationVerdict {
    /// Connect to the relay's "normal" WireGuard IP address.
    AcceptWireguardEndpoint,
    /// Connect to the relay using an IP address dedicated to
    /// this obfuscation method.
    AcceptObfuscationEndpoint,
    /// The requested obfuscation cannot be resolved on the relay
    /// with the given port or IP version.
    Reject(Reason),
}

fn obfuscation_criteria(
    shadowsocks_port_ranges: &[RangeInclusive<u16>],
    relay: &WireguardRelay,
    EntryConstraints {
        obfuscation_settings,
        ip_version,
        ..
    }: &EntryConstraints,
) -> ObfuscationVerdict {
    /// Whether the requested IP version (IPv4/IPv6) matches any of the given addresses.
    enum IpVersionMatch {
        Ok,
        /// No IP matches the request version, but some does match the _other_ version.
        Other,
        /// No IP matches any version, i.e. the list of IP addresses was empty.
        None,
    }
    fn any_ip_matches_version(
        requested_ip_version: &Constraint<IpVersion>,
        ip_list: impl IntoIterator<Item: Borrow<IpAddr>>,
    ) -> IpVersionMatch {
        let (has_ipv4, has_ipv6) = ip_list.into_iter().fold((false, false), |(v4, v6), addr| {
            (v4 || addr.borrow().is_ipv4(), v6 || addr.borrow().is_ipv6())
        });
        match requested_ip_version {
            Constraint::Any if has_ipv4 || has_ipv6 => IpVersionMatch::Ok,
            Constraint::Only(IpVersion::V4) if has_ipv4 => IpVersionMatch::Ok,
            Constraint::Only(IpVersion::V6) if has_ipv6 => IpVersionMatch::Ok,
            Constraint::Only(IpVersion::V4) if has_ipv6 => IpVersionMatch::Other,
            Constraint::Only(IpVersion::V6) if has_ipv4 => IpVersionMatch::Other,
            _ => IpVersionMatch::None,
        }
    }

    use ObfuscationVerdict::*;
    use mullvad_types::relay_constraints::SelectedObfuscation::*;
    match obfuscation_settings.selected_obfuscation {
        Shadowsocks => {
            // The relay may have IPs specifically meant for shadowsocks.
            // Use them if they match the requested IP version.
            match any_ip_matches_version(ip_version, &relay.endpoint().shadowsocks_extra_addr_in) {
                IpVersionMatch::Ok => AcceptObfuscationEndpoint,
                // Check if we can fall back to using the WireGuard endpoint instead.
                // A few port ranges on it are dedicated to shadowsocks. If a specific port
                // is requested it must lie within these ranges.
                _ if obfuscation_settings.shadowsocks.port.is_any_or(|port| {
                    shadowsocks_port_ranges
                        .iter()
                        .any(|range| range.contains(&port))
                }) =>
                {
                    AcceptWireguardEndpoint
                }
                // -- We cannot resolve the relay on any endpoint, so reject it --

                // Switching IP version would unblock the relay, so give that as the reject reason.
                // Note that the relay could also be unblocked by removing the port constraint
                // so that a normal WireGuard endpoint can be used IFF that endpoint
                // is available with the requested IP version. We cannot represent this, so we
                // opt to only inform the user about the IP version.
                IpVersionMatch::Other => Reject(Reason::IpVersion),
                // No extra addresses are available at all, the port must be changed
                // so that a Wireguard endpoint can be used. This endpoint must
                // then also be available with the requested IP version, which
                // is checked for outside this function.
                IpVersionMatch::None => Reject(Reason::Port),
            }
        }
        Quic => {
            // TODO: Refactor using `if-let guards` once 1.95 is stable.
            let Some(quic) = relay.endpoint().quic() else {
                // QUIC is disabled
                return Reject(Reason::Obfuscation);
            };
            match any_ip_matches_version(ip_version, quic.in_addr()) {
                IpVersionMatch::Ok => AcceptObfuscationEndpoint,
                // Switching IP version would unblock the relay.
                IpVersionMatch::Other => Reject(Reason::IpVersion),
                // The relay has quic but no IPv4 or IPv6 addresses to use it.
                // This scenario should be unreachable, but treat it as if obfuscation was
                // unavailable just in case.
                IpVersionMatch::None => Reject(Reason::Obfuscation),
            }
        }
        // LWO is only enabled on some relays
        Lwo if relay.endpoint().lwo => AcceptWireguardEndpoint,
        Lwo => Reject(Reason::Obfuscation),
        // Other relays are always valid
        // TODO:^ This might not be true. We might want to consider the selected port for
        // udp2tcp & wireguard port ..
        // Possible edge case that we have not implemented:
        // - User has set IPv6=only and anti-censorship=auto
        // - A relay doesn't have an IPv6 for its wg endpoint, but it does have an IPv6 extra shadowsocks addr.
        // In this scenario, we could conceivably allow the relay by enabling shadowsocks to resolve the IP constraint.
        // This would negatively affect the performance of the connection, so we have chosen to discard the relay for now.
        Off | Auto | WireguardPort | Udp2Tcp => AcceptWireguardEndpoint,
    }
}

/// A criteria is a function from a _single_ constraint and a relay to a [`Verdict`].
///
/// Multiple [`Criteria`] can be evaluated against a single relay at once by [`Criteria::eval`]. A
/// final verdict is then compiled. If applicable, all reject reasons are accumulated and presented
/// as a single [`Verdict::Reject`].
struct Criteria<'a, T> {
    // TODO:Store a &'static str with each Criteria, much like gotatun::Task. Makes for nicer
    // debugging/tracing of Criteria.
    f: Box<dyn Fn(&T) -> Verdict + 'a>,
}

impl<'a, T> Criteria<'a, T> {
    /// Create a new [`Criteria`].
    fn new(f: impl Fn(&T) -> Verdict + 'a) -> Self {
        Criteria { f: Box::new(f) }
    }
}

impl<'a> Criteria<'a, WireguardRelay> {
    /// Evaluate [`Criteria`] for a single [`Relay`].
    fn eval(&self, relay: &WireguardRelay) -> Verdict {
        (self.f)(relay)
    }

    /// Combine two [`Criteria`].
    ///
    /// This composition is biased towards the negative case, i.e. rejections always take
    /// precedence. If two rejecting criteria are composed, all of their reasons accumulate.
    fn and(self, other: Self) -> Self {
        Criteria::new(move |relay| self.eval(relay).and(other.eval(relay)))
    }

    /// Combine two [`Criteria`].
    ///
    /// This composition is biased towards the positive case, i.e. accepts always take
    /// precedence. If two rejecting criteria are composed, all of their reasons accumulate.
    fn or(self, other: Self) -> Self {
        Criteria::new(move |relay| self.eval(relay).or(other.eval(relay)))
    }

    /// Evaluate all criteria for a given relay, yield a final [`Verdict`].
    ///
    /// This function is biased towards [`Verdict::Accept`]. E.g. if `criterias` is emtpy, the
    /// scrutinized `relay` is accepted.
    fn fold(criterias: impl Iterator<Item = &'a Self>, relay: &WireguardRelay) -> Verdict {
        criterias
            .into_iter()
            .map(|criteria| criteria.eval(relay))
            .fold(Verdict::Accept, Verdict::and)
    }
}

/// If a relay is accepted or rejected. If it is rejected, all [reasons](Reason) for that judgment
/// is provided as well.
///
/// # Note
/// The associated relay is implied from the environment.
#[derive(Debug)]
enum Verdict {
    Accept,
    Reject(Vec<Reason>),
}

impl Verdict {
    /// Combine two [`Verdict`]s into one single verdict.
    ///
    /// This composition is biased towards the negative case, i.e. rejections always take
    /// precedence. If two rejecting verdicts are composed, all of their reasons accumulate.
    fn and(self, other: Verdict) -> Verdict {
        use Verdict::*;
        match (self, other) {
            (Accept, Accept) => Accept,
            (Accept, Reject(reasons)) | (Reject(reasons), Accept) => Reject(reasons),
            (Reject(left), Reject(right)) => Reject([left, right].concat()),
        }
    }

    /// Combine two [`Verdict`]s into one single verdict.
    ///
    /// This composition is biased towards the positive case, i.e. accepts always take
    /// precedence. If two rejecting verdicts are composed, all of their reasons accumulate.
    fn or(self, other: Verdict) -> Verdict {
        use Verdict::*;
        match (self, other) {
            (Reject(left), Reject(right)) => Reject([left, right].concat()),
            _ => Accept,
        }
    }

    /// Short-hand for creating a rejection for some reason.
    fn reject(reason: Reason) -> Verdict {
        Verdict::Reject(vec![reason])
    }
}

// Intended as an extension trait for `bool`.
trait VerdictExt {
    fn if_false(self, reason: Reason) -> Verdict;
    fn if_true(self, reason: Reason) -> Verdict;
}

impl VerdictExt for bool {
    /// Reject with `reason` if `self` is false.
    fn if_false(self, reason: Reason) -> Verdict {
        if self {
            Verdict::Accept
        } else {
            Verdict::reject(reason)
        }
    }

    /// Reject with `reason` if `self` is true.
    fn if_true(self, reason: Reason) -> Verdict {
        (!self).if_false(reason)
    }
}

// == End of new relay selector API. ==

fn apply_ip_availability(
    runtime_ip_availability: IpAvailability,
    user_query: &mut RelayQuery,
) -> Result<(), Error> {
    let ip_version = match runtime_ip_availability {
        IpAvailability::Ipv4 => Constraint::Only(IpVersion::V4),
        IpAvailability::Ipv6 => Constraint::Only(IpVersion::V6),
        IpAvailability::Ipv4AndIpv6 => Constraint::Any,
    };
    let wireguard_constraints = user_query
        .wireguard_constraints()
        .to_owned()
        .intersection(WireguardRelayQuery {
            ip_version,
            ..Default::default()
        })
        .ok_or_else(|| {
            // It is safe to call `unwrap` on `wireguard_constraints().ip_version` here
            // because this will only be called if intersection returns None
            // and the only way None can be returned is if both
            // ip_version and wireguard_constraints.ip_version are Constraint::Only and thus
            // guarantees that wireguard_constraints.ip_version is Constraint::Only
            let family = user_query.wireguard_constraints().ip_version.unwrap();
            Error::IpVersionUnavailable { family }
        })?;
    user_query.set_wireguard_constraints(wireguard_constraints);
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

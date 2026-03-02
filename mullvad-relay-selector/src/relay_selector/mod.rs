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
        LocationConstraint, ObfuscationSettings, Ownership, Providers, RelayConstraints,
        RelaySettings, WireguardConstraints,
    },
    relay_list::{Bridge, BridgeList, RelayList, WireguardRelay},
    settings::Settings,
    wireguard::{DaitaSettings, QuantumResistantState},
};
use std::ops::Deref;
use std::sync::{Arc, LazyLock, Mutex, RwLock};
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
    pub additional_constraints: AdditionalRelayConstraints,
    pub custom_lists: CustomListsSettings,
    // Wireguard specific data
    pub obfuscation_settings: ObfuscationSettings,
}

impl SelectorConfig {
    pub fn from_settings(settings: &Settings) -> Self {
        let additional_constraints = AdditionalRelayConstraints {
            wireguard: AdditionalWireguardConstraints {
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
            },
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
pub struct AdditionalRelayConstraints {
    pub wireguard: AdditionalWireguardConstraints,
}

/// Constraints to use when selecting WireGuard servers
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
    additional_preferences: &'a AdditionalRelayConstraints,
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
            additional_constraints: AdditionalRelayConstraints::default(),
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
            value.additional_preferences.wireguard.clone(),
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
                Self::get_relay_inner(&query, &relay_list, normal_config.custom_lists)
            }
        }
    }

    /// Returns a random relay and relay endpoint matching the current constraints corresponding to
    /// `retry_attempt` in one of the retry orders while considering
    /// [runtime_params][`RuntimeParameters`].
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
                    .filter_map(|query| {
                        Self::get_relay_inner(&query, &parsed_relays, custom_lists).ok()
                    })
                    .cycle() // If the above filters remove all relays, cycle will also return an empty iterator
                    .nth(retry_attempt);
                match maybe_relay {
                    Some(v) => Ok(v),
                    // If none of the queries in `retry_order` merged with `user_preferences` yield any relays,
                    // attempt to only consider the user's preferences.
                    None => Self::get_relay_inner(&user_query, &parsed_relays, custom_lists),
                }
            }
        }
    }

    /// "Execute" the given query, yielding a final set of relays and/or bridges which the VPN
    /// traffic shall be routed through.
    ///
    /// # Parameters
    /// - `query`: Constraints that filter the available relays, such as geographic location or
    ///   tunnel protocol.
    /// - `config`: Configuration settings that influence relay selection, including bridge state
    ///   and custom lists.
    /// - `parsed_relays`: The complete set of parsed relays available for selection.
    ///
    /// # Returns
    /// * A randomly selected relay that meets the specified constraints (and a random bridge/entry
    ///   relay if applicable). See [`GetRelay`] for more details.
    /// * An `Err` if no suitable relay is found
    /// * An `Err` if no suitable bridge is found
    fn get_relay_inner(
        query: &RelayQuery,
        parsed_relays: &RelayList,
        custom_lists: &CustomListsSettings,
    ) -> Result<GetRelay, Error> {
        Self::get_wireguard_relay_inner(query, custom_lists, parsed_relays)
    }

    /// Derive a valid relay configuration from `query`.
    ///
    /// # Note
    /// This function should *only* be called with a Wireguard query.
    /// It will panic if the tunnel type is not specified as Wireguard.
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

    /// As oppossed to the prior [`Self::get_relay_by_query`], this function is stateless with
    /// regards to any particular config / settings. <TODO: Document me more>
    //
    // # Algorithm
    // pseudo-code (implemented mostly in `Criteria`).
    //
    //
    // let criterias := [<is relay active?>, <is relay in expected location?>, ..]
    //
    // for each relay in relay list ..
    // let mut reject_reasons := []
    // for each criteria in criterias ..
    // if let Reject(reason) = critera.eval(relay) {
    //   reject_reasons.push(reason)
    // }
    // ..
    // if rejections_reasons.empty() {
    //   (relay, Accept),
    // } else {
    //   (relay, Reject(reject_reasons))
    // }
    pub fn partition_relays(&self, predicate: Predicate) -> RelayPartitions {
        let criteria = self.criteria(predicate);
        // The relay selection algorithm is embarrassingly parallel: https://en.wikipedia.org/wiki/Embarrassingly_parallel.
        // We may explore the entire search space (`relays` x `criteria`) without any synchronisation
        // between different branches.
        self.get_relays()
            .into_relays()
            .map(|relay| {
                let verdict = Criteria::fold(criteria.iter(), &relay);
                (relay, verdict)
            })
            // After this mapping, a single reduce is performed to partition the relays based on
            // their assigned verdict.
            .partition_map(|(relay, verdict)| match verdict {
                Verdict::Accept => Either::Left(relay),
                Verdict::Reject(rejected) => Either::Right((relay, rejected)),
            })
            .into()
    }

    /// Calculate the set of criteria each predicate will render for scrutinizing relays.
    fn criteria(&self, predicate: Predicate) -> Vec<Criteria<'_, WireguardRelay>> {
        match predicate {
            Predicate::Singlehop(constraints) => {
                let mut singlehop_criteria = self.singlehop_criteria(constraints.clone());
                let active =
                    Criteria::new(|relay: &WireguardRelay| relay.active.if_false(Reason::Inactive));
                let location = self.location_criteria(constraints.general);
                singlehop_criteria.extend([active, location]);
                singlehop_criteria
            }
            Predicate::Autohop(constraints) => {
                // This case is identical to `singlehop`, except that it does not generally care about obfuscation, DAITA, etc.
                // In those cases, the VPN traffic may be routed through an alternative entry relay.

                // If a specific exit is to be selected, it could occupy the only possible entry relay.
                // We may run `partition_relays` searching for the entry relay. If the result yields one
                // (and only one) specific relay, we know that it must be excluded from the list of
                // exit relays.
                let occupied = {
                    let mut constraints = constraints.clone();
                    constraints.general.location = Constraint::Any;
                    let entry_relay = self
                            // Compare with the equiv predicate for the `Predicate::Exit` case. Que
                            // interesante.
                            .partition_relays(Predicate::Singlehop(constraints))
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
                            Criteria::new(|_| Verdict::accept())
                        }
                    }
                };

                // Ugly hack for filters applying to both entry and exit, even if we're autohoping.
                let apply_entry_guards = {
                    let mut constraints = constraints.clone();
                    constraints.general.location = Constraint::Any;
                    self.singlehop_criteria(constraints)
                        .into_iter()
                        .reduce(Criteria::compose)
                        .unwrap()
                };

                vec![occupied, apply_entry_guards]
            }
            Predicate::Entry(MultihopConstraints { entry, exit }) => {
                // If an exit is already selected, it should be rejected as a possible entry relay.
                // To find out if a certain location is already selected as an exit relay, we may
                // run `partition_relays` searching for the exit relay. If the result yields one
                // (and only one) specific relay, we know that it must be excluded from the list of
                // entry relays.
                let occupied = {
                    let exit_relay = self
                            // Compare with the equiv predicate for the `Predicate::Exit` case. Que
                            // interesante.
                            .partition_relays(Predicate::Autohop(EntryConstraints { general: exit, ..Default::default()} ))
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
                            Criteria::new(|_| Verdict::accept())
                        }
                    }
                };

                // Except for the `occupied` condition, the remainder of the work is ~equiv
                // to `Predicate::Singlehop`.
                let mut criteria = self.singlehop_criteria(entry);
                criteria.extend([occupied]);
                criteria
            }
            Predicate::Exit(MultihopConstraints { entry, exit }) => {
                // If an entry is already selected, it should be rejected as a possible exit relay.
                // To find out if a certain location is already selected as an entry relay, we may
                // run `partition_relays` searching for the entry relay. If the result yields one
                // (and only one) specific relay, we know that it must be excluded from the list of
                // exit relays.
                let occupied = {
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
                            Criteria::new(|_| Verdict::accept())
                        }
                    }
                };

                // Here we *do not* have to consider any additional entry constraints, such as
                // obfuscation, DAITA, etc.
                let mut criteria = self.base_criteria(exit);
                criteria.extend([occupied]);
                criteria
            }
        }
    }

    /// TODO: Document me
    ///
    fn singlehop_criteria(
        &self,
        constraints: EntryConstraints,
    ) -> Vec<Criteria<'_, WireguardRelay>> {
        // Here we have to consider extra entry constraints, such as DAITA, obfuscation etc.
        let exit = constraints.general.clone();
        let filters = self.filter_criteria(exit);
        let obfuscation = self.obfuscation_criteria(constraints.clone());
        let daita = Criteria::new(move |relay| {
            let daita_on = constraints.daita.as_ref().map(|settings| settings.enabled);
            matcher::filter_on_daita(&daita_on, relay).if_false(Reason::Daita)
        });
        vec![filters, daita, obfuscation]
    }

    fn obfuscation_criteria(&self, constraints: EntryConstraints) -> Criteria<'_, WireguardRelay> {
        let EntryConstraints {
            obfuscation_settings,
            ip_version,
            ..
        } = constraints;
        Criteria::new(move |relay: &WireguardRelay| {
            match obfuscation_settings.as_ref() {
                Constraint::Any => true,
                Constraint::Only(settings) => {
                    matcher::filter_on_obfuscation_neo(relay, settings, ip_version.as_ref())
                }
            }
            .if_false(Reason::Obfuscation)
        })
    }

    fn location_criteria(&self, constraints: ExitConstraints) -> Criteria<'_, WireguardRelay> {
        let ExitConstraints { location, .. } = constraints;
        let custom_lists: CustomListsSettings = self.custom_lists();
        Criteria::new(move |relay| {
            let location = matcher::ResolvedLocationConstraint::from_constraint(
                location.as_ref(),
                &custom_lists,
            );
            matcher::filter_on_location(location.as_ref(), relay).if_false(Reason::Location)
        })
    }

    /// TODO: Document me
    /// * active
    /// * location
    /// * filters
    /// * providers
    fn base_criteria(&self, constraints: ExitConstraints) -> Vec<Criteria<'_, WireguardRelay>> {
        let ExitConstraints {
            location,
            providers,
            ownership,
        } = constraints;
        let custom_lists: CustomListsSettings = self.custom_lists();

        let active =
            Criteria::new(|relay: &WireguardRelay| relay.active.if_false(Reason::Inactive));

        let location = Criteria::new(move |relay| {
            let location = matcher::ResolvedLocationConstraint::from_constraint(
                location.as_ref(),
                &custom_lists,
            );
            matcher::filter_on_location(location.as_ref(), relay).if_false(Reason::Location)
        });
        // TODO: Use `filter_criteria`
        let ownership = Criteria::new(move |relay| {
            matcher::filter_on_ownership(ownership.as_ref(), relay).if_false(Reason::Ownership)
        });
        let providers = Criteria::new(move |relay| {
            matcher::filter_on_providers(providers.as_ref(), relay).if_false(Reason::Providers)
        });

        vec![active, location, ownership, providers]
    }

    /// All criteria for satisfying filter contraints.
    ///
    /// * ownership
    /// * providers
    fn filter_criteria(&self, constraints: ExitConstraints) -> Criteria<'_, WireguardRelay> {
        let ExitConstraints {
            providers,
            ownership,
            ..
        } = constraints;
        let ownership = Criteria::new(move |relay| {
            matcher::filter_on_ownership(ownership.as_ref(), relay).if_false(Reason::Ownership)
        });
        let providers = Criteria::new(move |relay| {
            matcher::filter_on_providers(providers.as_ref(), relay).if_false(Reason::Providers)
        });

        ownership.compose(providers)
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
    /// If the given criteria [`f`] evaulates to `false`, the second provided function `reason` is
    /// run to provide a single [`Reject`] reason. `reason` gets access to the failing relay, which
    /// means that `reason` may derivce additional information for why this particular relay was
    /// rejected.
    ///
    /// This is a short-hand for how most common [`Criteria`]s will be formulated, and it allows the
    /// caller to nicely separate the scrutinizing rejection logic from the logic extracting data to
    /// provide together with the final rejection. In the happy case this carries minimal additional
    /// runtime overhead compared to [`Criteria::new`], but upon a rejection two functions will run
    /// instead of one. For more fine-grained control over this behavior, prefer [`Criteria::new`].
    #[expect(unused)] // TODO: Use or remove.
    fn otherwise(
        f: impl Fn(&WireguardRelay) -> bool + 'a,
        reason: impl Fn(&WireguardRelay) -> Reason + 'a,
    ) -> Self {
        Criteria::new(move |relay| f(relay).if_false(reason(relay)))
    }

    /// Evaluate a single [`Criteria`] for a single [`Relay`].
    fn eval(&self, relay: &WireguardRelay) -> Verdict {
        (self.f)(relay)
    }

    /// Compose two [`Criteria`] into one.
    ///
    /// See [`Verdict::compose`].
    fn compose(self, other: Self) -> Self {
        Criteria::new(move |relay| {
            let verdict1 = self.eval(relay);
            let verdict2 = other.eval(relay);
            verdict1.compose(verdict2)
        })
    }

    /// Evaluate all criterias for a given relay, resulting in a single final verdict.
    ///
    /// This function is biased towards [`Verdict::Accept`]. E.g. if `criterias` is emtpy, the
    /// scrutinized `relay` is accepted.
    fn fold(criterias: impl Iterator<Item = &'a Self>, relay: &WireguardRelay) -> Verdict {
        criterias
            .into_iter()
            .map(|criteria| criteria.eval(relay))
            .fold(Verdict::Accept, Verdict::compose)
    }
}

/// If a relay is accepted or rejected .
///
/// # Note
/// The associated relay is implied from the environment.
#[derive(Debug)]
enum Verdict {
    Accept,
    Reject(Vec<Reason>),
}

#[expect(unused)]
impl Verdict {
    /// Compose two [`Verdict`]s into one single verdict.
    ///
    /// This composition is biased towards the negative case, i.e. rejections always take
    /// precedence. If two rejecting verdicts are composed, all of their reasons are composed as
    /// well.
    fn compose(self, other: Verdict) -> Verdict {
        use Verdict::*;
        match (self, other) {
            (Accept, Accept) => Accept,
            (Accept, Reject(reasons)) | (Reject(reasons), Accept) => Reject(reasons),
            (Reject(left), Reject(right)) => Reject([left, right].concat()),
        }
    }

    fn reject(reason: Reason) -> Verdict {
        Verdict::Reject(vec![reason])
    }

    fn accept() -> Verdict {
        Verdict::Accept
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
            Verdict::Reject(vec![reason])
        }
    }

    /// Reject with `reason` if `self` is true.
    fn if_true(self, reason: Reason) -> Verdict {
        (!self).if_false(reason)
    }
}

impl From<(Vec<WireguardRelay>, Vec<(WireguardRelay, Vec<Reason>)>)> for RelayPartitions {
    /// Map the result of [`Itertools::partition_map`] to [`RelayPartitions`].
    fn from(partitions: (Vec<WireguardRelay>, Vec<(WireguardRelay, Vec<Reason>)>)) -> Self {
        Self {
            matches: partitions.0,
            discards: partitions.1,
        }
    }
}

/// Specify the constraints that should be applied when selecting relays,
/// along with a context that may affect the selection behavior.
#[derive(Debug, Clone)]
pub enum Predicate {
    Singlehop(EntryConstraints),
    Autohop(EntryConstraints),
    // Multihop-only
    Entry(MultihopConstraints),
    Exit(MultihopConstraints),
}

// TODO: Document
// TODO: Should all fields be pub??
#[derive(Debug, Default, Clone)]
pub struct EntryConstraints {
    pub general: ExitConstraints,
    // Entry-specific constraints.
    pub obfuscation_settings: Constraint<ObfuscationSettings>,
    pub daita: Constraint<DaitaSettings>,
    pub ip_version: Constraint<IpVersion>,
}

// TODO: Document
// TODO: Should all fields be pub??
#[derive(Debug, Default, Clone)]
pub struct ExitConstraints {
    pub location: Constraint<LocationConstraint>,
    pub providers: Constraint<Providers>,
    pub ownership: Constraint<Ownership>,
}

// TODO: Document
// TODO: Should all fields be pub??
#[derive(Debug, Default, Clone)]
pub struct MultihopConstraints {
    pub entry: EntryConstraints,
    pub exit: ExitConstraints,
}

// TODO: Work with references instead of copies?
#[derive(Debug, Default, PartialEq)]
pub struct RelayPartitions {
    pub matches: Vec<WireguardRelay>,
    pub discards: Vec<(WireguardRelay, Vec<Reason>)>,
}

impl RelayPartitions {
    /// Collect all unique reasons why a relay was filtered out.
    pub fn unique_reasons(self) -> Vec<Vec<Reason>> {
        self.discards
            .into_iter()
            .map(|(_relay, reasons)| reasons)
            .unique()
            .collect()
    }
}

/// All possible reasons why a relay was filtered out for a particular query.
//
// TODO: Sort all variants in alphanumeric ordering.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Reason {
    /// TODO: Document
    Inactive,
    /// TODO: Document
    Location,
    /// TODO: Document
    Providers,
    /// TODO: Document
    Ownership,
    /// TODO: Document
    IpVersion,
    /// TODO: Document
    Daita,
    /// TODO: Document
    Obfuscation,
    /// TODO: Document
    Port,
    /// TODO: Document
    /// Conflict with other hop.
    // TODO: Rename to `Occupied`?
    Conflict,
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

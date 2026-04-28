//! The implementation of the relay selector.

pub mod detailer;
pub mod endpoint_set;
mod filter;
mod helpers;
pub mod query;
pub mod relays;

use relays::{Multihop, Singlehop, WireguardConfig};

use crate::{
    detailer::wireguard_endpoint,
    error::Error,
    query::{Hops, RelayQuery},
};

pub use mullvad_types::relay_list::Relay;
use mullvad_types::relay_selector::{EntrySpecificConstraints, MultihopConstraints};
use mullvad_types::{
    constraints::Constraint,
    custom_list::CustomListsSettings,
    endpoint::MullvadEndpoint,
    location::Coordinates,
    relay_constraints::RelaySettings,
    relay_list::{Bridge, BridgeList, RelayList, WireguardRelay},
    settings::Settings,
};
use std::ops::Deref;
use std::{
    collections::HashMap,
    sync::{Arc, LazyLock, Mutex, RwLock},
};
use talpid_types::net::{IpAvailability, IpVersion, obfuscation::Obfuscators, proxy::Shadowsocks};

/// [`RETRY_ORDER`] defines an ordered set of entry-relay parameters which the relay selector
/// should prioritize on successive connection attempts. Note that these will *never* override user
/// preferences. See [the documentation on `RelayQuery`][RelayQuery] for further details.
///
/// Each entry is an [`EntrySpecificConstraints`] that specifies only the axes that vary between
/// retry attempts (`ip_version` and `obfuscation`). All other fields are left as
/// `Constraint::Any` so that intersecting with the user's entry-specific constraints preserves
/// them. The user's hop count, exit constraints, allowed_ips, and quantum_resistant settings
/// are passed through unchanged when merging with the user query via
/// [`RelayQuery::merge_retry`].
///
/// This list should be kept in sync with the expected behavior defined in `docs/relay-selector.md`
pub static RETRY_ORDER: LazyLock<Vec<EntrySpecificConstraints>> = LazyLock::new(|| {
    vec![
        // 1: any wireguard relay
        EntrySpecificConstraints::default(),
        // 2: prefer IPv6
        EntrySpecificConstraints::default().ip_version(IpVersion::V6),
        // 3: shadowsocks
        EntrySpecificConstraints::shadowsocks(),
        // 4: quic
        EntrySpecificConstraints::quic(),
        // 5: udp2tcp
        EntrySpecificConstraints::udp2tcp(),
        // 6: udp2tcp + IPv6
        EntrySpecificConstraints::udp2tcp().ip_version(IpVersion::V6),
        // 7: lwo
        EntrySpecificConstraints::lwo(),
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
            .map(|relay| {
                let set = endpoint_set::RelayEndpointSet::new(relay, &list.wireguard)
                    .expect("Relay list has no WireGuard port ranges");
                (relay.hostname.clone(), set)
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
        Config {
            query: RelayQuery::from(settings),
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
            RelaySettings::Normal(_) => Ok(RelayQuery::from(&value)),
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
        self.bridge_list(get_proxy_settings)
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
        retry_order: &[EntrySpecificConstraints],
        runtime_ip_availability: IpAvailability,
    ) -> Result<GetRelay, Error> {
        let mut user_query = self.config.lock().unwrap().query.clone();

        // Runtime parameters may shrink the set of usable IP versions — apply that *before*
        // merging with retry_order so an IPv6-only retry attempt is correctly rejected when only
        // IPv4 is available.
        user_query.apply_ip_availability(runtime_ip_availability)?;
        log::trace!("Merging user preferences {user_query:?} with default retry strategy");

        // Select a relay using the user's preferences merged with the nth compatible retry entry,
        // looping back to the start if necessary.
        let maybe_relay = retry_order
            .iter()
            .filter_map(|retry| user_query.clone().merge_retry(retry.clone()))
            .filter_map(|query| self.get_relay_by_query(query).ok())
            .cycle()
            .nth(retry_attempt);

        match maybe_relay {
            Some(v) => Ok(v),
            // If no retry merged with `user_query` yields a relay, fall back to the user's
            // preferences alone.
            None => self.get_relay_by_query(user_query),
        }
    }

    /// Returns random relay and relay endpoint matching `query`.
    /// Note that this does not take custom config into consideration.
    pub fn get_relay_by_query(&self, query: RelayQuery) -> Result<GetRelay, Error> {
        // Hold a single read lock for the whole call so the relay we choose during
        // partitioning is the same one we look up in `endpoint_sets` afterwards.
        let annotated = self.relays.read().unwrap();
        let custom_lists = self.custom_lists();

        let inner = select_wireguard_relay(&annotated, &custom_lists, &query)?;

        let entry = match &inner {
            WireguardConfig::Singlehop { exit } => exit,
            WireguardConfig::Multihop { entry, .. } => entry,
        };

        let endpoint_set = annotated
            .endpoint_set_for(entry)
            .ok_or_else(|| Error::NoRelay(Box::new(query.clone())))?;

        let entry_specific = query.entry_specific();
        let (wg_addr, obfuscator) = endpoint_set
            .get_wireguard_obfuscator(&entry_specific.obfuscation, entry_specific.ip_version)?;

        let endpoint = wireguard_endpoint(
            &query.allowed_ips,
            &annotated.inner.wireguard,
            &inner,
            wg_addr,
        );

        Ok(GetRelay {
            endpoint,
            obfuscator,
            inner,
        })
    }
}

/// Select relay(s) matching the constraints, handling singlehop, autohop, and multihop routing.
fn select_wireguard_relay(
    relays: &AnnotatedRelayList,
    custom_lists: &CustomListsSettings,
    query: &RelayQuery,
) -> Result<WireguardConfig, Error> {
    match &query.hops {
        Hops::Single(constraints) => {
            let partitions = filter::partition_entry(relays, constraints, custom_lists);
            match helpers::pick_random_relay(&partitions.matches) {
                Some(exit) => Ok(WireguardConfig::from(Singlehop::new(exit.clone()))),
                None => Err(Error::NoRelay(Box::new(query.clone()))),
            }
        }
        Hops::Auto(constraints) => {
            let autohop = filter::partition_autohop(relays, constraints.clone(), custom_lists);
            // Attempt to pick a single relay that matches all constraints
            if let Some(exit) = helpers::pick_random_relay(&autohop.singlehop.matches) {
                return Ok(WireguardConfig::from(Singlehop::new(exit.clone())));
            }
            // Otherwise fall through to multihop using the pre-computed partition.
            let multihop_constraints = constraints.clone().into_autohop();
            select_from_multihop_partitions(autohop.multihop, multihop_constraints)
        }
        Hops::Multi(constraints) => {
            let partitions = filter::partition_multihop(relays, constraints, custom_lists);
            select_from_multihop_partitions(partitions, constraints.clone())
        }
    }
}

/// Select separate entry and exit relays for a multihop configuration.
///
/// If the entry location constraint is [`Constraint::Any`] (autohop), the entry relay
/// is chosen globally and biased towards the geographically closest relay to the exit.
/// Otherwise, entry and exit are picked randomly within their respective constraints.
fn select_from_multihop_partitions(
    partitions: filter::MultiHopPartitions,
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
    let endpoint =
        detailer::bridge_endpoint(&bridge_list.bridge_endpoint, &bridge).ok_or(Error::NoBridge)?;
    Ok((endpoint, bridge))
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

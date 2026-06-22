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
use mullvad_types::relay_selector::MultihopConstraints;
use mullvad_types::{
    constraints::Constraint,
    endpoint::MullvadEndpoint,
    location::Coordinates,
    relay_list::{Bridge, BridgeList, RelayList, WireguardRelay},
};
use std::ops::Deref;
use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};
use talpid_types::net::{obfuscation::Obfuscators, proxy::Shadowsocks};

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
    pub fn new(list: RelayList) -> Self {
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
    // Relays are updated very infrequently, but might conceivably be accessed by multiple readers at
    // the same time.
    relays: Arc<RwLock<AnnotatedRelayList>>,
    bridges: Arc<RwLock<BridgeList>>,
}

/// The return type of [`RelaySelector::get_relay_by_query`].
#[derive(Clone, Debug)]
pub struct GetRelay {
    pub endpoint: MullvadEndpoint,
    pub obfuscator: Option<Obfuscators>,
    pub inner: WireguardConfig,
}

impl RelaySelector {
    pub fn new(relays: RelayList, bridges: BridgeList) -> Self {
        RelaySelector {
            relays: Arc::new(RwLock::new(AnnotatedRelayList::new(relays))),
            bridges: Arc::new(RwLock::new(bridges)),
        }
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

    /// Returns random relay and relay endpoint matching `query`.
    /// Note that this does not take custom config into consideration.
    pub fn get_relay_by_query(&self, query: RelayQuery) -> Result<GetRelay, Error> {
        // Hold a single read lock for the whole call so the relay we choose during
        // partitioning is the same one we look up in `endpoint_sets` afterwards.
        let annotated = self.relays.read().unwrap();

        let inner = select_wireguard_relay(&annotated, &query)?;

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
            query.allowed_ips.as_ref(),
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
    query: &RelayQuery,
) -> Result<WireguardConfig, Error> {
    match &query.hops {
        Hops::Single(constraints) => {
            let partitions = filter::partition_entry(relays, constraints);
            match helpers::pick_random_relay(&partitions.matches) {
                Some(exit) => Ok(WireguardConfig::from(Singlehop::new(exit.clone()))),
                None => Err(Error::NoRelay(Box::new(query.clone()))),
            }
        }
        Hops::Auto(constraints) => {
            let autohop = filter::partition_autohop(relays, constraints.clone());
            // Attempt to pick a single relay that matches all constraints
            if let Some(exit) = helpers::pick_random_relay(&autohop.singlehop.matches) {
                return Ok(WireguardConfig::from(Singlehop::new(exit.clone())));
            }
            // Otherwise fall through to multihop using the pre-computed partition.
            let multihop_constraints = constraints.clone().into_autohop();
            select_from_multihop_partitions(autohop.multihop, multihop_constraints)
        }
        Hops::Multi(constraints) => {
            let partitions = filter::partition_multihop(relays, constraints);
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

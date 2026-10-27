//! The implementation of the relay selector.

pub mod detailer;
pub mod endpoint_set;
mod filter;
mod helpers;
pub mod query;
pub mod relays;

use relays::{Multihop, Singlehop, WireguardConfig};
use vec1::Vec1;

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
use std::{
    collections::HashMap,
    net::IpAddr,
    ops::Deref,
    sync::{Arc, RwLock},
};
use talpid_types::net::{
    obfuscation::{ObfuscatorConfig, Obfuscators, TransportConfig},
    proxy::Shadowsocks,
};

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

/// Remove the obfuscation methods that the selected relay offers from the query.
fn remove_supported_obfuscations(mut query: RelayQuery, relay: &GetRelay) -> RelayQuery {
    let obfuscation = &mut query.entry_specific_mut().obfuscation;
    for config in &*relay.obfuscator {
        match config {
            TransportConfig::Direct(_) => obfuscation.direct = None,
            TransportConfig::Obfuscation(config) => match config {
                ObfuscatorConfig::Udp2Tcp { .. } => obfuscation.udp2tcp = None,
                ObfuscatorConfig::Shadowsocks { .. } => obfuscation.shadowsocks = None,
                ObfuscatorConfig::Quic { .. } => obfuscation.quic = false,
                ObfuscatorConfig::Lwo { .. } => obfuscation.lwo = None,
            },
        }
    }
    query
}

/// The return type of [`RelaySelector::get_relay_by_query`].
#[derive(Clone, Debug)]
pub struct GetRelay {
    pub endpoint: MullvadEndpoint,
    pub obfuscator: Obfuscators,
    pub inner: WireguardConfig,
    pub server_override: bool,
}

/// Iterator of relays to use for connection attempts. Consumed in FIFO order.
///
/// When empty, it is refilled from [RelaySelector::get_relays_with_automatic_anti_censorship].
pub type RelayQueue = std::vec::IntoIter<GetRelay>;

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

    /// Returns inner [`RelayList`]. The cities in the object returned does not have any
    /// relays in them.
    pub fn get_relay_list(&self) -> RelayList {
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
    pub fn get_relay_by_query(&self, query: &RelayQuery) -> Result<GetRelay, Error> {
        // Hold a single read lock for the whole call so the relay we choose during
        // partitioning is the same one we look up in `endpoint_sets` afterwards.
        let annotated = self.relays.read().unwrap();

        let inner = select_wireguard_relay(&annotated, query)?;

        let entry = match &inner {
            WireguardConfig::Singlehop { exit } => exit,
            WireguardConfig::Multihop { entry, .. } => entry,
        };

        let endpoint_set = annotated
            .endpoint_set_for(entry)
            .ok_or_else(|| Error::NoRelay(Box::new(query.clone())))?;

        let entry_specific = query.entry_specific();

        let obfuscator = endpoint_set
            .get_wireguard_obfuscator(&entry_specific.obfuscation, entry_specific.ip_version)
            .ok_or_else(|| Error::NoRelay(Box::new(query.clone())))?;

        let server_override = has_overridden_address(entry, &obfuscator);

        // For now the peer config must have exactly one WG endpoint.
        // TODO: support having zero, one or two wg endpoints in the obfuscator list,
        // i.e. when multiplexing the direct IPv4 and IPv6 WireGuard connections.
        let wg_addr = endpoint_set.wireguard_endpoint_address(&obfuscator);

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
            server_override,
        })
    }

    /// Get the list of relays that should be connected to for a given query.
    ///
    /// If anti-censorship is disabled or set to a specific method, this will return a single relay.
    ///
    /// If automatic anti-censorship is enabled, the returned set of relays will
    /// together cover all available anti-censorship methods. As many methods as possible
    /// will be multiplexed on a given relay, without repeating methods on consecutive ones.
    /// Hence, all methods will be used once across the queue, if allowed by other constraints.
    ///
    /// See `docs/relay-selector.md#automatic-anti-censorship-selection`.
    pub fn get_relays_with_automatic_anti_censorship(
        &self,
        mut user_query: RelayQuery,
    ) -> Result<(GetRelay, RelayQueue), Error> {
        let mut relays = vec![];

        // First, we make a WireGuard-only attempt before using any obfuscations
        if let Some(wg_query) = user_query.clone().wireguard_only()
            && let Ok(wg_relay) = self.get_relay_by_query(&wg_query)
        {
            relays.push(wg_relay);
        }

        // Then, select a relay with multiplexed anti-censorship methods.
        loop {
            match self.get_relay_by_query(&user_query) {
                Ok(relay) => {
                    // All methods in the ObfuscationConstraints that are supported by the relay
                    // will be used by the multiplexer. The next relay is selected with only the
                    // remaining methods, until all methods have been used.
                    let new = remove_supported_obfuscations(user_query.clone(), &relay);
                    debug_assert_ne!(
                        user_query, new,
                        "Relay must have some obfuscation that we can remove"
                    );
                    user_query = new;
                    relays.push(relay);
                }
                Err(err) => {
                    let (relay, queue) = Vec1::try_from_vec(relays)
                        .map_err(|_| err)?
                        .split_off_first();
                    return Ok((relay, queue.into_iter()));
                }
            }
        }
    }
}

fn has_overridden_address(entry: &WireguardRelay, obfuscator: &Obfuscators) -> bool {
    obfuscator.0.iter().any(|t| {
        let ip = t.endpoint().address.ip();
        let overridden_v4 = entry.overridden_ipv4 && ip == IpAddr::V4(entry.ipv4_addr_in);
        let overridden_v6 =
            entry.overridden_ipv6 && entry.ipv6_addr_in.is_some_and(|v6| ip == IpAddr::V6(v6));
        overridden_v4 || overridden_v6
    })
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

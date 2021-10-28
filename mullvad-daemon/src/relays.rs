//! When changing relay selection, please verify if `docs/relay-selector.md` needs to be
//! updated as well.

use chrono::{DateTime, Local};
use futures::{
    channel::mpsc,
    future::{Fuse, FusedFuture},
    FutureExt, SinkExt, StreamExt,
};
use ipnetwork::IpNetwork;
use log::{debug, error, info, warn};
use mullvad_rpc::{availability::ApiAvailabilityHandle, rest::MullvadRestHandle, RelayListProxy};
use mullvad_types::{
    endpoint::MullvadEndpoint,
    location::Location,
    relay_constraints::{
        BridgeState, Constraint, InternalBridgeConstraints, LocationConstraint, Match,
        OpenVpnConstraints, Providers, RelayConstraints, Set, TransportPort, WireguardConstraints,
    },
    relay_list::{OpenVpnEndpointData, Relay, RelayList, RelayTunnels, WireguardEndpointData},
};
use parking_lot::Mutex;
use rand::{self, rngs::ThreadRng, seq::SliceRandom, Rng};
use std::{
    future::Future,
    io,
    net::{IpAddr, SocketAddr},
    path::{Path, PathBuf},
    sync::Arc,
    time::{self, Duration, Instant, SystemTime},
};
use talpid_core::future_retry::{retry_future, ExponentialBackoff, Jittered};
use talpid_types::{
    net::{
        all_of_the_internet, openvpn::ProxySettings, wireguard, IpVersion, TransportProtocol,
        TunnelType,
    },
    ErrorExt,
};
use tokio::fs::File;

const DATE_TIME_FORMAT_STR: &str = "%Y-%m-%d %H:%M:%S%.3f";
const RELAYS_FILENAME: &str = "relays.json";
/// How often the updater should wake up to check the cache of the in-memory cache of relays.
/// This check is very cheap. The only reason to not have it very often is because if downloading
/// constantly fails it will try very often and fill the logs etc.
const UPDATE_CHECK_INTERVAL: Duration = Duration::from_secs(60 * 15);
/// How old the cached relays need to be to trigger an update
const UPDATE_INTERVAL: Duration = Duration::from_secs(60 * 60);

const EXPONENTIAL_BACKOFF_INITIAL: Duration = Duration::from_secs(16);
const EXPONENTIAL_BACKOFF_FACTOR: u32 = 8;

const DEFAULT_WIREGUARD_PORT: u16 = 51820;
const WIREGUARD_EXIT_CONSTRAINTS: WireguardConstraints = WireguardConstraints {
    port: Constraint::Only(TransportPort {
        protocol: TransportProtocol::Udp,
        port: Constraint::Only(DEFAULT_WIREGUARD_PORT),
    }),
    ip_version: Constraint::Only(IpVersion::V4),
    entry_location: None,
};
const WIREGUARD_TCP_PORTS: [(u16, u16); 3] = [(80, 80), (443, 443), (5001, 5001)];


#[derive(err_derive::Error, Debug)]
#[error(no_from)]
pub enum Error {
    #[error(display = "Failed to open relay cache file")]
    OpenRelayCache(#[error(source)] io::Error),

    #[error(display = "Failed to write relay cache file to disk")]
    WriteRelayCache(#[error(source)] io::Error),

    #[error(display = "No relays matching current constraints")]
    NoRelay,

    #[error(display = "Failure in serialization of the relay list")]
    Serialize(#[error(source)] serde_json::Error),

    #[error(display = "Downloader already shut down")]
    DownloaderShutDown,
}

struct ParsedRelays {
    last_updated: SystemTime,
    locations: RelayList,
    relays: Vec<Relay>,
}

impl ParsedRelays {
    pub fn empty() -> Self {
        ParsedRelays {
            last_updated: time::UNIX_EPOCH,
            locations: RelayList::empty(),
            relays: Vec::new(),
        }
    }

    pub fn from_relay_list(relay_list: RelayList, last_updated: SystemTime) -> Self {
        let mut relays = Vec::new();
        for country in &relay_list.countries {
            let country_name = country.name.clone();
            let country_code = country.code.clone();
            for city in &country.cities {
                let city_name = city.name.clone();
                let city_code = city.code.clone();
                let latitude = city.latitude;
                let longitude = city.longitude;
                for relay in &city.relays {
                    let mut relay_with_location = relay.clone();
                    relay_with_location.location = Some(Location {
                        country: country_name.clone(),
                        country_code: country_code.clone(),
                        city: city_name.clone(),
                        city_code: city_code.clone(),
                        latitude,
                        longitude,
                    });

                    for wg_tunnel in &relay.tunnels.wireguard {
                        relay_with_location
                            .tunnels
                            .wireguard
                            .push(WireguardEndpointData {
                                protocol: TransportProtocol::Tcp,
                                port_ranges: WIREGUARD_TCP_PORTS.to_vec(),
                                ..wg_tunnel.clone()
                            });
                    }

                    relays.push(relay_with_location);
                }
            }
        }
        ParsedRelays {
            last_updated,
            locations: relay_list,
            relays,
        }
    }

    pub fn from_file(path: impl AsRef<Path>) -> Result<Self, Error> {
        debug!("Reading relays from {}", path.as_ref().display());
        let (last_modified, file) =
            Self::open_file(path.as_ref()).map_err(Error::OpenRelayCache)?;
        let relay_list =
            serde_json::from_reader(io::BufReader::new(file)).map_err(Error::Serialize)?;

        Ok(Self::from_relay_list(relay_list, last_modified))
    }

    fn open_file(path: &Path) -> io::Result<(SystemTime, std::fs::File)> {
        let file = std::fs::File::open(path)?;
        let last_modified = file.metadata()?.modified()?;
        Ok((last_modified, file))
    }

    pub fn last_updated(&self) -> SystemTime {
        self.last_updated
    }

    pub fn locations(&self) -> &RelayList {
        &self.locations
    }

    pub fn relays(&self) -> &Vec<Relay> {
        &self.relays
    }

    pub fn tag(&self) -> Option<&str> {
        self.locations.etag.as_deref()
    }
}

pub struct RelaySelector {
    parsed_relays: Arc<Mutex<ParsedRelays>>,
    rng: ThreadRng,
    updater: Option<RelayListUpdaterHandle>,
}

impl RelaySelector {
    /// Returns a new `RelaySelector` backed by relays cached on disk. Use the `update` method
    /// to refresh the relay list from the internet.
    pub fn new(
        rpc_handle: MullvadRestHandle,
        on_update: impl Fn(&RelayList) + Send + 'static,
        resource_dir: &Path,
        cache_dir: &Path,
        api_availability: ApiAvailabilityHandle,
    ) -> Self {
        let cache_path = cache_dir.join(RELAYS_FILENAME);
        let resource_path = resource_dir.join(RELAYS_FILENAME);
        let unsynchronized_parsed_relays = Self::read_relays_from_disk(&cache_path, &resource_path)
            .unwrap_or_else(|error| {
                error!(
                    "{}",
                    error.display_chain_with_msg("Unable to load cached relays")
                );
                ParsedRelays::empty()
            });
        info!(
            "Initialized with {} cached relays from {}",
            unsynchronized_parsed_relays.relays().len(),
            DateTime::<Local>::from(unsynchronized_parsed_relays.last_updated())
                .format(DATE_TIME_FORMAT_STR)
        );
        let parsed_relays = Arc::new(Mutex::new(unsynchronized_parsed_relays));

        let updater = RelayListUpdater::new(
            rpc_handle,
            cache_path,
            parsed_relays.clone(),
            Box::new(on_update),
            api_availability,
        );


        RelaySelector {
            parsed_relays,
            rng: rand::thread_rng(),
            updater: Some(updater),
        }
    }

    /// Download the newest relay list.
    pub fn update(&mut self) -> impl Future<Output = ()> {
        let mut updater = self.updater.as_ref().unwrap().clone();
        async move {
            updater
                .update_relay_list()
                .await
                .expect("Relay list updated thread has stopped unexpectedly");
        }
    }

    pub fn updater_handle(&self) -> RelayListUpdaterHandle {
        self.updater.as_ref().unwrap().clone()
    }

    /// Returns all countries and cities. The cities in the object returned does not have any
    /// relays in them.
    pub fn get_locations(&mut self) -> RelayList {
        self.parsed_relays.lock().locations().clone()
    }

    /// Returns a random relay and relay endpoint matching the given constraints and with
    /// preferences applied.
    pub fn get_tunnel_endpoint(
        &mut self,
        relay_constraints: &RelayConstraints,
        bridge_state: BridgeState,
        retry_attempt: u32,
        wg_key_exists: bool,
    ) -> Result<(Relay, MullvadEndpoint), Error> {
        let mut exit_relay_constraints = relay_constraints.clone();
        let wg_entry_is_subset = if let Some(entry_location) =
            exit_relay_constraints.wireguard_constraints.entry_location
        {
            let is_subset = entry_location.is_subset(&exit_relay_constraints.location);
            exit_relay_constraints.wireguard_constraints = WireguardConstraints {
                entry_location: Some(entry_location),
                ..WIREGUARD_EXIT_CONSTRAINTS
            };
            is_subset
        } else {
            false
        };

        let entry_endpoint = if wg_entry_is_subset
            && relay_constraints
                .wireguard_constraints
                .entry_location
                .is_some()
        {
            self.select_entry_endpoint(None, &relay_constraints, retry_attempt)
        } else {
            None
        };

        let (exit_relay, mut endpoint) = self.get_tunnel_exit_endpoint(
            &exit_relay_constraints,
            bridge_state,
            retry_attempt,
            wg_key_exists,
            entry_endpoint.as_ref().and_then(|(_relay, endpoint)| {
                if let MullvadEndpoint::Wireguard { peer, .. } = &endpoint {
                    Some(peer)
                } else {
                    None
                }
            }),
        )?;

        let mut entry_endpoint = entry_endpoint.or_else(|| {
            if !wg_entry_is_subset
                && relay_constraints
                    .wireguard_constraints
                    .entry_location
                    .is_some()
            {
                if let MullvadEndpoint::Wireguard { peer, .. } = &endpoint {
                    self.select_entry_endpoint(Some(peer), &relay_constraints, retry_attempt)
                } else {
                    None
                }
            } else {
                None
            }
        });

        if let MullvadEndpoint::Wireguard { peer, .. } = &mut endpoint {
            if let Some((entry_relay, mut entry_endpoint)) = entry_endpoint.take() {
                self.set_entry_peers(peer, &mut entry_endpoint);
                let addr_in = entry_endpoint.to_endpoint().address.ip();
                info!(
                    "Selected entry relay {} at {}",
                    entry_relay.hostname, addr_in
                );
                return Ok((exit_relay, entry_endpoint));
            } else if relay_constraints
                .wireguard_constraints
                .entry_location
                .is_some()
            {
                return Err(Error::NoRelay);
            }
        }

        Ok((exit_relay, endpoint))
    }

    fn get_tunnel_exit_endpoint(
        &mut self,
        relay_constraints: &RelayConstraints,
        bridge_state: BridgeState,
        retry_attempt: u32,
        wg_key_exists: bool,
        wg_entry_peer: Option<&wireguard::PeerConfig>,
    ) -> Result<(Relay, MullvadEndpoint), Error> {
        let preferred_constraints = self.preferred_constraints(
            &relay_constraints,
            bridge_state,
            retry_attempt,
            wg_key_exists,
        );
        if let Some((relay, endpoint)) =
            self.get_tunnel_endpoint_internal(&preferred_constraints, wg_entry_peer)
        {
            debug!(
                "Relay matched on highest preference for retry attempt {}",
                retry_attempt
            );
            Ok((relay, endpoint))
        } else if let Some((relay, endpoint)) =
            self.get_tunnel_endpoint_internal(&relay_constraints, wg_entry_peer)
        {
            debug!(
                "Relay matched on second preference for retry attempt {}",
                retry_attempt
            );
            Ok((relay, endpoint))
        } else {
            warn!("No relays matching {}", &relay_constraints);
            Err(Error::NoRelay)
        }
    }

    fn preferred_constraints(
        &self,
        original_constraints: &RelayConstraints,
        bridge_state: BridgeState,
        retry_attempt: u32,
        wg_key_exists: bool,
    ) -> RelayConstraints {
        let (preferred_port, preferred_protocol, preferred_tunnel) =
            if bridge_state != BridgeState::On {
                self.preferred_tunnel_constraints(
                    retry_attempt,
                    &original_constraints.location,
                    &original_constraints.providers,
                    wg_key_exists,
                )
            } else {
                (Constraint::Any, TransportProtocol::Tcp, TunnelType::OpenVpn)
            };


        let mut relay_constraints = original_constraints.clone();
        relay_constraints.openvpn_constraints = Default::default();

        // Highest priority preference. Where we prefer OpenVPN using UDP. But without changing
        // any constraints that are explicitly specified.
        match original_constraints.tunnel_protocol {
            // If no tunnel protocol is selected, use preferred constraints
            Constraint::Any => {
                if original_constraints.openvpn_constraints.port.is_any()
                    || bridge_state == BridgeState::On
                {
                    relay_constraints.openvpn_constraints = OpenVpnConstraints {
                        port: Constraint::Only(TransportPort {
                            protocol: preferred_protocol,
                            port: preferred_port,
                        }),
                    };
                } else {
                    relay_constraints.openvpn_constraints =
                        original_constraints.openvpn_constraints;
                }

                if relay_constraints.wireguard_constraints.port.is_any() {
                    relay_constraints.wireguard_constraints.port =
                        Constraint::Only(TransportPort {
                            protocol: preferred_protocol,
                            port: preferred_port,
                        });
                }

                relay_constraints.tunnel_protocol = Constraint::Only(preferred_tunnel);
            }
            Constraint::Only(TunnelType::OpenVpn) => {
                let openvpn_constraints = &mut relay_constraints.openvpn_constraints;
                *openvpn_constraints = original_constraints.openvpn_constraints;
                if bridge_state == BridgeState::On && openvpn_constraints.port.is_any() {
                    // FIXME: This is temporary while talpid-core only supports TCP proxies
                    openvpn_constraints.port = Constraint::Only(TransportPort {
                        protocol: TransportProtocol::Tcp,
                        port: Constraint::Any,
                    });
                } else if openvpn_constraints.port.is_any() {
                    let (preferred_port, preferred_protocol) =
                        Self::preferred_openvpn_constraints(retry_attempt);
                    openvpn_constraints.port = Constraint::Only(TransportPort {
                        protocol: preferred_protocol,
                        port: preferred_port,
                    });
                }
            }
            Constraint::Only(TunnelType::Wireguard) => {
                relay_constraints.wireguard_constraints =
                    original_constraints.wireguard_constraints.clone();
                // This ensures that if after the first 2 failed attempts the daemon does not
                // connect, then afterwards 2 of each 4 successive attempts will try to connect
                // on port 53.
                if retry_attempt % 4 > 1 && relay_constraints.wireguard_constraints.port.is_any() {
                    relay_constraints.wireguard_constraints.port =
                        Constraint::Only(TransportPort {
                            protocol: TransportProtocol::Udp,
                            port: Constraint::Only(53),
                        });
                }
            }
        }

        relay_constraints
    }

    fn select_entry_endpoint(
        &mut self,
        exit_peer: Option<&wireguard::PeerConfig>,
        relay_constraints: &RelayConstraints,
        retry_attempt: u32,
    ) -> Option<(Relay, MullvadEndpoint)> {
        let entry_location = relay_constraints
            .wireguard_constraints
            .entry_location
            .clone()?;
        let entry_constraints = RelayConstraints {
            location: entry_location,
            tunnel_protocol: Constraint::Only(TunnelType::Wireguard),
            ..relay_constraints.clone()
        };
        let entry_constraints =
            self.preferred_constraints(&entry_constraints, BridgeState::Off, retry_attempt, true);

        let matching_relays: Vec<Relay> = self
            .parsed_relays
            .lock()
            .relays()
            .iter()
            .filter(|relay| relay.active)
            .filter_map(|relay| Self::matching_relay(relay, &entry_constraints, exit_peer))
            .collect();

        let relay = self
            .pick_random_relay(&matching_relays)
            .map(|relay| relay.clone())?;
        let endpoint = self.get_random_tunnel(&relay, &entry_constraints)?;
        Some((relay, endpoint))
    }

    fn set_entry_peers(
        &mut self,
        new_exit_peer: &wireguard::PeerConfig,
        entry_endpoint: &mut MullvadEndpoint,
    ) {
        if let MullvadEndpoint::Wireguard {
            ref mut peer,
            exit_peer,
            ..
        } = entry_endpoint
        {
            peer.allowed_ips = vec![IpNetwork::from(new_exit_peer.endpoint.ip())];
            *exit_peer = Some(new_exit_peer.clone());
        }
    }

    pub fn get_auto_proxy_settings(
        &mut self,
        bridge_constraints: &InternalBridgeConstraints,
        location: &Location,
        retry_attempt: u32,
    ) -> Option<(ProxySettings, Relay)> {
        if !self.should_use_bridge(retry_attempt) {
            return None;
        }

        // For now, only TCP tunnels are supported.
        if let Constraint::Only(TransportProtocol::Udp) = bridge_constraints.transport_protocol {
            return None;
        }

        self.get_proxy_settings(bridge_constraints, location)
    }

    pub fn should_use_bridge(&self, retry_attempt: u32) -> bool {
        // shouldn't use a bridge for the first 3 times
        retry_attempt > 3 &&
            // i.e. 4th and 5th with bridge, 6th & 7th without
            // The test is to see whether the current _couple of connections_ is even or not.
            // | retry_attempt                | 4 | 5 | 6 | 7 | 8 | 9 |
            // | (retry_attempt % 4) < 2      | t | t | f | f | t | t |
            (retry_attempt % 4) < 2
    }

    pub fn get_proxy_settings(
        &mut self,
        constraints: &InternalBridgeConstraints,
        location: &Location,
    ) -> Option<(ProxySettings, Relay)> {
        let mut matching_relays: Vec<Relay> = self
            .parsed_relays
            .lock()
            .relays()
            .iter()
            .filter(|relay| relay.active)
            .filter_map(|relay| Self::matching_bridge_relay(relay, constraints))
            .collect();

        if matching_relays.is_empty() {
            return None;
        }

        matching_relays.sort_by_cached_key(|relay| {
            (relay.location.as_ref().unwrap().distance_from(&location) * 1000.0) as i64
        });
        matching_relays.get(0).and_then(|relay| {
            self.pick_random_bridge(&relay)
                .map(|bridge| (bridge, relay.clone()))
        })
    }

    /// Returns preferred constraints
    #[allow(unused_variables)]
    fn preferred_tunnel_constraints(
        &self,
        retry_attempt: u32,
        location_constraint: &Constraint<LocationConstraint>,
        providers_constraint: &Constraint<Providers>,
        wg_key_exists: bool,
    ) -> (Constraint<u16>, TransportProtocol, TunnelType) {
        #[cfg(target_os = "windows")]
        {
            let location_supports_openvpn =
                self.parsed_relays.lock().relays().iter().any(|relay| {
                    relay.active
                        && !relay.tunnels.openvpn.is_empty()
                        && location_constraint.matches(relay)
                        && providers_constraint.matches(relay)
                });
            if location_supports_openvpn {
                let (preferred_port, preferred_protocol) =
                    Self::preferred_openvpn_constraints(retry_attempt);
                return (preferred_port, preferred_protocol, TunnelType::OpenVpn);
            }
        }

        let location_supports_wireguard = self.parsed_relays.lock().relays().iter().any(|relay| {
            relay.active
                && !relay.tunnels.wireguard.is_empty()
                && location_constraint.matches(relay)
                && providers_constraint.matches(relay)
        });
        // If location does not support WireGuard, defer to preferred OpenVPN tunnel
        // constraints
        if !location_supports_wireguard || !wg_key_exists {
            let (preferred_port, preferred_protocol) =
                Self::preferred_openvpn_constraints(retry_attempt);
            return (preferred_port, preferred_protocol, TunnelType::OpenVpn);
        }

        // Try out WireGuard in the first two connection attempts, first with any port,
        // afterwards on port 53. Afterwards, connect through OpenVPN alternating between UDP
        // on any port twice and TCP on port 443 once.
        match retry_attempt {
            0 => (
                Constraint::Any,
                TransportProtocol::Udp,
                TunnelType::Wireguard,
            ),
            1 => (
                Constraint::Only(53),
                TransportProtocol::Udp,
                TunnelType::Wireguard,
            ),
            _ => {
                let (preferred_port, preferred_protocol) =
                    Self::preferred_openvpn_constraints(retry_attempt - 2);
                (preferred_port, preferred_protocol, TunnelType::OpenVpn)
            }
        }
    }

    fn preferred_openvpn_constraints(retry_attempt: u32) -> (Constraint<u16>, TransportProtocol) {
        // Prefer UDP by default. But if that has failed a couple of times, then try TCP port
        // 443, which works for many with UDP problems. After that, just alternate
        // between protocols.
        match retry_attempt {
            0 | 1 => (Constraint::Any, TransportProtocol::Udp),
            2 | 3 => (Constraint::Only(443), TransportProtocol::Tcp),
            attempt if attempt % 2 == 0 => (Constraint::Any, TransportProtocol::Udp),
            _ => (Constraint::Any, TransportProtocol::Tcp),
        }
    }


    /// Returns a random relay endpoint if any is matching the given constraints.
    fn get_tunnel_endpoint_internal(
        &mut self,
        constraints: &RelayConstraints,
        wg_entry_peer: Option<&wireguard::PeerConfig>,
    ) -> Option<(Relay, MullvadEndpoint)> {
        let matching_relays: Vec<Relay> = self
            .parsed_relays
            .lock()
            .relays()
            .iter()
            .filter(|relay| relay.active)
            .filter_map(|relay| Self::matching_relay(relay, constraints, wg_entry_peer))
            .collect();

        self.pick_random_relay(&matching_relays)
            .and_then(|selected_relay| {
                let endpoint = self.get_random_tunnel(&selected_relay, &constraints);
                let addr_in = endpoint
                    .as_ref()
                    .map(|endpoint| endpoint.to_endpoint().address.ip())
                    .unwrap_or(IpAddr::from(selected_relay.ipv4_addr_in));
                info!("Selected relay {} at {}", selected_relay.hostname, addr_in);
                endpoint.map(|endpoint| (selected_relay.clone(), endpoint))
            })
    }

    /// Takes a `Relay` and a corresponding `RelayConstraints` and returns a new `Relay` if the
    /// given relay matches the constraints.
    fn matching_relay(
        relay: &Relay,
        constraints: &RelayConstraints,
        skip_wg_peer: Option<&wireguard::PeerConfig>,
    ) -> Option<Relay> {
        if !constraints.location.matches(relay) {
            return None;
        }
        if !constraints.providers.matches(&relay) {
            return None;
        }

        let include_wg = if let Some(wg_peer) = skip_wg_peer {
            let peer_ip = wg_peer.endpoint.ip();
            peer_ip != IpAddr::V4(relay.ipv4_addr_in)
                && Some(peer_ip) != relay.ipv6_addr_in.map(IpAddr::V6)
        } else {
            true
        };

        let relay = match constraints.tunnel_protocol {
            Constraint::Any => {
                let mut relay = relay.clone();
                relay.tunnels = RelayTunnels {
                    wireguard: if include_wg {
                        Self::matching_wireguard_tunnels(
                            &relay.tunnels,
                            &constraints.wireguard_constraints,
                        )
                    } else {
                        vec![]
                    },
                    openvpn: Self::matching_openvpn_tunnels(
                        &relay.tunnels,
                        constraints.openvpn_constraints,
                    ),
                };
                relay
            }
            Constraint::Only(TunnelType::Wireguard) => {
                let mut relay = relay.clone();
                relay.tunnels = RelayTunnels {
                    wireguard: if include_wg {
                        Self::matching_wireguard_tunnels(
                            &relay.tunnels,
                            &constraints.wireguard_constraints,
                        )
                    } else {
                        vec![]
                    },
                    openvpn: vec![],
                };
                relay
            }

            Constraint::Only(TunnelType::OpenVpn) => {
                let mut relay = relay.clone();
                relay.tunnels = RelayTunnels {
                    openvpn: Self::matching_openvpn_tunnels(
                        &relay.tunnels,
                        constraints.openvpn_constraints,
                    ),
                    wireguard: vec![],
                };
                relay
            }
        };


        let relay_matches = match constraints.tunnel_protocol {
            Constraint::Any => {
                !relay.tunnels.openvpn.is_empty() || !relay.tunnels.wireguard.is_empty()
            }
            Constraint::Only(TunnelType::OpenVpn) => !relay.tunnels.openvpn.is_empty(),
            Constraint::Only(TunnelType::Wireguard) => !relay.tunnels.wireguard.is_empty(),
        };

        if relay_matches {
            Some(relay)
        } else {
            None
        }
    }

    fn matching_bridge_relay(
        relay: &Relay,
        constraints: &InternalBridgeConstraints,
    ) -> Option<Relay> {
        if !constraints.location.matches(relay) {
            return None;
        }
        if !constraints.providers.matches(relay) {
            return None;
        }

        let mut filtered_relay = relay.clone();
        filtered_relay
            .bridges
            .shadowsocks
            .retain(|bridge| constraints.transport_protocol.matches_eq(&bridge.protocol));
        if filtered_relay.bridges.shadowsocks.is_empty() {
            return None;
        }

        Some(filtered_relay)
    }

    fn matching_openvpn_tunnels(
        tunnels: &RelayTunnels,
        constraints: OpenVpnConstraints,
    ) -> Vec<OpenVpnEndpointData> {
        tunnels
            .openvpn
            .iter()
            .filter(|endpoint| constraints.matches(*endpoint))
            .cloned()
            .collect()
    }

    fn matching_wireguard_tunnels(
        tunnels: &RelayTunnels,
        constraints: &WireguardConstraints,
    ) -> Vec<WireguardEndpointData> {
        tunnels
            .wireguard
            .iter()
            .filter(|endpoint| constraints.matches(*endpoint))
            .cloned()
            .collect()
    }

    /// Pick a random relay from the given slice. Will return `None` if the given slice is empty
    /// or all relays in it has zero weight.
    fn pick_random_relay<'a>(&mut self, relays: &'a [Relay]) -> Option<&'a Relay> {
        let total_weight: u64 = relays.iter().map(|relay| relay.weight).sum();
        if total_weight == 0 {
            None
        } else {
            // Pick a random number in the range 0 - total_weight. This choses the relay.
            let mut i: u64 = self.rng.gen_range(0, total_weight + 1);
            Some(
                relays
                    .iter()
                    .find(|relay| {
                        i = i.saturating_sub(relay.weight);
                        i == 0
                    })
                    .unwrap(),
            )
        }
    }

    /// Picks a random bridge from a relay.
    fn pick_random_bridge(&mut self, relay: &Relay) -> Option<ProxySettings> {
        relay
            .bridges
            .shadowsocks
            .choose(&mut self.rng)
            .map(|shadowsocks_endpoint| {
                info!(
                    "Selected Shadowsocks bridge {} at {}:{}/{}",
                    relay.hostname,
                    relay.ipv4_addr_in,
                    shadowsocks_endpoint.port,
                    shadowsocks_endpoint.protocol
                );
                shadowsocks_endpoint
                    .clone()
                    .to_proxy_settings(relay.ipv4_addr_in.into())
            })
    }

    fn get_random_tunnel(
        &mut self,
        relay: &Relay,
        constraints: &RelayConstraints,
    ) -> Option<MullvadEndpoint> {
        #[cfg(not(target_os = "android"))]
        let mut thread_rng = self.rng.clone();
        #[cfg(not(target_os = "android"))]
        let mut new_openvpn_endpoint = || {
            relay
                .tunnels
                .openvpn
                .choose(&mut thread_rng)
                .cloned()
                .map(|endpoint| endpoint.into_mullvad_endpoint(relay.ipv4_addr_in.into()))
        };

        let mut new_wg_endpoint = || {
            relay
                .tunnels
                .wireguard
                .choose(&mut self.rng)
                .cloned()
                .and_then(|wg_tunnel| {
                    self.wg_data_to_endpoint(relay, wg_tunnel, &constraints.wireguard_constraints)
                })
        };

        #[cfg(not(target_os = "android"))]
        match constraints.tunnel_protocol {
            Constraint::Only(TunnelType::OpenVpn) => new_openvpn_endpoint(),

            Constraint::Any => vec![new_openvpn_endpoint(), new_wg_endpoint()]
                .into_iter()
                .filter_map(|relay| relay)
                .collect::<Vec<_>>()
                .choose(&mut self.rng)
                .cloned(),

            Constraint::Only(TunnelType::Wireguard) => new_wg_endpoint(),
        }
        #[cfg(target_os = "android")]
        new_wg_endpoint()
    }

    fn wg_data_to_endpoint(
        &mut self,
        relay: &Relay,
        data: WireguardEndpointData,
        constraints: &WireguardConstraints,
    ) -> Option<MullvadEndpoint> {
        let host = self.get_address_for_wireguard_relay(relay, constraints)?;
        let port = self.get_port_for_wireguard_relay(&data, constraints)?;
        let peer_config = wireguard::PeerConfig {
            public_key: data.public_key,
            endpoint: SocketAddr::new(host, port),
            allowed_ips: all_of_the_internet(),
            protocol: constraints
                .port
                .map(|port| port.protocol)
                .unwrap_or(TransportProtocol::Udp),
        };
        Some(MullvadEndpoint::Wireguard {
            peer: peer_config,
            exit_peer: None,
            ipv4_gateway: data.ipv4_gateway,
            ipv6_gateway: data.ipv6_gateway,
        })
    }

    fn get_address_for_wireguard_relay(
        &mut self,
        relay: &Relay,
        constraints: &WireguardConstraints,
    ) -> Option<IpAddr> {
        match constraints.ip_version {
            Constraint::Any | Constraint::Only(IpVersion::V4) => Some(relay.ipv4_addr_in.into()),
            Constraint::Only(IpVersion::V6) => relay.ipv6_addr_in.map(|addr| addr.into()),
        }
    }

    fn get_port_for_wireguard_relay(
        &mut self,
        data: &WireguardEndpointData,
        constraints: &WireguardConstraints,
    ) -> Option<u16> {
        match constraints
            .port
            .as_ref()
            .map(|port| port.port)
            .unwrap_or(Constraint::Any)
        {
            Constraint::Any => {
                let get_port_amount =
                    |range: &(u16, u16)| -> u64 { (1 + range.1 - range.0) as u64 };
                let port_amount: u64 = data.port_ranges.iter().map(get_port_amount).sum();

                if port_amount < 1 {
                    return None;
                }

                let mut port_index = self.rng.gen_range(0, port_amount);

                for range in data.port_ranges.iter() {
                    let ports_in_range = get_port_amount(range);
                    if port_index < ports_in_range {
                        return Some(port_index as u16 + range.0);
                    }
                    port_index -= ports_in_range;
                }
                error!("Port selection algorithm is broken!");
                None
            }
            Constraint::Only(port) => {
                if data
                    .port_ranges
                    .iter()
                    .any(|range| (range.0 <= port && port <= range.1))
                {
                    Some(port)
                } else {
                    None
                }
            }
        }
    }

    /// Try to read the relays from disk, preferring the newer ones.
    fn read_relays_from_disk(
        cache_path: &Path,
        resource_path: &Path,
    ) -> Result<ParsedRelays, Error> {
        // prefer the resource path's relay list if the cached one doesn't exist or was modified
        // before the resource one was created.
        let cached_relays = ParsedRelays::from_file(cache_path);
        let bundled_relays = match ParsedRelays::from_file(resource_path) {
            Ok(bundled_relays) => bundled_relays,
            Err(e) => {
                log::error!("Failed to load bundled relays - {}", e);
                return cached_relays;
            }
        };

        if cached_relays
            .as_ref()
            .map(|cached| cached.last_updated > bundled_relays.last_updated)
            .unwrap_or(false)
        {
            cached_relays
        } else {
            Ok(bundled_relays)
        }
    }
}

#[derive(Clone)]
pub struct RelayListUpdaterHandle {
    tx: mpsc::Sender<bool>,
}

impl RelayListUpdaterHandle {
    pub async fn update_relay_list(&mut self) -> Result<(), Error> {
        self.tx
            .send(false)
            .await
            .map_err(|_| Error::DownloaderShutDown)
    }

    pub async fn update_relay_list_deferred(&mut self) -> Result<(), Error> {
        self.tx
            .send(true)
            .await
            .map_err(|_| Error::DownloaderShutDown)
    }
}

struct RelayListUpdater {
    rpc_client: RelayListProxy,
    cache_path: PathBuf,
    parsed_relays: Arc<Mutex<ParsedRelays>>,
    on_update: Box<dyn Fn(&RelayList) + Send + 'static>,
    earliest_next_try: Instant,
    api_availability: ApiAvailabilityHandle,
}

impl RelayListUpdater {
    pub fn new(
        rpc_handle: MullvadRestHandle,
        cache_path: PathBuf,
        parsed_relays: Arc<Mutex<ParsedRelays>>,
        on_update: Box<dyn Fn(&RelayList) + Send + 'static>,
        api_availability: ApiAvailabilityHandle,
    ) -> RelayListUpdaterHandle {
        let (tx, cmd_rx) = mpsc::channel(1);
        let service = rpc_handle.service();
        let rpc_client = RelayListProxy::new(rpc_handle);
        let updater = RelayListUpdater {
            rpc_client,
            cache_path,
            parsed_relays,
            on_update,
            earliest_next_try: Instant::now() + UPDATE_INTERVAL,
            api_availability,
        };

        service.spawn(updater.run(cmd_rx));

        RelayListUpdaterHandle { tx }
    }

    async fn run(mut self, mut cmd_rx: mpsc::Receiver<bool>) {
        let mut check_interval = tokio_stream::wrappers::IntervalStream::new(
            tokio::time::interval(UPDATE_CHECK_INTERVAL),
        )
        .fuse();
        let mut download_future = Box::pin(Fuse::terminated());
        loop {
            futures::select! {
                _check_update = check_interval.next() => {
                    if download_future.is_terminated() && self.should_update() {
                        let tag = self.parsed_relays.lock().tag().map(|tag| tag.to_string());
                        download_future = Box::pin(Self::download_relay_list(self.api_availability.clone(), self.rpc_client.clone(), tag).fuse());
                        self.earliest_next_try = Instant::now() + UPDATE_INTERVAL;
                    }
                },

                new_relay_list = download_future => {
                    self.consume_new_relay_list(new_relay_list).await;

                },

                cmd = cmd_rx.next() => {
                    match cmd {
                        Some(defer) => {
                            let tag = self.parsed_relays.lock().tag().map(|tag| tag.to_string());
                            if defer {
                                let download_future = Self::download_relay_list(self.api_availability.clone(), self.rpc_client.clone(), tag);
                                self.consume_new_relay_list(download_future.await).await;
                            } else {
                                self.consume_new_relay_list(self.rpc_client.relay_list(tag).await.map_err(mullvad_rpc::Error::from)).await;
                            }
                        },
                        None => {
                            log::error!("Relay list updater shutting down");
                            return;
                        }
                    }
                }

            };
        }
    }

    async fn consume_new_relay_list(
        &mut self,
        result: Result<Option<RelayList>, mullvad_rpc::Error>,
    ) {
        match result {
            Ok(Some(relay_list)) => {
                if let Err(err) = self.update_cache(relay_list).await {
                    log::error!("Failed to update relay list cache: {}", err);
                }
            }
            Ok(None) => log::debug!("Relay list is up-to-date"),
            Err(err) => {
                log::error!(
                    "Failed to fetch new relay list: {}. Will retry in {} minutes",
                    err,
                    self.earliest_next_try
                        .saturating_duration_since(Instant::now())
                        .as_secs()
                        / 60
                );
            }
        }
    }

    /// Returns true if the current parsed_relays is older than UPDATE_INTERVAL
    fn should_update(&mut self) -> bool {
        match SystemTime::now().duration_since(self.parsed_relays.lock().last_updated()) {
            Ok(duration) => duration > UPDATE_INTERVAL && self.earliest_next_try <= Instant::now(),
            // If the clock is skewed we have no idea by how much or when the last update
            // actually was, better download again to get in sync and get a `last_updated`
            // timestamp corresponding to the new time.
            Err(_) => true,
        }
    }

    fn download_relay_list(
        api_handle: ApiAvailabilityHandle,
        rpc_handle: RelayListProxy,
        tag: Option<String>,
    ) -> impl Future<Output = Result<Option<RelayList>, mullvad_rpc::Error>> + 'static {
        let download_futures = move || {
            let available = api_handle.wait_background();
            let req = rpc_handle.relay_list(tag.clone());
            async move {
                available.await?;
                req.await.map_err(mullvad_rpc::Error::from)
            }
        };

        let exponential_backoff =
            ExponentialBackoff::new(EXPONENTIAL_BACKOFF_INITIAL, EXPONENTIAL_BACKOFF_FACTOR)
                .max_delay(UPDATE_INTERVAL * 2);

        let download_future = retry_future(
            download_futures,
            |result| result.is_err(),
            Jittered::jitter(exponential_backoff),
        );
        download_future
    }

    async fn update_cache(&mut self, new_relay_list: RelayList) -> Result<(), Error> {
        if let Err(error) = Self::cache_relays(&self.cache_path, &new_relay_list).await {
            error!(
                "{}",
                error.display_chain_with_msg("Failed to update relay cache on disk")
            );
        }

        let new_parsed_relays = ParsedRelays::from_relay_list(new_relay_list, SystemTime::now());
        info!(
            "Downloaded relay inventory has {} relays",
            new_parsed_relays.relays().len()
        );

        let mut parsed_relays = self.parsed_relays.lock();
        *parsed_relays = new_parsed_relays;
        (self.on_update)(parsed_relays.locations());
        Ok(())
    }

    /// Write a `RelayList` to the cache file.
    async fn cache_relays(cache_path: &Path, relays: &RelayList) -> Result<(), Error> {
        debug!("Writing relays cache to {}", cache_path.display());
        let mut file = File::create(cache_path)
            .await
            .map_err(Error::OpenRelayCache)?;
        let bytes = serde_json::to_vec_pretty(relays).map_err(Error::Serialize)?;
        let mut slice: &[u8] = bytes.as_slice();
        let _ = tokio::io::copy(&mut slice, &mut file)
            .await
            .map_err(Error::WriteRelayCache)?;
        Ok(())
    }
}


#[cfg(test)]
mod test {
    use super::*;
    use mullvad_types::{
        relay_constraints::RelayConstraints,
        relay_list::{
            Relay, RelayBridges, RelayListCity, RelayListCountry, RelayTunnels,
            WireguardEndpointData,
        },
    };
    use talpid_types::net::wireguard::PublicKey;

    lazy_static::lazy_static! {
        static ref RELAYS: RelayList = RelayList {
            etag: None,
            countries: vec![
                RelayListCountry {
                    name: "Sweden".to_string(),
                    code: "se".to_string(),
                    cities: vec![
                        RelayListCity {
                            name: "Gothenburg".to_string(),
                            code: "got".to_string(),
                            latitude: 57.70887,
                            longitude: 11.97456,
                            relays: vec![
                                Relay {
                                    hostname: "se9-wireguard".to_string(),
                                    ipv4_addr_in: "185.213.154.68".parse().unwrap(),
                                    ipv6_addr_in: Some("2a03:1b20:5:f011::a09f".parse().unwrap()),
                                    include_in_country: true,
                                    active: true,
                                    owned: true,
                                    provider: "31173".to_string(),
                                    weight: 1,
                                    tunnels: RelayTunnels {
                                        openvpn: vec![],
                                        wireguard: vec![
                                            WireguardEndpointData {
                                                port_ranges: vec![(53, 53), (4000, 33433), (33565, 51820), (52000, 60000)],
                                                ipv4_gateway: "10.64.0.1".parse().unwrap(),
                                                ipv6_gateway: "fc00:bbbb:bbbb:bb01::1".parse().unwrap(),
                                                public_key: PublicKey::from_base64("BLNHNoGO88LjV/wDBa7CUUwUzPq/fO2UwcGLy56hKy4=").unwrap(),
                                                protocol: TransportProtocol::Udp,
                                            },
                                        ],
                                    },
                                    bridges: RelayBridges {
                                        shadowsocks: vec![],
                                    },
                                    location: None,
                                },
                                Relay {
                                    hostname: "se10-wireguard".to_string(),
                                    ipv4_addr_in: "185.213.154.69".parse().unwrap(),
                                    ipv6_addr_in: Some("2a03:1b20:5:f011::a10f".parse().unwrap()),
                                    include_in_country: true,
                                    active: true,
                                    owned: true,
                                    provider: "31173".to_string(),
                                    weight: 1,
                                    tunnels: RelayTunnels {
                                        openvpn: vec![],
                                        wireguard: vec![
                                            WireguardEndpointData {
                                                port_ranges: vec![(53, 53), (4000, 33433), (33565, 51820), (52000, 60000)],
                                                ipv4_gateway: "10.64.0.1".parse().unwrap(),
                                                ipv6_gateway: "fc00:bbbb:bbbb:bb01::1".parse().unwrap(),
                                                public_key: PublicKey::from_base64("veGD6/aEY6sMfN3Ls7YWPmNgu3AheO7nQqsFT47YSws=").unwrap(),
                                                protocol: TransportProtocol::Udp,
                                            },
                                        ],
                                    },
                                    bridges: RelayBridges {
                                        shadowsocks: vec![],
                                    },
                                    location: None,
                                },
                                Relay {
                                    hostname: "se-got-001".to_string(),
                                    ipv4_addr_in: "185.213.154.131".parse().unwrap(),
                                    ipv6_addr_in: None,
                                    include_in_country: true,
                                    active: true,
                                    owned: true,
                                    provider: "31173".to_string(),
                                    weight: 1,
                                    tunnels: RelayTunnels {
                                        openvpn: vec![
                                            OpenVpnEndpointData {
                                                port: 1194,
                                                protocol: TransportProtocol::Udp,
                                            },
                                            OpenVpnEndpointData {
                                                port: 443,
                                                protocol: TransportProtocol::Tcp,
                                            },
                                            OpenVpnEndpointData {
                                                port: 80,
                                                protocol: TransportProtocol::Tcp,
                                            },
                                        ],
                                        wireguard: vec![],
                                    },
                                    bridges: RelayBridges {
                                        shadowsocks: vec![],
                                    },
                                    location: None,
                                },
                            ],
                        },
                    ],
                }
            ],
        };
    }

    fn new_relay_selector() -> RelaySelector {
        RelaySelector {
            parsed_relays: Arc::new(Mutex::new(ParsedRelays::from_relay_list(
                RELAYS.clone(),
                SystemTime::now(),
            ))),
            rng: rand::thread_rng(),
            updater: None,
        }
    }

    #[test]
    fn test_preferred_tunnel_protocol() {
        let mut relay_selector = new_relay_selector();

        // Prefer WG if the location only supports it
        let location = LocationConstraint::Hostname(
            "se".to_string(),
            "got".to_string(),
            "se9-wireguard".to_string(),
        );
        let relay_constraints = RelayConstraints {
            location: Constraint::Only(location.clone()),
            tunnel_protocol: Constraint::Any,
            ..RelayConstraints::default()
        };

        let preferred =
            relay_selector.preferred_constraints(&relay_constraints, BridgeState::Off, 0, true);
        assert_eq!(
            preferred.tunnel_protocol,
            Constraint::Only(TunnelType::Wireguard)
        );

        for attempt in 0..10 {
            assert!(relay_selector
                .get_tunnel_exit_endpoint(&relay_constraints, BridgeState::Off, attempt, true, None)
                .is_ok());
        }

        // Prefer OpenVPN if the location only supports it
        let location = LocationConstraint::Hostname(
            "se".to_string(),
            "got".to_string(),
            "se-got-001".to_string(),
        );
        let relay_constraints = RelayConstraints {
            location: Constraint::Only(location.clone()),
            tunnel_protocol: Constraint::Any,
            ..RelayConstraints::default()
        };

        let preferred =
            relay_selector.preferred_constraints(&relay_constraints, BridgeState::Off, 0, true);
        assert_eq!(
            preferred.tunnel_protocol,
            Constraint::Only(TunnelType::OpenVpn)
        );

        for attempt in 0..10 {
            assert!(relay_selector
                .get_tunnel_exit_endpoint(&relay_constraints, BridgeState::Off, attempt, true, None)
                .is_ok());
        }

        // Prefer OpenVPN on Windows when possible
        #[cfg(windows)]
        {
            let relay_constraints = RelayConstraints::default();
            for attempt in 0..10 {
                let preferred = relay_selector.preferred_constraints(
                    &relay_constraints,
                    BridgeState::Off,
                    attempt,
                    true,
                );
                assert_eq!(
                    preferred.tunnel_protocol,
                    Constraint::Only(TunnelType::OpenVpn)
                );
                match relay_selector.get_tunnel_exit_endpoint(
                    &relay_constraints,
                    BridgeState::Off,
                    attempt,
                    true,
                    None,
                ) {
                    Ok((_, MullvadEndpoint::OpenVpn(_))) => (),
                    _ => panic!("OpenVPN endpoint was not selected"),
                }
            }
        }
    }

    #[test]
    fn test_wg_entry_hostname_collision() {
        let mut relay_selector = new_relay_selector();

        let location1 = LocationConstraint::Hostname(
            "se".to_string(),
            "got".to_string(),
            "se9-wireguard".to_string(),
        );
        let location2 = LocationConstraint::Hostname(
            "se".to_string(),
            "got".to_string(),
            "se10-wireguard".to_string(),
        );

        let mut relay_constraints = RelayConstraints {
            location: Constraint::Only(location1.clone()),
            tunnel_protocol: Constraint::Only(TunnelType::Wireguard),
            ..RelayConstraints::default()
        };

        relay_constraints.wireguard_constraints.entry_location = Some(Constraint::Only(location1));

        // The same host cannot be used for entry and exit
        assert!(relay_selector
            .get_tunnel_endpoint(&relay_constraints, BridgeState::Off, 0, true)
            .is_err());

        relay_constraints.wireguard_constraints.entry_location = Some(Constraint::Only(location2));

        // If the entry and exit differ, this should succeed
        assert!(relay_selector
            .get_tunnel_endpoint(&relay_constraints, BridgeState::Off, 0, true)
            .is_ok());
    }

    #[test]
    fn test_wg_entry_filter() -> Result<(), String> {
        let mut relay_selector = new_relay_selector();

        let specific_hostname = "se10-wireguard";

        let location_general = LocationConstraint::City("se".to_string(), "got".to_string());
        let location_specific = LocationConstraint::Hostname(
            "se".to_string(),
            "got".to_string(),
            specific_hostname.to_string(),
        );

        let mut relay_constraints = RelayConstraints {
            location: Constraint::Only(location_general.clone()),
            tunnel_protocol: Constraint::Only(TunnelType::Wireguard),
            ..RelayConstraints::default()
        };

        relay_constraints.wireguard_constraints.entry_location =
            Some(Constraint::Only(location_specific.clone()));

        // The exit must not equal the entry
        let (exit_relay, _exit_endpoint) = relay_selector
            .get_tunnel_endpoint(&relay_constraints, BridgeState::Off, 0, true)
            .map_err(|error| error.to_string())?;

        assert_ne!(exit_relay.hostname, specific_hostname);


        relay_constraints.location = Constraint::Only(location_specific);
        relay_constraints.wireguard_constraints.entry_location =
            Some(Constraint::Only(location_general));

        // The entry must not equal the exit
        let (exit_relay, exit_endpoint) = relay_selector
            .get_tunnel_endpoint(&relay_constraints, BridgeState::Off, 0, true)
            .map_err(|error| error.to_string())?;

        assert_eq!(exit_relay.hostname, specific_hostname);
        match exit_endpoint {
            MullvadEndpoint::OpenVpn { .. } => return Err("Expected WireGuard relay".to_string()),
            MullvadEndpoint::Wireguard {
                peer, exit_peer, ..
            } => {
                assert_eq!(exit_relay.ipv4_addr_in, exit_peer.unwrap().endpoint.ip());
                assert_ne!(exit_relay.ipv4_addr_in, peer.endpoint.ip());
            }
        }

        Ok(())
    }
}

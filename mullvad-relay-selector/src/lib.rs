//! When changing relay selection, please verify if `docs/relay-selector.md` needs to be
//! updated as well.

use chrono::{DateTime, Local};
use ipnetwork::IpNetwork;
use mullvad_api::{availability::ApiAvailabilityHandle, rest::MullvadRestHandle};
use mullvad_types::{
    endpoint::{MullvadEndpoint, MullvadWireguardEndpoint},
    location::{Coordinates, Location},
    relay_constraints::{
        BridgeState, Constraint, InternalBridgeConstraints, LocationConstraint, Match,
        ObfuscationSettings, OpenVpnConstraints, Providers, RelayConstraints, SelectedObfuscation,
        Set, TransportPort, Udp2TcpObfuscationSettings, WireguardConstraints,
    },
    relay_list::{Relay, RelayList, Udp2TcpEndpointData},
};
use parking_lot::Mutex;
use rand::{self, seq::SliceRandom, Rng};
use std::{
    io,
    net::{IpAddr, SocketAddr},
    path::Path,
    sync::Arc,
    time::{self, SystemTime},
};
use talpid_types::{
    net::{
        obfuscation::ObfuscatorConfig, openvpn::ProxySettings, wireguard, IpVersion,
        TransportProtocol, TunnelType,
    },
    ErrorExt,
};

use self::{
    matcher::{RelayMatcher, TunnelMatcher, WireguardMatcher},
    updater::{RelayListUpdater, RelayListUpdaterHandle},
};

mod matcher;
mod updater;

const DATE_TIME_FORMAT_STR: &str = "%Y-%m-%d %H:%M:%S%.3f";
const RELAYS_FILENAME: &str = "relays.json";

const DEFAULT_WIREGUARD_PORT: u16 = 51820;
const WIREGUARD_EXIT_CONSTRAINTS: WireguardMatcher = WireguardMatcher {
    peer: None,
    port: Constraint::Only(DEFAULT_WIREGUARD_PORT),
    ip_version: Constraint::Only(IpVersion::V4),
};

const UDP2TCP_PORTS: [u16; 3] = [80, 443, 5001];

/// How much to favor bridges that are closer to the selected relay location. Each
/// bridge is assigned a base weight based on its rank order proximity to the location.
/// Its final weight equals `(base weight) ^ BRIDGE_PROXIMITY_BIAS`.
const BRIDGE_PROXIMITY_BIAS: u32 = 3;

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

                    Self::filter_invalid_relays(&mut relay_with_location);

                    // TODO: The WireGuard data is incorrectly modelled.
                    // Using a vector here suggests that a relay may use multiple key pairs at a
                    // time. This is incorrect and will never be the case.
                    //
                    // Currently, the `wireguard` vector will have 0 or 1 entries.
                    // This should be changed into e.g. using an Option<_> instead.
                    //

                    if !relay.tunnels.wireguard.is_empty() {
                        for port in UDP2TCP_PORTS {
                            relay_with_location
                                .obfuscators
                                .udp2tcp
                                .push(Udp2TcpEndpointData { port });
                        }
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

    fn filter_invalid_relays(relay: &mut Relay) {
        let total_openvpn_endpoints = relay.tunnels.openvpn.len();
        let openvpn_endpoints = &mut relay.tunnels.openvpn;
        openvpn_endpoints.retain(|data| data.port != 0);

        if openvpn_endpoints.len() < total_openvpn_endpoints {
            log::error!(
                "Relay {} contained {} invalid OpenVPN endpoints out of {} endpoints",
                relay.hostname,
                total_openvpn_endpoints - openvpn_endpoints.len(),
                total_openvpn_endpoints
            );
        }

        let total_wireguard_endpoints = relay.tunnels.wireguard.len();
        let wireguard_endpoints = &mut relay.tunnels.wireguard;
        wireguard_endpoints.retain(|data| {
            !data.port_ranges.is_empty() && data.port_ranges.iter().all(|(start, end)| start <= end)
        });

        if wireguard_endpoints.len() < total_wireguard_endpoints {
            log::error!(
                "Relay {} contained {} invalid WireGuard endpoints out of {} endpoints",
                relay.hostname,
                total_wireguard_endpoints - wireguard_endpoints.len(),
                total_wireguard_endpoints
            );
        }
    }

    pub fn from_file(path: impl AsRef<Path>) -> Result<Self, Error> {
        log::debug!("Reading relays from {}", path.as_ref().display());
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
    updater: Option<RelayListUpdaterHandle>,
}

impl RelaySelector {
    /// Returns a new `RelaySelector` backed by relays cached on disk. Use the `update` method
    /// to refresh the relay list from the internet.
    pub fn new(
        api_handle: MullvadRestHandle,
        on_update: impl Fn(&RelayList) + Send + 'static,
        resource_dir: &Path,
        cache_dir: &Path,
        api_availability: ApiAvailabilityHandle,
    ) -> Self {
        let cache_path = cache_dir.join(RELAYS_FILENAME);
        let resource_path = resource_dir.join(RELAYS_FILENAME);
        let unsynchronized_parsed_relays = Self::read_relays_from_disk(&cache_path, &resource_path)
            .unwrap_or_else(|error| {
                log::error!(
                    "{}",
                    error.display_chain_with_msg("Unable to load cached relays")
                );
                ParsedRelays::empty()
            });
        log::info!(
            "Initialized with {} cached relays from {}",
            unsynchronized_parsed_relays.relays().len(),
            DateTime::<Local>::from(unsynchronized_parsed_relays.last_updated())
                .format(DATE_TIME_FORMAT_STR)
        );
        let parsed_relays = Arc::new(Mutex::new(unsynchronized_parsed_relays));

        let updater = RelayListUpdater::new(
            api_handle,
            cache_path,
            parsed_relays.clone(),
            Box::new(on_update),
            api_availability,
        );

        RelaySelector {
            parsed_relays,
            updater: Some(updater),
        }
    }

    /// Download the newest relay list.
    pub async fn update(&self) {
        if let Some(mut updater) = self.updater.clone() {
            if let Err(err) = updater.update_relay_list().await {
                log::error!(
                    "{}",
                    err.display_chain_with_msg(
                        "Unable to send update command to relay list updater"
                    )
                );
            }
        }
    }

    /// Returns all countries and cities. The cities in the object returned does not have any
    /// relays in them.
    pub fn get_locations(&mut self) -> RelayList {
        self.parsed_relays.lock().locations().clone()
    }

    /// Returns a random relay and relay endpoint matching the given constraints and with
    /// preferences applied.
    pub fn get_tunnel_endpoint(
        &self,
        relay_constraints: &RelayConstraints,
        bridge_state: BridgeState,
        retry_attempt: u32,
    ) -> Result<RelaySelectorResult, Error> {
        match relay_constraints.tunnel_protocol {
            Constraint::Only(TunnelType::OpenVpn) => self.get_openvpn_endpoint(
                &relay_constraints.location,
                &relay_constraints.providers,
                relay_constraints.openvpn_constraints.clone(),
                bridge_state,
                retry_attempt,
            ),

            Constraint::Only(TunnelType::Wireguard) => self.get_wireguard_endpoint(
                &relay_constraints.location,
                &relay_constraints.providers,
                &relay_constraints.wireguard_constraints,
                retry_attempt,
            ),
            Constraint::Any => {
                self.get_any_tunnel_endpoint(relay_constraints, bridge_state, retry_attempt)
            }
        }
    }

    /// Returns the average location of relays that match the given constraints.
    /// This returns none if the location is `any` or if no relays match the constraints.
    pub fn get_relay_midpoint(&self, relay_constraints: &RelayConstraints) -> Option<Coordinates> {
        if relay_constraints.location.is_any() {
            return None;
        }

        let matcher = RelayMatcher::from(relay_constraints.clone());
        let mut matching_locations: Vec<Location> = self
            .parsed_relays
            .lock()
            .relays()
            .iter()
            .filter(|relay| relay.active)
            .filter_map(|relay| {
                matcher
                    .filter_matching_relay(relay)
                    .and_then(|relay| relay.location)
            })
            .collect();
        matching_locations.dedup_by(|a, b| a.has_same_city(b));

        if matching_locations.is_empty() {
            return None;
        }
        Some(Coordinates::midpoint(&matching_locations))
    }

    /// Returns an OpenVpn endpoint, should only ever be used when the user has specified the tunnel
    /// protocol as only OpenVPN.
    fn get_openvpn_endpoint(
        &self,
        location: &Constraint<LocationConstraint>,
        providers: &Constraint<Providers>,
        openvpn_constraints: OpenVpnConstraints,
        bridge_state: BridgeState,
        retry_attempt: u32,
    ) -> Result<RelaySelectorResult, Error> {
        let mut relay_matcher = RelayMatcher {
            location: location.clone(),
            providers: providers.clone(),
            tunnel: openvpn_constraints,
        };

        if relay_matcher.tunnel.port.is_any() && bridge_state == BridgeState::On {
            relay_matcher.tunnel.port = Constraint::Only(TransportPort {
                protocol: TransportProtocol::Tcp,
                port: Constraint::Any,
            });

            return self.get_tunnel_endpoint_internal(&relay_matcher);
        }

        let mut preferred_relay_matcher = relay_matcher.clone();

        let (preferred_port, preferred_protocol) =
            Self::preferred_openvpn_constraints(retry_attempt);
        let should_try_preferred = match &mut preferred_relay_matcher.tunnel.port {
            any @ Constraint::Any => {
                *any = Constraint::Only(TransportPort {
                    protocol: preferred_protocol,
                    port: preferred_port,
                });
                true
            }
            Constraint::Only(ref mut port_constraints)
                if port_constraints.protocol == preferred_protocol
                    && port_constraints.port.is_any() =>
            {
                port_constraints.port = preferred_port;
                true
            }
            _ => false,
        };

        if should_try_preferred {
            self.get_tunnel_endpoint_internal(&preferred_relay_matcher)
                .or_else(|_| self.get_tunnel_endpoint_internal(&relay_matcher))
        } else {
            self.get_tunnel_endpoint_internal(&relay_matcher)
        }
    }

    fn get_wireguard_multi_hop_endpoint(
        &self,
        mut entry_matcher: RelayMatcher<WireguardMatcher>,
        exit_location: Constraint<LocationConstraint>,
    ) -> Result<RelaySelectorResult, Error> {
        let mut exit_matcher = RelayMatcher {
            location: exit_location,
            tunnel: WIREGUARD_EXIT_CONSTRAINTS.clone().into(),
            ..entry_matcher.clone()
        };

        let (exit_relay, entry_relay, exit_endpoint, mut entry_endpoint) =
            if entry_matcher.location.is_subset(&exit_matcher.location) {
                let (entry_relay, entry_endpoint) = self.get_entry_endpoint(&entry_matcher)?;
                exit_matcher.set_peer(entry_relay.clone());
                let exit_result = self.get_tunnel_endpoint_internal(&exit_matcher)?;
                (
                    exit_result.exit_relay,
                    entry_relay,
                    exit_result.endpoint,
                    entry_endpoint,
                )
            } else {
                let exit_result = self.get_tunnel_endpoint_internal(&exit_matcher)?;

                entry_matcher.set_peer(exit_result.exit_relay.clone());
                let (entry_relay, entry_endpoint) = self.get_entry_endpoint(&entry_matcher)?;
                (
                    exit_result.exit_relay,
                    entry_relay,
                    exit_result.endpoint,
                    entry_endpoint,
                )
            };

        Self::set_entry_peers(&exit_endpoint.unwrap_wireguard().peer, &mut entry_endpoint);

        log::info!(
            "Selected entry relay {} at {} going through {} at {}",
            entry_relay.hostname,
            entry_endpoint.peer.endpoint.ip(),
            exit_relay.hostname,
            exit_endpoint.to_endpoint().address.ip(),
        );
        let result = RelaySelectorResult::wireguard_multihop_endpoint(
            exit_relay,
            entry_endpoint,
            entry_relay,
        );
        return Ok(result);
    }

    /// Returns a WireGuard endpoint, should only ever be used when the user has specified the
    /// tunnel protocol as only WireGuard.
    fn get_wireguard_endpoint(
        &self,
        location: &Constraint<LocationConstraint>,
        providers: &Constraint<Providers>,
        wireguard_constraints: &WireguardConstraints,
        retry_attempt: u32,
    ) -> Result<RelaySelectorResult, Error> {
        let mut entry_relay_matcher = RelayMatcher {
            location: location.clone(),
            providers: providers.clone(),
            tunnel: wireguard_constraints.clone().into(),
        };

        let mut preferred_matcher: RelayMatcher<WireguardMatcher> = entry_relay_matcher.clone();
        preferred_matcher.tunnel.port = preferred_matcher
            .tunnel
            .port
            .or(Self::preferred_wireguard_port(retry_attempt));

        if !wireguard_constraints.use_multihop {
            return self
                .get_tunnel_endpoint_internal(&preferred_matcher)
                .or_else(|_| self.get_tunnel_endpoint_internal(&entry_relay_matcher));
        }

        entry_relay_matcher.location = wireguard_constraints.entry_location.clone();
        entry_relay_matcher.tunnel.port = entry_relay_matcher
            .tunnel
            .port
            .or(Self::preferred_wireguard_port(retry_attempt));
        self.get_wireguard_multi_hop_endpoint(entry_relay_matcher, location.clone())
    }

    /// Returns a tunnel endpoint of any type, should only be used when the user hasn't specified a
    /// tunnel protocol.
    fn get_any_tunnel_endpoint(
        &self,
        relay_constraints: &RelayConstraints,
        bridge_state: BridgeState,
        retry_attempt: u32,
    ) -> Result<RelaySelectorResult, Error> {
        let preferred_constraints =
            self.preferred_constraints(&relay_constraints, bridge_state, retry_attempt);
        let original_matcher: RelayMatcher<_> = relay_constraints.clone().into();

        let preferred_tunnel_protocol = preferred_constraints.tunnel_protocol;
        let preferred_matcher: RelayMatcher<_> = preferred_constraints.into();

        match preferred_tunnel_protocol {
            Constraint::Only(TunnelType::Wireguard)
                if relay_constraints.wireguard_constraints.use_multihop =>
            {
                let exit_location = relay_constraints.location.clone();
                let mut preferred_entry_matcher = preferred_matcher.to_wireguard_matcher();
                preferred_entry_matcher.location = relay_constraints
                    .wireguard_constraints
                    .entry_location
                    .clone();
                let mut original_entry_matcher = original_matcher.to_wireguard_matcher();
                original_entry_matcher.location = relay_constraints
                    .wireguard_constraints
                    .entry_location
                    .clone();
                self.get_wireguard_multi_hop_endpoint(
                    preferred_entry_matcher,
                    exit_location.clone(),
                )
                .or_else(|_| {
                    self.get_wireguard_multi_hop_endpoint(original_entry_matcher, exit_location)
                })
            }

            _ => {
                if let Ok(result) = self.get_tunnel_endpoint_internal(&preferred_matcher) {
                    log::debug!(
                        "Relay matched on highest preference for retry attempt {}",
                        retry_attempt
                    );
                    Ok(result)
                } else if let Ok(result) = self.get_tunnel_endpoint_internal(&original_matcher) {
                    log::debug!(
                        "Relay matched on second preference for retry attempt {}",
                        retry_attempt
                    );
                    Ok(result)
                } else {
                    log::warn!("No relays matching {}", &relay_constraints);
                    Err(Error::NoRelay)
                }
            }
        }
    }

    // This function ignores the tunnel type constraint on purpose.
    fn preferred_constraints(
        &self,
        original_constraints: &RelayConstraints,
        bridge_state: BridgeState,
        retry_attempt: u32,
    ) -> RelayConstraints {
        let (preferred_port, preferred_protocol, preferred_tunnel) = self
            .preferred_tunnel_constraints(
                retry_attempt,
                &original_constraints.location,
                &original_constraints.providers,
            );

        let mut relay_constraints = original_constraints.clone();
        relay_constraints.openvpn_constraints = Default::default();

        // Highest priority preference. Where we prefer OpenVPN using UDP. But without changing
        // any constraints that are explicitly specified.
        match original_constraints.tunnel_protocol {
            // If no tunnel protocol is selected, use preferred constraints
            Constraint::Any => {
                if bridge_state == BridgeState::On {
                    relay_constraints.openvpn_constraints = OpenVpnConstraints {
                        port: Constraint::Only(TransportPort {
                            protocol: TransportProtocol::Tcp,
                            port: Constraint::Any,
                        }),
                    };
                } else if original_constraints.openvpn_constraints.port.is_any() {
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
                    relay_constraints.wireguard_constraints.port = preferred_port;
                }

                relay_constraints.tunnel_protocol = Constraint::Only(preferred_tunnel);
            }
            Constraint::Only(TunnelType::OpenVpn) => {
                let openvpn_constraints = &mut relay_constraints.openvpn_constraints;
                *openvpn_constraints = original_constraints.openvpn_constraints;
                if bridge_state == BridgeState::On && openvpn_constraints.port.is_any() {
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
                if relay_constraints.wireguard_constraints.port.is_any() {
                    relay_constraints.wireguard_constraints.port =
                        Self::preferred_wireguard_port(retry_attempt);
                }
            }
        };

        if relay_constraints.wireguard_constraints.port.is_any() {
            relay_constraints.wireguard_constraints.port = preferred_port;
        }

        relay_constraints.tunnel_protocol = Constraint::Only(preferred_tunnel);

        relay_constraints
    }

    fn get_entry_endpoint(
        &self,
        matcher: &RelayMatcher<WireguardMatcher>,
    ) -> Result<(Relay, MullvadWireguardEndpoint), Error> {
        let matching_relays: Vec<Relay> = self
            .parsed_relays
            .lock()
            .relays()
            .iter()
            .filter(|relay| relay.active)
            .filter_map(|relay| matcher.filter_matching_relay(relay))
            .collect();

        let relay = self
            .pick_random_relay(&matching_relays)
            .map(|relay| relay.clone())
            .ok_or(Error::NoRelay)?;
        let endpoint = matcher
            .mullvad_endpoint(&relay)
            .ok_or(Error::NoRelay)?
            .unwrap_wireguard()
            .clone();

        Ok((relay, endpoint))
    }

    fn set_entry_peers(
        exit_peer: &wireguard::PeerConfig,
        entry_endpoint: &mut MullvadWireguardEndpoint,
    ) {
        entry_endpoint.peer.allowed_ips = vec![IpNetwork::from(exit_peer.endpoint.ip())];
        entry_endpoint.exit_peer = Some(exit_peer.clone());
    }

    #[cfg(not(target_os = "android"))]
    pub fn get_auto_proxy_settings<T: Into<Coordinates>>(
        &self,
        bridge_constraints: &InternalBridgeConstraints,
        location: Option<T>,
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

    #[cfg(not(target_os = "android"))]
    pub fn should_use_bridge(&self, retry_attempt: u32) -> bool {
        // shouldn't use a bridge for the first 3 times
        retry_attempt > 3 &&
            // i.e. 4th and 5th with bridge, 6th & 7th without
            // The test is to see whether the current _couple of connections_ is even or not.
            // | retry_attempt                | 4 | 5 | 6 | 7 | 8 | 9 |
            // | (retry_attempt % 4) < 2      | t | t | f | f | t | t |
            (retry_attempt % 4) < 2
    }

    pub fn get_proxy_settings<T: Into<Coordinates>>(
        &self,
        constraints: &InternalBridgeConstraints,
        location: Option<T>,
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

        let relay = if let Some(location) = location {
            let location = location.into();
            matching_relays.sort_by_cached_key(|relay: &Relay| {
                relay.location.as_ref().unwrap().distance_from(&location) as usize
            });
            let max_weight = matching_relays.len();
            let weight_fn = |index, _relay: &Relay| {
                let w = (max_weight - index) as u64;
                w.saturating_pow(BRIDGE_PROXIMITY_BIAS)
            };
            self.pick_random_relay_fn(&matching_relays, weight_fn)
        } else {
            self.pick_random_relay(&matching_relays)
        };
        relay.and_then(|relay| {
            self.pick_random_bridge(&relay)
                .map(|bridge| (bridge, relay.clone()))
        })
    }

    pub fn get_obfuscator(
        &self,
        obfuscation_settings: &ObfuscationSettings,
        relay: &Relay,
        endpoint: &MullvadWireguardEndpoint,
        retry_attempt: u32,
    ) -> Option<(ObfuscatorConfig, Relay)> {
        match obfuscation_settings.selected_obfuscation {
            SelectedObfuscation::Auto => {
                self.get_auto_obfuscator(obfuscation_settings, relay, endpoint, retry_attempt)
            }
            SelectedObfuscation::Off => None,
            SelectedObfuscation::Udp2Tcp => self.get_udp2tcp_obfuscator(
                &obfuscation_settings.udp2tcp,
                relay,
                endpoint,
                retry_attempt,
            ),
        }
    }

    fn get_auto_obfuscator(
        &self,
        obfuscation_settings: &ObfuscationSettings,
        relay: &Relay,
        endpoint: &MullvadWireguardEndpoint,
        retry_attempt: u32,
    ) -> Option<(ObfuscatorConfig, Relay)> {
        if !self.should_use_auto_obfuscator(retry_attempt) {
            return None;
        }
        // TODO FIX: The third obfuscator entry will never be chosen
        // Because get_auto_obfuscator_retry_attempt() returns [0, 1]
        // And the udp2tcp endpoints are defined in a vector with entries [0, 1, 2]
        self.get_udp2tcp_obfuscator(
            &obfuscation_settings.udp2tcp,
            relay,
            endpoint,
            self.get_auto_obfuscator_retry_attempt(retry_attempt)
                .unwrap(),
        )
    }

    pub fn should_use_auto_obfuscator(&self, retry_attempt: u32) -> bool {
        self.get_auto_obfuscator_retry_attempt(retry_attempt)
            .is_some()
    }

    fn get_auto_obfuscator_retry_attempt(&self, retry_attempt: u32) -> Option<u32> {
        match retry_attempt % 4 {
            0 | 1 => None,
            filtered_retry => Some(filtered_retry - 2),
        }
    }

    fn get_udp2tcp_obfuscator(
        &self,
        obfuscation_settings: &Udp2TcpObfuscationSettings,
        relay: &Relay,
        _endpoint: &MullvadWireguardEndpoint,
        retry_attempt: u32,
    ) -> Option<(ObfuscatorConfig, Relay)> {
        let udp2tcp_endpoint = if obfuscation_settings.port.is_only() {
            relay
                .obfuscators
                .udp2tcp
                .iter()
                .find(|&candidate| obfuscation_settings.port.matches_eq(&candidate.port))
        } else {
            relay
                .obfuscators
                .udp2tcp
                .get(retry_attempt as usize % relay.obfuscators.udp2tcp.len())
        };
        udp2tcp_endpoint.map(|udp2tcp_endpoint| {
            (
                ObfuscatorConfig::Udp2Tcp {
                    endpoint: SocketAddr::new(relay.ipv4_addr_in.into(), udp2tcp_endpoint.port),
                },
                relay.clone(),
            )
        })
    }

    /// Returns preferred constraints
    #[allow(unused_variables)]
    fn preferred_tunnel_constraints(
        &self,
        retry_attempt: u32,
        location_constraint: &Constraint<LocationConstraint>,
        providers_constraint: &Constraint<Providers>,
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
        if !location_supports_wireguard {
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

    fn preferred_wireguard_port(retry_attempt: u32) -> Constraint<u16> {
        // This ensures that if after the first 2 failed attempts the daemon does not
        // connect, then afterwards 2 of each 4 successive attempts will try to connect
        // on port 53.
        match retry_attempt % 4 {
            0 | 1 => Constraint::Any,
            _ => Constraint::Only(53),
        }
    }

    fn preferred_openvpn_constraints(retry_attempt: u32) -> (Constraint<u16>, TransportProtocol) {
        // Prefer UDP by default. But if that has failed a couple of times, then try TCP port
        // 443, which works for many with UDP problems. After that, just alternate
        // between protocols.
        // If the tunnel type constraint is set OpenVpn, from the 4th attempt onwards, every two
        // retry attempts OpenVpn constraints should be set to TCP as a bridge will be used,
        // and to UDP for the next two attempts. If the tunnel type is specified to be _Any_
        // and on not-Windows, the first two tries are used for WireGuard and don't
        // affect counting here.
        match retry_attempt {
            0 | 1 => (Constraint::Any, TransportProtocol::Udp),
            2 | 3 => (Constraint::Only(443), TransportProtocol::Tcp),
            attempt if attempt % 2 == 0 => (Constraint::Any, TransportProtocol::Udp),
            _ => (Constraint::Any, TransportProtocol::Tcp),
        }
    }

    /// Returns a random relay endpoint if any is matching the given constraints.
    fn get_tunnel_endpoint_internal<T: TunnelMatcher>(
        &self,
        matcher: &RelayMatcher<T>,
    ) -> Result<RelaySelectorResult, Error> {
        let matching_relays: Vec<Relay> = self
            .parsed_relays
            .lock()
            .relays()
            .iter()
            .filter(|relay| relay.active)
            .filter_map(|relay| matcher.filter_matching_relay(relay))
            .collect();

        self.pick_random_relay(&matching_relays)
            .and_then(|selected_relay| {
                let endpoint = matcher.mullvad_endpoint(&selected_relay);
                let addr_in = endpoint
                    .as_ref()
                    .map(|endpoint| endpoint.to_endpoint().address.ip())
                    .unwrap_or(IpAddr::from(selected_relay.ipv4_addr_in));
                log::info!("Selected relay {} at {}", selected_relay.hostname, addr_in);
                endpoint.map(|endpoint| RelaySelectorResult::new(endpoint, selected_relay.clone()))
            })
            .ok_or(Error::NoRelay)
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

    /// Picks a relay using [Self::pick_random_relay_fn], using the `weight` member of each relay
    /// as the weight function.
    fn pick_random_relay<'a>(&self, relays: &'a [Relay]) -> Option<&'a Relay> {
        self.pick_random_relay_fn(relays, |_index, relay| relay.weight)
    }

    /// Pick a random relay from the given slice. Will return `None` if the given slice is empty.
    /// If all of the relays have a weight of 0, one will be picked at random without bias,
    /// otherwise roulette wheel selection will be used to pick only relays with non-zero
    /// weights.
    fn pick_random_relay_fn<'a>(
        &self,
        relays: &'a [Relay],
        weight_fn: impl Fn(usize, &Relay) -> u64,
    ) -> Option<&'a Relay> {
        let total_weight: u64 = relays
            .iter()
            .enumerate()
            .map(|(index, relay)| weight_fn(index, relay))
            .sum();
        let mut rng = rand::thread_rng();
        if total_weight == 0 {
            relays.choose(&mut rng)
        } else {
            // Pick a random number in the range 1..=total_weight. This choses the relay with a
            // non-zero weight.
            let mut i: u64 = rng.gen_range(1, total_weight + 1);
            Some(
                relays
                    .iter()
                    .enumerate()
                    .find(|(index, relay)| {
                        i = i.saturating_sub(weight_fn(*index, relay));
                        i == 0
                    })
                    .map(|(_, relay)| relay)
                    .expect("At least one relay must've had a weight above 0"),
            )
        }
    }

    /// Picks a random bridge from a relay.
    fn pick_random_bridge(&self, relay: &Relay) -> Option<ProxySettings> {
        relay
            .bridges
            .shadowsocks
            .choose(&mut rand::thread_rng())
            .map(|shadowsocks_endpoint| {
                log::info!(
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
                log::error!("Failed to load bundled relays: {}", e);
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

#[derive(Debug)]
pub struct RelaySelectorResult {
    pub exit_relay: Relay,
    pub endpoint: MullvadEndpoint,
    pub entry_relay: Option<Relay>,
}

impl RelaySelectorResult {
    fn new(endpoint: MullvadEndpoint, exit_relay: Relay) -> Self {
        Self {
            exit_relay,
            endpoint,
            entry_relay: None,
        }
    }

    fn wireguard_multihop_endpoint(
        exit_relay: Relay,
        endpoint: MullvadWireguardEndpoint,
        entry: Relay,
    ) -> Self {
        Self {
            exit_relay,
            endpoint: MullvadEndpoint::Wireguard(endpoint),
            entry_relay: Some(entry),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use mullvad_types::{
        relay_constraints::RelayConstraints,
        relay_list::{
            OpenVpnEndpointData, Relay, RelayBridges, RelayListCity, RelayListCountry,
            RelayObfuscators, RelayTunnels, WireguardEndpointData,
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
                                            },
                                        ],
                                    },
                                    bridges: RelayBridges {
                                        shadowsocks: vec![],
                                    },
                                    obfuscators: RelayObfuscators {
                                        udp2tcp: vec![],
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
                                            },
                                        ],
                                    },
                                    bridges: RelayBridges {
                                        shadowsocks: vec![],
                                    },
                                    obfuscators: RelayObfuscators {
                                        udp2tcp: vec![],
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
                                    obfuscators: RelayObfuscators {
                                        udp2tcp: vec![],
                                    },
                                    location: None,
                                },
                                Relay {
                                    hostname: "se11-wireguard-filtered".to_string(),
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
                                                port_ranges: vec![],
                                                ipv4_gateway: "10.64.0.1".parse().unwrap(),
                                                ipv6_gateway: "fc00:bbbb:bbbb:bb01::1".parse().unwrap(),
                                                public_key: PublicKey::from_base64("veGD6/aEY6sMfN3Ls7YWPmNgu3AheO7nQqsFT47YSws=").unwrap(),
                                            },
                                        ],
                                    },
                                    bridges: RelayBridges {
                                        shadowsocks: vec![],
                                    },
                                    obfuscators: RelayObfuscators {
                                        udp2tcp: vec![],
                                    },
                                    location: None,
                                },
                                Relay {
                                    hostname: "se-got-010-filtered".to_string(),
                                    ipv4_addr_in: "185.213.154.69".parse().unwrap(),
                                    ipv6_addr_in: Some("2a03:1b20:5:f011::a10f".parse().unwrap()),
                                    include_in_country: true,
                                    active: true,
                                    owned: true,
                                    provider: "31173".to_string(),
                                    weight: 1,
                                    tunnels: RelayTunnels {
                                        openvpn: vec![OpenVpnEndpointData{
                                            port: 0,
                                            protocol: TransportProtocol::Udp,
                                        }],
                                        wireguard: vec![],
                                    },
                                    bridges: RelayBridges {
                                        shadowsocks: vec![],
                                    },
                                    obfuscators: RelayObfuscators {
                                        udp2tcp: vec![],
                                    },
                                    location: None,
                                }
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
            updater: None,
        }
    }

    #[test]
    fn test_preferred_tunnel_protocol() {
        let relay_selector = new_relay_selector();

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
            relay_selector.preferred_constraints(&relay_constraints, BridgeState::Off, 0);
        assert_eq!(
            preferred.tunnel_protocol,
            Constraint::Only(TunnelType::Wireguard)
        );

        for attempt in 0..10 {
            assert!(relay_selector
                .get_any_tunnel_endpoint(&relay_constraints, BridgeState::Off, attempt)
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
            relay_selector.preferred_constraints(&relay_constraints, BridgeState::Off, 0);
        assert_eq!(
            preferred.tunnel_protocol,
            Constraint::Only(TunnelType::OpenVpn)
        );

        for attempt in 0..10 {
            assert!(relay_selector
                .get_any_tunnel_endpoint(&relay_constraints, BridgeState::Off, attempt)
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
                );
                assert_eq!(
                    preferred.tunnel_protocol,
                    Constraint::Only(TunnelType::OpenVpn)
                );
                match relay_selector.get_any_tunnel_endpoint(
                    &relay_constraints,
                    BridgeState::Off,
                    attempt,
                ) {
                    Ok(result) if matches!(result.endpoint, MullvadEndpoint::OpenVpn(_)) => (),
                    _ => panic!("OpenVPN endpoint was not selected"),
                }
            }
        }
    }

    #[test]
    fn test_wg_entry_hostname_collision() {
        let relay_selector = new_relay_selector();

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

        relay_constraints.wireguard_constraints.use_multihop = true;
        relay_constraints.wireguard_constraints.entry_location = Constraint::Only(location1);

        // The same host cannot be used for entry and exit
        assert!(relay_selector
            .get_tunnel_endpoint(&relay_constraints, BridgeState::Off, 0)
            .is_err());

        relay_constraints.wireguard_constraints.entry_location = Constraint::Only(location2);

        // If the entry and exit differ, this should succeed
        assert!(relay_selector
            .get_tunnel_endpoint(&relay_constraints, BridgeState::Off, 0)
            .is_ok());
    }

    #[test]
    fn test_wg_entry_filter() -> Result<(), String> {
        let relay_selector = new_relay_selector();

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

        relay_constraints.wireguard_constraints.use_multihop = true;
        relay_constraints.wireguard_constraints.entry_location =
            Constraint::Only(location_specific.clone());

        // The exit must not equal the entry
        let exit_relay = relay_selector
            .get_tunnel_endpoint(&relay_constraints, BridgeState::Off, 0)
            .map_err(|error| error.to_string())?
            .exit_relay;

        assert_ne!(exit_relay.hostname, specific_hostname);

        relay_constraints.location = Constraint::Only(location_specific);
        relay_constraints.wireguard_constraints.entry_location = Constraint::Only(location_general);

        // The entry must not equal the exit
        let RelaySelectorResult {
            exit_relay,
            endpoint,
            ..
        } = relay_selector
            .get_tunnel_endpoint(&relay_constraints, BridgeState::Off, 0)
            .map_err(|error| error.to_string())?;

        assert_eq!(exit_relay.hostname, specific_hostname);

        let endpoint = endpoint.unwrap_wireguard();
        assert_eq!(
            exit_relay.ipv4_addr_in,
            endpoint.exit_peer.as_ref().unwrap().endpoint.ip()
        );
        assert_ne!(exit_relay.ipv4_addr_in, endpoint.peer.endpoint.ip());

        Ok(())
    }

    #[test]
    fn test_bridge_constraints() -> Result<(), String> {
        let relay_selector = new_relay_selector();

        let location = LocationConstraint::Hostname(
            "se".to_string(),
            "got".to_string(),
            "se-got-001".to_string(),
        );
        let mut relay_constraints = RelayConstraints {
            location: Constraint::Only(location.clone()),
            tunnel_protocol: Constraint::Any,
            ..RelayConstraints::default()
        };
        relay_constraints.openvpn_constraints.port = Constraint::Only(TransportPort {
            protocol: TransportProtocol::Udp,
            port: Constraint::Any,
        });

        let preferred =
            relay_selector.preferred_constraints(&relay_constraints, BridgeState::On, 0);
        assert_eq!(
            preferred.tunnel_protocol,
            Constraint::Only(TunnelType::OpenVpn)
        );
        // NOTE: TCP is preferred for bridges
        assert_eq!(
            preferred.openvpn_constraints.port,
            Constraint::Only(TransportPort {
                protocol: TransportProtocol::Tcp,
                port: Constraint::Any,
            })
        );

        // Ignore bridge state where WireGuard is used
        let location = LocationConstraint::Hostname(
            "se".to_string(),
            "got".to_string(),
            "se10-wireguard".to_string(),
        );
        let relay_constraints = RelayConstraints {
            location: Constraint::Only(location),
            tunnel_protocol: Constraint::Any,
            ..RelayConstraints::default()
        };
        let preferred =
            relay_selector.preferred_constraints(&relay_constraints, BridgeState::On, 0);
        assert_eq!(
            preferred.tunnel_protocol,
            Constraint::Only(TunnelType::Wireguard)
        );

        // Handle bridge setting when falling back on OpenVPN
        let mut relay_constraints = RelayConstraints {
            location: Constraint::Any,
            tunnel_protocol: Constraint::Any,
            ..RelayConstraints::default()
        };
        relay_constraints.openvpn_constraints.port = Constraint::Only(TransportPort {
            protocol: TransportProtocol::Udp,
            port: Constraint::Any,
        });
        #[cfg(all(unix, not(target_os = "android")))]
        {
            let preferred =
                relay_selector.preferred_constraints(&relay_constraints, BridgeState::On, 0);
            assert_eq!(
                preferred.tunnel_protocol,
                Constraint::Only(TunnelType::Wireguard)
            );
        }
        let preferred =
            relay_selector.preferred_constraints(&relay_constraints, BridgeState::On, 2);
        assert_eq!(
            preferred.tunnel_protocol,
            Constraint::Only(TunnelType::OpenVpn)
        );
        assert_eq!(
            preferred.openvpn_constraints.port,
            Constraint::Only(TransportPort {
                protocol: TransportProtocol::Tcp,
                port: Constraint::Any,
            })
        );

        Ok(())
    }

    #[test]
    fn test_selecting_any_relay_will_consider_multihop() {
        let relay_constraints = RelayConstraints {
            wireguard_constraints: WireguardConstraints {
                use_multihop: true,
                ..WireguardConstraints::default()
            },
            // This has to be explicit otherwise Android will chose WireGuard when default
            // constructing.
            tunnel_protocol: Constraint::Any,
            ..RelayConstraints::default()
        };

        let relay_selector = new_relay_selector();

        let result = relay_selector.get_tunnel_endpoint(&relay_constraints, BridgeState::Off, 0)
            .expect("Failed to get relay when tunnel constraints are set to Any and retrying the selection");
        // Windows will ignore WireGuard until WireGuard is supported well enough
        // TODO: Remove this caveat once Windows defaults to using WireGuard
        #[cfg(target_os = "windows")]
        assert!(
            matches!(result.endpoint, MullvadEndpoint::OpenVpn(_)) && result.entry_relay.is_none()
        );

        #[cfg(not(target_os = "windows"))]
        assert!(
            matches!(result.endpoint, MullvadEndpoint::Wireguard(_))
                && result.entry_relay.is_some()
        );
    }

    const WIREGUARD_MULTIHOP_CONSTRAINTS: RelayConstraints = RelayConstraints {
        location: Constraint::Any,
        providers: Constraint::Any,
        wireguard_constraints: WireguardConstraints {
            use_multihop: true,
            port: Constraint::Any,
            ip_version: Constraint::Any,
            entry_location: Constraint::Any,
        },
        tunnel_protocol: Constraint::Only(TunnelType::Wireguard),
        openvpn_constraints: OpenVpnConstraints {
            port: Constraint::Any,
        },
    };

    const WIREGUARD_SINGLEHOP_CONSTRAINTS: RelayConstraints = RelayConstraints {
        location: Constraint::Any,
        providers: Constraint::Any,
        wireguard_constraints: WireguardConstraints {
            use_multihop: false,
            port: Constraint::Any,
            ip_version: Constraint::Any,
            entry_location: Constraint::Any,
        },
        tunnel_protocol: Constraint::Only(TunnelType::Wireguard),
        openvpn_constraints: OpenVpnConstraints {
            port: Constraint::Any,
        },
    };

    #[test]
    fn test_selecting_wireguard_location_will_consider_multihop() {
        let relay_selector = new_relay_selector();

        let result = relay_selector.get_tunnel_endpoint(&WIREGUARD_MULTIHOP_CONSTRAINTS, BridgeState::Off, 0)
            .expect("Failed to get relay when tunnel constraints are set to default WireGuard multihop constraints");

        assert!(result.entry_relay.is_some());
        // TODO: Verify that neither endpoint is using obfuscation for retry attempt 0
    }

    #[test]
    fn test_selecting_wg_endpoint_with_udp2tcp_obfuscation() {
        let relay_selector = new_relay_selector();

        let result = relay_selector.get_tunnel_endpoint(&WIREGUARD_SINGLEHOP_CONSTRAINTS, BridgeState::Off, 0)
            .expect("Failed to get relay when tunnel constraints are set to default WireGuard constraints");

        assert!(result.entry_relay.is_none());
        assert!(matches!(result.endpoint, MullvadEndpoint::Wireguard { .. }));

        let obfs_settings = ObfuscationSettings {
            selected_obfuscation: SelectedObfuscation::Udp2Tcp,
            ..ObfuscationSettings::default()
        };

        let (obfs_config, _obfs_relay) = relay_selector
            .get_obfuscator(
                &obfs_settings,
                &result.exit_relay,
                result.endpoint.unwrap_wireguard(),
                0,
            )
            .unwrap();

        assert!(matches!(obfs_config, ObfuscatorConfig::Udp2Tcp { .. }));
    }

    #[test]
    fn test_selecting_wg_endpoint_with_auto_obfuscation() {
        let relay_selector = new_relay_selector();

        let result = relay_selector.get_tunnel_endpoint(&WIREGUARD_SINGLEHOP_CONSTRAINTS, BridgeState::Off, 0)
            .expect("Failed to get relay when tunnel constraints are set to default WireGuard constraints");

        assert!(result.entry_relay.is_none());
        assert!(matches!(result.endpoint, MullvadEndpoint::Wireguard { .. }));

        let obfs_settings = ObfuscationSettings {
            selected_obfuscation: SelectedObfuscation::Auto,
            ..ObfuscationSettings::default()
        };

        assert!(relay_selector
            .get_obfuscator(
                &obfs_settings,
                &result.exit_relay,
                result.endpoint.unwrap_wireguard(),
                0,
            )
            .is_none());

        assert!(relay_selector
            .get_obfuscator(
                &obfs_settings,
                &result.exit_relay,
                result.endpoint.unwrap_wireguard(),
                1,
            )
            .is_none());

        assert!(relay_selector
            .get_obfuscator(
                &obfs_settings,
                &result.exit_relay,
                result.endpoint.unwrap_wireguard(),
                2,
            )
            .is_some());
    }

    #[test]
    fn test_selected_endpoints_use_correct_port_ranges() {
        let relay_selector = new_relay_selector();

        const TCP2UDP_PORTS: [u16; 3] = [80, 443, 5001];

        let obfs_settings = ObfuscationSettings {
            selected_obfuscation: SelectedObfuscation::Udp2Tcp,
            ..ObfuscationSettings::default()
        };

        for attempt in 0..1000 {
            let result = relay_selector
                .get_tunnel_endpoint(&WIREGUARD_SINGLEHOP_CONSTRAINTS, BridgeState::Off, attempt)
                .expect("Failed to select a WireGuard relay");
            assert!(result.entry_relay.is_none());

            let (obfs_config, _obfs_relay) = relay_selector
                .get_obfuscator(
                    &obfs_settings,
                    &result.exit_relay,
                    result.endpoint.unwrap_wireguard(),
                    attempt,
                )
                .expect("Failed to get Tcp2Udp endpoint");

            assert!(matches!(obfs_config, ObfuscatorConfig::Udp2Tcp { .. }));

            let ObfuscatorConfig::Udp2Tcp { endpoint } = obfs_config;
            assert!(TCP2UDP_PORTS.contains(&endpoint.port()));
        }
    }

    #[test]
    fn test_filtering_invalid_endpoint_relays() {
        let relay_selector = new_relay_selector();
        let mut constraints = RelayConstraints {
            location: Constraint::Only(LocationConstraint::Hostname(
                "se".to_string(),
                "got".to_string(),
                "se11-wireguard-filtered".to_string(),
            )),
            ..RelayConstraints::default()
        };
        relay_selector
            .get_tunnel_endpoint(&constraints, BridgeState::Off, 0)
            .expect_err("Successfully selected a relay that should be filtered");

        constraints.location = Constraint::Only(LocationConstraint::Hostname(
            "se".to_string(),
            "got".to_string(),
            "se-got-010-filtered".to_string(),
        ));

        relay_selector
            .get_tunnel_endpoint(&constraints, BridgeState::Off, 0)
            .expect_err("Successfully selected a relay that should be filtered");
    }
}

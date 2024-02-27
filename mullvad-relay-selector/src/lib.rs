//! When changing relay selection, please verify if `docs/relay-selector.md` needs to be
//! updated as well.

use chrono::{DateTime, Local};
use ipnetwork::IpNetwork;
use mullvad_types::{
    constraints::{Constraint, Match, Set},
    custom_list::CustomListsSettings,
    endpoint::{MullvadEndpoint, MullvadWireguardEndpoint},
    location::{Coordinates, Location},
    relay_constraints::{
        BridgeSettings, BridgeState, InternalBridgeConstraints, LocationConstraint,
        MissingCustomBridgeSettings, ObfuscationSettings, OpenVpnConstraints, Ownership, Providers,
        RelayConstraints, RelayConstraintsFormatter, RelayOverride, RelaySettings,
        ResolvedBridgeSettings, ResolvedLocationConstraint, SelectedObfuscation, TransportPort,
        Udp2TcpObfuscationSettings,
    },
    relay_list::{BridgeEndpointData, Relay, RelayEndpointData, RelayList},
    settings::Settings,
    CustomTunnelEndpoint,
};
use rand::{seq::SliceRandom, Rng};
use std::{
    collections::HashMap,
    io,
    net::{IpAddr, SocketAddr},
    path::Path,
    sync::{Arc, Mutex, MutexGuard},
    time::{self, SystemTime},
};
use talpid_types::{
    net::{
        obfuscation::ObfuscatorConfig, proxy::CustomProxy, wireguard, IpVersion, TransportProtocol,
        TunnelType,
    },
    ErrorExt,
};

use matcher::{BridgeMatcher, EndpointMatcher, OpenVpnMatcher, RelayMatcher, WireguardMatcher};

mod matcher;

const DATE_TIME_FORMAT_STR: &str = "%Y-%m-%d %H:%M:%S%.3f";

const WIREGUARD_EXIT_PORT: Constraint<u16> = Constraint::Only(51820);
const WIREGUARD_EXIT_IP_VERSION: Constraint<IpVersion> = Constraint::Only(IpVersion::V4);

const UDP2TCP_PORTS: [u16; 2] = [80, 5001];

/// Minimum number of bridges to keep for selection when filtering by distance.
const MIN_BRIDGE_COUNT: usize = 5;

/// Max distance of bridges to consider for selection (km).
const MAX_BRIDGE_DISTANCE: f64 = 1500f64;

#[derive(err_derive::Error, Debug)]
#[error(no_from)]
pub enum Error {
    #[error(display = "Failed to open relay cache file")]
    OpenRelayCache(#[error(source)] io::Error),

    #[error(display = "Failed to write relay cache file to disk")]
    WriteRelayCache(#[error(source)] io::Error),

    #[error(display = "No relays matching current constraints")]
    NoRelay,

    #[error(display = "No bridges matching current constraints")]
    NoBridge,

    #[error(display = "No obfuscators matching current constraints")]
    NoObfuscator,

    #[error(display = "Failure in serialization of the relay list")]
    Serialize(#[error(source)] serde_json::Error),

    #[error(display = "Downloader already shut down")]
    DownloaderShutDown,

    #[error(display = "Invalid bridge settings")]
    InvalidBridgeSettings(#[error(source)] MissingCustomBridgeSettings),
}

struct ParsedRelays {
    last_updated: SystemTime,
    parsed_list: RelayList,
    original_list: RelayList,
    overrides: Vec<RelayOverride>,
}

impl ParsedRelays {
    /// Return a flat iterator with all relays
    pub fn relays(&self) -> impl Iterator<Item = &Relay> + Clone + '_ {
        self.parsed_list.relays()
    }

    pub fn update(&mut self, new_relays: RelayList) {
        *self = Self::from_relay_list(new_relays, SystemTime::now(), &self.overrides);

        log::info!(
            "Updated relay inventory has {} relays",
            self.relays().count()
        );
    }

    pub fn last_updated(&self) -> SystemTime {
        self.last_updated
    }

    pub fn etag(&self) -> Option<String> {
        self.parsed_list.etag.clone()
    }

    fn set_overrides(&mut self, new_overrides: &[RelayOverride]) {
        self.parsed_list = Self::parse_relay_list(&self.original_list, new_overrides);
        self.overrides = new_overrides.to_vec();
    }

    fn empty() -> Self {
        ParsedRelays {
            last_updated: time::UNIX_EPOCH,
            parsed_list: RelayList::empty(),
            original_list: RelayList::empty(),
            overrides: vec![],
        }
    }

    /// Try to read the relays from disk, preferring the newer ones.
    fn from_file(
        cache_path: impl AsRef<Path>,
        resource_path: impl AsRef<Path>,
        overrides: &[RelayOverride],
    ) -> Result<Self, Error> {
        // prefer the resource path's relay list if the cached one doesn't exist or was modified
        // before the resource one was created.
        let cached_relays = Self::from_file_inner(cache_path, overrides);
        let bundled_relays = match Self::from_file_inner(resource_path, overrides) {
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

    fn from_file_inner(path: impl AsRef<Path>, overrides: &[RelayOverride]) -> Result<Self, Error> {
        log::debug!("Reading relays from {}", path.as_ref().display());
        let (last_modified, file) =
            Self::open_file(path.as_ref()).map_err(Error::OpenRelayCache)?;
        let relay_list =
            serde_json::from_reader(io::BufReader::new(file)).map_err(Error::Serialize)?;

        Ok(Self::from_relay_list(relay_list, last_modified, overrides))
    }

    fn open_file(path: &Path) -> io::Result<(SystemTime, std::fs::File)> {
        let file = std::fs::File::open(path)?;
        let last_modified = file.metadata()?.modified()?;
        Ok((last_modified, file))
    }

    fn from_relay_list(
        relay_list: RelayList,
        last_updated: SystemTime,
        overrides: &[RelayOverride],
    ) -> Self {
        ParsedRelays {
            last_updated,
            parsed_list: Self::parse_relay_list(&relay_list, overrides),
            original_list: relay_list,
            overrides: overrides.to_vec(),
        }
    }

    fn parse_relay_list(relay_list: &RelayList, overrides: &[RelayOverride]) -> RelayList {
        let mut remaining_overrides = HashMap::new();
        for relay_override in overrides {
            remaining_overrides.insert(
                relay_override.hostname.to_owned(),
                relay_override.to_owned(),
            );
        }

        let mut parsed_list = relay_list.clone();

        // Append data for obfuscation protocols ourselves, since the API does not provide it.
        if parsed_list.wireguard.udp2tcp_ports.is_empty() {
            parsed_list.wireguard.udp2tcp_ports.extend(UDP2TCP_PORTS);
        }

        // Add location and override relay data
        for country in &mut parsed_list.countries {
            for city in &mut country.cities {
                for relay in &mut city.relays {
                    // Append location data
                    relay.location = Some(Location {
                        country: country.name.clone(),
                        country_code: country.code.clone(),
                        city: city.name.clone(),
                        city_code: city.code.clone(),
                        latitude: city.latitude,
                        longitude: city.longitude,
                    });

                    // Append overrides
                    if let Some(overrides) = remaining_overrides.remove(&relay.hostname) {
                        overrides.apply_to_relay(relay);
                    }
                }
            }
        }

        parsed_list
    }
}

#[derive(Clone)]
pub struct SelectorConfig {
    pub relay_settings: RelaySettings,
    pub bridge_state: BridgeState,
    pub bridge_settings: BridgeSettings,
    pub obfuscation_settings: ObfuscationSettings,
    pub custom_lists: CustomListsSettings,
    pub relay_overrides: Vec<RelayOverride>,
}

impl Default for SelectorConfig {
    fn default() -> Self {
        let default_settings = Settings::default();
        SelectorConfig {
            relay_settings: default_settings.relay_settings,
            bridge_settings: default_settings.bridge_settings,
            obfuscation_settings: default_settings.obfuscation_settings,
            bridge_state: default_settings.bridge_state,
            custom_lists: default_settings.custom_lists,
            relay_overrides: default_settings.relay_overrides,
        }
    }
}

#[derive(Clone)]
pub struct RelaySelector {
    config: Arc<Mutex<SelectorConfig>>,
    parsed_relays: Arc<Mutex<ParsedRelays>>,
}

impl RelaySelector {
    /// Returns a new `RelaySelector` backed by relays cached on disk.
    pub fn new(
        config: SelectorConfig,
        resource_path: impl AsRef<Path>,
        cache_path: impl AsRef<Path>,
    ) -> Self {
        let unsynchronized_parsed_relays =
            ParsedRelays::from_file(&cache_path, &resource_path, &config.relay_overrides)
                .unwrap_or_else(|error| {
                    log::error!(
                        "{}",
                        error.display_chain_with_msg("Unable to load cached and bundled relays")
                    );
                    ParsedRelays::empty()
                });
        log::info!(
            "Initialized with {} cached relays from {}",
            unsynchronized_parsed_relays.relays().count(),
            DateTime::<Local>::from(unsynchronized_parsed_relays.last_updated())
                .format(DATE_TIME_FORMAT_STR)
        );

        RelaySelector {
            config: Arc::new(Mutex::new(config)),
            parsed_relays: Arc::new(Mutex::new(unsynchronized_parsed_relays)),
        }
    }

    pub fn from_list(config: SelectorConfig, relay_list: RelayList) -> Self {
        RelaySelector {
            parsed_relays: Arc::new(Mutex::new(ParsedRelays::from_relay_list(
                relay_list,
                SystemTime::now(),
                &config.relay_overrides,
            ))),
            config: Arc::new(Mutex::new(config)),
        }
    }

    pub fn set_config(&mut self, config: SelectorConfig) {
        self.set_overrides(&config.relay_overrides);
        let mut config_mutex = self.config.lock().unwrap();
        *config_mutex = config;
    }

    pub fn set_relays(&self, relays: RelayList) {
        let mut parsed_relays = self.parsed_relays.lock().unwrap();
        parsed_relays.update(relays);
    }

    fn set_overrides(&mut self, relay_overrides: &[RelayOverride]) {
        let mut parsed_relays = self.parsed_relays.lock().unwrap();
        parsed_relays.set_overrides(relay_overrides);
    }

    /// Returns all countries and cities. The cities in the object returned does not have any
    /// relays in them.
    pub fn get_relays(&mut self) -> RelayList {
        let parsed_relays = self.parsed_relays.lock().unwrap();
        parsed_relays.original_list.clone()
    }

    pub fn etag(&self) -> Option<String> {
        self.parsed_relays.lock().unwrap().etag()
    }

    pub fn last_updated(&self) -> SystemTime {
        self.parsed_relays.lock().unwrap().last_updated()
    }

    /// Returns a random relay and relay endpoint matching the current constraints.
    pub fn get_relay(
        &self,
        retry_attempt: u32,
    ) -> Result<
        (
            SelectedRelay,
            Option<SelectedBridge>,
            Option<SelectedObfuscator>,
        ),
        Error,
    > {
        let config_mutex = self.config.lock().unwrap();
        match &config_mutex.relay_settings {
            RelaySettings::CustomTunnelEndpoint(custom_relay) => {
                Ok((SelectedRelay::Custom(custom_relay.clone()), None, None))
            }
            RelaySettings::Normal(constraints) => {
                let relay = self.get_tunnel_endpoint(
                    constraints,
                    config_mutex.bridge_state,
                    retry_attempt,
                    &config_mutex.custom_lists,
                )?;
                let bridge = match relay.endpoint {
                    MullvadEndpoint::OpenVpn(endpoint)
                        if endpoint.protocol == TransportProtocol::Tcp =>
                    {
                        let location = relay
                            .exit_relay
                            .location
                            .as_ref()
                            .expect("Relay has no location set");
                        self.get_bridge_for(
                            &config_mutex,
                            location,
                            retry_attempt,
                            &config_mutex.custom_lists,
                        )?
                    }
                    _ => None,
                };
                let obfuscator = match relay.endpoint {
                    MullvadEndpoint::Wireguard(ref endpoint) => {
                        let obfuscator_relay =
                            relay.entry_relay.as_ref().unwrap_or(&relay.exit_relay);
                        self.get_obfuscator_inner(
                            &config_mutex,
                            obfuscator_relay,
                            endpoint,
                            retry_attempt,
                        )?
                    }
                    _ => None,
                };
                Ok((SelectedRelay::Normal(relay), bridge, obfuscator))
            }
        }
    }

    /// Returns a random relay and relay endpoint matching the given constraints and with
    /// preferences applied.
    #[cfg_attr(target_os = "android", allow(unused_variables))]
    fn get_tunnel_endpoint(
        &self,
        relay_constraints: &RelayConstraints,
        bridge_state: BridgeState,
        retry_attempt: u32,
        custom_lists: &CustomListsSettings,
    ) -> Result<NormalSelectedRelay, Error> {
        #[cfg(target_os = "android")]
        {
            self.get_wireguard_endpoint(relay_constraints, retry_attempt, custom_lists)
        }

        #[cfg(not(target_os = "android"))]
        match relay_constraints.tunnel_protocol {
            Constraint::Only(TunnelType::OpenVpn) => self.get_openvpn_endpoint(
                relay_constraints,
                bridge_state,
                retry_attempt,
                custom_lists,
            ),

            Constraint::Only(TunnelType::Wireguard) => {
                self.get_wireguard_endpoint(relay_constraints, retry_attempt, custom_lists)
            }
            Constraint::Any => self.get_any_tunnel_endpoint(
                relay_constraints,
                bridge_state,
                retry_attempt,
                custom_lists,
            ),
        }
    }

    /// Returns the average location of relays that match the given constraints.
    /// This returns none if the location is `any` or if no relays match the constraints.
    pub fn get_relay_midpoint(
        &self,
        relay_constraints: &RelayConstraints,
        custom_lists: &CustomListsSettings,
    ) -> Option<Coordinates> {
        if relay_constraints.location.is_any() {
            return None;
        }

        let (openvpn_data, wireguard_data) = {
            let relays = self.parsed_relays.lock().unwrap();
            (
                relays.parsed_list.openvpn.clone(),
                relays.parsed_list.wireguard.clone(),
            )
        };

        let matcher = RelayMatcher::new(
            relay_constraints.clone(),
            openvpn_data,
            wireguard_data,
            custom_lists,
        );

        let mut matching_locations: Vec<Location> = {
            let parsed_relays = self.parsed_relays.lock().unwrap();
            matcher
                .filter_matching_relay_list(parsed_relays.relays())
                .into_iter()
                .filter_map(|relay| relay.location)
                .collect()
        };
        matching_locations.dedup_by(|a, b| a.has_same_city(b));

        if matching_locations.is_empty() {
            return None;
        }
        Some(Coordinates::midpoint(&matching_locations))
    }

    /// Returns an OpenVpn endpoint, should only ever be used when the user has specified the tunnel
    /// protocol as only OpenVPN.
    #[cfg_attr(target_os = "android", allow(dead_code))]
    fn get_openvpn_endpoint(
        &self,
        relay_constraints: &RelayConstraints,
        bridge_state: BridgeState,
        retry_attempt: u32,
        custom_lists: &CustomListsSettings,
    ) -> Result<NormalSelectedRelay, Error> {
        let mut relay_matcher = RelayMatcher {
            locations: ResolvedLocationConstraint::from_constraint(
                relay_constraints.location.clone(),
                custom_lists,
            ),
            providers: relay_constraints.providers.clone(),
            ownership: relay_constraints.ownership,
            endpoint_matcher: OpenVpnMatcher::new(relay_constraints.openvpn_constraints, {
                let parsed_relays = self.parsed_relays.lock().unwrap();
                parsed_relays.parsed_list.openvpn.clone()
            }),
        };

        if relay_matcher.endpoint_matcher.constraints.port.is_any()
            && bridge_state == BridgeState::On
        {
            relay_matcher.endpoint_matcher.constraints.port = Constraint::Only(TransportPort {
                protocol: TransportProtocol::Tcp,
                port: Constraint::Any,
            });

            return self.get_tunnel_endpoint_internal(&relay_matcher);
        }

        let mut preferred_relay_matcher = relay_matcher.clone();

        let (preferred_port, preferred_protocol) =
            Self::preferred_openvpn_constraints(retry_attempt);
        let should_try_preferred =
            match &mut preferred_relay_matcher.endpoint_matcher.constraints.port {
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
        exit_locations: Constraint<LocationConstraint>,
        custom_lists: &CustomListsSettings,
    ) -> Result<NormalSelectedRelay, Error> {
        let mut exit_matcher = RelayMatcher {
            locations: ResolvedLocationConstraint::from_constraint(exit_locations, custom_lists),
            providers: entry_matcher.providers.clone(),
            ownership: entry_matcher.ownership,
            endpoint_matcher: self.wireguard_exit_matcher(),
        };

        let (exit_relay, entry_relay, exit_endpoint, mut entry_endpoint) =
            if entry_matcher.locations.is_subset(&exit_matcher.locations) {
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
        let result = NormalSelectedRelay::wireguard_multihop_endpoint(
            exit_relay,
            entry_endpoint,
            entry_relay,
        );
        Ok(result)
    }

    /// Returns a WireGuard endpoint, should only ever be used when the user has specified the
    /// tunnel protocol as only WireGuard.
    fn get_wireguard_endpoint(
        &self,
        relay_constraints: &RelayConstraints,
        retry_attempt: u32,
        custom_lists: &CustomListsSettings,
    ) -> Result<NormalSelectedRelay, Error> {
        let wg_endpoint_data = {
            let parsed_relays = self.parsed_relays.lock().unwrap();
            parsed_relays.parsed_list.wireguard.clone()
        };

        // NOTE: If not using multihop then `location` is set as the only location constraint.
        // If using multihop then location is the exit constraint and
        // `wireguard_constraints.entry_location` is set as the entry location constraint.
        if !relay_constraints.wireguard_constraints.use_multihop {
            let relay_matcher = RelayMatcher {
                locations: ResolvedLocationConstraint::from_constraint(
                    relay_constraints.location.clone(),
                    custom_lists,
                ),
                providers: relay_constraints.providers.clone(),
                ownership: relay_constraints.ownership,
                endpoint_matcher: WireguardMatcher::new(
                    relay_constraints.wireguard_constraints.clone(),
                    wg_endpoint_data,
                ),
            };

            // Nightly clippy seems wrong about this being a redundant clone
            #[allow(clippy::redundant_clone)]
            let mut preferred_matcher: RelayMatcher<WireguardMatcher> = relay_matcher.clone();
            preferred_matcher.endpoint_matcher.port = preferred_matcher
                .endpoint_matcher
                .port
                .or(Self::preferred_wireguard_port(retry_attempt));

            self.get_tunnel_endpoint_internal(&preferred_matcher)
                .or_else(|_| self.get_tunnel_endpoint_internal(&relay_matcher))
        } else {
            let mut entry_relay_matcher = RelayMatcher {
                locations: ResolvedLocationConstraint::from_constraint(
                    relay_constraints
                        .wireguard_constraints
                        .entry_location
                        .clone(),
                    custom_lists,
                ),
                providers: relay_constraints.providers.clone(),
                ownership: relay_constraints.ownership,
                endpoint_matcher: WireguardMatcher::new(
                    relay_constraints.wireguard_constraints.clone(),
                    wg_endpoint_data,
                ),
            };
            entry_relay_matcher.endpoint_matcher.port = entry_relay_matcher
                .endpoint_matcher
                .port
                .or(Self::preferred_wireguard_port(retry_attempt));

            self.get_wireguard_multi_hop_endpoint(
                entry_relay_matcher,
                relay_constraints.location.clone(),
                custom_lists,
            )
        }
    }

    /// Like [Self::get_tunnel_endpoint_internal] but also selects an entry endpoint if applicable.
    #[cfg_attr(target_os = "android", allow(dead_code))]
    fn get_multihop_tunnel_endpoint_internal(
        &self,
        relay_constraints: &RelayConstraints,
        custom_lists: &CustomListsSettings,
    ) -> Result<NormalSelectedRelay, Error> {
        let (openvpn_data, wireguard_data) = {
            let relays = self.parsed_relays.lock().unwrap();
            (
                relays.parsed_list.openvpn.clone(),
                relays.parsed_list.wireguard.clone(),
            )
        };
        let mut matcher = RelayMatcher::new(
            relay_constraints.clone(),
            openvpn_data,
            wireguard_data,
            custom_lists,
        );

        let mut selected_entry_relay = None;
        let mut selected_entry_endpoint = None;
        let mut entry_matcher = RelayMatcher {
            locations: ResolvedLocationConstraint::from_constraint(
                relay_constraints
                    .wireguard_constraints
                    .entry_location
                    .clone(),
                custom_lists,
            ),
            providers: relay_constraints.providers.clone(),
            ownership: relay_constraints.ownership,
            endpoint_matcher: matcher.endpoint_matcher.clone(),
        }
        .into_wireguard_matcher();

        // Pick the entry relay first if its location constraint is a subset of the exit location.
        if relay_constraints.wireguard_constraints.use_multihop {
            matcher.endpoint_matcher.wireguard = self.wireguard_exit_matcher();
            if entry_matcher.locations.is_subset(&matcher.locations) {
                if let Ok((entry_relay, entry_endpoint)) = self.get_entry_endpoint(&entry_matcher) {
                    matcher.endpoint_matcher.wireguard.peer = Some(entry_relay.clone());
                    selected_entry_relay = Some(entry_relay);
                    selected_entry_endpoint = Some(entry_endpoint);
                }
            }
        }

        let mut selected_relay = self.get_tunnel_endpoint_internal(&matcher)?;

        // Pick the entry relay last if its location constraint is NOT a subset of the exit
        // location.
        if matches!(selected_relay.endpoint, MullvadEndpoint::Wireguard(..))
            && relay_constraints.wireguard_constraints.use_multihop
        {
            if !entry_matcher.locations.is_subset(&matcher.locations) {
                entry_matcher.endpoint_matcher.peer = Some(selected_relay.exit_relay.clone());
                if let Ok((entry_relay, entry_endpoint)) = self.get_entry_endpoint(&entry_matcher) {
                    selected_entry_relay = Some(entry_relay);
                    selected_entry_endpoint = Some(entry_endpoint);
                }
            }

            match (selected_entry_endpoint, selected_entry_relay) {
                (Some(mut entry_endpoint), Some(entry_relay)) => {
                    Self::set_entry_peers(
                        &selected_relay.endpoint.unwrap_wireguard().peer,
                        &mut entry_endpoint,
                    );

                    log::info!(
                        "Selected entry relay {} at {} going through {} at {}",
                        entry_relay.hostname,
                        entry_endpoint.peer.endpoint.ip(),
                        selected_relay.exit_relay.hostname,
                        selected_relay.endpoint.to_endpoint().address.ip(),
                    );

                    selected_relay.endpoint = MullvadEndpoint::Wireguard(entry_endpoint);
                    selected_relay.entry_relay = Some(entry_relay);
                }
                _ => return Err(Error::NoRelay),
            }
        }

        Ok(selected_relay)
    }

    /// Returns a tunnel endpoint of any type, should only be used when the user hasn't specified a
    /// tunnel protocol.
    #[cfg_attr(target_os = "android", allow(dead_code))]
    fn get_any_tunnel_endpoint(
        &self,
        relay_constraints: &RelayConstraints,
        bridge_state: BridgeState,
        retry_attempt: u32,
        custom_lists: &CustomListsSettings,
    ) -> Result<NormalSelectedRelay, Error> {
        let preferred_constraints = self.preferred_constraints(
            relay_constraints,
            bridge_state,
            retry_attempt,
            custom_lists,
        );

        if let Ok(result) =
            self.get_multihop_tunnel_endpoint_internal(&preferred_constraints, custom_lists)
        {
            log::debug!(
                "Relay matched on highest preference for retry attempt {}",
                retry_attempt
            );
            Ok(result)
        } else if let Ok(result) =
            self.get_multihop_tunnel_endpoint_internal(relay_constraints, custom_lists)
        {
            log::debug!(
                "Relay matched on second preference for retry attempt {}",
                retry_attempt
            );
            Ok(result)
        } else {
            log::warn!(
                "No relays matching constraints: {}",
                RelayConstraintsFormatter {
                    constraints: relay_constraints,
                    custom_lists,
                }
            );
            Err(Error::NoRelay)
        }
    }

    // This function ignores the tunnel type constraint on purpose.
    #[cfg_attr(target_os = "android", allow(dead_code))]
    fn preferred_constraints(
        &self,
        original_constraints: &RelayConstraints,
        bridge_state: BridgeState,
        retry_attempt: u32,
        custom_lists: &CustomListsSettings,
    ) -> RelayConstraints {
        let location = ResolvedLocationConstraint::from_constraint(
            original_constraints.location.clone(),
            custom_lists,
        );
        let (preferred_port, preferred_protocol, preferred_tunnel) = self
            .preferred_tunnel_constraints_for_location(
                retry_attempt,
                &location,
                &original_constraints.providers,
                original_constraints.ownership,
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

        relay_constraints
    }

    fn get_entry_endpoint(
        &self,
        matcher: &RelayMatcher<WireguardMatcher>,
    ) -> Result<(Relay, MullvadWireguardEndpoint), Error> {
        let matching_relays: Vec<Relay> = {
            let parsed_relays = self.parsed_relays.lock().unwrap();
            matcher
                .filter_matching_relay_list(parsed_relays.relays())
                .into_iter()
                .collect()
        };

        let relay = self
            .pick_random_relay(&matching_relays)
            .cloned()
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

    fn get_bridge_for(
        &self,
        config: &MutexGuard<'_, SelectorConfig>,
        location: &mullvad_types::location::Location,
        retry_attempt: u32,
        custom_lists: &CustomListsSettings,
    ) -> Result<Option<SelectedBridge>, Error> {
        match config
            .bridge_settings
            .resolve()
            .map_err(Error::InvalidBridgeSettings)?
        {
            ResolvedBridgeSettings::Normal(settings) => {
                let bridge_constraints = InternalBridgeConstraints {
                    location: settings.location.clone(),
                    providers: settings.providers.clone(),
                    ownership: settings.ownership,
                    // FIXME: This is temporary while talpid-core only supports TCP proxies
                    transport_protocol: Constraint::Only(TransportProtocol::Tcp),
                };
                match config.bridge_state {
                    BridgeState::On => {
                        let (settings, relay) = self
                            .get_proxy_settings(&bridge_constraints, Some(location), custom_lists)
                            .ok_or(Error::NoBridge)?;
                        Ok(Some(SelectedBridge::Normal(NormalSelectedBridge {
                            settings,
                            relay,
                        })))
                    }
                    BridgeState::Auto if Self::should_use_bridge(retry_attempt) => Ok(self
                        .get_proxy_settings(&bridge_constraints, Some(location), custom_lists)
                        .map(|(settings, relay)| {
                            SelectedBridge::Normal(NormalSelectedBridge { settings, relay })
                        })),
                    BridgeState::Auto | BridgeState::Off => Ok(None),
                }
            }
            ResolvedBridgeSettings::Custom(bridge_settings) => match config.bridge_state {
                BridgeState::On => Ok(Some(SelectedBridge::Custom(bridge_settings.clone()))),
                BridgeState::Auto if Self::should_use_bridge(retry_attempt) => {
                    Ok(Some(SelectedBridge::Custom(bridge_settings.clone())))
                }
                BridgeState::Auto | BridgeState::Off => Ok(None),
            },
        }
    }

    /// Returns a non-custom bridge based on the relay and bridge constraints, ignoring the bridge
    /// state.
    pub fn get_bridge_forced(&self) -> Option<CustomProxy> {
        let config = self.config.lock().unwrap();
        // let relay_settings = {
        //     let config = self.config.lock().unwrap();
        //     config.relay_settings.clone()
        // };

        let near_location = match &config.relay_settings {
            RelaySettings::Normal(settings) => {
                let custom_lists = {
                    // let config = self.config.lock().unwrap();
                    config.custom_lists.clone()
                };
                self.get_relay_midpoint(settings, &custom_lists)
            }
            _ => None,
        };
        let bridge_settings = &config.bridge_settings;
        let constraints = match bridge_settings.resolve() {
            Ok(ResolvedBridgeSettings::Normal(settings)) => InternalBridgeConstraints {
                location: settings.location.clone(),
                providers: settings.providers.clone(),
                ownership: settings.ownership,
                transport_protocol: Constraint::Only(TransportProtocol::Tcp),
            },
            _ => InternalBridgeConstraints {
                location: Constraint::Any,
                providers: Constraint::Any,
                ownership: Constraint::Any,
                transport_protocol: Constraint::Only(TransportProtocol::Tcp),
            },
        };

        let custom_lists = &config.custom_lists;
        self.get_proxy_settings(&constraints, near_location, custom_lists)
            .map(|(settings, _relay)| settings)
    }

    fn should_use_bridge(retry_attempt: u32) -> bool {
        // shouldn't use a bridge for the first 3 times
        retry_attempt > 3 &&
            // i.e. 4th and 5th with bridge, 6th & 7th without
            // The test is to see whether the current _couple of connections_ is even or not.
            // | retry_attempt                | 4 | 5 | 6 | 7 | 8 | 9 |
            // | (retry_attempt % 4) < 2      | t | t | f | f | t | t |
            (retry_attempt % 4) < 2
    }

    fn get_proxy_settings<T: Into<Coordinates>>(
        &self,
        constraints: &InternalBridgeConstraints,
        location: Option<T>,
        custom_lists: &CustomListsSettings,
    ) -> Option<(CustomProxy, Relay)> {
        let matcher = RelayMatcher {
            locations: ResolvedLocationConstraint::from_constraint(
                constraints.location.clone(),
                custom_lists,
            ),
            providers: constraints.providers.clone(),
            ownership: constraints.ownership,
            endpoint_matcher: BridgeMatcher(()),
        };

        let matching_relays: Vec<Relay> = {
            let parsed_relays = self.parsed_relays.lock().unwrap();
            matcher.filter_matching_relay_list(parsed_relays.relays())
        };

        if matching_relays.is_empty() {
            return None;
        }

        let relay = if let Some(location) = location {
            let location = location.into();

            #[derive(Debug, Clone)]
            struct RelayWithDistance {
                relay: Relay,
                distance: f64,
            }

            let mut matching_relays: Vec<RelayWithDistance> = matching_relays
                .into_iter()
                .map(|relay| RelayWithDistance {
                    distance: relay.location.as_ref().unwrap().distance_from(&location),
                    relay,
                })
                .collect();
            matching_relays
                .sort_unstable_by_key(|relay: &RelayWithDistance| relay.distance as usize);

            let mut greatest_distance = 0f64;
            matching_relays = matching_relays
                .into_iter()
                .enumerate()
                .filter_map(|(i, relay)| {
                    if i < MIN_BRIDGE_COUNT || relay.distance <= MAX_BRIDGE_DISTANCE {
                        if relay.distance > greatest_distance {
                            greatest_distance = relay.distance;
                        }
                        return Some(relay);
                    }
                    None
                })
                .collect();

            let weight_fn =
                |relay: &RelayWithDistance| 1 + (greatest_distance - relay.distance) as u64;

            self.pick_random_relay_fn(&matching_relays, weight_fn)
                .cloned()
                .map(|relay_with_distance| relay_with_distance.relay)
        } else {
            self.pick_random_relay(&matching_relays).cloned()
        };
        relay.and_then(|relay| {
            let parsed_relays = self.parsed_relays.lock().unwrap();
            let bridge = &parsed_relays.parsed_list.bridge;
            self.pick_random_bridge(bridge, &relay)
                .map(|bridge| (bridge, relay.clone()))
        })
    }

    fn get_obfuscator_inner(
        &self,
        config: &MutexGuard<'_, SelectorConfig>,
        relay: &Relay,
        endpoint: &MullvadWireguardEndpoint,
        retry_attempt: u32,
    ) -> Result<Option<SelectedObfuscator>, Error> {
        match &config.obfuscation_settings.selected_obfuscation {
            SelectedObfuscation::Auto => Ok(self.get_auto_obfuscator(
                &config.obfuscation_settings,
                relay,
                endpoint,
                retry_attempt,
            )),
            SelectedObfuscation::Off => Ok(None),
            SelectedObfuscation::Udp2Tcp => Ok(Some(
                self.get_udp2tcp_obfuscator(
                    &config.obfuscation_settings.udp2tcp,
                    relay,
                    endpoint,
                    retry_attempt,
                )
                .ok_or(Error::NoObfuscator)?,
            )),
        }
    }

    fn get_auto_obfuscator(
        &self,
        obfuscation_settings: &ObfuscationSettings,
        relay: &Relay,
        endpoint: &MullvadWireguardEndpoint,
        retry_attempt: u32,
    ) -> Option<SelectedObfuscator> {
        let obfuscation_attempt = Self::get_auto_obfuscator_retry_attempt(retry_attempt)?;
        self.get_udp2tcp_obfuscator(
            &obfuscation_settings.udp2tcp,
            relay,
            endpoint,
            obfuscation_attempt,
        )
    }

    const fn get_auto_obfuscator_retry_attempt(retry_attempt: u32) -> Option<u32> {
        match retry_attempt % 4 {
            0 | 1 => None,
            // when the retry attempt is 2-3, 6-7, 10-11 ... obfuscation will be used
            filtered_retry => Some(retry_attempt / 4 + filtered_retry - 2),
        }
    }

    fn get_udp2tcp_obfuscator(
        &self,
        obfuscation_settings: &Udp2TcpObfuscationSettings,
        relay: &Relay,
        endpoint: &MullvadWireguardEndpoint,
        retry_attempt: u32,
    ) -> Option<SelectedObfuscator> {
        let udp2tcp_ports = {
            &self
                .parsed_relays
                .lock()
                .unwrap()
                .parsed_list
                .wireguard
                .udp2tcp_ports
        };
        let udp2tcp_endpoint = if obfuscation_settings.port.is_only() {
            udp2tcp_ports
                .iter()
                .find(|&candidate| obfuscation_settings.port == Constraint::Only(*candidate))
        } else {
            udp2tcp_ports.get(retry_attempt as usize % udp2tcp_ports.len())
        };
        udp2tcp_endpoint
            .map(|udp2tcp_endpoint| ObfuscatorConfig::Udp2Tcp {
                endpoint: SocketAddr::new(endpoint.peer.endpoint.ip(), *udp2tcp_endpoint),
            })
            .map(|config| SelectedObfuscator {
                config,
                relay: relay.clone(),
            })
    }

    /// Return the preferred constraints, on attempt `retry_attempt`, for matching locations
    fn preferred_tunnel_constraints_for_location(
        &self,
        retry_attempt: u32,
        location: &Constraint<ResolvedLocationConstraint>,
        providers: &Constraint<Providers>,
        ownership: Constraint<Ownership>,
    ) -> (Constraint<u16>, TransportProtocol, TunnelType) {
        let (location_supports_wg, location_supports_openvpn) = {
            let parsed_relays = self.parsed_relays.lock().unwrap();
            let mut active_location_relays = parsed_relays.relays().filter(|relay| {
                relay.active
                    && location.matches_with_opts(relay, true)
                    && providers.matches(relay)
                    && ownership.matches(relay)
            });
            let location_supports_wg = active_location_relays
                .clone()
                .any(|relay| matches!(relay.endpoint_data, RelayEndpointData::Wireguard(_)));
            let location_supports_openvpn = active_location_relays
                .any(|relay| matches!(relay.endpoint_data, RelayEndpointData::Openvpn));

            (location_supports_wg, location_supports_openvpn)
        };
        match (location_supports_wg, location_supports_openvpn) {
            (true, true) | (false, false) => Self::preferred_tunnel_constraints(retry_attempt),
            (true, false) => {
                let port = Self::preferred_wireguard_port(retry_attempt);
                (port, TransportProtocol::Udp, TunnelType::Wireguard)
            }
            (false, true) => {
                let (port, transport) = Self::preferred_openvpn_constraints(retry_attempt);
                (port, transport, TunnelType::OpenVpn)
            }
        }
    }

    /// Return the preferred constraints, on attempt `retry_attempt`, given no other constraints
    pub const fn preferred_tunnel_constraints(
        retry_attempt: u32,
    ) -> (Constraint<u16>, TransportProtocol, TunnelType) {
        // Use WireGuard on the first three attempts, then OpenVPN
        match retry_attempt {
            0..=2 => (
                Self::preferred_wireguard_port(retry_attempt),
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

    const fn preferred_wireguard_port(retry_attempt: u32) -> Constraint<u16> {
        // Alternate between using a random port and port 53
        if retry_attempt % 2 == 0 {
            Constraint::Any
        } else {
            Constraint::Only(53)
        }
    }

    const fn preferred_openvpn_constraints(
        retry_attempt: u32,
    ) -> (Constraint<u16>, TransportProtocol) {
        // Prefer UDP by default. But if that has failed a couple of times, then try TCP port
        // 443, which works for many with UDP problems. After that, just alternate
        // between protocols.
        // If the tunnel type constraint is set OpenVpn, from the 4th attempt onwards, the first
        // two retry attempts OpenVpn constraints should be set to TCP as a bridge will be used,
        // and to UDP or TCP for the next two attempts.
        match retry_attempt {
            0 | 1 => (Constraint::Any, TransportProtocol::Udp),
            2 | 3 => (Constraint::Only(443), TransportProtocol::Tcp),
            attempt if attempt % 4 < 2 => (Constraint::Any, TransportProtocol::Tcp),
            attempt if attempt % 4 == 2 => (Constraint::Any, TransportProtocol::Udp),
            _ => (Constraint::Any, TransportProtocol::Tcp),
        }
    }

    /// Returns a random relay endpoint if any is matching the given constraints.
    fn get_tunnel_endpoint_internal<T: EndpointMatcher>(
        &self,
        matcher: &RelayMatcher<T>,
    ) -> Result<NormalSelectedRelay, Error> {
        let matching_relays: Vec<Relay> = {
            let parsed_relays = self.parsed_relays.lock().unwrap();
            matcher
                .filter_matching_relay_list(parsed_relays.relays())
                .into_iter()
                .collect()
        };

        self.pick_random_relay(&matching_relays)
            .and_then(|selected_relay| {
                let endpoint = matcher.mullvad_endpoint(selected_relay);
                let addr_in = endpoint
                    .as_ref()
                    .map(|endpoint| endpoint.to_endpoint().address.ip())
                    .unwrap_or_else(|| IpAddr::from(selected_relay.ipv4_addr_in));
                log::info!("Selected relay {} at {}", selected_relay.hostname, addr_in);
                endpoint.map(|endpoint| NormalSelectedRelay::new(endpoint, selected_relay.clone()))
            })
            .ok_or(Error::NoRelay)
    }

    /// Picks a relay using [Self::pick_random_relay_fn], using the `weight` member of each relay
    /// as the weight function.
    fn pick_random_relay<'a>(&self, relays: &'a [Relay]) -> Option<&'a Relay> {
        self.pick_random_relay_fn(relays, |relay| relay.weight)
    }

    /// Pick a random relay from the given slice. Will return `None` if the given slice is empty.
    /// If all of the relays have a weight of 0, one will be picked at random without bias,
    /// otherwise roulette wheel selection will be used to pick only relays with non-zero
    /// weights.
    fn pick_random_relay_fn<'a, RelayType>(
        &self,
        relays: &'a [RelayType],
        weight_fn: impl Fn(&RelayType) -> u64,
    ) -> Option<&'a RelayType> {
        let total_weight: u64 = relays.iter().map(&weight_fn).sum();
        let mut rng = rand::thread_rng();
        if total_weight == 0 {
            relays.choose(&mut rng)
        } else {
            // Pick a random number in the range 1..=total_weight. This choses the relay with a
            // non-zero weight.
            let mut i: u64 = rng.gen_range(1..=total_weight);
            Some(
                relays
                    .iter()
                    .find(|relay| {
                        i = i.saturating_sub(weight_fn(relay));
                        i == 0
                    })
                    .expect("At least one relay must've had a weight above 0"),
            )
        }
    }

    /// Picks a random bridge from a relay.
    fn pick_random_bridge(&self, data: &BridgeEndpointData, relay: &Relay) -> Option<CustomProxy> {
        if relay.endpoint_data != RelayEndpointData::Bridge {
            return None;
        }
        data.shadowsocks
            .choose(&mut rand::thread_rng())
            .map(|shadowsocks_endpoint| {
                log::info!(
                    "Selected Shadowsocks bridge {} at {}:{}/{}",
                    relay.hostname,
                    relay.ipv4_addr_in,
                    shadowsocks_endpoint.port,
                    shadowsocks_endpoint.protocol
                );
                shadowsocks_endpoint.to_proxy_settings(relay.ipv4_addr_in.into())
            })
    }

    fn wireguard_exit_matcher(&self) -> WireguardMatcher {
        let wg = {
            self.parsed_relays
                .lock()
                .unwrap()
                .parsed_list
                .wireguard
                .clone()
        };
        let mut tunnel = WireguardMatcher::from_endpoint(wg);
        tunnel.ip_version = WIREGUARD_EXIT_IP_VERSION;
        tunnel.port = WIREGUARD_EXIT_PORT;
        tunnel
    }
}

#[derive(Debug)]
pub enum SelectedBridge {
    Normal(NormalSelectedBridge),
    Custom(CustomProxy),
}

#[derive(Debug)]
pub struct NormalSelectedBridge {
    pub settings: CustomProxy,
    pub relay: Relay,
}

#[derive(Debug)]
pub enum SelectedRelay {
    Normal(NormalSelectedRelay),
    Custom(CustomTunnelEndpoint),
}

#[derive(Debug)]
pub struct NormalSelectedRelay {
    pub exit_relay: Relay,
    pub endpoint: MullvadEndpoint,
    pub entry_relay: Option<Relay>,
}

#[derive(Debug)]
pub struct SelectedObfuscator {
    pub config: ObfuscatorConfig,
    pub relay: Relay,
}

impl NormalSelectedRelay {
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
        relay_constraints::{GeographicLocationConstraint, WireguardConstraints},
        relay_list::{
            OpenVpnEndpoint, OpenVpnEndpointData, RelayListCity, RelayListCountry,
            ShadowsocksEndpointData, WireguardEndpointData, WireguardRelayEndpointData,
        },
    };
    use once_cell::sync::Lazy;
    use std::collections::HashSet;
    use talpid_types::net::{wireguard::PublicKey, Endpoint};

    impl RelaySelector {
        fn get_obfuscator(
            &self,
            relay: &Relay,
            endpoint: &MullvadWireguardEndpoint,
            retry_attempt: u32,
        ) -> Result<Option<SelectedObfuscator>, Error> {
            self.get_obfuscator_inner(&self.config.lock().unwrap(), relay, endpoint, retry_attempt)
        }
    }

    static RELAYS: Lazy<RelayList> = Lazy::new(|| RelayList {
        etag: None,
        countries: vec![RelayListCountry {
            name: "Sweden".to_string(),
            code: "se".to_string(),
            cities: vec![RelayListCity {
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
                        provider: "provider0".to_string(),
                        weight: 1,
                        endpoint_data: RelayEndpointData::Wireguard(WireguardRelayEndpointData {
                            public_key: PublicKey::from_base64(
                                "BLNHNoGO88LjV/wDBa7CUUwUzPq/fO2UwcGLy56hKy4=",
                            )
                            .unwrap(),
                        }),
                        location: None,
                    },
                    Relay {
                        hostname: "se10-wireguard".to_string(),
                        ipv4_addr_in: "185.213.154.69".parse().unwrap(),
                        ipv6_addr_in: Some("2a03:1b20:5:f011::a10f".parse().unwrap()),
                        include_in_country: true,
                        active: true,
                        owned: false,
                        provider: "provider1".to_string(),
                        weight: 1,
                        endpoint_data: RelayEndpointData::Wireguard(WireguardRelayEndpointData {
                            public_key: PublicKey::from_base64(
                                "BLNHNoGO88LjV/wDBa7CUUwUzPq/fO2UwcGLy56hKy4=",
                            )
                            .unwrap(),
                        }),
                        location: None,
                    },
                    Relay {
                        hostname: "se-got-001".to_string(),
                        ipv4_addr_in: "185.213.154.131".parse().unwrap(),
                        ipv6_addr_in: None,
                        include_in_country: true,
                        active: true,
                        owned: true,
                        provider: "provider2".to_string(),
                        weight: 1,
                        endpoint_data: RelayEndpointData::Openvpn,
                        location: None,
                    },
                    Relay {
                        hostname: "se-got-002".to_string(),
                        ipv4_addr_in: "1.2.3.4".parse().unwrap(),
                        ipv6_addr_in: None,
                        include_in_country: true,
                        active: true,
                        owned: true,
                        provider: "provider0".to_string(),
                        weight: 1,
                        endpoint_data: RelayEndpointData::Openvpn,
                        location: None,
                    },
                    Relay {
                        hostname: "se-got-br-001".to_string(),
                        ipv4_addr_in: "1.3.3.7".parse().unwrap(),
                        ipv6_addr_in: None,
                        include_in_country: true,
                        active: true,
                        owned: true,
                        provider: "provider3".to_string(),
                        weight: 1,
                        endpoint_data: RelayEndpointData::Bridge,
                        location: None,
                    },
                ],
            }],
        }],
        openvpn: OpenVpnEndpointData {
            ports: vec![
                OpenVpnEndpoint {
                    port: 1194,
                    protocol: TransportProtocol::Udp,
                },
                OpenVpnEndpoint {
                    port: 443,
                    protocol: TransportProtocol::Tcp,
                },
                OpenVpnEndpoint {
                    port: 80,
                    protocol: TransportProtocol::Tcp,
                },
            ],
        },
        bridge: BridgeEndpointData {
            shadowsocks: vec![
                ShadowsocksEndpointData {
                    port: 443,
                    cipher: "aes-256-gcm".to_string(),
                    password: "mullvad".to_string(),
                    protocol: TransportProtocol::Tcp,
                },
                ShadowsocksEndpointData {
                    port: 1234,
                    cipher: "aes-256-cfb".to_string(),
                    password: "mullvad".to_string(),
                    protocol: TransportProtocol::Udp,
                },
                ShadowsocksEndpointData {
                    port: 1236,
                    cipher: "aes-256-gcm".to_string(),
                    password: "mullvad".to_string(),
                    protocol: TransportProtocol::Udp,
                },
            ],
        },
        wireguard: WireguardEndpointData {
            port_ranges: vec![(53, 53), (4000, 33433), (33565, 51820), (52000, 60000)],
            ipv4_gateway: "10.64.0.1".parse().unwrap(),
            ipv6_gateway: "fc00:bbbb:bbbb:bb01::1".parse().unwrap(),
            udp2tcp_ports: vec![],
        },
    });

    #[test]
    fn test_preferred_tunnel_protocol() {
        let relay_selector = RelaySelector::from_list(SelectorConfig::default(), RELAYS.clone());

        // Prefer WG if the location only supports it
        let location = GeographicLocationConstraint::Hostname(
            "se".to_string(),
            "got".to_string(),
            "se9-wireguard".to_string(),
        );
        let relay_constraints = RelayConstraints {
            location: Constraint::Only(LocationConstraint::from(location)),
            tunnel_protocol: Constraint::Any,
            ..RelayConstraints::default()
        };

        let preferred = relay_selector.preferred_constraints(
            &relay_constraints,
            BridgeState::Off,
            0,
            &CustomListsSettings::default(),
        );
        assert_eq!(
            preferred.tunnel_protocol,
            Constraint::Only(TunnelType::Wireguard)
        );

        for attempt in 0..10 {
            assert!(relay_selector
                .get_any_tunnel_endpoint(
                    &relay_constraints,
                    BridgeState::Off,
                    attempt,
                    &CustomListsSettings::default()
                )
                .is_ok());
        }

        // Prefer OpenVPN if the location only supports it
        let location = GeographicLocationConstraint::Hostname(
            "se".to_string(),
            "got".to_string(),
            "se-got-001".to_string(),
        );
        let relay_constraints = RelayConstraints {
            location: Constraint::Only(LocationConstraint::from(location)),
            tunnel_protocol: Constraint::Any,
            ..RelayConstraints::default()
        };

        let preferred = relay_selector.preferred_constraints(
            &relay_constraints,
            BridgeState::Off,
            0,
            &CustomListsSettings::default(),
        );
        assert_eq!(
            preferred.tunnel_protocol,
            Constraint::Only(TunnelType::OpenVpn)
        );

        for attempt in 0..10 {
            assert!(relay_selector
                .get_any_tunnel_endpoint(
                    &relay_constraints,
                    BridgeState::Off,
                    attempt,
                    &CustomListsSettings::default()
                )
                .is_ok());
        }
    }

    #[test]
    fn test_wg_entry_hostname_collision() {
        let relay_selector = RelaySelector::from_list(SelectorConfig::default(), RELAYS.clone());

        let location1 = GeographicLocationConstraint::Hostname(
            "se".to_string(),
            "got".to_string(),
            "se9-wireguard".to_string(),
        );
        let location2 = GeographicLocationConstraint::Hostname(
            "se".to_string(),
            "got".to_string(),
            "se10-wireguard".to_string(),
        );

        let mut relay_constraints = RelayConstraints {
            location: Constraint::Only(LocationConstraint::from(location1.clone())),
            tunnel_protocol: Constraint::Only(TunnelType::Wireguard),
            ..RelayConstraints::default()
        };

        relay_constraints.wireguard_constraints.use_multihop = true;
        relay_constraints.wireguard_constraints.entry_location =
            Constraint::Only(LocationConstraint::from(location1));

        // The same host cannot be used for entry and exit
        assert!(relay_selector
            .get_tunnel_endpoint(
                &relay_constraints,
                BridgeState::Off,
                0,
                &CustomListsSettings::default()
            )
            .is_err());

        relay_constraints.wireguard_constraints.entry_location =
            Constraint::Only(LocationConstraint::from(location2));

        // If the entry and exit differ, this should succeed
        assert!(relay_selector
            .get_tunnel_endpoint(
                &relay_constraints,
                BridgeState::Off,
                0,
                &CustomListsSettings::default()
            )
            .is_ok());
    }

    #[test]
    fn test_wg_entry_filter() -> Result<(), String> {
        let relay_selector = RelaySelector::from_list(SelectorConfig::default(), RELAYS.clone());

        let specific_hostname = "se10-wireguard";

        let location_general = LocationConstraint::from(GeographicLocationConstraint::City(
            "se".to_string(),
            "got".to_string(),
        ));
        let location_specific = LocationConstraint::from(GeographicLocationConstraint::Hostname(
            "se".to_string(),
            "got".to_string(),
            specific_hostname.to_string(),
        ));

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
            .get_tunnel_endpoint(
                &relay_constraints,
                BridgeState::Off,
                0,
                &CustomListsSettings::default(),
            )
            .map_err(|error| error.to_string())?
            .exit_relay;

        assert_ne!(exit_relay.hostname, specific_hostname);

        relay_constraints.location = Constraint::Only(location_specific);
        relay_constraints.wireguard_constraints.entry_location = Constraint::Only(location_general);

        // The entry must not equal the exit
        let NormalSelectedRelay {
            exit_relay,
            endpoint,
            ..
        } = relay_selector
            .get_tunnel_endpoint(
                &relay_constraints,
                BridgeState::Off,
                0,
                &CustomListsSettings::default(),
            )
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
    fn test_openvpn_constraints() -> Result<(), String> {
        let relay_selector = RelaySelector::from_list(SelectorConfig::default(), RELAYS.clone());

        const ACTUAL_TCP_PORT: u16 = 443;
        const ACTUAL_UDP_PORT: u16 = 1194;
        const NON_EXISTENT_PORT: u16 = 1337;

        // Test all combinations of constraints, and whether they should
        // match some relay
        const CONSTRAINT_COMBINATIONS: [(OpenVpnConstraints, bool); 7] = [
            (
                OpenVpnConstraints {
                    port: Constraint::Any,
                },
                true,
            ),
            (
                OpenVpnConstraints {
                    port: Constraint::Only(TransportPort {
                        protocol: TransportProtocol::Udp,
                        port: Constraint::Any,
                    }),
                },
                true,
            ),
            (
                OpenVpnConstraints {
                    port: Constraint::Only(TransportPort {
                        protocol: TransportProtocol::Tcp,
                        port: Constraint::Any,
                    }),
                },
                true,
            ),
            (
                OpenVpnConstraints {
                    port: Constraint::Only(TransportPort {
                        protocol: TransportProtocol::Udp,
                        port: Constraint::Only(ACTUAL_UDP_PORT),
                    }),
                },
                true,
            ),
            (
                OpenVpnConstraints {
                    port: Constraint::Only(TransportPort {
                        protocol: TransportProtocol::Udp,
                        port: Constraint::Only(NON_EXISTENT_PORT),
                    }),
                },
                false,
            ),
            (
                OpenVpnConstraints {
                    port: Constraint::Only(TransportPort {
                        protocol: TransportProtocol::Tcp,
                        port: Constraint::Only(ACTUAL_TCP_PORT),
                    }),
                },
                true,
            ),
            (
                OpenVpnConstraints {
                    port: Constraint::Only(TransportPort {
                        protocol: TransportProtocol::Tcp,
                        port: Constraint::Only(NON_EXISTENT_PORT),
                    }),
                },
                false,
            ),
        ];

        let matches_constraints =
            |endpoint: Endpoint, constraints: &OpenVpnConstraints| match constraints.port {
                Constraint::Any => true,
                Constraint::Only(TransportPort { protocol, port }) => {
                    if endpoint.protocol != protocol {
                        return false;
                    }
                    match port {
                        Constraint::Any => true,
                        Constraint::Only(port) => port == endpoint.address.port(),
                    }
                }
            };

        let mut relay_constraints = RelayConstraints {
            tunnel_protocol: Constraint::Only(TunnelType::OpenVpn),
            ..RelayConstraints::default()
        };

        for (openvpn_constraints, should_match) in &CONSTRAINT_COMBINATIONS {
            relay_constraints.openvpn_constraints = *openvpn_constraints;

            for retry_attempt in 0..10 {
                let relay = relay_selector.get_tunnel_endpoint(
                    &relay_constraints,
                    BridgeState::Auto,
                    retry_attempt,
                    &CustomListsSettings::default(),
                );

                println!("relay: {relay:?}, constraints: {relay_constraints:?}");

                if !should_match {
                    relay.expect_err("unexpected relay");
                    continue;
                }

                let relay = relay.expect("expected to find a relay");

                assert!(
                    matches_constraints(
                        relay.endpoint.to_endpoint(),
                        &relay_constraints.openvpn_constraints,
                    ),
                    "{relay:?}, on attempt {retry_attempt}, did not match constraints: {relay_constraints:?}"
                );
            }
        }

        Ok(())
    }

    #[test]
    fn test_bridge_constraints() -> Result<(), String> {
        let relay_selector = RelaySelector::from_list(SelectorConfig::default(), RELAYS.clone());

        let location = LocationConstraint::from(GeographicLocationConstraint::Hostname(
            "se".to_string(),
            "got".to_string(),
            "se-got-001".to_string(),
        ));
        let mut relay_constraints = RelayConstraints {
            location: Constraint::Only(location),
            tunnel_protocol: Constraint::Any,
            ..RelayConstraints::default()
        };
        relay_constraints.openvpn_constraints.port = Constraint::Only(TransportPort {
            protocol: TransportProtocol::Udp,
            port: Constraint::Any,
        });

        let preferred = relay_selector.preferred_constraints(
            &relay_constraints,
            BridgeState::On,
            0,
            &CustomListsSettings::default(),
        );
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
        let location = LocationConstraint::from(GeographicLocationConstraint::Hostname(
            "se".to_string(),
            "got".to_string(),
            "se10-wireguard".to_string(),
        ));
        let relay_constraints = RelayConstraints {
            location: Constraint::Only(location),
            tunnel_protocol: Constraint::Any,
            ..RelayConstraints::default()
        };
        let preferred = relay_selector.preferred_constraints(
            &relay_constraints,
            BridgeState::On,
            0,
            &CustomListsSettings::default(),
        );
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
        let preferred = relay_selector.preferred_constraints(
            &relay_constraints,
            BridgeState::On,
            0,
            &CustomListsSettings::default(),
        );
        assert_eq!(
            preferred.tunnel_protocol,
            Constraint::Only(TunnelType::Wireguard)
        );
        let preferred = relay_selector.preferred_constraints(
            &relay_constraints,
            BridgeState::On,
            3,
            &CustomListsSettings::default(),
        );
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

        let relay_selector = RelaySelector::from_list(SelectorConfig::default(), RELAYS.clone());

        let result = relay_selector.get_tunnel_endpoint(&relay_constraints, BridgeState::Off, 0, &CustomListsSettings::default())
            .expect("Failed to get relay when tunnel constraints are set to Any and retrying the selection");

        assert!(
            matches!(result.endpoint, MullvadEndpoint::Wireguard(_))
                && result.entry_relay.is_some()
        );
    }

    const WIREGUARD_MULTIHOP_CONSTRAINTS: RelayConstraints = RelayConstraints {
        location: Constraint::Any,
        providers: Constraint::Any,
        ownership: Constraint::Any,
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
        ownership: Constraint::Any,
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
        let relay_selector = RelaySelector::from_list(SelectorConfig::default(), RELAYS.clone());

        let result = relay_selector.get_tunnel_endpoint(&WIREGUARD_MULTIHOP_CONSTRAINTS, BridgeState::Off, 0, &CustomListsSettings::default())
            .expect("Failed to get relay when tunnel constraints are set to default WireGuard multihop constraints");

        assert!(result.entry_relay.is_some());
        // TODO: Verify that neither endpoint is using obfuscation for retry attempt 0
    }

    #[test]
    fn test_selecting_wg_endpoint_with_udp2tcp_obfuscation() {
        let relay_selector = RelaySelector::from_list(SelectorConfig::default(), RELAYS.clone());

        let result = relay_selector.get_tunnel_endpoint(&WIREGUARD_SINGLEHOP_CONSTRAINTS, BridgeState::Off, 0, &CustomListsSettings::default())
            .expect("Failed to get relay when tunnel constraints are set to default WireGuard constraints");

        assert!(result.entry_relay.is_none());
        assert!(matches!(result.endpoint, MullvadEndpoint::Wireguard { .. }));

        {
            relay_selector.config.lock().unwrap().obfuscation_settings = ObfuscationSettings {
                selected_obfuscation: SelectedObfuscation::Udp2Tcp,
                ..ObfuscationSettings::default()
            };
        }

        let obfs_config = relay_selector
            .get_obfuscator(&result.exit_relay, result.endpoint.unwrap_wireguard(), 0)
            .unwrap()
            .unwrap();

        assert!(matches!(
            obfs_config,
            SelectedObfuscator {
                config: ObfuscatorConfig::Udp2Tcp { .. },
                ..
            }
        ));
    }

    #[test]
    fn test_selecting_wg_endpoint_with_auto_obfuscation() {
        let relay_selector = RelaySelector::from_list(SelectorConfig::default(), RELAYS.clone());

        let result = relay_selector.get_tunnel_endpoint(&WIREGUARD_SINGLEHOP_CONSTRAINTS, BridgeState::Off, 0, &CustomListsSettings::default())
            .expect("Failed to get relay when tunnel constraints are set to default WireGuard constraints");

        assert!(result.entry_relay.is_none());
        assert!(matches!(result.endpoint, MullvadEndpoint::Wireguard { .. }));

        {
            relay_selector.config.lock().unwrap().obfuscation_settings = ObfuscationSettings {
                selected_obfuscation: SelectedObfuscation::Auto,
                ..ObfuscationSettings::default()
            };
        }

        assert!(relay_selector
            .get_obfuscator(&result.exit_relay, result.endpoint.unwrap_wireguard(), 0,)
            .unwrap()
            .is_none());

        assert!(relay_selector
            .get_obfuscator(&result.exit_relay, result.endpoint.unwrap_wireguard(), 1,)
            .unwrap()
            .is_none());

        assert!(relay_selector
            .get_obfuscator(&result.exit_relay, result.endpoint.unwrap_wireguard(), 2,)
            .unwrap()
            .is_some());
    }

    #[test]
    fn test_selected_endpoints_use_correct_port_ranges() {
        let relay_selector = RelaySelector::from_list(SelectorConfig::default(), RELAYS.clone());

        const TCP2UDP_PORTS: [u16; 3] = [80, 443, 5001];

        {
            relay_selector.config.lock().unwrap().obfuscation_settings = ObfuscationSettings {
                selected_obfuscation: SelectedObfuscation::Udp2Tcp,
                ..ObfuscationSettings::default()
            };
        }

        for attempt in 0..1000 {
            let result = relay_selector
                .get_tunnel_endpoint(
                    &WIREGUARD_SINGLEHOP_CONSTRAINTS,
                    BridgeState::Off,
                    attempt,
                    &CustomListsSettings::default(),
                )
                .expect("Failed to select a WireGuard relay");
            assert!(result.entry_relay.is_none());

            let obfs_config = relay_selector
                .get_obfuscator(
                    &result.exit_relay,
                    result.endpoint.unwrap_wireguard(),
                    attempt,
                )
                .unwrap()
                .expect("Failed to get Tcp2Udp endpoint");

            assert!(matches!(
                obfs_config,
                SelectedObfuscator {
                    config: ObfuscatorConfig::Udp2Tcp { .. },
                    ..
                }
            ));

            let SelectedObfuscator {
                config: ObfuscatorConfig::Udp2Tcp { endpoint },
                ..
            } = obfs_config;
            assert!(TCP2UDP_PORTS.contains(&endpoint.port()));
        }
    }

    #[test]
    fn test_ownership() {
        let relay_selector = RelaySelector::from_list(SelectorConfig::default(), RELAYS.clone());
        let mut constraints = RelayConstraints::default();
        for i in 0..10 {
            constraints.ownership = Constraint::Only(Ownership::MullvadOwned);
            let relay = relay_selector
                .get_tunnel_endpoint(
                    &constraints,
                    BridgeState::Auto,
                    i,
                    &CustomListsSettings::default(),
                )
                .unwrap();
            assert!(matches!(
                relay,
                NormalSelectedRelay {
                    exit_relay: Relay { owned: true, .. },
                    ..
                }
            ));

            constraints.ownership = Constraint::Only(Ownership::Rented);
            let relay = relay_selector
                .get_tunnel_endpoint(
                    &constraints,
                    BridgeState::Auto,
                    i,
                    &CustomListsSettings::default(),
                )
                .unwrap();
            assert!(matches!(
                relay,
                NormalSelectedRelay {
                    exit_relay: Relay { owned: false, .. },
                    ..
                }
            ));
        }
    }

    // Make sure server and port selection varies between retry attempts.
    #[test]
    fn test_load_balancing() {
        let relay_selector = RelaySelector::from_list(SelectorConfig::default(), RELAYS.clone());

        for tunnel_protocol in [
            Constraint::Any,
            Constraint::Only(TunnelType::Wireguard),
            Constraint::Only(TunnelType::OpenVpn),
        ] {
            {
                let mut config = relay_selector.config.lock().unwrap();
                config.relay_settings = RelaySettings::Normal(RelayConstraints {
                    tunnel_protocol,
                    location: Constraint::Only(LocationConstraint::from(
                        GeographicLocationConstraint::Country("se".to_string()),
                    )),
                    ..RelayConstraints::default()
                });
            }

            let mut actual_ports = HashSet::new();
            let mut actual_ips = HashSet::new();

            for retry_attempt in 0..30 {
                let (relay, ..) = relay_selector.get_relay(retry_attempt).unwrap();
                match relay {
                    SelectedRelay::Normal(relay) => {
                        let address = relay.endpoint.to_endpoint().address;
                        actual_ports.insert(address.port());
                        actual_ips.insert(address.ip());
                    }
                    SelectedRelay::Custom(_) => unreachable!("not using custom relay"),
                }
            }

            assert!(
                actual_ports.len() > 1,
                "expected more than 1 port, got {actual_ports:?}, for tunnel protocol {tunnel_protocol:?}",
            );
            assert!(
                actual_ips.len() > 1,
                "expected more than 1 server, got {actual_ips:?}, for tunnel protocol {tunnel_protocol:?}",
            );
        }
    }

    #[test]
    fn test_providers() {
        const EXPECTED_PROVIDERS: [&str; 2] = ["provider0", "provider2"];

        let relay_selector = RelaySelector::from_list(SelectorConfig::default(), RELAYS.clone());
        let mut constraints = RelayConstraints::default();

        for i in 0..10 {
            constraints.providers = Constraint::Only(
                Providers::new(EXPECTED_PROVIDERS.into_iter().map(|p| p.to_owned())).unwrap(),
            );
            let relay = relay_selector
                .get_tunnel_endpoint(
                    &constraints,
                    BridgeState::Auto,
                    i,
                    &CustomListsSettings::default(),
                )
                .unwrap();
            assert!(
                EXPECTED_PROVIDERS.contains(&relay.exit_relay.provider.as_str()),
                "cannot find provider {} in {:?}",
                relay.exit_relay.provider,
                EXPECTED_PROVIDERS
            );
        }
    }

    /// Verify that bridges are automatically used when bridge mode is set
    /// to automatic.
    #[test]
    fn test_auto_bridge() {
        let relay_selector = RelaySelector::from_list(SelectorConfig::default(), RELAYS.clone());

        {
            let mut config = relay_selector.config.lock().unwrap();
            config.bridge_state = BridgeState::Auto;
        }

        const ATTEMPT_SHOULD_USE_BRIDGE: [bool; 5] = [false, false, false, false, true];

        for (i, should_use_bridge) in ATTEMPT_SHOULD_USE_BRIDGE.iter().enumerate() {
            let (_relay, bridge, _obfs) = relay_selector.get_relay(i as u32).unwrap();
            assert_eq!(*should_use_bridge, bridge.is_some());
        }

        // Verify that bridges are ignored when tunnel protocol is WireGuard
        {
            let mut config = relay_selector.config.lock().unwrap();
            config.relay_settings = RelaySettings::Normal(RelayConstraints {
                tunnel_protocol: Constraint::Only(TunnelType::Wireguard),
                ..RelayConstraints::default()
            });
        }
        for i in 0..20 {
            let (_relay, bridge, _obfs) = relay_selector.get_relay(i).unwrap();
            assert!(bridge.is_none());
        }
    }

    /// Ensure that `include_in_country` is ignored if all relays have it set to false (i.e., some
    /// relay is returned). Also ensure that `include_in_country` is respected if some relays
    /// have it set to true (i.e., that relay is never returned)
    #[test]
    fn test_include_in_country() {
        let mut relay_list = RelayList {
            etag: None,
            countries: vec![RelayListCountry {
                name: "Sweden".to_string(),
                code: "se".to_string(),
                cities: vec![RelayListCity {
                    name: "Gothenburg".to_string(),
                    code: "got".to_string(),
                    latitude: 57.70887,
                    longitude: 11.97456,
                    relays: vec![
                        Relay {
                            hostname: "se9-wireguard".to_string(),
                            ipv4_addr_in: "185.213.154.68".parse().unwrap(),
                            ipv6_addr_in: Some("2a03:1b20:5:f011::a09f".parse().unwrap()),
                            include_in_country: false,
                            active: true,
                            owned: true,
                            provider: "31173".to_string(),
                            weight: 1,
                            endpoint_data: RelayEndpointData::Wireguard(
                                WireguardRelayEndpointData {
                                    public_key: PublicKey::from_base64(
                                        "BLNHNoGO88LjV/wDBa7CUUwUzPq/fO2UwcGLy56hKy4=",
                                    )
                                    .unwrap(),
                                },
                            ),
                            location: None,
                        },
                        Relay {
                            hostname: "se10-wireguard".to_string(),
                            ipv4_addr_in: "185.213.154.69".parse().unwrap(),
                            ipv6_addr_in: Some("2a03:1b20:5:f011::a10f".parse().unwrap()),
                            include_in_country: false,
                            active: true,
                            owned: false,
                            provider: "31173".to_string(),
                            weight: 1,
                            endpoint_data: RelayEndpointData::Wireguard(
                                WireguardRelayEndpointData {
                                    public_key: PublicKey::from_base64(
                                        "BLNHNoGO88LjV/wDBa7CUUwUzPq/fO2UwcGLy56hKy4=",
                                    )
                                    .unwrap(),
                                },
                            ),
                            location: None,
                        },
                    ],
                }],
            }],
            openvpn: OpenVpnEndpointData {
                ports: vec![
                    OpenVpnEndpoint {
                        port: 1194,
                        protocol: TransportProtocol::Udp,
                    },
                    OpenVpnEndpoint {
                        port: 443,
                        protocol: TransportProtocol::Tcp,
                    },
                    OpenVpnEndpoint {
                        port: 80,
                        protocol: TransportProtocol::Tcp,
                    },
                ],
            },
            bridge: BridgeEndpointData {
                shadowsocks: vec![],
            },
            wireguard: WireguardEndpointData {
                port_ranges: vec![(53, 53), (4000, 33433), (33565, 51820), (52000, 60000)],
                ipv4_gateway: "10.64.0.1".parse().unwrap(),
                ipv6_gateway: "fc00:bbbb:bbbb:bb01::1".parse().unwrap(),
                udp2tcp_ports: vec![],
            },
        };

        // If include_in_country is false for all relays, a relay must be selected anyway.
        //

        let relay_selector =
            RelaySelector::from_list(SelectorConfig::default(), relay_list.clone());
        assert!(relay_selector.get_relay(0).is_ok());

        // If include_in_country is true for some relay, it must always be selected.
        //

        relay_list.countries[0].cities[0].relays[0].include_in_country = true;
        let expected_hostname = relay_list.countries[0].cities[0].relays[0].hostname.clone();

        let relay_selector = RelaySelector::from_list(SelectorConfig::default(), relay_list);
        let (relay, ..) = relay_selector.get_relay(0).expect("expected match");

        assert!(
            matches!(
                relay,
                SelectedRelay::Normal(NormalSelectedRelay {
                    exit_relay: Relay {
                        ref hostname,
                        ..
                    },
                    ..
                }) if hostname == &expected_hostname,
            ),
            "found {relay:?}, expected {expected_hostname:?}",
        )
    }
}
